use std::num::{NonZeroU32, NonZeroU64};

use eyre::Result;
use iroha_client::{
    client::{asset, Client, QueryResult},
    data_model::{asset::AssetDefinition, prelude::*, query::Pagination},
};
use test_network::*;

#[test]
fn limits_should_work() -> Result<()> {
    let (_rt, _peer, client) = <PeerBuilder>::new().with_port(10_690).start_with_runtime();
    wait_for_genesis_committed(&vec![client.clone()], 0);

    register_assets(&client)?;

    let vec = &client
        .build_query(asset::all_definitions())
        .with_pagination(Pagination {
            limit: NonZeroU32::new(7),
            start: NonZeroU64::new(1),
        })
        .execute()?
        .collect::<QueryResult<Vec<_>>>()?;
    assert_eq!(vec.len(), 7);
    Ok(())
}

#[test]
fn fetch_size_should_work() -> Result<()> {
    let (_rt, _peer, client) = <PeerBuilder>::new().with_port(11_120).start_with_runtime();
    wait_for_genesis_committed(&vec![client.clone()], 0);

    register_assets(&client)?;

    let iter = client
        .build_query(asset::all_definitions())
        .with_pagination(Pagination {
            limit: NonZeroU32::new(7),
            start: NonZeroU64::new(1),
        })
        .with_fetch_size(FetchSize::new(Some(NonZeroU32::new(3).expect("Valid"))))
        .execute()?;
    assert_eq!(iter.batch_len(), 3);
    Ok(())
}

fn register_assets(client: &Client) -> Result<()> {
    // FIXME transaction is rejected for more than a certain number of instructions
    let register: Vec<InstructionBox> = ('a'..='j')
        .map(|c| c.to_string())
        .map(|name| (name + "#wonderland").parse().expect("Valid"))
        .map(|asset_definition_id| {
            Register::asset_definition(AssetDefinition::numeric(asset_definition_id)).into()
        })
        .collect();
    let _ = client.submit_all_blocking(register)?;
    Ok(())
}
