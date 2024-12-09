use eyre::Result;
use iroha::{client, data_model::prelude::*};
use iroha_test_network::*;
use iroha_test_samples::{gen_account_in, ALICE_ID};

/// # Scenario
///
/// 0. transaction: register carol
/// 0. account created (carol) -> mint a rose for carol
/// 0. transaction: transfer the rose from carol ... depends on the last trigger execution
/// 0. block commit
#[test]
fn executes_on_every_transaction() -> Result<()> {
    todo!()
}

mod matches_a_batch_of_events {
    use super::*;

    /// # Scenario
    ///
    /// 0. instruction: mint a rose
    /// 0. instruction: mint a rose
    /// 0. asset created (2 roses) -> transfer the 2 roses
    #[test]
    fn accumulation() -> Result<()> {
        todo!()
    }

    /// # Scenario
    ///
    /// 0. instruction: register carol
    /// 0. instruction: register dave
    /// 0. account created (carol | dave) -> mint a rose for carol and dave
    #[test]
    fn enumeration() -> Result<()> {
        todo!()
    }

    /// # Scenario
    ///
    /// 0. instruction: register carol
    /// 0. instruction: unregister carol
    /// 0. instruction: register carol
    /// 0. account created (carol) -> mint a rose for carol
    #[test]
    fn cancellation() -> Result<()> {
        todo!()
    }
}

/// # Scenario
///
/// 0. register trigger_1 with filter_1
/// 0. register trigger_2 with filter_2
/// 0. emit an event that matches both filter_1 and filter_2
/// 0. both trigger_1 and trigger_2 execute
#[test]
fn subscribe_events() -> Result<()> {
    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    let account_id = ALICE_ID.clone();
    let asset_definition_id = "rose#wonderland".parse()?;
    let asset_id = AssetId::new(asset_definition_id, account_id.clone());

    let get_asset_value = |iroha: &client::Client, asset_id: AssetId| -> Numeric {
        match *iroha
            .query(FindAssets::new())
            .filter_with(|asset| asset.id.eq(asset_id))
            .execute_single()
            .unwrap()
            .value()
        {
            AssetValue::Numeric(val) => val,
            _ => panic!("Expected u32 asset value"),
        }
    };

    let prev_value = get_asset_value(&test_client, asset_id.clone());

    let instruction = Mint::asset_numeric(1u32, asset_id.clone());
    let register_trigger = Register::trigger(Trigger::new(
        "mint_rose_1".parse()?,
        Action::new(
            [instruction.clone()],
            Repeats::Indefinitely,
            account_id.clone(),
            AccountEventFilter::new().for_events(AccountEventSet::Created),
        ),
    ));
    test_client.submit_blocking(register_trigger)?;

    let register_trigger = Register::trigger(Trigger::new(
        "mint_rose_2".parse()?,
        Action::new(
            [instruction],
            Repeats::Indefinitely,
            account_id,
            DomainEventFilter::new().for_events(DomainEventSet::Created),
        ),
    ));
    test_client.submit_blocking(register_trigger)?;

    test_client.submit_blocking(Register::account(Account::new(
        gen_account_in("wonderland").0,
    )))?;

    let new_value = get_asset_value(&test_client, asset_id.clone());
    assert_eq!(new_value, prev_value.checked_add(numeric!(1)).unwrap());

    test_client.submit_blocking(Register::domain(Domain::new("neverland".parse()?)))?;

    let newer_value = get_asset_value(&test_client, asset_id);
    assert_eq!(newer_value, new_value.checked_add(numeric!(1)).unwrap());

    Ok(())
}
