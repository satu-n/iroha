//! This module contains data events
#![allow(missing_docs)]

use derive_more::Constructor;
use getset::Getters;
use iroha_data_model_derive::{model, EventSet, HasOrigin};
use iroha_primitives::numeric::Numeric;

pub use self::model::*;
use super::*;

macro_rules! data_event {
    ($item:item) => {
        iroha_data_model_derive::model_single! {
            #[derive(
                Debug,
                Clone,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                HasOrigin,
                EventSet,
                parity_scale_codec::Decode,
                parity_scale_codec::Encode,
                serde::Deserialize,
                serde::Serialize,
                iroha_schema::IntoSchema,
            )]
            #[non_exhaustive]
            #[ffi_type]
            $item
        }
    };
}

macro_rules! entity_path {
    ($ident:ident: $tail:ty > $head:ty) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Constructor,
            parity_scale_codec::Decode,
            parity_scale_codec::Encode,
            serde::Deserialize,
            serde::Serialize,
            iroha_schema::IntoSchema,
        )]
        pub struct $ident {
            head: $head,
            tail: $tail,
        }

        impl EntityPath for $ident {
            type Head = $head;
            type Tail = $tail;
            fn head(&self) -> &Self::Head {
                &self.head
            }
            fn tail(&self) -> &Self::Tail {
                &self.tail
            }
        }
    };
}

// NOTE: if adding/editing events here, make sure to update the corresponding event filter in [`super::filter`]

#[model]
mod model {
    use super::*;
    use crate::metadata::MetadataValueBox;

    /// Generic [`MetadataChanged`] struct.
    /// Contains the changed metadata (`(key, value)` pair), either inserted or removed, which is determined by the wrapping event.
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Decode,
        Encode,
        Deserialize,
        Serialize,
        IntoSchema,
    )]
    // TODO: Generics are not supported. Figure out what to do
    //#[getset(get = "pub")]
    #[ffi_type]
    pub struct MetadataChanged<PATH> {
        pub target_path: PATH,
        pub key: Name,
        pub value: MetadataValueBox,
    }

    /// Event
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        FromVariant,
        Decode,
        Encode,
        Deserialize,
        Serialize,
        IntoSchema,
    )]
    #[ffi_type]
    pub enum DataEvent {
        /// Peer event
        Peer(peer::PeerEvent),
        /// Domain event
        Domain(domain::DomainEvent),
        /// Trigger event
        Trigger(trigger::TriggerEvent),
        /// Role event
        Role(role::RoleEvent),
        /// Permission token event
        PermissionToken(permission::PermissionTokenSchemaUpdateEvent),
        /// Configuration event
        Configuration(config::ConfigurationEvent),
        /// Executor event
        Executor(executor::ExecutorEvent),
    }
}

mod asset {
    //! This module contains `AssetEvent`, `AssetDefinitionEvent` and its impls

    use iroha_data_model_derive::model;

    pub use self::model::*;
    use super::*;

    type AssetMetadataChanged = MetadataChanged<AssetPath>;
    type AssetDefinitionMetadataChanged = MetadataChanged<AssetDefinitionPath>;

    data_event! {
        #[has_origin(origin = AssetPath)]
        pub enum AssetEvent {
            #[has_origin(asset => &asset.id().clone().into())]
            Created(Asset),
            Deleted(AssetPath),
            #[has_origin(asset_changed => &asset_changed.asset_path)]
            Added(AssetChanged),
            #[has_origin(asset_changed => &asset_changed.asset_path)]
            Removed(AssetChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataInserted(AssetMetadataChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataRemoved(AssetMetadataChanged),
        }
    }

    entity_path!(AssetPath: account::AccountPath > AssetDefinitionId);

    impl From<AssetId> for AssetPath {
        fn from(id: AssetId) -> Self {
            Self::new(id.definition_id().clone(), id.account_id().clone().into())
        }
    }

    data_event! {
        #[has_origin(origin = AssetDefinitionPath)]
        pub enum AssetDefinitionEvent {
            #[has_origin(asset_definition => &asset_definition.id().clone().into())]
            Created(AssetDefinition),
            MintabilityChanged(AssetDefinitionPath),
            #[has_origin(ownership_changed => &ownership_changed.asset_definition_path)]
            OwnerChanged(AssetDefinitionOwnerChanged),
            Deleted(AssetDefinitionPath),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataInserted(AssetDefinitionMetadataChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataRemoved(AssetDefinitionMetadataChanged),
            #[has_origin(total_quantity_changed => &total_quantity_changed.asset_definition_path)]
            TotalQuantityChanged(AssetDefinitionTotalQuantityChanged),
        }
    }

    entity_path!(AssetDefinitionPath: domain::DomainPath > Name);

    impl From<AssetDefinitionId> for AssetDefinitionPath {
        fn from(id: AssetDefinitionId) -> Self {
            Self::new(id.name().clone(), id.domain_id().clone().into())
        }
    }

    #[model]
    mod model {
        use super::*;

        /// Depending on the wrapping event, [`Self`] represents the added or removed asset quantity.
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct AssetChanged {
            pub asset_path: AssetPath,
            pub amount: AssetValue,
        }

        /// [`Self`] represents updated total asset quantity.
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct AssetDefinitionTotalQuantityChanged {
            pub asset_definition_path: AssetDefinitionPath,
            pub total_amount: Numeric,
        }

        /// [`Self`] represents updated total asset quantity.
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct AssetDefinitionOwnerChanged {
            /// Id of asset definition being updated
            pub asset_definition_path: AssetDefinitionPath,
            /// Id of new owning account
            pub new_owner: AccountId,
        }
    }
}

mod peer {
    //! This module contains `PeerEvent` and its impls

    use super::*;

    data_event! {
        #[has_origin(origin = PeerPath)]
        pub enum PeerEvent {
            Added(PeerPath),
            Removed(PeerPath),
        }
    }

    entity_path!(PeerPath: () > PeerId);
}

mod role {
    //! This module contains `RoleEvent` and its impls

    use iroha_data_model_derive::model;

    pub use self::model::*;
    use super::*;

    data_event! {
        #[has_origin(origin = RolePath)]
        pub enum RoleEvent {
            #[has_origin(role => &role.id().clone().into())]
            Created(Role),
            Deleted(RolePath),
            /// [`PermissionToken`]s with particular [`Id`](crate::permission::token::PermissionTokenId)
            /// were removed from the role.
            #[has_origin(permission_removed => &permission_removed.role_path)]
            PermissionRemoved(RolePermissionChanged),
            /// [`PermissionToken`]s with particular [`Id`](crate::permission::token::PermissionTokenId)
            /// were removed added to the role.
            #[has_origin(permission_added => &permission_added.role_path)]
            PermissionAdded(RolePermissionChanged),
        }
    }

    entity_path!(RolePath: () > RoleId);

    impl From<RoleId> for RolePath {
        fn from(id: RoleId) -> Self {
            Self::new(id.clone(), ())
        }
    }

    #[model]
    mod model {
        use super::*;

        /// Depending on the wrapping event, [`RolePermissionChanged`] role represents the added or removed role's permission
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct RolePermissionChanged {
            pub role_path: RolePath,
            // TODO: Skipped temporarily because of FFI
            #[getset(skip)]
            pub permission_token_id: PermissionTokenId,
        }
    }
}

mod permission {
    //! This module contains [`PermissionTokenSchemaUpdateEvent`]

    pub use self::model::*;
    use super::*;
    use crate::permission::PermissionTokenSchema;

    #[model]
    mod model {
        use super::*;

        /// Information about permission tokens update.
        /// Only happens when registering new executor
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct PermissionTokenSchemaUpdateEvent {
            /// Previous set of permission tokens
            pub old_schema: PermissionTokenSchema,
            /// New set of permission tokens
            pub new_schema: PermissionTokenSchema,
        }
    }
}

mod account {
    //! This module contains `AccountEvent` and its impls

    use iroha_data_model_derive::model;

    pub use self::model::*;
    use super::*;
    use crate::name::Name;

    type AccountMetadataChanged = MetadataChanged<AccountPath>;

    data_event! {
        #[has_origin(origin = AccountPath)]
        pub enum AccountEvent {
            #[has_origin(asset_event => &asset_event.origin_path().tail())]
            Asset(AssetEvent),
            #[has_origin(account => &account.id().clone().into())]
            Created(Account),
            Deleted(AccountPath),
            AuthenticationAdded(AccountPath),
            AuthenticationRemoved(AccountPath),
            #[has_origin(permission_changed => &permission_changed.account_path)]
            PermissionAdded(AccountPermissionChanged),
            #[has_origin(permission_changed => &permission_changed.account_path)]
            PermissionRemoved(AccountPermissionChanged),
            #[has_origin(role_changed => &role_changed.account_path)]
            RoleRevoked(AccountRoleChanged),
            #[has_origin(role_changed => &role_changed.account_path)]
            RoleGranted(AccountRoleChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataInserted(AccountMetadataChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataRemoved(AccountMetadataChanged),
        }
    }

    entity_path!(AccountPath: domain::DomainPath > PublicKey);

    impl From<AccountId> for AccountPath {
        fn from(id: AccountId) -> Self {
            Self::new(id.signatory().clone(), id.domain_id().clone().into())
        }
    }

    #[model]
    mod model {
        use super::*;

        /// Depending on the wrapping event, [`AccountPermissionChanged`] role represents the added or removed account role
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct AccountPermissionChanged {
            pub account_path: AccountPath,
            // TODO: Skipped temporarily because of FFI
            #[getset(skip)]
            pub permission_id: PermissionTokenId,
        }

        /// Depending on the wrapping event, [`AccountRoleChanged`] represents the granted or revoked role
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct AccountRoleChanged {
            pub account_path: AccountPath,
            pub role_id: RoleId,
        }
    }

    impl AccountPermissionChanged {
        /// Get permission id
        pub fn permission_id(&self) -> &Name {
            &self.permission_id
        }
    }
}

mod domain {
    //! This module contains `DomainEvent` and its impls

    pub use self::model::*;
    use super::*;

    type DomainMetadataChanged = MetadataChanged<DomainPath>;

    data_event! {
        #[has_origin(origin = DomainPath)]
        pub enum DomainEvent {
            #[has_origin(account_event => &account_event.origin_path().tail())]
            Account(AccountEvent),
            #[has_origin(asset_definition_event => &asset_definition_event.origin_path().tail())]
            AssetDefinition(AssetDefinitionEvent),
            #[has_origin(domain => &domain.id().clone().into())]
            Created(Domain),
            Deleted(DomainPath),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataInserted(DomainMetadataChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataRemoved(DomainMetadataChanged),
            #[has_origin(owner_changed => &owner_changed.domain_path)]
            OwnerChanged(DomainOwnerChanged),
        }
    }

    entity_path!(DomainPath: () > DomainId);

    impl From<DomainId> for DomainPath {
        fn from(id: DomainId) -> Self {
            Self::new(id.clone(), ())
        }
    }

    #[model]
    mod model {
        use super::*;

        /// Event indicate that owner of the [`Domain`] is changed
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct DomainOwnerChanged {
            pub domain_path: DomainPath,
            pub new_owner: AccountId,
        }
    }
}

mod trigger {
    //! This module contains `TriggerEvent` and its impls

    use iroha_data_model_derive::model;

    pub use self::model::*;
    use super::*;

    type TriggerMetadataChanged = MetadataChanged<TriggerPath>;

    data_event! {
        #[has_origin(origin = TriggerPath)]
        pub enum TriggerEvent {
            Created(TriggerPath),
            Deleted(TriggerPath),
            #[has_origin(number_of_executions_changed => &number_of_executions_changed.trigger_path)]
            Extended(TriggerNumberOfExecutionsChanged),
            #[has_origin(number_of_executions_changed => &number_of_executions_changed.trigger_path)]
            Shortened(TriggerNumberOfExecutionsChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataInserted(TriggerMetadataChanged),
            #[has_origin(metadata_changed => &metadata_changed.target_path)]
            MetadataRemoved(TriggerMetadataChanged),
        }
    }

    entity_path!(TriggerPath: () > TriggerId);

    #[model]
    mod model {
        use super::*;

        /// Depending on the wrapping event, [`Self`] represents the increased or decreased number of event executions.
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Getters,
            Decode,
            Encode,
            Deserialize,
            Serialize,
            IntoSchema,
        )]
        #[getset(get = "pub")]
        #[ffi_type]
        pub struct TriggerNumberOfExecutionsChanged {
            pub trigger_path: TriggerPath,
            pub by: u32,
        }
    }
}

mod config {
    use super::*;

    data_event! {
        #[has_origin(origin = ParameterPath)]
        pub enum ConfigurationEvent {
            Changed(ParameterPath),
            Created(ParameterPath),
            Deleted(ParameterPath),
        }
    }

    entity_path!(ParameterPath: () > ParameterId);
}

mod executor {
    use iroha_data_model_derive::model;

    pub use self::model::*;
    // this is used in no_std
    #[allow(unused)]
    use super::*;

    #[model]
    mod model {

        use iroha_data_model_derive::EventSet;

        // this is used in no_std
        #[allow(unused)]
        use super::*;

        #[derive(
            Debug,
            Copy,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            parity_scale_codec::Decode,
            parity_scale_codec::Encode,
            serde::Deserialize,
            serde::Serialize,
            iroha_schema::IntoSchema,
            EventSet,
        )]
        #[non_exhaustive]
        #[ffi_type]
        #[serde(untagged)] // Unaffected by #3330, as single unit variant
        #[repr(transparent)]
        pub enum ExecutorEvent {
            Upgraded,
        }
    }
}

trait EntityPath {
    type Head;
    type Tail: EntityPath;
    fn head(&self) -> &Self::Head;
    fn tail(&self) -> &Self::Tail;
}

impl EntityPath for () {
    type Head = ();
    type Tail = ();
    fn head(&self) -> &Self::Head {
        &()
    }
    fn tail(&self) -> &Self::Tail {
        &()
    }
}

/// Trait for events originating from [`HasOrigin::Origin`].
pub trait HasOrigin {
    /// Entity where the event originates.
    type Origin: EntityPath;
    /// Entity where the event originates.
    fn origin_path(&self) -> &Self::Origin;
}

impl From<AccountEvent> for DataEvent {
    fn from(value: AccountEvent) -> Self {
        DomainEvent::Account(value).into()
    }
}

impl From<AssetDefinitionEvent> for DataEvent {
    fn from(value: AssetDefinitionEvent) -> Self {
        DomainEvent::AssetDefinition(value).into()
    }
}

impl From<AssetEvent> for DataEvent {
    fn from(value: AssetEvent) -> Self {
        AccountEvent::Asset(value).into()
    }
}

impl DataEvent {
    /// Return the domain id of [`Event`]
    pub fn domain_id(&self) -> Option<&DomainId> {
        match self {
            Self::Domain(event) => Some(event.origin_path().head()),
            Self::Trigger(event) => event.origin_path().head().domain_id().as_ref(),
            Self::Peer(_)
            | Self::Configuration(_)
            | Self::Role(_)
            | Self::PermissionToken(_)
            | Self::Executor(_) => None,
        }
    }
}

pub mod prelude {
    pub use super::{
        account::{
            AccountEvent, AccountEventSet, AccountPath, AccountPermissionChanged,
            AccountRoleChanged,
        },
        asset::{
            AssetChanged, AssetDefinitionEvent, AssetDefinitionEventSet,
            AssetDefinitionOwnerChanged, AssetDefinitionPath, AssetDefinitionTotalQuantityChanged,
            AssetEvent, AssetEventSet, AssetPath,
        },
        config::{ConfigurationEvent, ConfigurationEventSet, ParameterPath},
        domain::{DomainEvent, DomainEventSet, DomainOwnerChanged, DomainPath},
        executor::{ExecutorEvent, ExecutorEventSet},
        peer::{PeerEvent, PeerEventSet, PeerPath},
        permission::PermissionTokenSchemaUpdateEvent,
        role::{RoleEvent, RoleEventSet, RolePath, RolePermissionChanged},
        trigger::{TriggerEvent, TriggerEventSet, TriggerNumberOfExecutionsChanged, TriggerPath},
        DataEvent, HasOrigin, MetadataChanged,
    };
}
