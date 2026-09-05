#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

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

    /// Mints a soulbound reputation token for `subject`, but only if `subject` actually
    /// holds a verified `credential_type` credential according to `credential_verifier` —
    /// checked for real via a cross-contract call, not assumed. Without this, reputation
    /// could be minted for anyone regardless of whether they ever verified anything, which
    /// would make the whole "reputation" claim meaningless.
    ///
    /// Uses a raw `env.invoke_contract` rather than a typed client: depending on
    /// `credential_verifier`'s crate directly would link its own `#[contract]` code into
    /// this contract's wasm, which is exactly what caused `credential_verifier`'s own
    /// exports to collide and vanish from its compiled wasm on a real testnet deploy
    /// earlier — see that contract's `Cargo.toml` for the full account of that bug.
    pub fn mint(
        env: Env,
        admin: Address,
        subject: Address,
        initial_score: i64,
        credential_type: String,
    ) -> u64 {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        assert_eq!(admin, stored_admin, "caller is not the admin");

        let cv: Address = env
            .storage()
            .instance()
            .get(&DataKey::CredentialVerifier)
            .expect("contract not initialized");
        let has_credential: bool = env.invoke_contract(
            &cv,
            &Symbol::new(&env, "has_credential"),
            Vec::from_array(&env, [subject.to_val(), credential_type.to_val()]),
        );
        if !has_credential {
            panic!("subject does not hold a verified credential of this type");
        }

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
