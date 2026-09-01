#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofRecord {
    pub user: Address,
    pub credential_type: String,
    pub proof_hash: String,
    pub timestamp: u64,
    pub expiration_time: u64,
}

#[contracttype]
pub enum DataKey {
    Proof(Address, String),
}

#[contract]
pub struct CredentialVerifierContract;

#[contractimpl]
impl CredentialVerifierContract {
    /// NOTE — despite the name, this does NOT perform any zero-knowledge or cryptographic
    /// verification. `proof_hash` is accepted as an opaque string and stored as-is; nothing
    /// checks it against the actual Noir circuits in `circuits/membership_proof` or
    /// `circuits/age_proof`, and nothing checks it against `asp_registry`'s stored Merkle
    /// roots either — the two are currently disconnected. Any caller can currently get
    /// `has_credential()` to return true for any `credential_type` by calling this with an
    /// arbitrary string. Real proof verification (checking `proof_hash` against a Noir/PLONK
    /// verifier and an ASP's registered Merkle root) is unimplemented.
    pub fn verify_proof(
        env: Env,
        user: Address,
        credential_type: String,
        proof_hash: String,
        expiration_time: u64,
    ) -> bool {
        user.require_auth();

        let current_time = env.ledger().timestamp();
        if expiration_time > 0 && current_time > expiration_time {
            panic!("Credential proof has expired");
        }

        let record = ProofRecord {
            user: user.clone(),
            credential_type: credential_type.clone(),
            proof_hash,
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
}
