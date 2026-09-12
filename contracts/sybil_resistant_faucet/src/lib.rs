#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, String, Symbol};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    CredentialVerifier,
    Token,
    ClaimAmount,
    RequiredCredentialType,
    Claimed(Address),
}

#[contract]
pub struct SybilResistantFaucetContract;

/// A faucet/airdrop primitive that pays out a fixed amount of a real token, at most once per
/// address, gated on that address genuinely holding a verified zkident credential —
/// checked live via a real cross-contract call to `credential_verifier`, not assumed. This is
/// the standard "prevent bots from draining a giveaway with infinite addresses" pattern,
/// reusing `credential_verifier`'s existing `has_credential` the exact same way
/// `reputation_nft::mint` already does, so any project pointing this at their own (or our)
/// `credential_verifier` instance gets Sybil resistance without writing any proof logic
/// itself.
#[contractimpl]
impl SybilResistantFaucetContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        credential_verifier: Address,
        token: Address,
        claim_amount: i128,
        required_credential_type: String,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CredentialVerifier, &credential_verifier);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::ClaimAmount, &claim_amount);
        env.storage()
            .instance()
            .set(&DataKey::RequiredCredentialType, &required_credential_type);
    }

    pub fn set_claim_amount(env: Env, admin: Address, claim_amount: i128) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        assert_eq!(admin, stored_admin, "caller is not the admin");
        env.storage().instance().set(&DataKey::ClaimAmount, &claim_amount);
    }

    /// Pays `claim_amount` of `token` to `user`, but only once per address, and only if
    /// `user` genuinely holds a verified `required_credential_type` credential according to
    /// `credential_verifier` — checked for real via a cross-contract call. Without this gate,
    /// a bot could drain the faucet with an unbounded number of fresh addresses; the gate
    /// makes each claim cost a real, already-verified identity instead of just a keypair.
    ///
    /// Uses a raw `env.invoke_contract` rather than a typed client, the same reasoning as
    /// `reputation_nft::mint`'s own cross-contract call to this same function: depending on
    /// `credential_verifier`'s crate directly would link its `#[contract]` code into this
    /// contract's wasm and collide with this contract's own exports (see that contract's
    /// Cargo.toml for the full account of that real bug).
    pub fn claim(env: Env, user: Address) -> i128 {
        user.require_auth();

        let claimed_key = DataKey::Claimed(user.clone());
        if env.storage().persistent().has(&claimed_key) {
            panic!("this address has already claimed from this faucet");
        }

        let cv: Address = env
            .storage()
            .instance()
            .get(&DataKey::CredentialVerifier)
            .expect("contract not initialized");
        let required_type: String = env
            .storage()
            .instance()
            .get(&DataKey::RequiredCredentialType)
            .expect("contract not initialized");
        let verified: bool = env.invoke_contract(
            &cv,
            &Symbol::new(&env, "has_credential"),
            soroban_sdk::Vec::from_array(&env, [user.to_val(), required_type.to_val()]),
        );
        if !verified {
            panic!("claim requires a verified credential — this address does not hold one");
        }

        let amount: i128 = env.storage().instance().get(&DataKey::ClaimAmount).expect("contract not initialized");
        let token_id: Address = env.storage().instance().get(&DataKey::Token).expect("contract not initialized");
        token::Client::new(&env, &token_id).transfer(&env.current_contract_address(), &user, &amount);

        env.storage().persistent().set(&claimed_key, &true);
        env.storage().persistent().extend_ttl(&claimed_key, 172800, 5184000);

        env.events().publish((symbol_short!("claimed"),), (user, amount));
        amount
    }

    pub fn has_claimed(env: Env, user: Address) -> bool {
        env.storage().persistent().has(&DataKey::Claimed(user))
    }

    pub fn get_claim_amount(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::ClaimAmount).expect("contract not initialized")
    }
}

mod test;
