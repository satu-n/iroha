//! Trigger given per domain to control multi-signature accounts and corresponding triggers

#![no_std]

extern crate alloc;
#[cfg(not(test))]
extern crate panic_halt;

use alloc::format;

use dlmalloc::GlobalDlmalloc;
use executor_custom_data_model::multisig::MultisigAccountArgs;
use iroha_executor_data_model::permission::trigger::CanExecuteTrigger;
use iroha_trigger::{
    debug::{dbg_panic, DebugExpectExt as _},
    prelude::*,
};

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

getrandom::register_custom_getrandom!(iroha_trigger::stub_getrandom);

// Binary containing common logic to each multisig account for handling multisig transactions
const WASM: &[u8] = core::include_bytes!(concat!(
    core::env!("OUT_DIR"),
    "/multisig_transactions.wasm"
));

#[iroha_trigger::main]
fn main(host: Iroha, context: Context) {
    let EventBox::ExecuteTrigger(event) = context.event else {
        dbg_panic("Only work as by call trigger");
    };
    let args: MultisigAccountArgs = event
        .args()
        .try_into_any()
        .dbg_expect("failed to parse args");
    let account_id = args.account.id().clone();

    host.submit(&Register::account(args.account))
        .dbg_expect("failed to register multisig account");

    let multisig_transactions_registry_id: TriggerId = format!(
        "multisig_transactions_{}_{}",
        account_id.signatory(),
        account_id.domain()
    )
    .parse()
    .dbg_expect("failed to parse trigger id");
    let multisig_transactions_registry = Trigger::new(
        multisig_transactions_registry_id.clone(),
        Action::new(
            WasmSmartContract::from_compiled(WASM.to_vec()),
            Repeats::Indefinitely,
            account_id.clone(),
            ExecuteTriggerEventFilter::new().for_trigger(multisig_transactions_registry_id.clone()),
        ),
    );

    host.submit(&Register::trigger(multisig_transactions_registry))
        .dbg_expect("failed to register multisig transactions registry");

    let role_id: RoleId = format!(
        "multisig_signatory_{}_{}",
        account_id.signatory(),
        account_id.domain()
    )
    .parse()
    .dbg_expect("failed to parse role");
    let can_execute_multisig_transactions_registry = CanExecuteTrigger {
        trigger: multisig_transactions_registry_id.clone(),
    };

    host.submit(&Register::role(
        // Temporarily grant a multisig role to the trigger authority to propagate the role to the signatories
        Role::new(role_id.clone(), context.authority.clone())
            .add_permission(can_execute_multisig_transactions_registry),
    ))
    .dbg_expect("failed to register multisig role");

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id,
        "signatories".parse().unwrap(),
        Json::new(&args.signatories),
    ))
    .dbg_unwrap();

    for signatory in args.signatories {
        host.submit(&Grant::account_role(role_id.clone(), signatory))
            .dbg_expect("failed to grant multisig role to account");
    }

    host.submit(&Revoke::account_role(role_id.clone(), context.authority))
        .dbg_expect("failed to revoke multisig role from owner");
}
