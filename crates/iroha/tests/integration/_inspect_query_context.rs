use eyre::Result;
use iroha::data_model::{prelude::*, Level};
use iroha_test_network::*;
use iroha_test_samples::{load_sample_wasm, ALICE_ID};

#[test]
fn trigger() -> Result<()> {
    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    let trigger_str = "_inspect_query_context_in_trigger";
    let trigger_id: TriggerId = trigger_str.parse().unwrap();

    let register_trigger = Register::trigger(Trigger::new(
        trigger_id.clone(),
        Action::new(
            load_sample_wasm(trigger_str),
            Repeats::Indefinitely,
            ALICE_ID.clone(),
            ExecuteTriggerEventFilter::new().for_trigger(trigger_id.clone()),
        ),
    ));
    test_client.submit_blocking(register_trigger)?;

    let call_trigger = ExecuteTrigger::new(trigger_id);
    test_client.submit_blocking(call_trigger)?;

    test_client.submit_blocking(Log::new(Level::ERROR, format!("just ticking time")))?;

    Ok(())
}

#[test]
fn wasm() -> Result<()> {
    let (network, _rt) = NetworkBuilder::new().start_blocking()?;
    let test_client = network.client();

    let wasm_str = "_inspect_query_context_in_txn_wasm";

    let transaction =
        test_client.build_transaction(load_sample_wasm(wasm_str), Metadata::default());
    test_client.submit_transaction_blocking(&transaction)?;

    test_client.submit_blocking(Log::new(Level::ERROR, format!("just ticking time")))?;

    Ok(())
}
