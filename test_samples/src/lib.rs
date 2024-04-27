//! Utility crate for standardized and random signatories.

use core::str::FromStr;

use iroha_crypto::KeyPair;
use iroha_data_model::prelude::AccountId;
use once_cell::sync::Lazy;

/// Generate [`AccountId`](iroha_data_model::account::AccountId) in the given `domain`.
///
/// # Panics
///
/// Panics if the given `domain` is invalid as [`Name`](iroha_data_model::name::Name).
#[cfg(feature = "rand")]
pub fn gen_account_in(domain: impl core::fmt::Display) -> (AccountId, KeyPair) {
    let key_pair = KeyPair::random();
    let account_id = format!("{}@{}", key_pair.public_key(), domain)
        .parse()
        .expect("domain name should be valid");
    (account_id, key_pair)
}

macro_rules! static_signatory_ed25519 {
    ( $kp:ident, $vk:expr, $sk:expr ) => {
        /// A standardized [`KeyPair`](iroha_crypto::KeyPair).
        pub static $kp: Lazy<KeyPair> = Lazy::new(|| {
            KeyPair::new(
                iroha_crypto::PublicKey::from_str($vk).unwrap(),
                iroha_crypto::PrivateKey::from_hex(iroha_crypto::Algorithm::Ed25519, $sk).unwrap(),
            )
            .unwrap()
        });
    };
    ( $id:ident, $dm:literal, $kp:ident, $vk:literal, $sk:literal ) => {
        /// A standardized [`AccountId`](iroha_data_model::account::AccountId).
        pub static $id: Lazy<AccountId> =
            Lazy::new(|| format!("{}@{}", $kp.public_key(), $dm).parse().unwrap());

        static_signatory_ed25519!($kp, $vk, $sk);
    };
}
static_signatory_ed25519!(PEER_KEYPAIR, "ed01207233BFC89DCBD68C19FDE6CE6158225298EC1131B6A130D1AEB454C1AB5183C0", "9AC47ABF59B356E0BD7DCBBBB4DEC080E302156A48CA907E47CB6AEA1D32719E7233BFC89DCBD68C19FDE6CE6158225298EC1131B6A130D1AEB454C1AB5183C0");
static_signatory_ed25519!(ALICE_ID, "wonderland", ALICE_KEYPAIR, "ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03", "CCF31D85E3B32A4BEA59987CE0C78E3B8E2DB93881468AB2435FE45D5C9DCD53CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03");
static_signatory_ed25519!(BOB_ID, "wonderland", BOB_KEYPAIR, "ed012004FF5B81046DDCCF19E2E451C45DFB6F53759D4EB30FA2EFA807284D1CC33016", "AF3F96DEEF44348FEB516C057558972CEC4C75C4DB9C5B3AAC843668854BF82804FF5B81046DDCCF19E2E451C45DFB6F53759D4EB30FA2EFA807284D1CC33016");
static_signatory_ed25519!(CARPENTER_ID, "garden_of_live_flowers", CARPENTER_KEYPAIR, "ed0120E9F632D3034BAB6BB26D92AC8FD93EF878D9C5E69E01B61B4C47101884EE2F99", "B5DD003D106B273F3628A29E6087C31CE12C9F32223BE26DD1ADB85CEBB48E1DE9F632D3034BAB6BB26D92AC8FD93EF878D9C5E69E01B61B4C47101884EE2F99");
static_signatory_ed25519!(ADMIN_ID, "admin", ADMIN_KEYPAIR, "ed012076E5CA9698296AF9BE2CA45F525CB3BCFDEB7EE068BA56F973E9DD90564EF4FC", "A4DE33BCA99A254ED6265D1F0FB69DFE42B77F89F6C2E478498E1831BF6D81F276E5CA9698296AF9BE2CA45F525CB3BCFDEB7EE068BA56F973E9DD90564EF4FC");
