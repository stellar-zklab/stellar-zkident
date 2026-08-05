# stellar-zkident 🔐🪪

> **Zero-Knowledge Decentralized Identity (DID) & Soulbound Reputation Protocol on Stellar (Soroban)**  
> *Self-Sovereign `did:stellar:` Identity, Noir ZK Credentials, and Compliance-First Merkle ASP Sets*

[![CI](https://github.com/stellar-zklab/stellar-zkident/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-zklab/stellar-zkident/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Soroban Version](https://img.shields.io/badge/Soroban-v22.0.0-orange)](https://developers.stellar.org)

---

## Executive Summary

`stellar-zkident` is a privacy-preserving decentralized identity (DID) and on-chain reputation protocol built natively on **Soroban**, Stellar's smart contract engine.

It establishes an identity layer for the Stellar ecosystem: allowing users to anchor self-sovereign DIDs (`did:stellar:<address>`), prove real-world credentials (Age, KYC level, Residency, Employment, ASP Set Membership) via **Noir Zero-Knowledge Proofs**, and earn non-transferable **Soulbound Reputation NFTs** — all while maintaining complete data privacy and regulatory compliance.

---

## Key Features & Protocol Innovations

- 🪪 **`did:stellar:` Method Specification**: On-chain DID registry anchoring W3C-compliant DID documents directly to Stellar account keys without external blockchain bridges.
- 🔐 **Noir ZK Credential Proofs**: Users generate ZK proofs locally using Noir circuits. Prove age $\ge 18$, KYC tier $\ge \text{Silver}$, or country residency without revealing birthdates, personal names, or exact locations.
- 🏆 **Soulbound Reputation Tokens (SBTs)**: Non-transferable on-chain reputation score minted per DID. Transfers are strictly blocked at the contract level.
- ⚖️ **Association Set Provider (ASP) Merkle Sets**: Compliance-first ZK architecture allowing regulatory approved entities (ASPs) to maintain Merkle roots of verified users on-chain. Users prove Merkle membership without revealing their specific index.
- 🔗 **Soroban Composability Primitive**: Any external DeFi protocol or DAO contract on Soroban can call `has_credential(address, type)` or `is_active(address)` in a single line of Rust.

---

## Protocol Architecture & System Flow

```
┌────────────────────────────────────────────────────────────────────────┐
│                              CLIENT SIDE                               │
│                                                                        │
│  User Private Data (Birthdate, KYC Level, Merkle Path)                 │
│       │                                                                │
│       ▼                                                                │
│  [Noir Circuits] (age_proof, kyc_tier_proof, membership_proof)         │
│       │                                                                │
│       ▼                                                                │
│  UltraPlonk ZK Proof Generation (Barretenberg)                         │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        SOROBAN CONTRACT LAYER                          │
│                                                                        │
│   credential_verifier ──────► did_registry                            │
│           │                   (did:stellar:<address> status check)     │
│           │                                                            │
│           ├─────────────────► asp_registry                             │
│           │                   (Verify Merkle root for compliance)      │
│           │                                                            │
│           └─────────────────► reputation_nft                           │
│                               (Mint/Update Soulbound Score)            │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Cryptographic & Mathematical Specification

### 1. `did:stellar:` W3C DID Method Specification
Each DID is uniquely generated from a Stellar public address:
$$\text{DID} = \text{did:stellar:}\langle\text{Stellar-Address}\rangle$$

### 2. Noir ZK Circuit Equations

#### Age Proof Circuit (`circuits/age_proof`)
Proves user birth year $Y_{\text{birth}} \le Y_{\text{current}} - 18$:
$$C = \text{Poseidon}(Y_{\text{birth}}, M_{\text{birth}}, D_{\text{birth}}, \text{salt})$$
$$\text{Assert: } Y_{\text{birth}} \le Y_{\text{current}} - 18 \quad \land \quad C == \text{PublicCommitment}$$

---

## Smart Contract API Reference

### 1. `DIDRegistryContract` (`contracts/did_registry`)

#### `register_did(env: Env, owner: Address, document: String) -> String`
Registers a new DID document. Owner authentication required (`owner.require_auth()`).

#### `is_active(env: Env, address: Address) -> bool`
Composable query returning `true` if an address has an active DID record.

---

### 2. `CredentialVerifierContract` (`contracts/credential_verifier`)

#### `verify_credential_proof(env: Env, subject: Address, credential_type: CredentialType, _proof: Bytes, _public_inputs: Vec<BytesN<32>>, issuer: Address, expires_at: u64) -> bool`
Verifies ZK proof, checks trusted issuer status, validates DID registration, and records credential on-chain.

#### `has_credential(env: Env, subject: Address, credential_type: CredentialType) -> bool`
Single-line composable helper callable by any external Soroban smart contract to check user credential status.

---

## Developer Quick Start

### Build Contracts

```bash
git clone https://github.com/stellar-zklab/stellar-zkident.git
cd stellar-zkident

# Run unit tests across all 4 contracts
cargo test --all --features testutils

# Compile release WASM binaries
cargo build --release --target wasm32v1-none
```

---

## 🤝 Contributing & Community Roadmap

`stellar-zkident` is an open-source identity primitive for Stellar. We welcome contributions from developers, security auditors, and identity protocols!

### How to Contribute
1. **Explore Issues**: Check out open tasks tagged [`good-first-issue`](https://github.com/stellar-zklab/stellar-zkident/issues?q=is%3Aissue+is%3Aopen+label%3A%22good-first-issue%22) or [`help-wanted`](https://github.com/stellar-zklab/stellar-zkident/issues).
2. **Fork & Branch**: Create a feature branch (`git checkout -b feat/your-feature`).
3. **Test Your Changes**: Ensure all unit tests pass (`cargo test --all --features testutils`).
4. **Submit a Pull Request**: Open a PR with a clear summary of your changes.

---

## License

Licensed under **Apache License 2.0**. See [LICENSE](LICENSE).
