//! Arguments attached on executing triggers for multisig accounts or transactions

use alloc::{collections::btree_set::BTreeSet, vec::Vec};

use iroha_data_model::{account::NewAccount, prelude::*};
use serde::{Deserialize, Serialize};

/// Arguments to register multisig account
#[derive(Serialize, Deserialize)]
pub struct MultisigAccountArgs {
    /// Multisig account to be registered
    /// WARNING: any corresponding private key allows the owner to manipulate this account as a ordinary personal account
    pub account: NewAccount,
    /// List of accounts responsible for handling multisig account
    pub signatories: BTreeSet<AccountId>,
    /// Multisig transaction time-to-live based on block timestamps. Defaults to [`DEFAULT_MULTISIG_TTL_SECS`]
    pub transaction_ttl_secs: Option<u32>,
}

// Default multisig transaction time-to-live based on block timestamps
pub const DEFAULT_MULTISIG_TTL_SECS: u32 = 60 * 60; // 1 hour

/// Arguments to propose or approve multisig transaction
#[derive(Serialize, Deserialize)]
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
