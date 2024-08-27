use std::{collections::BTreeMap, str::FromStr};

use executor_custom_data_model::multisig::{MultisigAccountArgs, MultisigTransactionArgs};
use eyre::Result;
use iroha::{
    client,
    crypto::KeyPair,
    data_model::{prelude::*, query::trigger::FindTriggers, transaction::TransactionBuilder},
};
use iroha_test_network::*;
use iroha_test_samples::gen_account_in;

#[test]
fn mutlisig() -> Result<()> {
    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    // Predefined in default genesis
    let multisig_accounts_registry_id = TriggerId::from_str("multisig_accounts_wonderland")?;

    // Create multisig account id and destroy it's private key
    // FIXME #5022 Should not allow arbitrary IDs. Otherwise, after #4426 pre-registration account will be hijacked as a multisig account
    let multisig_account_id = gen_account_in("wonderland").0;

    let multisig_transactions_registry_id: TriggerId = format!(
        "multisig_transactions_{}_{}",
        multisig_account_id.signatory(),
        multisig_account_id.domain()
    )
    .parse()?;

    let signatories = core::iter::repeat_with(|| gen_account_in("wonderland"))
        .take(5)
        .collect::<BTreeMap<AccountId, KeyPair>>();

    let args = &MultisigAccountArgs {
        account: Account::new(multisig_account_id.clone()),
        signatories: signatories.keys().cloned().collect(),
    };

    test_client.submit_all_blocking(
        signatories
            .keys()
            .cloned()
            .map(Account::new)
            .map(Register::account),
    )?;

    let register_multisig_account =
        ExecuteTrigger::new(multisig_accounts_registry_id).with_args(args);
    test_client.submit_blocking(register_multisig_account)?;

    // Check that multisig account exist
    test_client
        .query(client::account::all())
        .filter_with(|account| account.id.eq(multisig_account_id.clone()))
        .execute_single()
        .expect("multisig account should be created by calling the multisig accounts registry");

    // Check that multisig transactions registry exist
    let trigger = test_client
        .query(FindTriggers::new())
        .filter_with(|trigger| trigger.id.eq(multisig_transactions_registry_id.clone()))
        .execute_single()
        .expect("multisig transactions registry should be created along with the corresponding multisig account");

    assert_eq!(trigger.id(), &multisig_transactions_registry_id);

    let key: Name = "key".parse().unwrap();
    let instructions = vec![SetKeyValue::account(
        multisig_account_id.clone(),
        key.clone(),
        "value".parse::<Json>().unwrap(),
    )
    .into()];
    let instructions_hash = HashOf::new(&instructions);

    let mut signatories_iter = signatories.into_iter();

    if let Some((signatory, key_pair)) = signatories_iter.next() {
        let args = &MultisigTransactionArgs::Propose(instructions);
        let propose =
            ExecuteTrigger::new(multisig_transactions_registry_id.clone()).with_args(args);
        test_client.submit_transaction_blocking(
            &TransactionBuilder::new(test_client.chain.clone(), signatory)
                .with_instructions([propose])
                .sign(key_pair.private_key()),
        )?;
    }

    // Check that the multisig transaction has not yet executed
    let _err = test_client
        .query_single(FindAccountMetadata::new(
            multisig_account_id.clone(),
            key.clone(),
        ))
        .expect_err("key-value shouldn't be set without enough approvals");

    for (signatory, key_pair) in signatories_iter {
        let args = &MultisigTransactionArgs::Approve(instructions_hash);
        let approve =
            ExecuteTrigger::new(multisig_transactions_registry_id.clone()).with_args(args);
        test_client.submit_transaction_blocking(
            &TransactionBuilder::new(test_client.chain.clone(), signatory)
                .with_instructions([approve])
                .sign(key_pair.private_key()),
        )?;
    }

    // Check that the multisig transaction has executed
    test_client
        .query_single(FindAccountMetadata::new(
            multisig_account_id.clone(),
            key.clone(),
        ))
        .expect("key-value should be set with enough approvals");

    Ok(())
}
