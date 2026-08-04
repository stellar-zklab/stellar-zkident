# stellar-zkident 🔐🪪

> **Zero-Knowledge Decentralized Identity (DID) & Soulbound Reputation Protocol on Stellar (Soroban)**  
> *Self-Sovereign `did:stellar:` Identity, Noir ZK Credentials, and Compliance-First Merkle ASP Sets*

[![CI](https://github.com/stellar-zklab/stellar-zkident/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-zklab/stellar-zkident/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Stellar Drips Wave](https://img.shields.io/badge/Stellar-Drips%20Wave-blueviolet)](https://drips.network)
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

**DID Document Schema (JSON-LD)**:
```json
{
  "@context": ["https://www.w3.org/ns/did/v1"],
  "id": "did:stellar:GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN",
  "verificationMethod": [{
    "id": "did:stellar:GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN#key-1",
    "type": "Ed25519VerificationKey2020",
    "controller": "did:stellar:GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"
  }],
  "authentication": ["did:stellar:GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN#key-1"]
}
```

### 2. Noir ZK Circuit Equations

#### Age Proof Circuit (`circuits/age_proof`)
Proves user birth year $Y_{\text{birth}} \le Y_{\text{current}} - 18$:
$$C = \text{Poseidon}(Y_{\text{birth}}, M_{\text{birth}}, D_{\text{birth}}, \text{salt})$$
$$\text{Assert: } Y_{\text{birth}} \le Y_{\text{current}} - 18 \quad \land \quad C == \text{PublicCommitment}$$

#### ASP Merkle Set Membership Circuit (`circuits/membership_proof`)
Proves inclusion of leaf $L$ in Merkle tree with root $R$ at depth $D=20$:
$$H_0 = \text{Poseidon}(L)$$
$$H_{i+1} = \text{Poseidon}(H_i, \text{Sibling}_i) \quad \text{or} \quad \text{Poseidon}(\text{Sibling}_i, H_i)$$
$$\text{Assert: } H_{20} == R$$

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

### 3. `ReputationNFTContract` (`contracts/reputation_nft`)

#### `mint(env: Env, admin: Address, subject: Address, initial_score: i64) -> u64`
Mints a soulbound reputation token for a subject.

#### `transfer(env: Env, from: Address, to: Address, token_id: u64)`
**Always Panics**: `panic!("soulbound: transfer not allowed")`.

---

### 4. `ASPRegistryContract` (`contracts/asp_registry`)

#### `register_asp(env: Env, admin: Address, asp: Address, merkle_root: BytesN<32>)`
Registers an Association Set Provider with its compliance Merkle root.

---

## Directory Structure

```
stellar-zkident/
├── contracts/
│   ├── did_registry/           # W3C did:stellar method registry
│   ├── credential_verifier/    # ZK proof verification & has_credential()
│   ├── reputation_nft/         # Soulbound Token (SBT) reputation engine
│   └── asp_registry/           # Association Set Provider Merkle roots
├── circuits/
│   ├── age_proof/              # Noir circuit: Age >= 18 proof
│   ├── kyc_tier_proof/         # Noir circuit: KYC level proof
│   └── membership_proof/       # Noir circuit: Merkle set inclusion proof
├── sdk/                        # TypeScript SDK
├── frontend/                   # React identity manager dashboard
├── docs/                       # Architecture, DID spec, credential types
└── scripts/
    └── deploy.sh               # Testnet deployment script
```

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

### Compile Noir Circuits

```bash
# Install Nargo (Noir CLI)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash

# Compile circuits
cd circuits/age_proof && nargo compile
cd ../kyc_tier_proof && nargo compile
cd ../membership_proof && nargo compile
```

---

## 🌊 Contributing — Stellar Drips Wave

`stellar-zkident` participates in the **[Stellar Drips Wave](https://drips.network)** program.

| Category | Points | Tasks |
|---|---|---|
| 🔴 **High Complexity** | 200 pts | Noir circuit specs, credential verifier, reputation SBT |
| 🟡 **Medium Complexity** | 150 pts | DID registry CRUD, ASP Merkle manager, SDK |
| 🟢 **Trivial Complexity** | 100 pts | Documentation, DID spec, testnet deploy script |

Browse open issues on [GitHub Issues](https://github.com/stellar-zklab/stellar-zkident/issues).

---

## License

Licensed under **Apache License 2.0**. See [LICENSE](LICENSE).
