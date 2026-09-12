#![cfg(test)]
use super::*;
use asp_registry::{ASPRegistryContract, ASPRegistryContractClient};
use credential_verifier::{CredentialVerifierContract, CredentialVerifierContractClient};
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Bytes, BytesN, Env, Vec,
};

/// Mirrors `credential_verifier::compute_leaf` exactly — see reputation_nft's own test.rs for
/// the same duplication and the same reasoning (that function is private to its crate).
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
    faucet: SybilResistantFaucetContractClient<'static>,
    verifier: CredentialVerifierContractClient<'static>,
    registry: ASPRegistryContractClient<'static>,
    admin: Address,
    token: Address,
    credential_type: String,
}

const CLAIM_AMOUNT: i128 = 5_000_000; // 0.5 XLM-equivalent units, matching the fixed-amount
                                       // convention used elsewhere in the ecosystem (see
                                       // stellar-zkstream's 0.5 XLM fixed stream amount).

fn setup(env: &Env) -> Fixture {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    let credential_type = String::from_str(env, "kyc_tier_2");

    let registry_id = env.register(ASPRegistryContract, ());
    let registry = ASPRegistryContractClient::new(env, &registry_id);
    registry.initialize(&admin);

    let verifier_id = env.register(CredentialVerifierContract, ());
    let verifier = CredentialVerifierContractClient::new(env, &verifier_id);
    verifier.initialize(&admin, &registry_id);

    let faucet_id = env.register(SybilResistantFaucetContract, ());
    let faucet = SybilResistantFaucetContractClient::new(env, &faucet_id);
    faucet.initialize(&admin, &verifier_id, &token, &CLAIM_AMOUNT, &credential_type);

    // Fund the faucet with a real token balance, the same way it would be funded on testnet.
    StellarAssetClient::new(env, &token).mint(&faucet_id, &(CLAIM_AMOUNT * 10));

    Fixture { faucet, verifier, registry, admin, token, credential_type }
}

/// Registers a real 4-leaf Merkle tree with `asp` for `subject`'s `credential_type`, and
/// actually calls `verify_proof` so `subject` genuinely holds a verified credential
/// afterward — the same real path reputation_nft's tests use, not a shortcut.
fn give_subject_a_real_verified_credential(env: &Env, fixture: &Fixture, asp: &Address, subject: &Address) {
    let leaf0 = compute_leaf(env, subject, &fixture.credential_type);
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
    let ok = fixture.verifier.verify_proof(subject, &fixture.credential_type, asp, &proof, &0u32, &0u64);
    assert!(ok, "test setup: the real Merkle proof must verify");
}

#[test]
fn test_claim_pays_out_real_tokens_to_a_verified_subject() {
    let env = Env::default();
    env.mock_all_auths();
    let fixture = setup(&env);
    let asp = Address::generate(&env);
    let subject = Address::generate(&env);
    give_subject_a_real_verified_credential(&env, &fixture, &asp, &subject);

    let token_client = soroban_sdk::token::TokenClient::new(&env, &fixture.token);
    assert_eq!(token_client.balance(&subject), 0);

    let paid = fixture.faucet.claim(&subject);
    assert_eq!(paid, CLAIM_AMOUNT);
    assert_eq!(token_client.balance(&subject), CLAIM_AMOUNT);
    assert!(fixture.faucet.has_claimed(&subject));
}

#[test]
#[should_panic(expected = "claim requires a verified credential")]
fn test_claim_rejects_an_address_with_no_verified_credential() {
    let env = Env::default();
    env.mock_all_auths();
    let fixture = setup(&env);
    let stranger = Address::generate(&env);

    // No ASP registered, no proof ever verified for `stranger` — claim must not just pay out
    // to anyone who asks.
    fixture.faucet.claim(&stranger);
}

#[test]
#[should_panic(expected = "already claimed")]
fn test_claim_rejects_a_second_claim_from_the_same_verified_subject() {
    let env = Env::default();
    env.mock_all_auths();
    let fixture = setup(&env);
    let asp = Address::generate(&env);
    let subject = Address::generate(&env);
    give_subject_a_real_verified_credential(&env, &fixture, &asp, &subject);

    fixture.faucet.claim(&subject);
    fixture.faucet.claim(&subject);
}
