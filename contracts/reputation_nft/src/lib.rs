#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[derive(Clone)]
#[contracttype]
pub struct ReputationData {
    pub owner: Address,
    pub score: i64,
    pub token_id: u64,
    pub minted_at: u64,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey { Admin, CredentialVerifier, Reputation(Address), TokenCount }

#[contract]
pub struct ReputationNFTContract;

#[contractimpl]
impl ReputationNFTContract {
    pub fn initialize(env: Env, admin: Address, cv: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CredentialVerifier, &cv);
        env.storage().instance().set(&DataKey::TokenCount, &0u64);
    }

    pub fn mint(env: Env, admin: Address, subject: Address, initial_score: i64) -> u64 {
        admin.require_auth();
        let count: u64 = env.storage().instance().get(&DataKey::TokenCount).unwrap_or(0);
        let now = env.ledger().timestamp();
        let data = ReputationData { owner: subject.clone(), score: initial_score, token_id: count, minted_at: now, updated_at: now };
        env.storage().persistent().set(&DataKey::Reputation(subject.clone()), &data);
        env.storage().instance().set(&DataKey::TokenCount, &(count + 1));
        env.events().publish((symbol_short!("rep"), symbol_short!("minted")), (count, subject, initial_score));
        count
    }

    pub fn get_reputation(env: Env, subject: Address) -> Option<ReputationData> {
        env.storage().persistent().get(&DataKey::Reputation(subject))
    }

    pub fn transfer(_env: Env, _from: Address, _to: Address, _token_id: u64) {
        panic!("soulbound: transfer not allowed");
    }
}

mod test;
