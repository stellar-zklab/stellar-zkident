#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

#[test]
fn test_verify_and_has_credential() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let zk = Address::generate(&env);
    let did = Address::generate(&env);
    let cid = env.register(CredentialVerifierContract, ());
    let client = CredentialVerifierContractClient::new(&env, &cid);
    client.initialize(&admin, &zk, &did);

    let issuer = Address::generate(&env);
    client.register_issuer(&admin, &issuer);

    let subject = Address::generate(&env);
    client.verify_credential_proof(&subject, &CredentialType::Age, &Bytes::new(&env), &Vec::new(&env), &issuer, &0u64);
    assert!(client.has_credential(&subject, &CredentialType::Age));
}
