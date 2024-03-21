use iroha_data_model::account::AccountId;
use iroha_sample_params::alias::Alias;

fn main() {
    let account_id: AccountId = "alice@wonderland".parse_alias();
    println!("ID: {}", account_id.signatory);
}
