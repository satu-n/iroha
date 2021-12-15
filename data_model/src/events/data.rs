//! Data events.

use iroha_macro::FromVariant;
use iroha_schema::prelude::*;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Event.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, IntoSchema)]
pub struct Event {
    entity: Entity,
    status: Status,
}

/// Enumeration of all possible Iroha data entities.
#[derive(Clone, Debug, Decode, Encode, Deserialize, Serialize, FromVariant, IntoSchema)]
pub enum Entity {
    /// [`Account`].
    Account(Box<Account>),
    /// [`AssetDefinition`].
    AssetDefinition(AssetDefinition),
    /// [`Asset`].
    Asset(Asset),
    /// [`Domain`].
    Domain(Domain),
    /// [`Peer`].
    Peer(Peer),
}

/// Entity status.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, IntoSchema)]
pub enum Status {
    /// Entity was added, registered, minted or another action was made to make entity appear on
    /// the blockchain for the first time.
    Created,
    /// Entity's state was changed, any parameter updated it's value.
    Updated,
    /// Entity was archived or by any other way was put into state that guarantees absence of
    /// [`Updated`](`Status::Updated`) events for this entity.
    Deleted,
}

// SATO
// /// Entity type to filter events.
// #[derive(Debug, Decode, Encode, Deserialize, Serialize, Eq, PartialEq, Copy, Clone, IntoSchema)]
// pub enum EntityType {
//     /// [`Account`]s.
//     Account,
//     /// [`AssetDefinition`]s.
//     AssetDefinition,
//     /// [`Asset`]s.
//     Asset,
//     /// [`Domain`]s.
//     Domain,
//     /// [`Peer`]s.
//     Peer,
// }
//
// impl From<Entity> for EntityType {
//     fn from(entity: Entity) -> Self {
//         match entity {
//             Entity::Account(_) => EntityType::Account,
//             Entity::AssetDefinition(_) => EntityType::AssetDefinition,
//             Entity::Asset(_) => EntityType::Asset,
//             Entity::Domain(_) => EntityType::Domain,
//             Entity::Peer(_) => EntityType::Peer,
//         }
//     }
// }

/// Filter to select [`Event`] which matches the `entity` and `status` conditions
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, IntoSchema)]
pub struct EventFilter {
    /// Filter by [`Entity`].
    entity: Option<EntityFilter>,
    /// Filter by [`Status`]. [`None`] means accept all the [`Status`]es.
    status: Option<StatusFilter>,
}

/// Select entities under the entity of specified id, or all the entities of the entity type.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, IntoSchema)]
enum EntityFilter {
    /// Filter by [`Account`].
    Account(Option<AccountId>),
    /// Filter by [`AssetDefinition`].
    AssetDefinition(Option<AssetDefinitionId>),
    /// Filter by [`Asset`].
    Asset(Option<AssetId>),
    /// Filter by [`Domain`].
    Domain(Option<Name>),
    /// Filter by [`Peer`].
    Peer(Option<PeerId>),
}

type StatusFilter = Status;

impl Event {
    pub fn new(entity: Entity, status: Status) -> Self {
        Self { entity, status }
    }
}

impl EventFilter {
    /// Apply filter to `event`: check if `event` is accepted.
    pub fn apply(&self, event: &Event) -> bool {
        let entity_check = self
            .entity
            .as_ref()
            .map_or(true, |entity| entity.apply(&event.entity));
        let status_check = self
            .status
            .map_or(true, |status| status.apply(&event.status));
        entity_check && status_check
    }
}

impl EntityFilter {
    fn apply(&self, entity: &Entity) -> bool {
        match self {
            Self::Account(opt) => match entity {
                Entity::Account(account) => opt.as_ref().map_or(true, |id| account.id == *id),
                Entity::Asset(asset) => opt.as_ref().map_or(false, |id| asset.id.account_id == *id),
                _ => false,
            },
            Self::AssetDefinition(opt) => match entity {
                Entity::AssetDefinition(asset_definition) => {
                    opt.as_ref().map_or(true, |id| asset_definition.id == *id)
                }
                Entity::Asset(asset) => opt
                    .as_ref()
                    .map_or(false, |id| asset.id.definition_id == *id),
                _ => false,
            },
            Self::Asset(opt) => match entity {
                Entity::Asset(asset) => opt.as_ref().map_or(true, |id| asset.id == *id),
                _ => false,
            },
            Self::Domain(opt) => match entity {
                Entity::Account(account) => opt
                    .as_ref()
                    .map_or(false, |name| account.id.domain_name == *name),
                Entity::AssetDefinition(asset_definition) => opt
                    .as_ref()
                    .map_or(false, |name| asset_definition.id.domain_name == *name),
                Entity::Asset(asset) => opt.as_ref().map_or(false, |name| {
                    asset.id.account_id.domain_name == *name
                        || asset.id.definition_id.domain_name == *name
                }),
                Entity::Domain(domain) => opt.as_ref().map_or(true, |name| domain.name == *name),
                _ => false,
            },
            Self::Peer(opt) => match entity {
                Entity::Peer(peer) => opt.as_ref().map_or(true, |id| peer.id == *id),
                _ => false,
            },
        }
    }
}

impl StatusFilter {
    fn apply(&self, status: &Status) -> bool {
        self == status
    }
}

mod world {
    use crate::prelude::*;

    impl From<Register<Peer>> for DataEvent {
        fn from(src: Register<Peer>) -> Self {
            Self::new(DataEntity::Peer(src.object), DataStatus::Created)
        }
    }

    impl From<Unregister<Peer>> for DataEvent {
        fn from(src: Unregister<Peer>) -> Self {
            Self::new(DataEntity::Peer(src.object), DataStatus::Deleted)
        }
    }

    impl From<Register<Domain>> for DataEvent {
        fn from(src: Register<Domain>) -> Self {
            Self::new(DataEntity::Domain(src.object), DataStatus::Created)
        }
    }

    impl From<Unregister<Domain>> for DataEvent {
        fn from(src: Unregister<Domain>) -> Self {
            Self::new(DataEntity::Domain(src.object), DataStatus::Deleted)
        }
    }

    #[cfg(feature = "roles")]
    impl From<Register<Role>> for DataEvent {
        fn from(src: Register<Role>) -> Self {
            Self::new(DataEntity::Role(src.object), DataStatus::Created)
        }
    }

    #[cfg(feature = "roles")]
    impl From<Unregister<Role>> for DataEvent {
        fn from(src: Unregister<Role>) -> Self {
            Self::new(DataEntity::Role(src.object), DataStatus::Deleted)
        }
    }
}

mod domain {
    use crate::prelude::*;

    impl From<Register<NewAccount>> for DataEvent {
        fn from(src: Register<NewAccount>) -> Self {
            Self::new(DataEntity::NewAccount(src.object), DataStatus::Created)
        }
    }

    impl From<Unregister<Account>> for DataEvent {
        fn from(src: Unregister<Account>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Deleted)
        }
    }

    impl From<Register<AssetDefinition>> for DataEvent {
        fn from(src: Register<AssetDefinition>) -> Self {
            Self::new(DataEntity::AssetDefinition(src.object), DataStatus::Created)
        }
    }

    impl From<Unregister<AssetDefinition>> for DataEvent {
        fn from(src: Unregister<AssetDefinition>) -> Self {
            Self::new(DataEntity::AssetDefinition(src.object), DataStatus::Deleted)
        }
    }

    impl From<SetKeyValue<AssetDefinition, String, Value>> for DataEvent {
        fn from(src: SetKeyValue<AssetDefinition, String, Value>) -> Self {
            Self::new(
                DataEntity::AssetDefinition(src.object),
                // SATO Inserted
                DataStatus::Updated,
            )
        }
    }

    impl From<RemoveKeyValue<AssetDefinition, String>> for DataEvent {
        fn from(src: RemoveKeyValue<AssetDefinition, String>) -> Self {
            Self::new(
                DataEntity::AssetDefinition(src.object),
                // SATO Removed
                DataStatus::Updated,
            )
        }
    }

    impl From<SetKeyValue<Domain, String, Value>> for DataEvent {
        fn from(src: SetKeyValue<Domain, String, Value>) -> Self {
            Self::new(
                DataEntity::Domain(src.object),
                // SATO Inserted
                DataStatus::Updated,
            )
        }
    }

    impl From<RemoveKeyValue<Domain, String>> for DataEvent {
        fn from(src: RemoveKeyValue<Domain, String>) -> Self {
            Self::new(
                DataEntity::Domain(src.object),
                // SATO Removed
                DataStatus::Updated,
            )
        }
    }
}

mod account {
    use iroha_crypto::PublicKey;

    use crate::prelude::*;

    // SATO DataStatus::Updated(...)

    impl From<Mint<Account, PublicKey>> for DataEvent {
        fn from(src: Mint<Account, PublicKey>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }

    impl From<Mint<Account, SignatureCheckCondition>> for DataEvent {
        fn from(src: Mint<Account, SignatureCheckCondition>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }

    impl From<Burn<Account, PublicKey>> for DataEvent {
        fn from(src: Burn<Account, PublicKey>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }

    impl From<SetKeyValue<Account, String, Value>> for DataEvent {
        fn from(src: SetKeyValue<Account, String, Value>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }

    impl From<RemoveKeyValue<Account, String>> for DataEvent {
        fn from(src: RemoveKeyValue<Account, String>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }

    impl From<Grant<Account, PermissionToken>> for DataEvent {
        fn from(src: Grant<Account, PermissionToken>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }

    #[cfg(feature = "roles")]
    impl From<Grant<Account, RoleId>> for DataEvent {
        fn from(src: Grant<Account, RoleId>) -> Self {
            Self::new(DataEntity::Account(src.object), DataStatus::Updated)
        }
    }
}

mod asset {
    use crate::prelude::*;

    // SATO DataStatus::Created(...)
    // SATO DataStatus::Updated(...)

    impl From<Mint<Asset, u32>> for DataEvent {
        fn from(src: Mint<Asset, u32>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Created)
        }
    }

    impl From<Mint<Asset, u128>> for DataEvent {
        fn from(src: Mint<Asset, u128>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Created)
        }
    }

    impl From<Mint<Asset, Fixed>> for DataEvent {
        fn from(src: Mint<Asset, Fixed>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Created)
        }
    }

    impl From<SetKeyValue<Asset, String, Value>> for DataEvent {
        fn from(src: SetKeyValue<Asset, String, Value>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Updated)
        }
    }

    impl From<Burn<Asset, u32>> for DataEvent {
        fn from(src: Burn<Asset, u32>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Deleted)
        }
    }

    impl From<Burn<Asset, u128>> for DataEvent {
        fn from(src: Burn<Asset, u128>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Deleted)
        }
    }

    impl From<Burn<Asset, Fixed>> for DataEvent {
        fn from(src: Burn<Asset, Fixed>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Deleted)
        }
    }

    impl From<RemoveKeyValue<Asset, String>> for DataEvent {
        fn from(src: RemoveKeyValue<Asset, String>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Updated)
        }
    }

    impl From<Transfer<Asset, u32, Asset>> for DataEvent {
        fn from(src: Transfer<Asset, u32, Asset>) -> Self {
            Self::new(DataEntity::Asset(src.object), DataStatus::Updated)
        }
    }
}

/// Exports common structs and enums from this module.
pub mod prelude {
    pub use super::{
        Entity as DataEntity, Event as DataEvent, EventFilter as DataEventFilter,
        Status as DataStatus,
    };
}
