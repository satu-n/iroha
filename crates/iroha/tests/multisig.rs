use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use eyre::Result;
use iroha::{
    client::Client,
    crypto::KeyPair,
    data_model::{prelude::*, query::trigger::FindTriggers, Level},
    multisig_data_model::*,
};
use iroha_multisig_data_model::approvals_key;
use iroha_test_network::*;
use iroha_test_samples::{
    gen_account_in, ALICE_ID, BOB_ID, BOB_KEYPAIR, CARPENTER_ID, CARPENTER_KEYPAIR,
};

#[test]
fn multisig() -> Result<()> {
    multisig_base(None)
}

#[test]
fn multisig_expires() -> Result<()> {
    multisig_base(Some(2))
}

#[allow(clippy::cast_possible_truncation)]
fn multisig_base(transaction_ttl_ms: Option<u64>) -> Result<()> {
    const N_SIGNATORIES: usize = 5;

    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    let kingdom: DomainId = "kingdom".parse().unwrap();

    // Assume some domain registered after genesis
    let register_and_transfer_kingdom: [InstructionBox; 2] = [
        Register::domain(Domain::new(kingdom.clone())).into(),
        Transfer::domain(ALICE_ID.clone(), kingdom.clone(), BOB_ID.clone()).into(),
    ];
    test_client.submit_all_blocking(register_and_transfer_kingdom)?;

    // One more block to generate a multisig accounts registry for the domain
    // SATO
    // test_client.submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))?;

    // Populate residents in the domain
    let mut residents = core::iter::repeat_with(|| gen_account_in(&kingdom))
        .take(1 + N_SIGNATORIES)
        .collect::<BTreeMap<AccountId, KeyPair>>();
    alt_client((BOB_ID.clone(), BOB_KEYPAIR.clone()), &test_client).submit_all_blocking(
        residents
            .keys()
            .cloned()
            .map(Account::new)
            .map(Register::account),
    )?;

    // Create a multisig account ID and discard the corresponding private key
    let multisig_account_id = gen_account_in(&kingdom).0;

    let not_signatory = residents.pop_first().unwrap();
    let mut signatories = residents;

    let register_multisig_account = MultisigRegister::new(
        multisig_account_id.clone(),
        signatories
            .keys()
            .enumerate()
            .map(|(weight, id)| (id.clone(), 1 + weight as u8))
            .collect(),
        // Can be met without the first signatory
        (1..=N_SIGNATORIES).skip(1).sum::<usize>() as u16,
        transaction_ttl_ms.unwrap_or(u64::MAX),
    );

    // Any account in another domain cannot register a multisig account without special permission
    let _err = alt_client(
        (CARPENTER_ID.clone(), CARPENTER_KEYPAIR.clone()),
        &test_client,
    )
    .submit_blocking(register_multisig_account.clone())
    .expect_err("multisig account should not be registered by account of another domain");

    // Any account in the same domain can register a multisig account without special permission
    alt_client(not_signatory, &test_client)
        .submit_blocking(register_multisig_account)
        .expect("multisig account should be registered by account of the same domain");

    // Check that the multisig account has been registered
    test_client
        .query(FindAccounts::new())
        .filter_with(|account| account.id.eq(multisig_account_id.clone()))
        .execute_single()
        .expect("multisig account should be created");

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

    let propose = MultisigPropose::new(multisig_account_id.clone(), instructions);

    alt_client(proposer, &test_client).submit_blocking(propose)?;

    // Check that the multisig transaction has not yet executed
    let _err = test_client
        .query_single(FindAccountMetadata::new(
            multisig_account_id.clone(),
            key.clone(),
        ))
        .expect_err("instructions shouldn't execute without enough approvals");

    // Allow time to elapse to test the expiration
    if let Some(ms) = transaction_ttl_ms {
        std::thread::sleep(Duration::from_millis(ms))
    };
    test_client.submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))?;

    // All but the first signatory approve the multisig transaction
    for approver in approvers.into_iter().skip(1) {
        let approve = MultisigApprove::new(multisig_account_id.clone(), instructions_hash);

        alt_client(approver, &test_client).submit_blocking(approve)?;
    }

    // Check that the multisig transaction has executed
    let res = test_client.query_single(FindAccountMetadata::new(
        multisig_account_id.clone(),
        key.clone(),
    ));

    if transaction_ttl_ms.is_some() {
        let _err = res.expect_err("instructions shouldn't execute despite enough approvals");
    } else {
        res.expect("instructions should execute with enough approvals");
    }

    Ok(())
}

/// # Scenario
///
/// ```
///         012345 <--- root multisig account
///        /      \
///       /        12345
///      /        /     \
///     /       12       345
///    /       /  \     / | \
///   0       1    2   3  4  5 <--- personal signatories
/// ```
#[test]
#[allow(clippy::similar_names, clippy::too_many_lines)]
fn multisig_recursion() -> Result<()> {
    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    let wonderland = "wonderland";

    // Populate signatories in the domain
    let signatories = core::iter::repeat_with(|| gen_account_in(wonderland))
        .take(6)
        .collect::<BTreeMap<AccountId, KeyPair>>();
    test_client.submit_all_blocking(
        signatories
            .keys()
            .cloned()
            .map(Account::new)
            .map(Register::account),
    )?;

    // Recursively register multisig accounts from personal signatories to the root one
    let mut sigs = signatories.clone();
    let sigs_345 = sigs.split_off(signatories.keys().nth(3).unwrap());
    let sigs_12 = sigs.split_off(signatories.keys().nth(1).unwrap());
    let mut sigs_0 = sigs;

    let register_ms_accounts = |sigs_list: Vec<Vec<&AccountId>>| {
        sigs_list
            .into_iter()
            .map(|sigs| {
                let ms_account_id = gen_account_in(wonderland).0;
                let register_ms_account = MultisigRegister::new(
                    ms_account_id.clone(),
                    sigs.iter().copied().map(|id| (id.clone(), 1)).collect(),
                    sigs.len().try_into().unwrap(),
                    u64::MAX,
                );

                test_client
                    .submit_blocking(register_ms_account)
                    .expect("multisig account should be registered by account of the same domain");

                ms_account_id
            })
            .collect::<Vec<AccountId>>()
    };

    let sigs_list: Vec<Vec<&AccountId>> = [&sigs_12, &sigs_345]
        .into_iter()
        .map(|sigs| sigs.keys().collect())
        .collect();
    let msas = register_ms_accounts(sigs_list);
    let msa_12 = msas[0].clone();
    let msa_345 = msas[1].clone();

    let sigs_list = vec![vec![&msa_12, &msa_345]];
    let msas = register_ms_accounts(sigs_list);
    let msa_12345 = msas[0].clone();

    let sig_0 = sigs_0.keys().next().unwrap().clone();
    let sigs_list = vec![vec![&sig_0, &msa_12345]];
    let msas = register_ms_accounts(sigs_list);
    // The root multisig account with 6 personal signatories under its umbrella
    let msa_012345 = msas[0].clone();

    // One of personal signatories proposes a multisig transaction
    let key: Name = "key".parse().unwrap();
    let instructions = vec![SetKeyValue::account(
        msa_012345.clone(),
        key.clone(),
        "value".parse::<Json>().unwrap(),
    )
    .into()];
    let instructions_hash = HashOf::new(&instructions);

    let proposer = sigs_0.pop_last().unwrap();
    let propose = MultisigPropose::new(msa_012345.clone(), instructions);

    alt_client(proposer, &test_client).submit_blocking(propose)?;

    // Ticks as many times as the multisig recursion
    (0..2).for_each(|_| {
        test_client
            .submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))
            .unwrap();
    });

    // Check that the entire authentication policy has been deployed down to one of the leaf signatories
    let approval_hash_to_12345 = {
        let approval_hash_to_012345 = {
            let approve: InstructionBox =
                MultisigApprove::new(msa_012345.clone(), instructions_hash).into();

            HashOf::new(&vec![approve])
        };
        let approve: InstructionBox =
            MultisigApprove::new(msa_12345.clone(), approval_hash_to_012345).into();

        HashOf::new(&vec![approve])
    };

    let approvals_at_12: BTreeSet<AccountId> = test_client
        .query_single(FindAccountMetadata::new(
            msa_12.clone(),
            approvals_key(&approval_hash_to_12345),
        ))
        .expect("leaf approvals should be initialized by the root proposal")
        .try_into_any()
        .unwrap();

    assert!(1 == approvals_at_12.len() && approvals_at_12.contains(&msa_12345));

    // Check that the multisig transaction has not yet executed
    let _err = test_client
        .query_single(FindAccountMetadata::new(msa_012345.clone(), key.clone()))
        .expect_err("instructions shouldn't execute without enough approvals");

    // All the rest signatories approve the multisig transaction
    let approve_for_each = |approvers: BTreeMap<AccountId, KeyPair>,
                            instructions_hash: HashOf<Vec<InstructionBox>>,
                            ms_account: &AccountId| {
        for approver in approvers {
            let approve = MultisigApprove::new(ms_account.clone(), instructions_hash);

            alt_client(approver, &test_client)
                .submit_blocking(approve)
                .expect("should successfully approve the proposal");
        }
    };

    approve_for_each(sigs_12, approval_hash_to_12345, &msa_12);
    approve_for_each(sigs_345, approval_hash_to_12345, &msa_345);

    // Let the intermediate registry (12345) collect approvals and approve the original proposal
    // SATO
    // test_client.submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))?;

    // Let the root registry (012345) collect approvals and execute the original proposal
    // SATO
    // test_client.submit_blocking(Log::new(Level::DEBUG, "Just ticking time".to_string()))?;

    // Check that the multisig transaction has executed
    test_client
        .query_single(FindAccountMetadata::new(msa_012345.clone(), key.clone()))
        .expect("instructions should execute with enough approvals");

    Ok(())
}

#[test]
fn reserved_names() {
    let (network, _rt) = NetworkBuilder::new().start_blocking().unwrap();
    let test_client = network.client();

    let account_in_another_domain = gen_account_in("garden_of_live_flowers").0;

    {
        let register = {
            let role = multisig_role_for(&account_in_another_domain);
            Register::role(Role::new(role, ALICE_ID.clone()))
        };
        let _err = test_client.submit_blocking(register).expect_err(
            "role with this name shouldn't be registered by anyone other than domain owner",
        );
    }
}

fn alt_client(signatory: (AccountId, KeyPair), base_client: &Client) -> Client {
    Client {
        account: signatory.0,
        key_pair: signatory.1,
        ..base_client.clone()
    }
}

#[expect(dead_code)]
fn debug_account(account_id: &AccountId, client: &Client) {
    let account = client
        .query(FindAccounts)
        .filter_with(|account| account.id.eq(account_id.clone()))
        .execute_single()
        .unwrap();

    iroha_logger::error!(?account);
}
