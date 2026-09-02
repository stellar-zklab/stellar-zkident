# stellar-zkident Deployment Guide

Deploys `asp_registry`, `credential_verifier`, `did_registry`, and `reputation_nft` to
Stellar testnet, and wires up their real cross-contract dependencies. All four pass their
real test suite (`cargo test --all --features testutils`, see the repo README) before this
guide is relevant — deployment doesn't substitute for that.

## Prerequisites
- **Stellar CLI**: `cargo install --locked stellar-cli`
- **Rust Wasm target**: `rustup target add wasm32v1-none`

## Network
- **Network**: `testnet`
- **RPC URL**: `https://soroban-testnet.stellar.org:443`
- **Passphrase**: `"Test SDF Network ; September 2015"`

## Deploy

```bash
bash scripts/deploy.sh
```

This generates and friendbot-funds a `deployer` testnet identity if one doesn't already
exist, builds all four contracts, and deploys + initializes them in dependency order:

1. `asp_registry.initialize(admin)`
2. `credential_verifier.initialize(admin, asp_registry=<asp_registry's real deployed ID>)`
   — this is the address the deployed `credential_verifier` will actually call
   `get_merkle_root()` on; it's not a placeholder.
3. `did_registry.initialize(admin)`
4. `reputation_nft.initialize(admin, cv=<credential_verifier's real deployed ID>)`

Resulting contract IDs are written to `deployments/testnet.json`.

## What this does NOT set up

No ASP is registered against `asp_registry` by this script — `register_asp(admin, asp,
merkle_root)` needs a real Merkle root built by an actual attestation service over real
credential leaves, which is application-level setup, not deployment. Until an ASP is
registered, `credential_verifier.verify_proof()` will correctly reject every proof (see the
`test_verify_proof_rejects_an_unregistered_asp` test) — that's the contract behaving
correctly, not a deployment bug.
