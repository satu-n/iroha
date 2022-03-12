//! Pipeline events.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

use iroha_crypto::Hash;
use iroha_macro::FromVariant;
use iroha_schema::prelude::IntoSchema;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub use crate::transaction::RejectionReason as PipelineRejectionReason;

/// [`Event`] filter.
#[derive(
    Default,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Decode,
    Encode,
    IntoSchema,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct EventFilter {
    /// If `Some::<EntityType>` filters by the [`EntityType`]. If `None` accepts all the [`EntityType`].
    pub entity_type: Option<EntityType>,
    /// If `Some::<StatusType>` filters by the [`StatusType`]. If `None` accepts all the [`StatusType`].
    pub status_type: Option<StatusType>,
    /// If `Some::<Hash>` filters by the [`Hash`]. If `None` accepts all the [`Hash`].
    pub hash: Option<Hash>,
}

impl EventFilter {
    /// Construct [`EventFilter`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by [`EntityType`].
    pub const fn entity_type(mut self, entity_type: EntityType) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Filter by [`StatusType`].
    pub const fn status_type(mut self, status_type: StatusType) -> Self {
        self.status_type = Some(status_type);
        self
    }

    /// Filter by [`Hash`].
    pub const fn hash(mut self, hash: Hash) -> Self {
        self.hash = Some(hash);
        self
    }

    /// Check if `self` accepts the `event`.
    pub fn matches(&self, event: &Event) -> bool {
        [
            Self::field_matches(&self.entity_type, &event.entity_type),
            Self::field_matches(&self.status_type, &event.status.type_()),
            Self::field_matches(&self.hash, &event.hash),
        ]
        .into_iter()
        .all(core::convert::identity)
    }

    fn field_matches<T: Eq>(filter: &Option<T>, event: &T) -> bool {
        filter.as_ref().map_or(true, |field| field == event)
    }
}

/// Type of the pipeline entity.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Decode,
    Encode,
    IntoSchema,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum EntityType {
    /// Block.
    Block,
    /// Transaction.
    Transaction,
}

/// [`Event`].
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode, IntoSchema)]
pub struct Event {
    /// [`EntityType`] of the entity that caused this [`Event`].
    pub entity_type: EntityType,
    /// [`Status`] of the entity that caused this [`Event`].
    pub status: Status,
    /// [`Hash`] of the entity that caused this [`Event`].
    pub hash: Hash,
}

impl Event {
    /// Construct [`Event`].
    pub const fn new(entity_type: EntityType, status: Status, hash: Hash) -> Self {
        Event {
            entity_type,
            status,
            hash,
        }
    }
}

/// [`Status`] of the entity.
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode, FromVariant, IntoSchema)]
pub enum Status {
    /// Entity has been seen in blockchain, but has not passed validation.
    Validating,
    /// Entity was rejected in one of the validation stages.
    Rejected(PipelineRejectionReason),
    /// Entity has passed validation.
    Committed,
}

/// Type of [`Status`].
#[derive(
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Decode,
    Encode,
    IntoSchema,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum StatusType {
    /// Represents [`Status::Validating`].
    Validating,
    /// Represents [`Status::Rejected`].
    Rejected,
    /// Represents [`Status::Committed`].
    Committed,
}

impl Status {
    fn type_(&self) -> StatusType {
        use Status::*;
        match self {
            Validating => StatusType::Validating,
            Rejected(_) => StatusType::Rejected,
            Committed => StatusType::Committed,
        }
    }
}

/// Exports common structs and enums from this module.
pub mod prelude {
    pub use super::{
        EntityType as PipelineEntityType, Event as PipelineEvent,
        EventFilter as PipelineEventFilter, PipelineRejectionReason, Status as PipelineStatus,
    };
}

#[cfg(test)]
mod tests {
    #![allow(clippy::restriction)]

    #[cfg(not(feature = "std"))]
    use alloc::{string::ToString as _, vec, vec::Vec};

    use super::*;
    use crate::transaction::{NotPermittedFail, RejectionReason::*, TransactionRejectionReason::*};

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
                .filter(|event| EventFilter::new().hash(Hash([0_u8; 32])).matches(event))
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
                .filter(|event| EventFilter::new()
                    .entity_type(EntityType::Block)
                    .matches(event))
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
                .filter(|event| EventFilter::new()
                    .entity_type(EntityType::Transaction)
                    .hash(Hash([2_u8; 32]))
                    .matches(event))
                .collect::<Vec<Event>>()
        );
        assert_eq!(
            events,
            events
                .iter()
                .cloned()
                .filter(|event| EventFilter::new().matches(event))
                .collect::<Vec<Event>>()
        )
    }
}
