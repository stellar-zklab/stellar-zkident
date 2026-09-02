#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env};

#[derive(Clone)]
#[contracttype]
pub struct ASPRecord {
    pub asp: Address,
    pub merkle_root: BytesN<32>,
    pub active: bool,
    pub registered_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey { Admin, ASP(Address) }

#[contract]
pub struct ASPRegistryContract;

#[contractimpl]
impl ASPRegistryContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn register_asp(env: Env, admin: Address, asp: Address, merkle_root: BytesN<32>) {
        admin.require_auth();
        let record = ASPRecord { asp: asp.clone(), merkle_root, active: true, registered_at: env.ledger().timestamp() };
        env.storage().persistent().set(&DataKey::ASP(asp.clone()), &record);
        env.events().publish((symbol_short!("asp"), symbol_short!("register")), asp);
    }

    pub fn is_registered_asp(env: Env, asp: Address) -> bool {
        env.storage().persistent().get::<DataKey, ASPRecord>(&DataKey::ASP(asp)).map(|r| r.active).unwrap_or(false)
    }

    /// Returns the ASP's currently registered Merkle root, or `None` if the ASP is
    /// unregistered or has been deactivated. This is the value other contracts (e.g.
    /// `credential_verifier`) must call to get a trustworthy root — it must never be
    /// accepted as a caller-supplied argument, since that would let anyone point at an
    /// attacker-controlled root instead of this registry's real one.
    pub fn get_merkle_root(env: Env, asp: Address) -> Option<BytesN<32>> {
        env.storage()
            .persistent()
            .get::<DataKey, ASPRecord>(&DataKey::ASP(asp))
            .and_then(|r| if r.active { Some(r.merkle_root) } else { None })
    }
}

mod test;
