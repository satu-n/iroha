//! Aliases that resolve with reference to [`crate::SampleParams`].
//! Provides readability and concise notation for testing.

use std::str::FromStr;

use iroha_data_model::prelude::{AccountId, AssetId};

/// Alias of `T`. Can be parsed to `T: FromStr` when the string is provided by [`resolve`](Alias::resolve).
pub trait Alias<T: FromStr>
where
    <T as FromStr>::Err: core::fmt::Debug,
{
    /// CAUTION: This is just a testing utility. Aliases other than predefined ones should cause panic!
    ///
    /// Parse [`self`] to `T`, assuming [`self`] should successfully [`resolve`](Alias::resolve).
    ///
    /// # Panics
    ///
    /// - [`resolve`](Alias::resolve) implementation is not compatible with [`FromStr`] for `T`
    ///
    /// # Example
    ///
    /// ```rust
    /// use iroha_data_model::prelude::{AccountId, AssetId};
    /// use iroha_sample_params::gen_account_in;
    ///
    /// let (alice_from_alias, _alice_from_alias_keypair) = gen_account_in("wonderland"); // ACC_NAME alice
    /// let alice: AccountId = "ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland".parse().expect("should be valid");
    /// assert_eq!(alice, alice_from_alias);
    ///
    /// let rose_from_alias: AssetId = "rose##alice@wonderland".parse_alias();
    /// let rose: AssetId = "rose##ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland".parse().expect("should be valid");
    /// assert_eq!(rose, rose_from_alias);
    ///
    /// use iroha_sample_params::gen_account_in;
    ///
    /// let alice_from_alias_alt = AccountId::from_alias("alice@wonderland");
    /// assert_eq!(alice_from_alias, alice_from_alias_alt);
    /// ```
    #[inline]
    fn parse_alias(&self) -> T {
        self.resolve()
            .as_ref()
            .parse()
            .expect("alias should resolve to compatible string with FromStr")
    }
    /// Resolve [`self`] to string that should be parsed to `T: FromStr`.
    fn resolve(&self) -> impl AsRef<str>;
}

/// Can be constructed from an alias via [`Alias::resolve`] and [`FromStr`] implementation for `Self`.
pub trait FromAlias
where
    Self: FromStr,
    <Self as FromStr>::Err: core::fmt::Debug,
    str: Alias<Self>,
{
    /// CAUTION: This is just a testing utility. Aliases other than predefined ones should cause panic!
    ///
    /// An alternative to [`Alias::parse_alias`].
    /// Construct `Self` from `alias`, assuming `alias` should successfully [`resolve`](Alias::resolve).
    ///
    /// # Panics
    ///
    /// - [`Alias::resolve`] implementation is not compatible with [`FromStr`] for `Self`
    ///
    /// # Example
    ///
    /// ```rust
    /// use iroha_data_model::prelude::AccountId;
    /// use iroha_sample_params::gen_account_in;
    ///
    /// let (alice_from_alias, _alice_from_alias_keypair) = gen_account_in("wonderland"); // ACC_NAME alice
    /// let alice_from_alias_alt = AccountId::from_alias("alice@wonderland");
    /// assert_eq!(alice_from_alias, alice_from_alias_alt);
    /// ```
    fn from_alias(alias: &str) -> Self {
        alias.parse_alias()
    }
}

impl<T> FromAlias for T
where
    T: FromStr,
    <T as FromStr>::Err: core::fmt::Debug,
    str: Alias<T>,
{
}

impl Alias<AccountId> for str {
    fn resolve(&self) -> impl AsRef<str> {
        let (name, domain) = self
            .rsplit_once('@')
            .expect("name@domain format should be given");
        let sp = &super::SAMPLE_PARAMS;
        let signatory = &*sp
            .signatory
            .get(name)
            .expect("signatory.name should be defined in SampleParams source file")
            .public_key;
        [signatory, domain].join("@")
    }
}

impl Alias<AssetId> for str {
    fn resolve(&self) -> impl AsRef<str> {
        let (asset_definition, account) = self
            .rsplit_once('#')
            .expect("asset#domain#account@domain format should be given");
        let account = Alias::<AccountId>::resolve(account);
        [asset_definition, account.as_ref()].join("#")
    }
}

#[cfg(test)]
mod tests {
    use iroha_data_model::prelude::{AssetDefinitionId, DomainId, PublicKey};

    use super::*;

    #[test]
    fn parse_sample_account_alias() {
        let (alice, _alice_keypair) = gen_account_in("wonderland"); // ACC_NAME alice
        let sp = &crate::SAMPLE_PARAMS;
        assert_eq!(
            *alice.signatory(),
            sp.signatory
                .get("alice")
                .expect("signatory.alice should be defined in SampleParams source file")
                .public_key
                .parse::<PublicKey>()
                .expect("sample keys should be valid")
        );
        assert_eq!(
            *alice.domain_id(),
            "wonderland".parse::<DomainId>().expect("should be valid")
        );
    }

    #[test]
    fn parse_sample_asset_alias() {
        let rose: AssetId = "rose##alice@wonderland".parse_alias();
        assert_eq!(
            *rose.definition_id(),
            "rose#wonderland"
                .parse::<AssetDefinitionId>()
                .expect("should be valid")
        );
        assert_eq!(*rose.account_id(), gen_account_in("wonderland").0); // ACC_NAME alice
    }
}
