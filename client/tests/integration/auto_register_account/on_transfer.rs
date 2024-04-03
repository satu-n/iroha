//! A new account should be automatically registered when being a destination of a transfer

use iroha_client::data_model::prelude::*;
use iroha_sample_params::{alias::Alias, SampleParams};
use serde_json::json;
use test_network::*;

/// A new account should be automatically registered when being a destination of a transfer of numeric asset e.g. "rose"
///
/// # Scenario
///
/// 0. alice@wonderland: domain owner
/// 0. alice -> new carol ... ok (domain owner)
/// 0. carol -> new dave ... err (permission denied)
/// 0. grant `CanRegisterAccountInDomain` permission to carol
/// 0. carol -> new dave ... ok (authorized)
#[test]
fn asset_numeric() {
    let (_rt, _peer, client_alice) = <PeerBuilder>::new().with_port(10_810).start_with_runtime();
    wait_for_genesis_committed(&[client_alice.clone()], 0);
    let observer = client_alice.clone();

    // alice@wonderland: domain owner
    let alice: AccountId = "alice@wonderland".parse_alias();
    let wonderland = observer
        .request(FindDomainById::new("wonderland".parse().unwrap()))
        .expect("should be found");
    assert_eq!(*wonderland.owned_by(), alice);

    // alice -> new carol ... ok (domain owner)
    let carol: AccountId = "carol@wonderland".parse_alias();
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect_err("carol should not be on chain at this point");
    let rose_alice: AssetId = "rose##alice@wonderland".parse_alias();
    let n_roses_alice = observer
        .request(FindAssetQuantityById::new(rose_alice.clone()))
        .expect("alice should have roses");
    assert!(numeric!(3) < n_roses_alice);
    let transfer = Transfer::asset_numeric(rose_alice, 3_u32, carol.clone());
    client_alice
        .submit_blocking(transfer)
        .expect("alice the domain owner should succeed to register carol in the domain");
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect("carol should be on chain now");
    let rose_carol: AssetId = "rose##carol@wonderland".parse_alias();
    let n_roses_carol = observer
        .request(FindAssetQuantityById::new(rose_carol.clone()))
        .expect("carol should have roses");
    assert_eq!(n_roses_carol, numeric!(3));

    // carol -> new dave ... err (permission denied)
    let client_carol = {
        let mut client = client_alice.clone();
        let sp = SampleParams::default();
        client.key_pair = sp.signatory["carol"].make_key_pair();
        client.account_id = carol.clone();
        client
    };
    let dave: AccountId = "dave@wonderland".parse_alias();
    let _ = observer
        .request(FindAccountById::new(dave.clone()))
        .expect_err("dave should not be on chain at this point");
    let transfer = Transfer::asset_numeric(rose_carol, 1_u32, dave.clone());
    let _ = client_carol
        .submit_blocking(transfer.clone())
        .expect_err("carol should fail to register dave in the domain");
    let _ = observer
        .request(FindAccountById::new(dave.clone()))
        .expect_err("dave should not be on chain yet");
    let rose_dave: AssetId = "rose##dave@wonderland".parse_alias();
    let _ = observer
        .request(FindAssetQuantityById::new(rose_dave.clone()))
        .expect_err("dave should not have roses yet");

    // grant `CanRegisterAccountInDomain` permission to carol
    let grant = {
        let token = PermissionToken::new(
            "CanRegisterAccountInDomain".parse().unwrap(),
            &json!({ "domain_id": wonderland.id }),
        );
        Grant::permission(token, carol.clone())
    };
    client_alice
        .submit_blocking(grant)
        .expect("alice should succeed to grant");

    // carol -> new dave ... ok (authorized)
    client_carol
        .submit_blocking(transfer)
        .expect("carol now authorized should succeed to register dave in the domain");
    let _ = observer
        .request(FindAccountById::new(dave.clone()))
        .expect("dave should be on chain now");
    let n_roses_dave = observer
        .request(FindAssetQuantityById::new(rose_dave))
        .expect("dave should have roses");
    assert_eq!(n_roses_dave, numeric!(1));
}

/// A new account should be automatically registered when being a destination of a transfer of asset store e.g. "dict"
///
/// # Scenario
///
/// 0. alice@wonderland: domain owner
/// 0. alice -> new carol ... ok (domain owner)
///
/// And how to authorize an account other than the domain owner is the same way as [`asset_numeric()`]
#[test]
fn asset_store() {
    let (_rt, _peer, client_alice) = <PeerBuilder>::new().with_port(10_815).start_with_runtime();
    wait_for_genesis_committed(&[client_alice.clone()], 0);
    let observer = client_alice.clone();

    let dict_alice: AssetId = "dict##alice@wonderland".parse_alias();
    let register: InstructionBox =
        Register::asset_definition(AssetDefinition::store(dict_alice.definition_id.clone())).into();
    let dict_key: Name = "raven".parse().unwrap();
    let dict_value: String = "nevar".into();
    let set_key_value =
        SetKeyValue::asset(dict_alice.clone(), dict_key.clone(), dict_value.clone()).into();
    client_alice
        .submit_all_blocking([register, set_key_value])
        .expect("should be committed");

    // alice@wonderland: domain owner
    let alice: AccountId = "alice@wonderland".parse_alias();
    let wonderland = observer
        .request(FindDomainById::new("wonderland".parse().unwrap()))
        .expect("should be found");
    assert_eq!(*wonderland.owned_by(), alice);

    // alice -> new carol ... ok (domain owner)
    let carol: AccountId = "carol@wonderland".parse_alias();
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect_err("carol should not be on chain at this point");
    let transfer = Transfer::asset_store(dict_alice, carol.clone());
    client_alice
        .submit_blocking(transfer)
        .expect("alice the domain owner should succeed to register carol in the domain");
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect("carol should be on chain now");
    let dict_carol: AssetId = "dict##carol@wonderland".parse_alias();
    let dict_value_carol = observer
        .request(FindAssetKeyValueByIdAndKey::new(
            dict_carol.clone(),
            dict_key,
        ))
        .expect("carol should have dicts");
    assert_eq!(dict_value_carol, dict_value.into());
}

/// A new account should be automatically registered when being a destination of a transfer of asset definition e.g. "rose"
///
/// # Scenario
///
/// 0. alice@wonderland: domain owner
/// 0. alice -> new carol ... ok (domain owner)
///
/// And how to authorize an account other than the domain owner is the same way as [`asset_numeric()`]
#[test]
fn asset_definition() {
    let (_rt, _peer, client_alice) = <PeerBuilder>::new().with_port(10_820).start_with_runtime();
    wait_for_genesis_committed(&[client_alice.clone()], 0);
    let observer = client_alice.clone();

    // alice@wonderland: domain owner
    let alice: AccountId = "alice@wonderland".parse_alias();
    let wonderland = observer
        .request(FindDomainById::new("wonderland".parse().unwrap()))
        .expect("should be found");
    assert_eq!(*wonderland.owned_by(), alice);

    // alice -> new carol ... ok (domain owner)
    let carol: AccountId = "carol@wonderland".parse_alias();
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect_err("carol should not be on chain at this point");
    let rose: AssetDefinitionId = "rose#wonderland".parse().unwrap();
    let rose_def = observer
        .request(FindAssetDefinitionById::new(rose.clone()))
        .expect("wonderland should have rose definition");
    assert_eq!(*rose_def.owned_by(), alice);
    let transfer = Transfer::asset_definition(alice, rose.clone(), carol.clone());
    client_alice
        .submit_blocking(transfer)
        .expect("alice the domain owner should succeed to register carol in the domain");
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect("carol should be on chain now");
    let rose_def = observer
        .request(FindAssetDefinitionById::new(rose.clone()))
        .expect("wonderland should have rose definition");
    assert_eq!(*rose_def.owned_by(), carol);
}

/// A new account should be automatically registered when being a destination of a transfer of domain e.g. "wonderland"
///
/// # Scenario
///
/// 0. alice@wonderland: domain owner
/// 0. alice -> new carol ... ok (domain owner)
///
/// And how to authorize an account other than the domain owner is the same way as [`asset_numeric()`]
#[test]
fn domain() {
    let (_rt, _peer, client_alice) = <PeerBuilder>::new().with_port(10_830).start_with_runtime();
    wait_for_genesis_committed(&[client_alice.clone()], 0);
    let observer = client_alice.clone();

    // alice@wonderland: domain owner
    let alice: AccountId = "alice@wonderland".parse_alias();
    let wonderland = observer
        .request(FindDomainById::new("wonderland".parse().unwrap()))
        .expect("should be found");
    assert_eq!(*wonderland.owned_by(), alice);

    // alice -> new carol ... ok (domain owner)
    let carol: AccountId = "carol@wonderland".parse_alias();
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect_err("carol should not be on chain at this point");
    let transfer = Transfer::domain(alice, wonderland.id, carol.clone());
    client_alice
        .submit_blocking(transfer)
        .expect("alice the domain owner should succeed to register carol in the domain");
    let _ = observer
        .request(FindAccountById::new(carol.clone()))
        .expect("carol should be on chain now");
    let wonderland = observer
        .request(FindDomainById::new("wonderland".parse().unwrap()))
        .expect("should be found");
    assert_eq!(*wonderland.owned_by(), carol);
}
