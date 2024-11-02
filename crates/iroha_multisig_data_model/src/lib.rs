// SATO crate description
//! Arguments attached on executing triggers for multisig accounts or transactions

#![no_std]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};

pub use constants::*;
use derive_more::{Constructor, From};
use iroha_data_model::{
    isi::{CustomInstruction, Instruction, InstructionBox},
    prelude::Json,
};
use iroha_schema::IntoSchema;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
mod constants {
    use super::*;

    pub const SIGNATORIES: &str = "signatories";
    pub const QUORUM: &str = "quorum";
    pub const TRANSACTION_TTL_MS: &str = "transaction_ttl_ms";
    pub const PROPOSALS: &str = "proposals";

    pub const MULTISIG_SIGNATORY_: &str = "MULTISIG_SIGNATORY_";

    pub fn instructions_key(hash: &HashOf<Vec<InstructionBox>>) -> Name {
        format!("{PROPOSALS}/{hash}/instructions").parse().unwrap()
    }
    pub fn proposed_at_ms_key(hash: &HashOf<Vec<InstructionBox>>) -> Name {
        format!("{PROPOSALS}/{hash}/proposed_at_ms")
            .parse()
            .unwrap()
    }
    pub fn approvals_key(hash: &HashOf<Vec<InstructionBox>>) -> Name {
        format!("{PROPOSALS}/{hash}/approvals").parse().unwrap()
    }
    pub fn multisig_role_for(account: &AccountId) -> RoleId {
        format!(
            "{MULTISIG_SIGNATORY_}{}_{}",
            account.signatory(),
            account.domain()
        )
        .parse()
        .unwrap()
    }
    pub fn multisig_account_from(role: &RoleId) -> Option<AccountId> {
        role.name()
            .as_ref()
            .strip_prefix(MULTISIG_SIGNATORY_)?
            .replacen('_', "@", 1)
            .parse()
            .ok()
    }
}

/// SATO doc
#[derive(Debug, Clone, Deserialize, Serialize, IntoSchema, From)]
pub enum MultisigInstructionBox {
    /// SATO
    Register(MultisigRegister),
    /// SATO
    Propose(MultisigPropose),
    /// SATO
    Approve(MultisigApprove),
}

/// SATO doc
#[derive(Debug, Clone, Deserialize, Serialize, IntoSchema, Constructor)]
pub struct MultisigRegister {
    /// Multisig account to be registered
    /// <div class="warning">
    ///
    /// Any corresponding private key allows the owner to manipulate this account as a ordinary personal account
    ///
    /// </div>
    // FIXME #5022 prevent multisig monopoly
    // FIXME #5022 stop accepting user input: otherwise, after #4426 pre-registration account will be hijacked as a multisig account
    pub account: AccountId,
    /// List of accounts and their relative weights of responsibility for the multisig
    pub signatories: BTreeMap<AccountId, Weight>,
    /// Threshold of total weight at which the multisig is considered authenticated
    pub quorum: u16,
    /// Multisig transaction time-to-live in milliseconds based on block timestamps. Defaults to [`DEFAULT_MULTISIG_TTL_MS`]
    pub transaction_ttl_ms: u64,
}

/// SATO doc
#[derive(Debug, Clone, Deserialize, Serialize, IntoSchema, Constructor)]
pub struct MultisigPropose {
    /// SATO doc
    pub account: AccountId,
    /// SATO doc
    pub instructions: Vec<InstructionBox>,
}
/// SATO doc
#[derive(Debug, Clone, Deserialize, Serialize, IntoSchema, Constructor)]
pub struct MultisigApprove {
    /// SATO doc
    pub account: AccountId,
    /// SATO doc
    pub instructions_hash: HashOf<Vec<InstructionBox>>,
}

macro_rules! impl_custom_instruction {
    ($box:ty, $($instruction:ty)|+) => {
        impl Instruction for $box {}

        impl From<$box> for InstructionBox {
            fn from(value: $box) -> Self {
                Self::Custom(value.into())
            }
        }

        impl From<$box> for CustomInstruction {
            fn from(value: $box) -> Self {
                let payload = serde_json::to_value(&value)
                    .expect(concat!("INTERNAL BUG: Couldn't serialize ", stringify!($box)));

                Self::new(payload)
            }
        }

        impl TryFrom<&Json> for $box {
            type Error = serde_json::Error;

            fn try_from(payload: &Json) -> serde_json::Result<Self> {
                serde_json::from_str::<Self>(payload.as_ref())
            }
        }

        $(
            impl Instruction for $instruction {}

            impl From<$instruction> for InstructionBox {
                fn from(value: $instruction) -> Self {
                    Self::Custom(<$box>::from(value).into())
                }
            }
        )+
    };
}

impl_custom_instruction!(
    MultisigInstructionBox,
    MultisigRegister | MultisigPropose | MultisigApprove
);

// SATO remove

use alloc::collections::btree_map::BTreeMap;

use iroha_data_model::prelude::*;
use parity_scale_codec::{Decode, Encode};

/// Arguments to register multisig account
#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize, IntoSchema)]
pub struct MultisigAccountArgs {
    /// Multisig account to be registered
    /// <div class="warning">
    ///
    /// Any corresponding private key allows the owner to manipulate this account as a ordinary personal account
    ///
    /// </div>
    // FIXME #5022 prevent multisig monopoly
    // FIXME #5022 stop accepting user input: otherwise, after #4426 pre-registration account will be hijacked as a multisig account
    pub account: PublicKey,
    /// List of accounts and their relative weights of responsibility for the multisig
    pub signatories: BTreeMap<AccountId, Weight>,
    /// Threshold of total weight at which the multisig is considered authenticated
    pub quorum: u16,
    /// Multisig transaction time-to-live in milliseconds based on block timestamps. Defaults to [`DEFAULT_MULTISIG_TTL_MS`]
    pub transaction_ttl_ms: u64,
}

type Weight = u8;

/// Default multisig transaction time-to-live in milliseconds based on block timestamps
pub const DEFAULT_MULTISIG_TTL_MS: u64 = 60 * 60 * 1_000; // 1 hour

/// Arguments to propose or approve multisig transaction
#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize, IntoSchema)]
pub enum MultisigTransactionArgs {
    /// Propose instructions and initialize approvals with the proposer's one
    Propose(Vec<InstructionBox>),
    /// Approve certain instructions
    Approve(HashOf<Vec<InstructionBox>>),
}

impl From<MultisigAccountArgs> for Json {
    fn from(details: MultisigAccountArgs) -> Self {
        Json::new(details)
    }
}

impl TryFrom<&Json> for MultisigAccountArgs {
    type Error = serde_json::Error;

    fn try_from(payload: &Json) -> serde_json::Result<Self> {
        serde_json::from_str::<Self>(payload.as_ref())
    }
}

impl From<MultisigTransactionArgs> for Json {
    fn from(details: MultisigTransactionArgs) -> Self {
        Json::new(details)
    }
}

impl TryFrom<&Json> for MultisigTransactionArgs {
    type Error = serde_json::Error;

    fn try_from(payload: &Json) -> serde_json::Result<Self> {
        serde_json::from_str::<Self>(payload.as_ref())
    }
}
