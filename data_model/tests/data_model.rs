use iroha_data_model::prelude::*;
use iroha_sample_params::gen_account_in;

#[test]
fn transfer_isi_should_be_valid() {
    let _instruction = Transfer::asset_numeric(
        "btc##seller@crypto".parse_alias(),
        12u32,
        gen_account_in("crypto").0, // ACC_NAME buyer
    );
}
