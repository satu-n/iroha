//! This module provides `WorldStateView` - in-memory representations of the current blockchain
//! state.

use std::collections::HashMap;

use config::Configuration;
use iroha_data_model::prelude::*;
use iroha_error::{error, Result};

use crate::prelude::*;

/// Current state of the blockchain alligned with `Iroha` module.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WorldStateView {
    /// The world - contains `domains`, `triggers`, etc..
    pub world: World,
    /// Hashes of the committed and rejected transactions.
    pub transactions: HashMap<Hash, TransactionValue>,
    /// Configuration of World State View.
    pub config: Configuration,
}

/// WARNING!!! INTERNAL USE ONLY!!!
impl WorldStateView {
    /// Default `WorldStateView` constructor.
    pub fn new(world: World) -> Self {
        WorldStateView {
            world,
            transactions: HashMap::new(),
            config: Configuration::default(),
        }
    }

    /// [`WorldStateView`] constructor with configuration.
    pub fn from_config(config: Configuration, world: World) -> Self {
        WorldStateView {
            world,
            transactions: HashMap::new(),
            config,
        }
    }

    /// Initializes WSV with the blocks from block storage.
    pub fn init(&mut self, blocks: &[VersionedValidBlock]) {
        for block in blocks {
            self.apply(&block.clone().commit());
        }
    }

    /// Apply `CommittedBlock` with changes in form of **Iroha Special Instructions** to `self`.
    pub fn apply(&mut self, block: &VersionedCommittedBlock) {
        for transaction in &block.as_inner_v1().transactions {
            if let Err(e) = transaction.proceed(self) {
                iroha_logger::warn!("Failed to proceed transaction on WSV: {}", e);
            }
            let _ = self.transactions.insert(
                transaction.hash(),
                TransactionValue::Transaction(transaction.clone().into()),
            );
        }
        for transaction in &block.as_inner_v1().rejected_transactions {
            let _ = self.transactions.insert(
                transaction.hash(),
                TransactionValue::RejectedTransaction(transaction.clone()),
            );
        }
    }

    /// Get `World` without an ability to modify it.
    pub const fn read_world(&self) -> &World {
        &self.world
    }

    /// Get `World` with an ability to modify it.
    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }

    /// Add new `Domain` entity.
    pub fn add_domain(&mut self, domain: Domain) {
        let _ = self.world.domains.insert(domain.name.clone(), domain);
    }

    /// Get `Domain` without an ability to modify it.
    pub fn read_domain(&self, name: &str) -> Option<&Domain> {
        self.world.domains.get(name)
    }

    /// Get `Domain` with an ability to modify it.
    pub fn domain(&mut self, name: &str) -> Option<&mut Domain> {
        self.world.domains.get_mut(name)
    }

    /// Get all `Domain`s without an ability to modify them.
    pub fn read_all_domains(&self) -> Vec<&Domain> {
        let mut vec = self.world.domains.values().collect::<Vec<&Domain>>();
        vec.sort();
        vec
    }

    /// Get all `Account`s without an ability to modify them.
    pub fn read_all_accounts(&self) -> Vec<&Account> {
        let mut vec = self
            .world
            .domains
            .values()
            .flat_map(|domain| domain.accounts.values())
            .collect::<Vec<&Account>>();
        vec.sort();
        vec
    }

    /// Get `Account` without an ability to modify it.
    pub fn read_account(&self, id: &<Account as Identifiable>::Id) -> Option<&Account> {
        self.read_domain(&id.domain_name)?.accounts.get(id)
    }

    /// Get `Account`'s `Asset`s without an ability to modify it.
    pub fn read_account_assets(&self, id: &<Account as Identifiable>::Id) -> Option<Vec<&Asset>> {
        let mut vec = self
            .read_account(id)?
            .assets
            .values()
            .collect::<Vec<&Asset>>();
        vec.sort();
        Some(vec)
    }

    /// Get `Account` with an ability to modify it.
    pub fn account(&mut self, id: &<Account as Identifiable>::Id) -> Option<&mut Account> {
        self.domain(&id.domain_name)?.accounts.get_mut(id)
    }

    /// Get all `PeerId`s without an ability to modify them.
    pub fn read_all_peers(&self) -> Vec<Peer> {
        let mut vec = self
            .read_world()
            .trusted_peers_ids
            .iter()
            .cloned()
            .map(Peer::new)
            .collect::<Vec<Peer>>();
        vec.sort();
        vec
    }

    /// Get all `Asset`s without an ability to modify them.
    pub fn read_all_assets(&self) -> Vec<&Asset> {
        let mut vec = self
            .world
            .domains
            .values()
            .flat_map(|domain| domain.accounts.values())
            .flat_map(|account| account.assets.values())
            .collect::<Vec<&Asset>>();
        vec.sort();
        vec
    }

    /// Get all `Asset Definition Entry`s without an ability to modify them.
    pub fn read_all_assets_definitions_entries(&self) -> Vec<&AssetDefinitionEntry> {
        let mut vec = self
            .world
            .domains
            .values()
            .flat_map(|domain| domain.asset_definitions.values())
            .collect::<Vec<&AssetDefinitionEntry>>();
        vec.sort();
        vec
    }

    /// Get `Asset` without an ability to modify it.
    pub fn read_asset(&self, id: &<Asset as Identifiable>::Id) -> Option<&Asset> {
        self.read_account(&id.account_id)?.assets.get(id)
    }

    /// Get `Asset` with an ability to modify it.
    pub fn asset(&mut self, id: &<Asset as Identifiable>::Id) -> Option<&mut Asset> {
        self.account(&id.account_id)?.assets.get_mut(id)
    }

    /// Get `Asset` with an ability to modify it.
    /// If no asset is present - create it. Similar to Entry API.
    ///
    /// Returns `None` if no corresponding account was found.
    pub fn asset_or_insert<V: Into<AssetValue>>(
        &mut self,
        id: &<Asset as Identifiable>::Id,
        default_value: V,
    ) -> Option<&mut Asset> {
        Some(
            self.account(&id.account_id)?
                .assets
                .entry(id.clone())
                .or_insert_with(|| Asset::new(id.clone(), default_value)),
        )
    }

    /// Add new `Asset` entity.
    /// # Errors
    /// Fails if there is no account for asset
    pub fn add_asset(&mut self, asset: Asset) -> Result<()> {
        let _ = self
            .account(&asset.id.account_id)
            .ok_or_else(|| error!("Failed to find account"))?
            .assets
            .insert(asset.id.clone(), asset);
        Ok(())
    }

    /// Get `AssetDefinitionEntry` without an ability to modify it.
    pub fn read_asset_definition_entry(
        &self,
        id: &<AssetDefinition as Identifiable>::Id,
    ) -> Option<&AssetDefinitionEntry> {
        self.read_domain(&id.domain_name)?.asset_definitions.get(id)
    }

    /// Get `AssetDefinitionEntry` with an ability to modify it.
    pub fn asset_definition_entry(
        &mut self,
        id: &<AssetDefinition as Identifiable>::Id,
    ) -> Option<&mut AssetDefinitionEntry> {
        self.domain(&id.domain_name)?.asset_definitions.get_mut(id)
    }

    /// Checks if this `transaction_hash` is already committed or rejected.
    pub fn has_transaction(&self, transaction_hash: Hash) -> bool {
        self.transactions.contains_key(&transaction_hash)
    }

    /// Get committed and rejected transaction of the account.
    pub fn read_transactions(&self, account_id: &AccountId) -> Vec<&TransactionValue> {
        let mut vec: Vec<&TransactionValue> = self
            .transactions
            .values()
            .filter(|tx| &tx.payload().account_id == account_id)
            .collect();
        vec.sort();
        vec
    }
}

/// This module contains all configuration related logic.
pub mod config {
    use iroha_config::derive::Configurable;
    use iroha_data_model::metadata::Limits as MetadataLimits;
    use iroha_data_model::LengthLimits;
    use serde::{Deserialize, Serialize};

    const DEFAULT_ASSET_LIMITS: MetadataLimits = MetadataLimits::new(2_u32.pow(20), 2_u32.pow(12));
    const DEFAULT_ACCOUNT_LIMITS: MetadataLimits =
        MetadataLimits::new(2_u32.pow(20), 2_u32.pow(12));
    const DEFAULT_LENGTH_LIMITS: LengthLimits = LengthLimits::new(1, 2_u32.pow(7));
    /// [`WorldStateView`](super::WorldStateView) configuration.
    #[derive(Clone, Deserialize, Serialize, Debug, Copy, Configurable)]
    #[config(env_prefix = "WSV_")]
    #[serde(rename_all = "UPPERCASE", default)]
    pub struct Configuration {
        /// [`MetadataLimits`] for every asset with store.
        pub asset_metadata_limits: MetadataLimits,
        /// [`MetadataLimits`] of any account's metadata.
        pub account_metadata_limits: MetadataLimits,
        /// [`LengthLimits`] of identifiers in bytes that can be stored in the WSV.
        pub length_limits: LengthLimits,
    }

    impl Default for Configuration {
        fn default() -> Self {
            Configuration {
                asset_metadata_limits: DEFAULT_ASSET_LIMITS,
                account_metadata_limits: DEFAULT_ACCOUNT_LIMITS,
                length_limits: DEFAULT_LENGTH_LIMITS,
            }
        }
    }
}
