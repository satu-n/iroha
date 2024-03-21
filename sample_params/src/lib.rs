//! Utility crate for testing.
//! Provides readability and concise notation along with [`alias`] module.
//!
//! # Example
//!
//! ```rust
//! use iroha_data_model::prelude::AccountId;
//! use iroha_sample_params::{alias::Alias, SAMPLE_PARAMS};
//!
//! let alice_id: AccountId = "alice@wonderland".parse_alias();
//!
//! let sp = &SAMPLE_PARAMS;
//! let alice_keypair = sp.signatory["alice"].make_key_pair();
//!
//! assert_eq!(alice_id.signatory(), alice_keypair.public_key())
//! ```

use once_cell::sync::Lazy;
use serde::Deserialize;

pub mod alias;

const TOML_PATH: &str = "src/.toml";

/// [`SampleParams`] from [`TOML_PATH`] file contents which is initialized on the first access.
///
/// # Panics
///
/// - [`TOML_PATH`] file does not exist
/// - [`TOML_PATH`] file contents is not [`SampleParams`] compatible
pub static SAMPLE_PARAMS: Lazy<SampleParams> = Lazy::new(|| {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(TOML_PATH);
    let buf = std::fs::read_to_string(path).expect("file should exist and be utf-8 format");
    toml::from_str(&buf).expect("should deserialize to SampleParams")
});

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
