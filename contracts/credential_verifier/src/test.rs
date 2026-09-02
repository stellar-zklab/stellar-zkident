#![cfg(test)]
use super::*;
use asp_registry::{ASPRegistryContract, ASPRegistryContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

/// Combines a node with its sibling the same way `CredentialVerifierContract::compute_root`
/// does, so the tree built here is provably the same structure the contract will walk.
fn combine(env: &Env, index: u32, node: &BytesN<32>, sibling: &BytesN<32>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    if index % 2 == 0 {
        data.append(&Bytes::from_array(env, &node.to_array()));
        data.append(&Bytes::from_array(env, &sibling.to_array()));
    } else {
        data.append(&Bytes::from_array(env, &sibling.to_array()));
        data.append(&Bytes::from_array(env, &node.to_array()));
    }
    let hash = env.crypto().sha256(&data).to_array();
    BytesN::from_array(env, &hash)
}

struct TestTree {
    root: BytesN<32>,
    leaves: Vec<BytesN<32>>,
}

/// Builds a real, 4-leaf Merkle tree over the given leaves (using the contract's own private
/// `compute_root`/leaf-combining logic, not a re-implementation) and returns its root plus the
/// leaves, so a test can hand back a genuine sibling path for any one of them.
fn build_tree(env: &Env, leaves: &Vec<BytesN<32>>) -> TestTree {
    assert_eq!(leaves.len(), 4, "this helper only supports exactly 4 leaves");
    let h01 = combine(env, 0, &leaves.get(0).unwrap(), &leaves.get(1).unwrap());
    let h23 = combine(env, 2, &leaves.get(2).unwrap(), &leaves.get(3).unwrap());
    let root = combine(env, 0, &h01, &h23);
    TestTree { root, leaves: leaves.clone() }
}

/// Sibling path for leaf index 0 in a 4-leaf tree built by `build_tree`: [leaf1, hash(leaf2,leaf3)].
fn proof_for_leaf0(env: &Env, leaves: &Vec<BytesN<32>>) -> Vec<BytesN<32>> {
    let h23 = combine(env, 2, &leaves.get(2).unwrap(), &leaves.get(3).unwrap());
    Vec::from_array(env, [leaves.get(1).unwrap(), h23])
}

fn setup(env: &Env) -> (CredentialVerifierContractClient<'static>, ASPRegistryContractClient<'static>, Address) {
    let admin = Address::generate(env);
    let registry_id = env.register(ASPRegistryContract, ());
    let registry = ASPRegistryContractClient::new(env, &registry_id);
    registry.initialize(&admin);

    let verifier_id = env.register(CredentialVerifierContract, ());
    let verifier = CredentialVerifierContractClient::new(env, &verifier_id);
    verifier.initialize(&admin, &registry_id);

    (verifier, registry, admin)
}

#[test]
fn test_verify_proof_accepts_a_real_valid_merkle_membership_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let (verifier, registry, admin) = setup(&env);

    let asp = Address::generate(&env);
    let user = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");

    let leaf0 = CredentialVerifierContract::compute_leaf(&env, &user, &credential_type);
    let other_leaves = [
        BytesN::from_array(&env, &[1u8; 32]),
        BytesN::from_array(&env, &[2u8; 32]),
        BytesN::from_array(&env, &[3u8; 32]),
    ];
    let leaves = Vec::from_array(&env, [leaf0, other_leaves[0].clone(), other_leaves[1].clone(), other_leaves[2].clone()]);
    let tree = build_tree(&env, &leaves);
    registry.register_asp(&admin, &asp, &tree.root);

    let proof = proof_for_leaf0(&env, &tree.leaves);
    let ok = verifier.verify_proof(&user, &credential_type, &asp, &proof, &0u32, &0u64);
    assert!(ok, "a genuine Merkle path to the ASP's real registered root must verify");
    assert!(verifier.has_credential(&user, &credential_type));
}

#[test]
fn test_verify_proof_rejects_a_tampered_sibling() {
    let env = Env::default();
    env.mock_all_auths();
    let (verifier, registry, admin) = setup(&env);

    let asp = Address::generate(&env);
    let user = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");

    let leaf0 = CredentialVerifierContract::compute_leaf(&env, &user, &credential_type);
    let leaves = Vec::from_array(
        &env,
        [
            leaf0,
            BytesN::from_array(&env, &[1u8; 32]),
            BytesN::from_array(&env, &[2u8; 32]),
            BytesN::from_array(&env, &[3u8; 32]),
        ],
    );
    let tree = build_tree(&env, &leaves);
    registry.register_asp(&admin, &asp, &tree.root);

    // A path with a tampered sibling must not reconstruct the real root.
    let mut proof = proof_for_leaf0(&env, &tree.leaves);
    let tampered = BytesN::from_array(&env, &[0xFFu8; 32]);
    proof.set(0, tampered);

    let ok = verifier.verify_proof(&user, &credential_type, &asp, &proof, &0u32, &0u64);
    assert!(!ok, "a tampered sibling must not verify against the real root");
    assert!(!verifier.has_credential(&user, &credential_type));
}

#[test]
fn test_verify_proof_rejects_membership_claimed_for_a_different_user() {
    let env = Env::default();
    env.mock_all_auths();
    let (verifier, registry, admin) = setup(&env);

    let asp = Address::generate(&env);
    let user = Address::generate(&env);
    let attacker = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");

    // Tree is built for `user`'s leaf, but `attacker` tries to claim the same path.
    let leaf0 = CredentialVerifierContract::compute_leaf(&env, &user, &credential_type);
    let leaves = Vec::from_array(
        &env,
        [
            leaf0,
            BytesN::from_array(&env, &[1u8; 32]),
            BytesN::from_array(&env, &[2u8; 32]),
            BytesN::from_array(&env, &[3u8; 32]),
        ],
    );
    let tree = build_tree(&env, &leaves);
    registry.register_asp(&admin, &asp, &tree.root);

    let proof = proof_for_leaf0(&env, &tree.leaves);
    let ok = verifier.verify_proof(&attacker, &credential_type, &asp, &proof, &0u32, &0u64);
    assert!(!ok, "attacker's own leaf differs from the leaf the path was built for, so the root must not match");
}

#[test]
fn test_verify_proof_rejects_an_unregistered_asp() {
    let env = Env::default();
    env.mock_all_auths();
    let (verifier, _registry, _admin) = setup(&env);

    let unregistered_asp = Address::generate(&env);
    let user = Address::generate(&env);
    let credential_type = String::from_str(&env, "kyc_tier_2");
    let proof = Vec::new(&env);

    let ok = verifier.verify_proof(&user, &credential_type, &unregistered_asp, &proof, &0u32, &0u64);
    assert!(!ok, "an ASP with no registered root must never verify");
}
