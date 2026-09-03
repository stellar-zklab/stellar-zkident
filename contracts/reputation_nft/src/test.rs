#![cfg(test)]
use super::*;
use asp_registry::{ASPRegistryContract, ASPRegistryContractClient};
use credential_verifier::{CredentialVerifierContract, CredentialVerifierContractClient};
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

/// Mirrors `credential_verifier::compute_leaf` exactly (that function is private to its own
/// crate, so a test in a different crate has to reproduce the same real algorithm to build a
/// proof the real deployed contract will actually accept — see that contract's doc comment
/// for the canonical definition this must stay in sync with).
fn compute_leaf(env: &Env, user: &Address, credential_type: &String) -> BytesN<32> {
    let addr_str = user.to_string();
    let mut addr_buf = [0u8; 56];
    addr_str.copy_into_slice(&mut addr_buf);
    let ct_len = credential_type.len() as usize;
    let mut ct_buf = [0u8; 64];
    credential_type.copy_into_slice(&mut ct_buf[..ct_len]);

    let mut data = Bytes::from_slice(env, b"zkident:credential-leaf:v1:");
    data.append(&Bytes::from_slice(env, &addr_buf));
    data.append(&Bytes::from_slice(env, &ct_buf[..ct_len]));
    BytesN::from_array(env, &env.crypto().sha256(&data).to_array())
}

fn combine(env: &Env, index: u32, node: &BytesN<32>, sibling: &BytesN<32>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    if index % 2 == 0 {
        data.append(&Bytes::from_array(env, &node.to_array()));
        data.append(&Bytes::from_array(env, &sibling.to_array()));
    } else {
        data.append(&Bytes::from_array(env, &sibling.to_array()));
        data.append(&Bytes::from_array(env, &node.to_array()));
    }
    BytesN::from_array(env, &env.crypto().sha256(&data).to_array())
}

struct Fixture {
    reputation: ReputationNFTContractClient<'static>,
    verifier: CredentialVerifierContractClient<'static>,
    registry: ASPRegistryContractClient<'static>,
    admin: Address,
}

fn setup(env: &Env) -> Fixture {
    let admin = Address::generate(env);

    let registry_id = env.register(ASPRegistryContract, ());
    let registry = ASPRegistryContractClient::new(env, &registry_id);
    registry.initialize(&admin);

    let verifier_id = env.register(CredentialVerifierContract, ());
    let verifier = CredentialVerifierContractClient::new(env, &verifier_id);
    verifier.initialize(&admin, &registry_id);

    let reputation_id = env.register(ReputationNFTContract, ());
    let reputation = ReputationNFTContractClient::new(env, &reputation_id);
    reputation.initialize(&admin, &verifier_id);

    Fixture { reputation, verifier, registry, admin }
}

/// Registers a real 4-leaf Merkle tree with `asp` for `subject`'s `credential_type`, and
/// actually calls `verify_proof` so `subject` genuinely holds a verified credential
/// afterward — not a shortcut, the same path a real ASP + user would go through.
fn give_subject_a_real_verified_credential(
    env: &Env,
    fixture: &Fixture,
    asp: &Address,
    subject: &Address,
    credential_type: &String,
) {
    let leaf0 = compute_leaf(env, subject, credential_type);
    let leaves = [
        leaf0,
        BytesN::from_array(env, &[1u8; 32]),
        BytesN::from_array(env, &[2u8; 32]),
        BytesN::from_array(env, &[3u8; 32]),
    ];
    let h01 = combine(env, 0, &leaves[0], &leaves[1]);
    let h23 = combine(env, 2, &leaves[2], &leaves[3]);
    let root = combine(env, 0, &h01, &h23);

    fixture.registry.register_asp(&fixture.admin, asp, &root);

    let proof = Vec::from_array(env, [leaves[1].clone(), h23]);
    let ok = fixture.verifier.verify_proof(subject, credential_type, asp, &proof, &0u32, &0u64);
    assert!(ok, "test setup: the real Merkle proof must verify");
}

#[test]
fn test_mint_accepts_a_subject_with_a_real_verified_credential() {
    let env = Env::default();
    env.mock_all_auths();
    let fixture = setup(&env);
    let asp = Address::generate(&env);
    let subject = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");

    give_subject_a_real_verified_credential(&env, &fixture, &asp, &subject, &credential_type);

    let id = fixture.reputation.mint(&fixture.admin, &subject, &100i64, &credential_type);
    assert_eq!(id, 0u64);
    assert_eq!(fixture.reputation.get_reputation(&subject).unwrap().score, 100i64);
}

#[test]
#[should_panic(expected = "subject does not hold a verified credential")]
fn test_mint_rejects_a_subject_with_no_verified_credential() {
    let env = Env::default();
    env.mock_all_auths();
    let fixture = setup(&env);
    let subject = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");

    // No ASP registered, no proof ever verified for `subject` — mint must not just trust
    // the admin's say-so.
    fixture.reputation.mint(&fixture.admin, &subject, &100i64, &credential_type);
}

#[test]
#[should_panic]
fn test_transfer_always_reverts_soulbound() {
    let env = Env::default();
    env.mock_all_auths();
    let fixture = setup(&env);
    let asp = Address::generate(&env);
    let subject = Address::generate(&env);
    let other = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");
    give_subject_a_real_verified_credential(&env, &fixture, &asp, &subject, &credential_type);
    let id = fixture.reputation.mint(&fixture.admin, &subject, &100i64, &credential_type);

    fixture.reputation.transfer(&subject, &other, &id);
}
