# stellar-zkident 🔐🪪

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
![Build](https://img.shields.io/badge/Cargo_Test-Passing-brightgreen)
![DID](https://img.shields.io/badge/Standard-W3C_DID_v1.0-violet)

Self-Sovereign `did:stellar:` Decentralized Identity, Noir UltraPlonk ZK Credentials, and Soulbound Reputation (SBT) Framework.

## Current Status — what's real vs. not

**`contracts/did_registry` — real.** Full DID document CRUD: register, add verification keys, update, deactivate, resolve. No shortcuts.

**`contracts/reputation_nft` — real.** Mint, read, and a soulbound `transfer()` that correctly always reverts (non-transferable by design).

**`contracts/asp_registry` — real, but unused by anything else.** Stores a Merkle root per registered Attestation Service Provider. Nothing currently reads from it — see below.

**`contracts/credential_verifier` — the name is misleading right now.** `verify_proof()` does **not** perform any cryptographic verification. It accepts `proof_hash` as an opaque string, stores it, and returns `true`. It doesn't call into the real Noir circuits in `circuits/membership_proof` or `circuits/age_proof`, and doesn't check anything against the Merkle roots stored in `asp_registry`. As it stands, anyone can get `has_credential()` to return `true` for any credential type by submitting an arbitrary string — there's no actual identity or credential guarantee yet.

**`circuits/` — real Noir source, not yet connected to anything on-chain.**

## 🚀 Quick Start
```bash
cargo test --all --features testutils
cd frontend && npm run dev
```
