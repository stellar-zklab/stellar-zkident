#![no_std]
use asp_registry::ASPRegistryContractClient;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, String, Vec};

/// Stellar strkey addresses (both `G...` account keys and `C...` contract keys) are always
/// exactly 56 ASCII characters — 32-byte payload + 1 version byte + 2-byte checksum, base32
/// encoded. This is a protocol constant, not a guess.
const STRKEY_LEN: usize = 56;
/// Bound on `credential_type` length so leaf hashing has a fixed-size stack buffer. Generous
/// for identifiers like "kyc_tier_2" or "residency_us".
const MAX_CREDENTIAL_TYPE_LEN: usize = 64;
/// Bound on Merkle proof depth (supports trees with up to 2^32 leaves) so verification cost
/// stays predictable.
const MAX_PROOF_DEPTH: usize = 32;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofRecord {
    pub user: Address,
    pub credential_type: String,
    pub asp: Address,
    pub timestamp: u64,
    pub expiration_time: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    AspRegistry,
    Proof(Address, String),
}

#[contract]
pub struct CredentialVerifierContract;

#[contractimpl]
impl CredentialVerifierContract {
    /// `asp_registry` is stored, not taken as a per-call argument, precisely so a caller can
    /// never point verification at an attacker-controlled contract that would report back
    /// whatever root matches a fabricated proof.
    pub fn initialize(env: Env, admin: Address, asp_registry: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::AspRegistry, &asp_registry);
    }

    pub fn set_asp_registry(env: Env, admin: Address, asp_registry: Address) {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        assert_eq!(admin, stored_admin, "caller is not the admin");
        env.storage().instance().set(&DataKey::AspRegistry, &asp_registry);
    }

    /// Verifies that `user` is a member of the credential set attested by `asp`'s Merkle
    /// root — currently registered ASP roots only, fetched from `asp_registry` (never from
    /// caller input) — by walking `merkle_proof` up from a leaf this contract computes itself
    /// as `sha256(b"zkident:credential-leaf:v1:" || strkey(user) || credential_type)`.
    ///
    /// This is classical Merkle inclusion verification, not zero-knowledge proof
    /// verification: it proves `user` is one of the leaves an ASP committed to, but it does
    /// not verify any of the actual Noir circuits in `circuits/` (age_proof.nr,
    /// kyc_tier_proof.nr, membership_proof.nr) or hide *which* leaf matched. An ASP wanting
    /// real privacy-preserving disclosure still needs those circuits verified separately;
    /// this only wires the on-chain membership check up to the ASP's real, registered root.
    pub fn verify_proof(
        env: Env,
        user: Address,
        credential_type: String,
        asp: Address,
        merkle_proof: Vec<BytesN<32>>,
        leaf_index: u32,
        expiration_time: u64,
    ) -> bool {
        user.require_auth();

        let current_time = env.ledger().timestamp();
        if expiration_time > 0 && current_time > expiration_time {
            panic!("Credential proof has expired");
        }
        if (merkle_proof.len() as usize) > MAX_PROOF_DEPTH {
            panic!("Merkle proof is deeper than the supported maximum");
        }

        let asp_registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::AspRegistry)
            .expect("contract not initialized");
        let registry_client = ASPRegistryContractClient::new(&env, &asp_registry);
        let root = match registry_client.get_merkle_root(&asp) {
            Some(root) => root,
            None => return false,
        };

        let leaf = Self::compute_leaf(&env, &user, &credential_type);
        let computed_root = Self::compute_root(&env, &leaf, &merkle_proof, leaf_index);
        if computed_root != root {
            return false;
        }

        let record = ProofRecord {
            user: user.clone(),
            credential_type: credential_type.clone(),
            asp,
            timestamp: current_time,
            expiration_time,
        };

        let key = DataKey::Proof(user.clone(), credential_type.clone());
        env.storage().persistent().set(&key, &record);
        env.storage().persistent().extend_ttl(&key, 172800, 5184000);

        env.events().publish(
            (symbol_short!("verified"), user, credential_type),
            current_time,
        );

        true
    }

    pub fn has_credential(env: Env, user: Address, credential_type: String) -> bool {
        let key = DataKey::Proof(user, credential_type);
        if let Some(record) = env.storage().persistent().get::<DataKey, ProofRecord>(&key) {
            let current_time = env.ledger().timestamp();
            record.expiration_time == 0 || current_time <= record.expiration_time
        } else {
            false
        }
    }

    fn compute_leaf(env: &Env, user: &Address, credential_type: &String) -> BytesN<32> {
        let addr_str = user.to_string();
        let addr_len = addr_str.len() as usize;
        if addr_len != STRKEY_LEN {
            panic!("unexpected address strkey length");
        }
        let mut addr_buf = [0u8; STRKEY_LEN];
        addr_str.copy_into_slice(&mut addr_buf);

        let ct_len = credential_type.len() as usize;
        if ct_len > MAX_CREDENTIAL_TYPE_LEN {
            panic!("credential_type is longer than the supported maximum");
        }
        let mut ct_buf = [0u8; MAX_CREDENTIAL_TYPE_LEN];
        credential_type.copy_into_slice(&mut ct_buf[..ct_len]);

        let mut data = Bytes::from_slice(env, b"zkident:credential-leaf:v1:");
        data.append(&Bytes::from_slice(env, &addr_buf));
        data.append(&Bytes::from_slice(env, &ct_buf[..ct_len]));

        let hash = env.crypto().sha256(&data).to_array();
        BytesN::from_array(env, &hash)
    }

    fn compute_root(
        env: &Env,
        leaf: &BytesN<32>,
        proof: &Vec<BytesN<32>>,
        leaf_index: u32,
    ) -> BytesN<32> {
        let mut current = leaf.clone();
        let mut index = leaf_index;
        for sibling in proof.iter() {
            let mut combined = Bytes::new(env);
            if index % 2 == 0 {
                combined.append(&Bytes::from_array(env, &current.to_array()));
                combined.append(&Bytes::from_array(env, &sibling.to_array()));
            } else {
                combined.append(&Bytes::from_array(env, &sibling.to_array()));
                combined.append(&Bytes::from_array(env, &current.to_array()));
            }
            let hash = env.crypto().sha256(&combined).to_array();
            current = BytesN::from_array(env, &hash);
            index /= 2;
        }
        current
    }
}

mod test;
