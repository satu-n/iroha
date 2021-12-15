//! Pipeline events.

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

use iroha_crypto::{Hash, SignatureVerificationFail};
use iroha_macro::FromVariant;
use iroha_schema::prelude::*;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{isi::Instruction, transaction::Payload};

/// Event filter.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Copy, Clone, IntoSchema)]
pub struct EventFilter {
    /// Filter by Entity if `Some`, if `None` all entities are accepted.
    pub entity: Option<EntityType>,
    /// Filter by Hash if `Some`, if `None` all hashes are accepted.
    pub hash: Option<Hash>,
}

impl EventFilter {
    /// Do not filter at all.
    pub const fn identity() -> EventFilter {
        EventFilter {
            entity: None,
            hash: None,
        }
    }

    /// Filter by entity.
    pub const fn by_entity(entity: EntityType) -> EventFilter {
        EventFilter {
            entity: Some(entity),
            hash: None,
        }
    }

    /// Filter by hash.
    pub const fn by_hash(hash: Hash) -> EventFilter {
        EventFilter {
            hash: Some(hash),
            entity: None,
        }
    }

    /// Filter by entity and hash.
    pub const fn by_entity_and_hash(entity: EntityType, hash: Hash) -> EventFilter {
        EventFilter {
            entity: Some(entity),
            hash: Some(hash),
        }
    }

    /// Apply filter to event.
    pub fn apply(&self, event: &Event) -> bool {
        let entity_check = self
            .entity
            .map_or(true, |entity| entity == event.entity_type);
        let hash_check = self.hash.map_or(true, |hash| hash == event.hash);
        entity_check && hash_check
    }
}

/// Entity type to filter events.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, IntoSchema)]
pub enum EntityType {
    /// Block.
    Block,
    /// Transaction.
    Transaction,
}

/// Transaction was reject because it doesn't satisfy signature condition
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Decode, Encode, IntoSchema)]
pub struct UnsatisfiedSignatureConditionFail {
    /// Reason why signature condition failed
    pub reason: String,
}

impl Display for UnsatisfiedSignatureConditionFail {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Failed to verify signature condition specified in the account: {}",
            self.reason,
        )
    }
}

impl StdError for UnsatisfiedSignatureConditionFail {}

/// Transaction was rejected because of one of its instructions failing.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Decode, Encode, IntoSchema)]
pub struct InstructionExecutionFail {
    /// Instruction which execution failed
    pub instruction: Instruction,
    /// Error which happened during execution
    pub reason: String,
}

impl Display for InstructionExecutionFail {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Instruction::*;
        let type_ = match self.instruction {
            Burn(_) => "burn",
            Fail(_) => "fail",
            If(_) => "if",
            Mint(_) => "mint",
            Pair(_) => "pair",
            Register(_) => "register",
            Sequence(_) => "sequence",
            Transfer(_) => "transfer",
            Unregister(_) => "unregister",
            SetKeyValue(_) => "set key-value pair",
            RemoveKeyValue(_) => "remove key-value pair",
            Grant(_) => "grant",
        };
        write!(
            f,
            "Failed to execute instruction of type {}: {}",
            type_, self.reason
        )
    }
}
impl StdError for InstructionExecutionFail {}

/// Transaction was reject because of low authority
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Decode, Encode, IntoSchema)]
pub struct NotPermittedFail {
    /// Reason of failure
    pub reason: String,
}

impl Display for NotPermittedFail {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Action not permitted: {}", self.reason)
    }
}

impl StdError for NotPermittedFail {}

/// The reason for rejecting transaction which happened because of new blocks.
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Decode,
    Encode,
    FromVariant,
    Error,
    IntoSchema,
)]
pub enum BlockRejectionReason {
    /// Block was rejected during consensus.
    //TODO: store rejection reasons for blocks?
    #[error("Block was rejected during consensus.")]
    ConsensusBlockRejection,
}

/// The reason for rejecting transaction which happened because of transaction.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Decode,
    Encode,
    FromVariant,
    Error,
    IntoSchema,
)]
pub enum TransactionRejectionReason {
    /// Insufficient authorisation.
    #[error("Transaction rejected due to insufficient authorisation")]
    NotPermitted(#[source] NotPermittedFail),
    /// Failed to verify signature condition specified in the account.
    #[error("Transaction rejected due to an unsatisfied signature condition")]
    UnsatisfiedSignatureCondition(#[source] UnsatisfiedSignatureConditionFail),
    /// Failed to execute instruction.
    #[error("Transaction rejected due to failure in instruction execution")]
    InstructionExecution(#[source] InstructionExecutionFail),
    /// Failed to verify signatures.
    #[error("Transaction rejected due to failed signature verification")]
    SignatureVerification(#[source] SignatureVerificationFail<Payload>),
    /// Genesis account can sign only transactions in the genesis block.
    #[error("The genesis account can only sign transactions in the genesis block.")]
    UnexpectedGenesisAccountSignature,
}

/// The reason for rejecting pipeline entity such as transaction or block.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Decode,
    Encode,
    FromVariant,
    Error,
    IntoSchema,
)]
pub enum RejectionReason {
    /// The reason for rejecting the block.
    #[error("Block was rejected")]
    Block(#[source] BlockRejectionReason),
    /// The reason for rejecting transaction.
    #[error("Transaction was rejected")]
    Transaction(#[source] TransactionRejectionReason),
}

/// Entity type to filter events.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Eq, PartialEq, Clone, IntoSchema)]
pub struct Event {
    /// Type of entity that caused this event.
    pub entity_type: EntityType,
    /// The status of this entity.
    pub status: Status,
    /// The hash of this entity.
    pub hash: Hash,
}

impl Event {
    /// Constructs pipeline event.
    pub const fn new(entity_type: EntityType, status: Status, hash: Hash) -> Self {
        Event {
            entity_type,
            status,
            hash,
        }
    }
}

/// Entity type to filter events.
#[derive(
    Debug, Decode, Encode, Deserialize, Serialize, Eq, PartialEq, Clone, FromVariant, IntoSchema,
)]
pub enum Status {
    /// Entity has been seen in blockchain, but has not passed validation.
    Validating,
    /// Entity was rejected in one of the validation stages.
    Rejected(RejectionReason),
    /// Entity has passed validation.
    Committed,
}

/// Exports common structs and enums from this module.
pub mod prelude {
    pub use super::{
        BlockRejectionReason, EntityType as PipelineEntityType, Event as PipelineEvent,
        EventFilter as PipelineEventFilter, InstructionExecutionFail, NotPermittedFail,
        RejectionReason as PipelineRejectionReason, Status as PipelineStatus,
        TransactionRejectionReason, UnsatisfiedSignatureConditionFail,
    };
}

#[cfg(test)]
mod tests {
    #![allow(clippy::restriction)]

    use RejectionReason::*;
    use TransactionRejectionReason::*;

    use super::*;

    #[test]
    fn events_are_correctly_filtered() {
        let events = vec![
            Event {
                entity_type: EntityType::Transaction,
                status: Status::Validating,
                hash: Hash([0_u8; 32]),
            },
            Event {
                entity_type: EntityType::Transaction,
                status: Status::Rejected(Transaction(NotPermitted(NotPermittedFail {
                    reason: "Some reason".to_string(),
                }))),
                hash: Hash([0_u8; 32]),
            },
            Event {
                entity_type: EntityType::Transaction,
                status: Status::Committed,
                hash: Hash([2_u8; 32]),
            },
            Event {
                entity_type: EntityType::Block,
                status: Status::Committed,
                hash: Hash([2_u8; 32]),
            },
        ];
        assert_eq!(
            vec![
                Event {
                    entity_type: EntityType::Transaction,
                    status: Status::Validating,
                    hash: Hash([0_u8; 32]),
                },
                Event {
                    entity_type: EntityType::Transaction,
                    status: Status::Rejected(Transaction(NotPermitted(NotPermittedFail {
                        reason: "Some reason".to_string(),
                    }))),
                    hash: Hash([0_u8; 32]),
                },
            ],
            events
                .iter()
                .cloned()
                .filter(|event| EventFilter::by_hash(Hash([0_u8; 32])).apply(event))
                .collect::<Vec<Event>>()
        );
        assert_eq!(
            vec![Event {
                entity_type: EntityType::Block,
                status: Status::Committed,
                hash: Hash([2_u8; 32]),
            }],
            events
                .iter()
                .cloned()
                .filter(|event| EventFilter::by_entity(EntityType::Block).apply(event))
                .collect::<Vec<Event>>()
        );
        assert_eq!(
            vec![Event {
                entity_type: EntityType::Transaction,
                status: Status::Committed,
                hash: Hash([2_u8; 32]),
            }],
            events
                .iter()
                .cloned()
                .filter(|event| EventFilter::by_entity_and_hash(
                    EntityType::Transaction,
                    Hash([2_u8; 32])
                )
                .apply(event))
                .collect::<Vec<Event>>()
        );
        assert_eq!(
            events,
            events
                .iter()
                .cloned()
                .filter(|event| EventFilter::identity().apply(event))
                .collect::<Vec<Event>>()
        )
    }
}
