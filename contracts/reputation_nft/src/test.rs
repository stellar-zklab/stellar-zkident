#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_mint_and_soulbound() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let cv = Address::generate(&env);
    let cid = env.register(ReputationNFTContract, ());
    let client = ReputationNFTContractClient::new(&env, &cid);
    client.initialize(&admin, &cv);

    let subject = Address::generate(&env);
    let id = client.mint(&admin, &subject, &100i64);
    assert_eq!(id, 0u64);
    assert_eq!(client.get_reputation(&subject).unwrap().score, 100i64);
}
