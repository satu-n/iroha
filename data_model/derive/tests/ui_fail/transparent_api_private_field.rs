use iroha_data_model::account::AccountId;

fn main() {
    let account_id: AccountId = format!("{}@wonderland", KeyPair::random().public_key()).parse().unwrap();
    println!("ID: {}", account_id.signatory);
}
