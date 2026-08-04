#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

#[derive(Clone)]
#[contracttype]
pub struct DIDRecord {
    pub owner: Address,
    pub document: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    DID(Address),
    Admin,
}

#[contract]
pub struct DIDRegistryContract;

#[contractimpl]
impl DIDRegistryContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn register_did(env: Env, owner: Address, document: String) -> String {
        owner.require_auth();
        assert!(
            !env.storage().persistent().has(&DataKey::DID(owner.clone())),
            "DID already registered"
        );

        let now = env.ledger().timestamp();
        let record = DIDRecord {
            owner: owner.clone(),
            document: document.clone(),
            active: true,
            created_at: now,
            updated_at: now,
        };
        env.storage().persistent().set(&DataKey::DID(owner.clone()), &record);

        env.events().publish(
            (symbol_short!("did"), symbol_short!("register")),
            owner,
        );
        document
    }

    pub fn update_did(env: Env, owner: Address, document: String) {
        owner.require_auth();
        let mut record: DIDRecord = env.storage()
            .persistent()
            .get(&DataKey::DID(owner.clone()))
            .expect("DID not found");
        assert!(record.active, "DID is deactivated");

        record.document = document;
        record.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::DID(owner.clone()), &record);

        env.events().publish(
            (symbol_short!("did"), symbol_short!("update")),
            owner,
        );
    }

    pub fn deactivate_did(env: Env, owner: Address) {
        owner.require_auth();
        let mut record: DIDRecord = env.storage()
            .persistent()
            .get(&DataKey::DID(owner.clone()))
            .expect("DID not found");
        record.active = false;
        record.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::DID(owner.clone()), &record);

        env.events().publish(
            (symbol_short!("did"), symbol_short!("deact")),
            owner,
        );
    }

    pub fn resolve_did(env: Env, address: Address) -> Option<DIDRecord> {
        env.storage().persistent().get(&DataKey::DID(address))
    }

    pub fn is_active(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, DIDRecord>(&DataKey::DID(address))
            .map(|r| r.active)
            .unwrap_or(false)
    }
}

mod test;
