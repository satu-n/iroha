//! alias resolution

use iroha_data_model::prelude::AccountId;
use iroha_data_model::prelude::Name;
use iroha_data_model::prelude::DomainId;
use crate::client::Client;

enum AccountAddr {
    Alias(AccountAlias),
    Id(AccountId),
}

struct AccountAlias {
    name: Name,
    domain: DomainId,
}

impl Client {
    fn hoge(&self) {
        let _ = self.clone();
    }
}
