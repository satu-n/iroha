//! Aliases that resolve with reference to [`crate::SampleParams`].

use std::str::FromStr;

use iroha_data_model::prelude::{AccountId, AssetId};

/// Alias of `T`. Can be parsed to `T: FromStr` when the string is provided by [`resolve`](Alias::resolve).
pub trait Alias<T: FromStr>
where
    <T as FromStr>::Err: core::fmt::Debug,
{
    /// Parse [`self`] to `T`, assuming [`self`] should successfully [`resolve`](Alias::resolve).
    ///
    /// # Panics
    ///
    /// - [`resolve`](Alias::resolve) implementation is not compatible with [`FromStr`] for `T`
    ///
    /// # Example
    ///
    /// See [`crate`] documentation.
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

impl Alias<AccountId> for str {
    fn resolve(&self) -> impl AsRef<str> {
        let (name, domain) = self
            .rsplit_once('@')
            .expect("name@domain format should be given");
        let sp = super::SampleParams::default();
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
        let alice: AccountId = "alice@wonderland".parse_alias();
        let sp = crate::SampleParams::default();
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
        assert_eq!(*rose.account_id(), "alice@wonderland".parse_alias());
    }
}
