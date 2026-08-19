#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

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
