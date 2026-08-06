#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_register_add_key_and_resolve() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(DIDRegistryContract, ());
    let client = DIDRegistryContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let owner = Address::generate(&env);
    let doc = String::from_str(&env, r#"{"id":"did:stellar:test"}"#);
    client.register_did(&owner, &doc);

    let key_id = String::from_str(&env, "key-1");
    let key_type = String::from_str(&env, "Ed25519VerificationKey2020");
    let pub_key = String::from_str(&env, "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN");
    client.add_verification_key(&owner, &key_id, &key_type, &pub_key);

    let record = client.resolve_did(&owner).unwrap();
    assert!(record.active);
    assert_eq!(record.keys.len(), 1);
}
