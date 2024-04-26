use iroha_data_model::prelude::*;
use iroha_sample_params::gen_account_in;

#[test]
fn transfer_isi_should_be_valid() {
    let _instruction = Transfer::asset_numeric(
        format!("btc##{}", gen_account_in("crypto").0).parse().expect("should be valid"), // ACC_NAME seller
        12u32,
        gen_account_in("crypto").0, // ACC_NAME buyer
    );
}
