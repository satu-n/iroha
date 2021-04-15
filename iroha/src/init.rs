use std::collections::BTreeMap;

use iroha_data_model::prelude::*;

use crate::config::Configuration;

/// Returns the a map of a form `domain_name -> domain`, for initial domains.
pub fn domains(configuration: &Configuration) -> BTreeMap<String, Domain> {
    std::iter::once((
        GENESIS_DOMAIN_NAME.to_owned(),
        GenesisDomain::new(
            configuration
                .genesis_configuration
                .genesis_account_public_key
                .clone(),
        )
        .into(),
    ))
    .collect()
}
