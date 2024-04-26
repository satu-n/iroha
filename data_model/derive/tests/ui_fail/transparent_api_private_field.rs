use iroha_data_model::account::AccountId;
use iroha_sample_params::gen_account_in;

fn main() {
    let (account_id, _account_keypair) = gen_account_in("wonderland"); // ACC_NAME alice
    println!("ID: {}", account_id.signatory);
}
