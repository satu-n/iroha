//! Utility crate for testing.
//! Provides readability and concise notation along with [`alias`] module.
//!
//! # Example
//!
//! ```rust
//! use iroha_data_model::prelude::{AccountId, AssetId};
//! use iroha_sample_params::alias::Alias;
//!
//! let alice_from_alias: AccountId = "alice@wonderland".parse_alias();
//! let alice: AccountId = "ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland".parse().expect("should be valid");
//! assert_eq!(alice, alice_from_alias);
//!
//! let rose_from_alias: AssetId = "rose##alice@wonderland".parse_alias();
//! let rose: AssetId = "rose##ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland".parse().expect("should be valid");
//! assert_eq!(rose, rose_from_alias);
//! ```

use serde::Deserialize;

pub mod alias;

const TOML_PATH: &str = "src/.toml";

impl Default for SampleParams {
    /// Construct [`SampleParams`] from [`TOML_PATH`] file contents.
    ///
    /// # Panics
    ///
    /// - [`TOML_PATH`] file does not exist
    /// - [`TOML_PATH`] file contents is not [`SampleParams`] compatible
    fn default() -> Self {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(TOML_PATH);
        let buf = std::fs::read_to_string(path).expect("file should exist and be utf-8 format");
        toml::from_str(&buf).expect("should deserialize to SampleParams")
    }
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct SampleParams {
    pub signatory: std::collections::BTreeMap<String, Signatory>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Signatory {
    pub public_key: String,
    pub private_key: PrivateKey,
}

impl Signatory {
    /// Make a [`iroha_crypto::PublicKey`] from the deserialized [`Signatory`]
    pub fn make_public_key(&self) -> iroha_crypto::PublicKey {
        self.public_key.parse().expect("sample should be valid")
    }
    /// Make a [`iroha_crypto::PrivateKey`] from the deserialized [`Signatory`]
    pub fn make_private_key(&self) -> iroha_crypto::PrivateKey {
        self.private_key.make()
    }
    /// Make a [`iroha_crypto::KeyPair`] from the deserialized [`Signatory`]
    pub fn make_key_pair(&self) -> iroha_crypto::KeyPair {
        iroha_crypto::KeyPair::new(self.make_public_key(), self.make_private_key())
            .expect("should be valid pair")
    }
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct PrivateKey {
    pub algorithm: String,
    pub payload: String,
}

impl PrivateKey {
    /// Make a [`iroha_crypto::PrivateKey`] from the deserialized [`PrivateKey`]
    pub fn make(&self) -> iroha_crypto::PrivateKey {
        let algorithm = self.algorithm.parse().expect("sample should be valid");
        iroha_crypto::PrivateKey::from_hex(algorithm, &self.payload)
            .expect("sample should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_sample_params() {
        let _sp = SampleParams::default();
    }
}
