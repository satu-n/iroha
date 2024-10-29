//! Trigger given per multi-signature account to control multi-signature transactions

#![no_std]

extern crate alloc;
#[cfg(not(test))]
extern crate panic_halt;

use alloc::{collections::btree_set::BTreeSet, format, vec::Vec};

use dlmalloc::GlobalDlmalloc;
use executor_custom_data_model::multisig::MultisigTransactionArgs;
use iroha_trigger::{
    debug::{dbg_panic, DebugExpectExt as _},
    prelude::*,
};

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

getrandom::register_custom_getrandom!(iroha_trigger::stub_getrandom);

#[iroha_trigger::main]
fn main(host: Iroha, context: Context) {
    let trigger_id = context.id;
    let EventBox::ExecuteTrigger(event) = context.event else {
        dbg_panic("only work as by call trigger");
    };

    let args: MultisigTransactionArgs = event
        .args()
        .try_into_any()
        .dbg_expect("failed to parse arguments");
    let signatory = event.authority().clone();
    let instructions_hash = match &args {
        MultisigTransactionArgs::Propose(instructions) => HashOf::new(instructions),
        MultisigTransactionArgs::Approve(instructions_hash) => *instructions_hash,
    };
    let approvals_metadata_key: Name = format!("{instructions_hash}/approvals").parse().unwrap();
    let instructions_metadata_key: Name =
        format!("{instructions_hash}/instructions").parse().unwrap();

    let (approvals, instructions) = match args {
        MultisigTransactionArgs::Propose(instructions) => {
            host.query_single(FindTriggerMetadata::new(
                trigger_id.clone(),
                approvals_metadata_key.clone(),
            ))
            .expect_err("instructions are already submitted");

            let approvals = BTreeSet::from([signatory.clone()]);

            host.submit(&SetKeyValue::trigger(
                trigger_id.clone(),
                instructions_metadata_key.clone(),
                Json::new(&instructions),
            ))
            .dbg_unwrap();

            host.submit(&SetKeyValue::trigger(
                trigger_id.clone(),
                approvals_metadata_key.clone(),
                Json::new(&approvals),
            ))
            .dbg_unwrap();

            (approvals, instructions)
        }
        MultisigTransactionArgs::Approve(_instructions_hash) => {
            let mut approvals: BTreeSet<AccountId> = host
                .query_single(FindTriggerMetadata::new(
                    trigger_id.clone(),
                    approvals_metadata_key.clone(),
                ))
                .dbg_expect("instructions should be submitted first")
                .try_into_any()
                .dbg_unwrap();

            approvals.insert(signatory.clone());

            host.submit(&SetKeyValue::trigger(
                trigger_id.clone(),
                approvals_metadata_key.clone(),
                Json::new(&approvals),
            ))
            .dbg_unwrap();

            let instructions: Vec<InstructionBox> = host
                .query_single(FindTriggerMetadata::new(
                    trigger_id.clone(),
                    instructions_metadata_key.clone(),
                ))
                .dbg_unwrap()
                .try_into_any()
                .dbg_unwrap();

            (approvals, instructions)
        }
    };

    let signatories: BTreeSet<AccountId> = host
        .query_single(FindTriggerMetadata::new(
            trigger_id.clone(),
            "signatories".parse().unwrap(),
        ))
        .dbg_unwrap()
        .try_into_any()
        .dbg_unwrap();

    // Require N of N signatures
    if approvals.is_superset(&signatories) {
        // Cleanup approvals and instructions
        host.submit(&RemoveKeyValue::trigger(
            trigger_id.clone(),
            approvals_metadata_key,
        ))
        .dbg_unwrap();
        host.submit(&RemoveKeyValue::trigger(
            trigger_id.clone(),
            instructions_metadata_key,
        ))
        .dbg_unwrap();

        // Execute instructions proposal which collected enough approvals
        for isi in &instructions {
            host.submit(isi).dbg_unwrap();
        }
    }
}
