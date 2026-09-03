# stellar-zkident 🔐🪪

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
![Build](https://img.shields.io/badge/Cargo_Test-Passing-brightgreen)
![DID](https://img.shields.io/badge/Standard-W3C_DID_v1.0-violet)

Self-Sovereign `did:stellar:` Decentralized Identity, Noir UltraPlonk ZK Credentials, and Soulbound Reputation (SBT) Framework.

## Current Status — what's real vs. not

**`contracts/did_registry` — real.** Full DID document CRUD: register, add verification keys, update, deactivate, resolve. No shortcuts.

**`contracts/reputation_nft` — real, and now actually gated by a verified credential.** `mint()` used to accept an admin's say-so alone — it stored `credential_verifier`'s address at `initialize()` but never called it, so reputation could be minted for anyone regardless of whether they'd ever verified anything. It now calls `credential_verifier.has_credential(subject, credential_type)` for real (via a raw `env.invoke_contract`, not a crate dependency — see `credential_verifier`'s own note on why) and rejects the mint if that comes back false. `get_reputation()` and a soulbound `transfer()` that correctly always reverts (non-transferable by design) are unchanged.

**`contracts/asp_registry` — real.** Stores a Merkle root per registered Attestation Service Provider, plus `get_merkle_root()` for other contracts to read it. Now actually consumed by `credential_verifier` — see below.

**`contracts/credential_verifier` — real Merkle membership verification, not a ZK proof.** `verify_proof()` now performs genuine cryptographic verification: it computes a leaf as `sha256(b"zkident:credential-leaf:v1:" || strkey(user) || credential_type)`, walks a caller-supplied sibling path up to a root, and rejects unless that root matches the ASP's *currently registered* root — fetched directly from `asp_registry` (stored at `initialize()`, never taken as caller input, so a proof can't be checked against an attacker-controlled fake registry). This proves on-chain that `user` is one of the leaves a specific ASP committed to. It is **not** zero-knowledge: the leaf is derived from the caller's real address, so membership is not hidden, and it still doesn't call into the real Noir circuits in `circuits/membership_proof` or `circuits/age_proof` — those would provide the actual privacy-preserving disclosure and remain unconnected. What changed is that `has_credential()` can no longer be made `true` by submitting an arbitrary string; it now requires a real path to the ASP's real root.

**`circuits/` — real Noir source, not yet connected to anything on-chain.**

## Deployment

All four contracts are live on Stellar testnet (deployed 2026-09-03, see
[`deployments/testnet.json`](deployments/testnet.json) — independently checkable on
[stellar.expert](https://stellar.expert/explorer/testnet)):

| Contract | Address |
|---|---|
| `asp_registry` | `CD5BJLK36ZHKLHRE4L3LUDEEM5SM2ME5XLALIIGTCZ6OTMXJSPLWR75C` |
| `credential_verifier` | `CAHGQ3OH2MSJBVQ4DGDPBX7VPROCKVB2YA32QCLIPYX2766WZJPJCWTA` |
| `did_registry` | `CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6` |
| `reputation_nft` | `CCP6BTCMMYBKEZ2T2DL32JSFSEVPGLWXMNXWPXEHNSTGTPC7BDU4Z5AT` |

`credential_verifier` is initialized with `asp_registry`'s real deployed address above, and
`reputation_nft` with `credential_verifier`'s — these aren't independently deployed
instances that merely coexist, they're actually wired to each other on-chain.
`scripts/deploy.sh` reproduces this from scratch — see
[`docs/DEPLOYMENT_GUIDE.md`](docs/DEPLOYMENT_GUIDE.md).

## 🚀 Quick Start
```bash
cargo test --all --features testutils
cd frontend && npm run dev
```
