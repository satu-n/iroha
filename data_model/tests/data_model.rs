use iroha_data_model::prelude::*;
use iroha_sample_params::alias::Alias;

#[test]
fn transfer_isi_should_be_valid() {
    let _instruction = Transfer::asset_numeric(
        "btc##seller@crypto".parse_alias(),
        12u32,
        "buyer@crypto".parse_alias(),
    );
}
