# Contributing to stellar-zkident 🔐🪪

Welcome to **`stellar-zkident`**! We are building the **Zero-Knowledge Decentralized Identity (DID) and Soulbound Reputation Primitive** for the Stellar (Soroban) ecosystem.

We welcome contributions from Rust developers, Noir ZK circuit engineers, W3C identity specialists, and frontend builders.

---

## 🚀 About the Protocol & Ecosystem Impact

`stellar-zkident` establishes a privacy-first identity layer for Stellar:
- Anchors self-sovereign DIDs (`did:stellar:<address>`) on-chain.
- Verifies real-world credentials (Age, KYC tier, Residency, Employment, ASP Set Membership) via **Noir Zero-Knowledge Proofs**.
- Mints non-transferable **Soulbound Reputation Tokens (SBTs)**.
- Enables external DeFi protocols to verify user compliance (`has_credential()`) in a single line of Rust without exposing user personal data.

---

## 🗺️ Technical Architecture & Contribution Roadmap

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DEVELOPMENT ROADMAP PHASES                          │
│                                                                         │
│  Phase 1: Core Contracts & Noir Circuits (Scaffolded & Verified)        │
│    ├── did_registry (W3C did:stellar method)                           │
│    ├── credential_verifier & asp_registry                              │
│    └── Noir circuits (age_proof, kyc_tier_proof, membership_proof)     │
│                                                                         │
│  Phase 2: Off-Chain Attestation & SDK (Active Contribution)            │
│    ├── TypeScript SDK (@stellar-zklab/zkident-sdk)                     │
│    ├── Ed25519 issuer off-chain attestation CLI tool                   │
│    └── IPFS DID Document CID storage integration                       │
│                                                                         │
│  Phase 3: Identity Dashboard & Wallet Sign-In (Upcoming)               │
│    ├── React DID & Credential Manager dashboard                        │
│    ├── Sign-In-With-Stellar (SIWS) authentication component             │
│    └── Multi-credential ZK proof aggregation circuit                   │
│                                                                         │
│  Phase 4: Ecosystem Composability & Compliance (Future)                │
│    ├── ASP Merkle root automatic sync daemon                           │
│    ├── Fuzz testing for raw DID JSON-LD strings                        │
│    └── Soulbound reputation tier lookup helpers                        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Developer Environment Quickstart

### Prerequisites
- **Rust Toolchain**: `rustup target add wasm32v1-none`
- **Nargo (Noir CLI)**: v0.30.0+ (`noirup`)
- **Stellar CLI**: v22.0.0+

### Build & Run Tests

```bash
# Clone the repository
git clone https://github.com/stellar-zklab/stellar-zkident.git
cd stellar-zkident

# Run unit tests across all 4 contracts
cargo test --all --features testutils

# Compile Noir ZK circuits
cd circuits/age_proof && nargo compile
cd ../kyc_tier_proof && nargo compile
cd ../membership_proof && nargo compile
```

---

## 🌿 Git Branch & Conventional Commits

| Prefix | Usage | Example |
|---|---|---|
| `feat:` | New feature or contract function | `feat(did): add batch_is_active query helper` |
| `fix:` | Bug fix or logic patch | `fix(verifier): update credential expiry validation` |
| `docs:` | Documentation updates | `docs(did): add W3C did:stellar specification` |
| `circuit:` | Noir circuit changes | `circuit(kyc): add proof aggregation constraints` |

---

## 📋 How to Claim an Issue & Submit a PR

1. **Pick an Issue**: Browse open tasks on [GitHub Issues](https://github.com/stellar-zklab/stellar-zkident/issues). Look for [`good-first-issue`](https://github.com/stellar-zklab/stellar-zkident/issues?q=is%3Aissue+is%3Aopen+label%3A%22good-first-issue%22).
2. **Create a Branch**: `git checkout -b feat/your-feature-name`
3. **Verify Locally**: Ensure `cargo test --all --features testutils` passes.
4. **Submit PR**: Open a Pull Request referencing the issue number (e.g. `Closes #8`).

Thank you for building self-sovereign identity on Stellar! 🪪
