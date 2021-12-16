use std::collections::BTreeMap;

use eyre::{eyre, Result};
use iroha_data_model::prelude::*;

use crate::config::Configuration;

/// Returns the a map of a form `domain_id -> domain`, for initial domains.
pub fn domains(configuration: &Configuration) -> Result<BTreeMap<DomainId, Domain>> {
    let key = configuration
        .genesis
        .account_public_key
        .clone()
        .ok_or_else(|| eyre!("Genesis account public key is not specified."))?;
    #[allow(clippy::expect_used)]
    Ok(std::iter::once((
        Name::new(GENESIS_DOMAIN_NAME)
            .expect("Valid names never fail")
            .into(),
        Domain::from(GenesisDomain::new(key)),
    ))
    .collect())
}
