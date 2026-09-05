#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_register_asp() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let cid = env.register(ASPRegistryContract, ());
    let client = ASPRegistryContractClient::new(&env, &cid);
    client.initialize(&admin);

    let asp = Address::generate(&env);
    let root = BytesN::from_array(&env, &[1u8; 32]);
    client.register_asp(&admin, &asp, &root);
    assert!(client.is_registered_asp(&asp));
}
