use std::{collections::BTreeMap, time::Duration};

use executor_custom_data_model::multisig::{MultisigAccountArgs, MultisigTransactionArgs};
use eyre::Result;
use iroha::{
    client,
    crypto::KeyPair,
    data_model::{
        prelude::*, query::trigger::FindTriggers, transaction::TransactionBuilder, Level,
    },
};
use iroha_test_network::*;
use iroha_test_samples::{gen_account_in, CARPENTER_ID, CARPENTER_KEYPAIR};

#[test]
fn multisig() -> Result<()> {
    multisig_base(None)
}

#[test]
fn multisig_expires() -> Result<()> {
    multisig_base(Some(2))
}

/// # Scenario
///
/// | world level               | domain level                | account level                   | transaction level    |
/// |---------------------------|-----------------------------|---------------------------------|----------------------|
/// | given domains initializer |                             |                                 |                      |
/// |                           | creates domain              |                                 |                      |
/// |       domains initializer | generates accounts registry |                                 |                      |
/// |                           |                             | creates signatories             |                      |
/// |                           |      call accounts registry | creates multisig account        |                      |
/// |                           |           accounts registry | generates transactions registry |                      |
/// |                           |                             |      call transactions registry | proposes transaction |
/// |                           |                             |      call transactions registry | approves transaction |
/// |                           |                             |           transactions registry | executes transaction |
fn multisig_base(transaction_ttl_secs: Option<u32>) -> Result<()> {
    const N_SIGNATORIES: usize = 5;

    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    let kingdom = "kingdom";
    // Assume some domain registered after genesis
    test_client.submit_blocking(Register::domain(Domain::new(kingdom.parse().unwrap())))?;
    // One more block to generate a multisig accounts registry for the domain
    test_client.submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))?;

    // Check that the multisig accounts registry has been generated
    let multisig_accounts_registry_id: TriggerId =
        format!("multisig_accounts_{kingdom}").parse()?;
    let _trigger = test_client
        .query(FindTriggers::new())
        .filter_with(|trigger| trigger.id.eq(multisig_accounts_registry_id.clone()))
        .execute_single()
        .expect("multisig accounts registry should be generated after domain creation");

    // Populate residents in the domain
    let mut residents = core::iter::repeat_with(|| gen_account_in(kingdom))
        .take(1 + N_SIGNATORIES)
        .collect::<BTreeMap<AccountId, KeyPair>>();
    test_client.submit_all_blocking(
        residents
            .keys()
            .cloned()
            .map(Account::new)
            .map(Register::account),
    )?;

    // Create a multisig account ID and discard the corresponding private key
    // FIXME #5022 Should not allow arbitrary IDs. Otherwise, after #4426 pre-registration account will be hijacked as a multisig account
    let multisig_account_id = gen_account_in(kingdom).0;

    let not_signatory = residents.pop_first().unwrap();
    let mut signatories = residents;

    let args = &MultisigAccountArgs {
        account: Account::new(multisig_account_id.clone()),
        signatories: signatories
            .keys()
            .enumerate()
            .map(|(weight, id)| (id.clone(), 1 + weight as u8))
            .collect(),
        // Can be met without the first signatory
        quorum: (1..=N_SIGNATORIES).skip(1).sum::<usize>() as u16,
        transaction_ttl_secs,
    };
    let register_multisig_account =
        ExecuteTrigger::new(multisig_accounts_registry_id).with_args(args);

    let client = |account: AccountId, key_pair: KeyPair| client::Client {
        account,
        key_pair,
        ..test_client.clone()
    };

    // Any account in another domain cannot register a multisig account without special permission
    let carpenter_client = client(CARPENTER_ID.clone(), CARPENTER_KEYPAIR.clone());
    let _err = carpenter_client
        .submit_blocking(register_multisig_account.clone())
        .expect_err("multisig account should not be registered by account of another domain");

    // Any account in the same domain can register a multisig account without special permission
    let not_signatory_client = client(not_signatory.0, not_signatory.1);
    not_signatory_client
        .submit_blocking(register_multisig_account)
        .expect("multisig account should be registered by account of the same domain");

    // Check that the multisig account has been registered
    test_client
        .query(client::account::all())
        .filter_with(|account| account.id.eq(multisig_account_id.clone()))
        .execute_single()
        .expect("multisig account should be created by calling the multisig accounts registry");

    // Check that the multisig transactions registry has been generated
    let multisig_transactions_registry_id: TriggerId = format!(
        "multisig_transactions_{}_{}",
        multisig_account_id.signatory(),
        multisig_account_id.domain()
    )
    .parse()?;
    let _trigger = test_client
        .query(FindTriggers::new())
        .filter_with(|trigger| trigger.id.eq(multisig_transactions_registry_id.clone()))
        .execute_single()
        .expect("multisig transactions registry should be generated along with the corresponding multisig account");

    let key: Name = "key".parse().unwrap();
    let instructions = vec![SetKeyValue::account(
        multisig_account_id.clone(),
        key.clone(),
        "value".parse::<Json>().unwrap(),
    )
    .into()];
    let instructions_hash = HashOf::new(&instructions);

    let proposer = signatories.pop_last().unwrap();
    let approvers = signatories;

    let args = &MultisigTransactionArgs::Propose(instructions);
    let propose = ExecuteTrigger::new(multisig_transactions_registry_id.clone()).with_args(args);

    // One of signatories proposes a multisig transaction
    test_client.submit_transaction_blocking(
        &TransactionBuilder::new(test_client.chain.clone(), proposer.0)
            .with_instructions([propose])
            .sign(proposer.1.private_key()),
    )?;
    // Check that the multisig transaction has not yet executed
    let _err = test_client
        .query_single(FindAccountMetadata::new(
            multisig_account_id.clone(),
            key.clone(),
        ))
        .expect_err("key-value shouldn't be set without enough approvals");

    // Allow time to elapse to test the expiration
    if let Some(s) = transaction_ttl_secs {
        std::thread::sleep(Duration::from_secs(s.into()))
    };
    test_client.submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))?;

    // All but the first signatory approve the multisig transaction
    for approver in approvers.into_iter().skip(1) {
        let args = &MultisigTransactionArgs::Approve(instructions_hash);
        let approve =
            ExecuteTrigger::new(multisig_transactions_registry_id.clone()).with_args(args);

        test_client.submit_transaction_blocking(
            &TransactionBuilder::new(test_client.chain.clone(), approver.0)
                .with_instructions([approve])
                .sign(approver.1.private_key()),
        )?;
    }
    // Check that the multisig transaction has executed
    let res = test_client.query_single(FindAccountMetadata::new(
        multisig_account_id.clone(),
        key.clone(),
    ));

    if transaction_ttl_secs.is_some() {
        let _err = res.expect_err("key-value shouldn't be set despite enough approvals");
    } else {
        res.expect("key-value should be set with enough approvals");
    }

    Ok(())
}
