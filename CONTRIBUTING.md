# Contributing to stellar-zkident 🔐🪪

Welcome to **`stellar-zkident`**! We are building the **Zero-Knowledge Decentralized Identity (DID) and Soulbound Reputation Primitive** for the Stellar (Soroban) ecosystem.

We welcome contributions from Rust developers, Circom/snarkjs ZK circuit engineers, W3C identity specialists, and frontend builders.

---

## 🚀 About the Protocol & Ecosystem Impact

`stellar-zkident` establishes a privacy-first identity layer for Stellar:
- Anchors self-sovereign DIDs (`did:stellar:<address>`) on-chain.
- Verifies age, KYC tier, and Merkle-tree membership claims via real **Groth16 zero-knowledge proofs**, without revealing the underlying private data.
- Also verifies classical Merkle-inclusion credentials (a separate, non-ZK mechanism — see README) for ASP-attested claims.
- Mints non-transferable **Soulbound Reputation Tokens (SBTs)**, gated on a real verified credential.
- Enables external DeFi protocols to verify user compliance (`has_credential()`) in a single line of Rust without exposing user personal data.

---

## 🗺️ Technical Architecture & Contribution Roadmap

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DEVELOPMENT ROADMAP PHASES                          │
│                                                                         │
│  Phase 1: Core Contracts & Real ZK Circuits (Built & Tested)           │
│    ├── did_registry (W3C did:stellar method)                          │
│    ├── credential_verifier & asp_registry (classical Merkle proofs)   │
│    ├── zk_verifier x3 (real Groth16 BN254 verifiers, one per circuit) │
│    └── Circom circuits (age_proof, kyc_tier_proof, membership_proof)  │
│                                                                         │
│  Phase 2: Off-Chain Attestation & SDK (Active Contribution)            │
│    ├── TypeScript SDK (@stellar-zklab/zkident-sdk) — real, tested     │
│    ├── In-browser proof generation (snarkjs) for the SDK              │
│    └── Off-chain attestation issuer tooling for ASPs                  │
│                                                                         │
│  Phase 3: Identity Dashboard & Wallet Sign-In (Upcoming)               │
│    ├── React DID & Credential Manager dashboard                        │
│    ├── Sign-In-With-Stellar (SIWS) authentication component             │
│    └── Multi-credential ZK proof aggregation circuit                   │
│                                                                         │
│  Phase 4: Ecosystem Composability & Compliance (Future)                │
│    ├── ASP Merkle root automatic sync daemon                           │
│    └── Soulbound reputation tier lookup helpers                        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Developer Environment Quickstart

### Prerequisites
- **Rust Toolchain**: `rustup target add wasm32v1-none`
- **Circom & snarkjs** (for circuit work only): `npm install -g snarkjs`, [circom 2.1.6+](https://docs.circom.io/getting-started/installation/)
- **Stellar CLI**: v22.0.0+

### Build & Run Tests

```bash
# Clone the repository
git clone https://github.com/stellar-zklab/stellar-zkident.git
cd stellar-zkident

# Run unit tests across all 5 contracts (did_registry, credential_verifier,
# reputation_nft, asp_registry, zk_verifier — 21 tests total)
cargo test --all --features testutils

# Reproduce the real ZK circuits' trusted-setup pipeline (only needed if you're
# changing a circuit — the resulting VKs/proofs are already committed under
# circuits/build/): see circuits/README.md for the full, exact steps.
```

---

## 🌿 Git Branch & Conventional Commits

| Prefix | Usage | Example |
|---|---|---|
| `feat:` | New feature or contract function | `feat(did): add batch_is_active query helper` |
| `fix:` | Bug fix or logic patch | `fix(verifier): update credential expiry validation` |
| `docs:` | Documentation updates | `docs(did): add W3C did:stellar specification` |
| `circuit:` | Circom circuit changes | `circuit(kyc): add proof aggregation constraints` |

---

## 📋 How to Claim an Issue & Submit a PR

1. **Pick an Issue**: Browse open tasks on [GitHub Issues](https://github.com/stellar-zklab/stellar-zkident/issues). Look for [`good-first-issue`](https://github.com/stellar-zklab/stellar-zkident/issues?q=is%3Aissue+is%3Aopen+label%3A%22good-first-issue%22).
2. **Create a Branch**: `git checkout -b feat/your-feature-name`
3. **Verify Locally**: Ensure `cargo test --all --features testutils` passes.
4. **Submit PR**: Open a Pull Request referencing the issue number (e.g. `Closes #8`).

Thank you for building self-sovereign identity on Stellar! 🪪
