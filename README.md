# stellar-zkident 🔐🪪

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
![Build](https://img.shields.io/badge/Cargo_Test-Passing-brightgreen)
![DID](https://img.shields.io/badge/Standard-W3C_DID_v1.0-violet)

Self-Sovereign `did:stellar:` Decentralized Identity, real Groth16 Zero-Knowledge Credentials, and Soulbound Reputation (SBT) Framework.

## Current Status — what's real vs. not

**`contracts/did_registry` — real.** Full DID document CRUD: register, add verification keys, update, deactivate, resolve. No shortcuts.

**`contracts/reputation_nft` — real, and now actually gated by a verified credential.** `mint()` used to accept an admin's say-so alone — it stored `credential_verifier`'s address at `initialize()` but never called it, so reputation could be minted for anyone regardless of whether they'd ever verified anything. It now calls `credential_verifier.has_credential(subject, credential_type)` for real (via a raw `env.invoke_contract`, not a crate dependency — see `credential_verifier`'s own note on why) and rejects the mint if that comes back false. `get_reputation()` and a soulbound `transfer()` that correctly always reverts (non-transferable by design) are unchanged.

**`contracts/asp_registry` — real.** Stores a Merkle root per registered Attestation Service Provider, plus `get_merkle_root()` for other contracts to read it. Now actually consumed by `credential_verifier` — see below.

**`contracts/credential_verifier` — real Merkle membership verification, not a ZK proof.** `verify_proof()` now performs genuine cryptographic verification: it computes a leaf as `sha256(b"zkident:credential-leaf:v1:" || strkey(user) || credential_type)`, walks a caller-supplied sibling path up to a root, and rejects unless that root matches the ASP's *currently registered* root — fetched directly from `asp_registry` (stored at `initialize()`, never taken as caller input, so a proof can't be checked against an attacker-controlled fake registry). This proves on-chain that `user` is one of the leaves a specific ASP committed to. It is **not** zero-knowledge: the leaf is derived from the caller's real address, so membership is not hidden. What changed is that `has_credential()` can no longer be made `true` by submitting an arbitrary string; it now requires a real path to the ASP's real root. Genuine zero-knowledge verification is a separate, real capability — see `contracts/zk_verifier` below.

**`contracts/zk_verifier` — real, genuine zero-knowledge verification, deployed three times over.** Three real Groth16 BN254 verifier instances (reusing `stellar-zkstream`'s already-proven verifier contract unmodified), one per real circuit in `circuits/`: proving age ≥ 18, KYC tier ≥ N, and Merkle-tree membership — each **without revealing the underlying private data** (birth date, actual tier, or which leaf/path). This is a real, complete Groth16 trusted-setup pipeline (circom → snarkjs powers-of-tau → phase 2 → contribution → export), not hand-crafted bytes — see `circuits/README.md`. 12 tests pass, including 3 that feed real generated proofs for these exact circuits through the real contract's `vrfy_prf()` and 2 that confirm a tampered public input is correctly rejected.

**`circuits/` — real Circom circuits (rebuilt from this repo's original Noir source) that a deployed contract actually verifies.** Originally written in Noir, which defaults to the UltraHonk proving system — that needs a BN254+Grumpkin curve cycle Soroban has no native support for (there's an active, unfinished official proposal to build this; see `circuits/README.md`). Rebuilt in Circom/Groth16 instead, which only needs the BN254 pairing checks Soroban already supports natively — the same approach `stellar-zkstream` already proved works end to end.

## Deployment

All four contracts are live on Stellar testnet (deployed 2026-09-03, see
[`deployments/testnet.json`](deployments/testnet.json) — independently checkable on
[stellar.expert](https://stellar.expert/explorer/testnet)):

| Contract | Address |
|---|---|
| `asp_registry` | `CACMQJV7SRSRKQDBXJWORGYKVWMXL6LDGJZXGY6A3DYPV3RXFHF4AR52` |
| `credential_verifier` | `CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF` |
| `did_registry` | `CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6` |
| `reputation_nft` | `CDA34SUCSQDOCCY5B6HJJH4CQ5PUDWII6CY3BDONGKT5E3KTEWZJ47GD` |

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
