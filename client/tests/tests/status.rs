#![allow(clippy::pedantic, clippy::restriction)]

use std::thread;

use iroha_client::client::Client;
use iroha_core::config::Configuration;
use iroha_crypto::KeyPair;
use iroha_data_model::prelude::*;
use test_network::{Network as TestNetwork, TestConfiguration};

fn ready_for_mint(client: &mut Client) -> MintBox {
    let create_domain = RegisterBox::new(IdentifiableBox::Domain(Domain::new("domain").into()));
    let account_id = AccountId::new("account", "domain");
    let create_account = RegisterBox::new(IdentifiableBox::NewAccount(
        NewAccount::with_signatory(
            account_id.clone(),
            KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key,
        )
        .into(),
    ));
    let asset_definition_id = AssetDefinitionId::new("asset", "domain");
    let create_asset = RegisterBox::new(IdentifiableBox::AssetDefinition(
        AssetDefinition::new_quantity(asset_definition_id.clone()).into(),
    ));

    client
        .submit_all(vec![
            create_domain.into(),
            create_account.into(),
            create_asset.into(),
        ])
        .expect("Failed to prepare state.");

    MintBox::new(
        Value::U32(1),
        IdBox::AssetId(AssetId::new(asset_definition_id, account_id)),
    )
}

#[test]
fn test_status() {
    const N_PEERS: u64 = 4;
    let mut status;

    let (rt, network, mut client) = <TestNetwork>::start_test_with_runtime(N_PEERS as u32, 1);
    let pipeline_time = Configuration::pipeline_time();
    client.status_url.insert_str(0, "http://");
    thread::sleep(pipeline_time * 2);

    // Confirm all peers connected
    status = client.get_status().unwrap();
    assert_eq!(status.peers, N_PEERS - 1);
    assert_eq!(status.blocks, 1);

    // Add a peer then #peers should be incremented
    let (peer, _) = rt.block_on(network.add_peer());
    thread::sleep(pipeline_time * 2);
    status = client.get_status().unwrap();
    assert_eq!(status.peers, N_PEERS);
    assert_eq!(status.blocks, 2);

    // Remove the peer then #peers should be decremented
    let remove_peer = UnregisterBox::new(IdBox::PeerId(peer.id.clone()));
    client.submit(remove_peer).expect("Failed to remove peer");
    thread::sleep(pipeline_time * 2);
    status = client.get_status().unwrap();
    assert_eq!(status.peers, N_PEERS - 1);
    assert_eq!(status.blocks, 3);
}
