//! Trigger given per domain to control multi-signature accounts and corresponding triggers

#![no_std]

extern crate alloc;
#[cfg(not(test))]
extern crate panic_halt;

use alloc::format;

use dlmalloc::GlobalDlmalloc;
use iroha_executor_data_model::permission::trigger::CanExecuteTrigger;
use iroha_multisig_data_model::MultisigAccountArgs;
use iroha_trigger::{
    debug::{dbg_panic, DebugExpectExt as _},
    prelude::*,
};

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

getrandom::register_custom_getrandom!(iroha_trigger::stub_getrandom);

// Binary containing common logic to each multisig account for handling multisig transactions
const MULTISIG_TRANSACTIONS_WASM: &[u8] = core::include_bytes!(concat!(
    core::env!("CARGO_MANIFEST_DIR"),
    "/../../target/prebuilt/libs/multisig_transactions.wasm"
));

#[iroha_trigger::main]
fn main(host: Iroha, context: Context) {
    let EventBox::ExecuteTrigger(event) = context.event else {
        dbg_panic("trigger misused: must be triggered only by a call");
    };
    let args: MultisigAccountArgs = event
        .args()
        .try_into_any()
        .dbg_expect("args should be for a multisig account");
    let domain_id = context
        .id
        .name()
        .as_ref()
        .strip_prefix("multisig_accounts_")
        .and_then(|s| s.parse::<DomainId>().ok())
        .dbg_unwrap();
    let account_id = AccountId::new(domain_id, args.account);

    // SATO wip

    let account_hash = match &args {
        MultisigAccountArgs::Propose(proposal) => proposal.account.id().signatory().clone(),
        MultisigAccountArgs::Approve(public_key) => public_key,
    };
    let instructions_metadata_key: Name = format!("proposals/{account_hash}/instructions")
        .parse()
        .unwrap();
    let proposed_at_ms_metadata_key: Name = format!("proposals/{account_hash}/proposed_at_ms")
        .parse()
        .unwrap();
    let approvals_metadata_key: Name = format!("proposals/{account_hash}/approvals")
        .parse()
        .unwrap();

    let signatories: BTreeMap<AccountId, u8> = query_single(FindTriggerMetadata::new(
        id.clone(),
        "signatories".parse().unwrap(),
    ))
    .dbg_unwrap()
    .try_into_any()
    .dbg_unwrap();

    // Recursively deploy multisig authentication down to the terminal personal signatories
    for account_id in signatories.keys() {
        let sub_transactions_registry_id: TriggerId = format!(
            "multisig_transactions_{}_{}",
            account_id.signatory(),
            account_id.domain()
        )
        .parse()
        .unwrap();

        if let Ok(_sub_registry) = query(FindTriggers::new())
            .filter_with(|trigger| trigger.id.eq(sub_transactions_registry_id.clone()))
            .execute_single()
        {
            let propose_to_approve_me: InstructionBox = {
                let approve_me: InstructionBox = {
                    let args = MultisigTransactionArgs::Approve(account_hash);
                    ExecuteTrigger::new(id.clone()).with_args(&args).into()
                };
                let args = MultisigTransactionArgs::Propose([approve_me].to_vec());

                ExecuteTrigger::new(sub_transactions_registry_id.clone())
                    .with_args(&args)
                    .into()
            };
            propose_to_approve_me
                .execute()
                .dbg_expect("should successfully write to sub registry");
        }
    }

    let mut block_headers = query(FindBlockHeaders).execute().dbg_unwrap();
    let now_ms: u64 = block_headers
        .next()
        .dbg_unwrap()
        .dbg_unwrap()
        .creation_time()
        .as_millis()
        .try_into()
        .dbg_unwrap();

    let (approvals, instructions) = match args {
        MultisigTransactionArgs::Propose(instructions) => {
            query_single(FindTriggerMetadata::new(
                id.clone(),
                approvals_metadata_key.clone(),
            ))
            .expect_err("instructions shouldn't already be proposed");

            let approvals = BTreeSet::from([signatory.clone()]);

            SetKeyValue::trigger(
                id.clone(),
                instructions_metadata_key.clone(),
                JsonString::new(&instructions),
            )
            .execute()
            .dbg_unwrap();

            SetKeyValue::trigger(
                id.clone(),
                proposed_at_ms_metadata_key.clone(),
                JsonString::new(&now_ms),
            )
            .execute()
            .dbg_unwrap();

            SetKeyValue::trigger(
                id.clone(),
                approvals_metadata_key.clone(),
                JsonString::new(&approvals),
            )
            .execute()
            .dbg_unwrap();

            (approvals, instructions)
        }
        MultisigTransactionArgs::Approve(account_hash) => {
            let mut approvals: BTreeSet<AccountId> = query_single(FindTriggerMetadata::new(
                id.clone(),
                approvals_metadata_key.clone(),
            ))
            .dbg_expect("instructions should be proposed first")
            .try_into_any()
            .dbg_unwrap();

            approvals.insert(signatory.clone());

            SetKeyValue::trigger(
                id.clone(),
                approvals_metadata_key.clone(),
                JsonString::new(&approvals),
            )
            .execute()
            .dbg_unwrap();

            let instructions: Vec<InstructionBox> = query_single(FindTriggerMetadata::new(
                id.clone(),
                instructions_metadata_key.clone(),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();

            (approvals, instructions)
        }
    };

    let quorum: u16 = query_single(FindTriggerMetadata::new(
        id.clone(),
        "quorum".parse().unwrap(),
    ))
    .dbg_unwrap()
    .try_into_any()
    .dbg_unwrap();

    let is_authenticated = quorum
        <= signatories
            .into_iter()
            .filter(|(id, _)| approvals.contains(&id))
            .map(|(_, weight)| weight as u16)
            .sum();

    let is_expired = {
        let proposed_at_ms: u64 = query_single(FindTriggerMetadata::new(
            id.clone(),
            proposed_at_ms_metadata_key.clone(),
        ))
        .dbg_unwrap()
        .try_into_any()
        .dbg_unwrap();

        let transaction_ttl_secs: u32 = query_single(FindTriggerMetadata::new(
            id.clone(),
            "transaction_ttl_secs".parse().unwrap(),
        ))
        .dbg_unwrap()
        .try_into_any()
        .dbg_unwrap();

        proposed_at_ms + transaction_ttl_secs as u64 * 1_000 < now_ms
    };

    if is_authenticated || is_expired {
        // Cleanup approvals and instructions
        RemoveKeyValue::trigger(id.clone(), approvals_metadata_key)
            .execute()
            .dbg_unwrap();
        RemoveKeyValue::trigger(id.clone(), proposed_at_ms_metadata_key)
            .execute()
            .dbg_unwrap();
        RemoveKeyValue::trigger(id.clone(), instructions_metadata_key)
            .execute()
            .dbg_unwrap();

        if !is_expired {
            // Execute instructions proposal which collected enough approvals
            for isi in instructions {
                isi.execute().dbg_unwrap();
            }
        }
    }

    // SATO wip

    let account_id = args.account.id().clone();

    Register::account(args.account)
        .execute()
        .dbg_expect("accounts registry should successfully register a multisig account");

    let multisig_transactions_registry_id: TriggerId = format!(
        "multisig_transactions_{}_{}",
        account_id.signatory(),
        account_id.domain()
    )
    .parse()
    .dbg_unwrap();

    let multisig_transactions_registry = Trigger::new(
        multisig_transactions_registry_id.clone(),
        Action::new(
            WasmSmartContract::from_compiled(MULTISIG_TRANSACTIONS_WASM.to_vec()),
            Repeats::Indefinitely,
            account_id.clone(),
            ExecuteTriggerEventFilter::new().for_trigger(multisig_transactions_registry_id.clone()),
        ),
    );

    host.submit(&Register::trigger(multisig_transactions_registry))
        .dbg_expect("accounts registry should successfully register a transactions registry");

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id.clone(),
        "signatories".parse().unwrap(),
        Json::new(&args.signatories),
    ))
    .dbg_unwrap();

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id.clone(),
        "quorum".parse().unwrap(),
        Json::new(&args.quorum),
    ))
    .dbg_unwrap();

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id.clone(),
        "transaction_ttl_ms".parse().unwrap(),
        Json::new(&args.transaction_ttl_ms),
    ))
    .dbg_unwrap();

    let role_id: RoleId = format!(
        "multisig_signatory_{}_{}",
        account_id.signatory(),
        account_id.domain()
    )
    .parse()
    .dbg_unwrap();

    host.submit(&Register::role(
        // Temporarily grant a multisig role to the trigger authority to delegate the role to the signatories
        Role::new(role_id.clone(), context.authority.clone()),
    ))
    .dbg_expect("accounts registry should successfully register a multisig role");

    for signatory in args.signatories.keys().cloned() {
        let is_multisig_again = {
            let sub_role_id: RoleId = format!(
                "multisig_signatory_{}_{}",
                signatory.signatory(),
                signatory.domain()
            )
            .parse()
            .dbg_unwrap();

            host.query(FindRoleIds)
                .filter_with(|role_id| role_id.eq(sub_role_id))
                .execute_single_opt()
                .dbg_unwrap()
                .is_some()
        };

        if is_multisig_again {
            // Allow the transactions registry to write to the sub registry
            let sub_registry_id: TriggerId = format!(
                "multisig_transactions_{}_{}",
                signatory.signatory(),
                signatory.domain()
            )
            .parse()
            .dbg_unwrap();

            host.submit(&Grant::account_permission(
                CanExecuteTrigger {
                    trigger: sub_registry_id,
                },
                account_id.clone(),
            ))
            .dbg_expect(
                "accounts registry should successfully grant permission to the multisig account",
            );
        }

        host.submit(&Grant::account_role(role_id.clone(), signatory))
            .dbg_expect(
                "accounts registry should successfully grant the multisig role to signatories",
            );
    }

    host.submit(&Revoke::account_role(role_id.clone(), context.authority))
        .dbg_expect(
        "accounts registry should successfully revoke the multisig role from the trigger authority",
    );
}
