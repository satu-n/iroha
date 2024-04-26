//! Utility crate for testing.

use iroha_data_model::prelude::{AccountId, DomainId};
use iroha_crypto::KeyPair;

/// Generate [`AccountId`] in the given `domain`.
/// 
/// # Panics
/// 
/// Panics if the given `domain` is invalid as [`Name`].
/// 
/// SATO doc link
/// [AccountId]: iroha_data_model::prelude::AccountId
/// [Name]: iroha_data_model::prelude::Name
pub fn gen_account_in(domain: impl AsRef<str>) -> (AccountId, KeyPair) {
    let domain_id: DomainId = domain.as_ref().parse().expect("domain name should be valid");
    let key_pair = KeyPair::random();
    let account_id = AccountId::new(domain_id, key_pair.public_key().clone());
    (account_id, key_pair)
}
