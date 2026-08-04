#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_register_and_resolve() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(DIDRegistryContract, ());
    let client = DIDRegistryContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let owner = Address::generate(&env);
    let doc = String::from_str(&env, r#"{"id":"did:stellar:test"}"#);
    client.register_did(&owner, &doc);

    let record = client.resolve_did(&owner).unwrap();
    assert!(record.active);
    assert_eq!(record.owner, owner);
    assert!(client.is_active(&owner));
}
