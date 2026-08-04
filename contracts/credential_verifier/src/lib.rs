#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Bytes, BytesN, Env, Vec,
};

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum CredentialType {
    Age,
    KYCBronze,
    KYCSilver,
    KYCGold,
    Residency,
    Employment,
    ASPMembership,
}

#[derive(Clone)]
#[contracttype]
pub struct CredentialRecord {
    pub subject: Address,
    pub credential_type: CredentialType,
    pub issuer: Address,
    pub verified_at: u64,
    pub expires_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    ZkVerifier,
    DIDRegistry,
    TrustedIssuer(Address),
    Credential(Address, CredentialType),
}

#[contract]
pub struct CredentialVerifierContract;

#[contractimpl]
impl CredentialVerifierContract {
    pub fn initialize(env: Env, admin: Address, zk_verifier: Address, did_registry: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ZkVerifier, &zk_verifier);
        env.storage().instance().set(&DataKey::DIDRegistry, &did_registry);
    }

    pub fn register_issuer(env: Env, admin: Address, issuer: Address) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        assert!(admin == stored, "unauthorized");
        env.storage().instance().set(&DataKey::TrustedIssuer(issuer), &true);
    }

    pub fn verify_credential_proof(
        env: Env,
        subject: Address,
        credential_type: CredentialType,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
        issuer: Address,
        expires_at: u64,
    ) -> bool {
        subject.require_auth();
        assert!(env.storage().instance().has(&DataKey::TrustedIssuer(issuer.clone())), "issuer not trusted");

        let record = CredentialRecord {
            subject: subject.clone(),
            credential_type: credential_type.clone(),
            issuer,
            verified_at: env.ledger().timestamp(),
            expires_at,
        };
        env.storage().persistent().set(&DataKey::Credential(subject.clone(), credential_type.clone()), &record);
        env.events().publish((symbol_short!("cred"), symbol_short!("verified")), (subject, credential_type));
        true
    }

    pub fn has_credential(env: Env, subject: Address, credential_type: CredentialType) -> bool {
        let key = DataKey::Credential(subject, credential_type);
        if let Some(record) = env.storage().persistent().get::<DataKey, CredentialRecord>(&key) {
            if record.expires_at == 0 { return true; }
            return env.ledger().timestamp() < record.expires_at;
        }
        false
    }
}

mod test;
