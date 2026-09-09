# stellar-zkident 🔐🪪

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
[![CI](https://github.com/stellar-zklab/stellar-zkident/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-zklab/stellar-zkident/actions/workflows/ci.yml)
![DID](https://img.shields.io/badge/Standard-W3C_DID_v1.0-violet)
[![Live Demo](https://img.shields.io/badge/Live_Demo-stellar--zkident.vercel.app-black?style=flat&logo=vercel)](https://stellar-zkident.vercel.app/)

**Self-sovereign `did:stellar:` decentralized identity on Soroban** — real Groth16 zero-knowledge credentials (age, KYC tier, membership) that prove a claim on-chain without revealing the private data behind it, plus a soulbound reputation (SBT) framework gated on real verification.

**[🔗 Try the live demo](https://stellar-zkident.vercel.app/)** — wired to the real deployed testnet contracts listed under [Deployment](#deployment), not a mockup.

## Contents

- [Why this is real zero-knowledge, not a buzzword](#why-this-is-real-zero-knowledge-not-a-buzzword)
- [Architecture](#architecture)
- [What's built](#whats-built)
- [Deployment](#deployment)
- [Usage](#usage)
- [Quick start](#-quick-start)
- [Ecosystem](#ecosystem)
- [Contributing](#contributing)
- [License](#license)

## Why this is real zero-knowledge, not a buzzword

- **Genuine Groth16 proofs verified on-chain, not simulated.** Three deployed `zk_verifier` instances perform real BN254 pairing checks via Soroban Protocol 25's native `env.crypto().bn254()` host functions — proving age ≥ 18, KYC tier ≥ N, or Merkle-tree membership **without revealing the underlying private data**.
- **A real trusted-setup pipeline, not hand-crafted bytes.** Every verification key comes from an actual circom → snarkjs Powers-of-Tau → phase 2 → contribution → export run — see [`circuits/README.md`](circuits/README.md) for the exact reproducible steps.
- **A hard technical call made and documented, not glossed over.** The original circuits were written in Noir, whose default proving system needs a BN254+Grumpkin curve cycle Soroban doesn't support. Rather than ship something that couldn't actually verify on-chain, the circuits were rebuilt in Circom/Groth16 — see [Deployment](#deployment) for why that's a real, working tradeoff, not a downgrade.
- **Classical and zero-knowledge verification are kept honestly separate.** `credential_verifier`'s Merkle-inclusion check is explicitly labeled as *not* zero-knowledge (the leaf is derived from the caller's real address) — real ZK privacy lives specifically in `zk_verifier`, so the two capabilities are never conflated.

## Architecture

```
+----------------------------+
|        did_registry        |
+----------------------------+
(register / update / resolve DID documents)

Classical credential path:
+----------------------------+
|        asp_registry        |
|        (Merkle root        |
|          per ASP)          |
+----------------------------+
            |  get_merkle_root()
            v
+----------------------------+
|    credential_verifier     |
|    (Merkle inclusion --    |
|    NOT zero-knowledge)     |
+----------------------------+
            |  has_credential()
            v
+----------------------------+
|       reputation_nft       |
|     (soulbound, gated      |
|   on real verification)    |
+----------------------------+

Real zero-knowledge path (separate from the above):
+----------------------------+
|      Circom circuit +      |
|       Groth16 proof        |
|   (generated off-chain)    |
+----------------------------+
            |  vrfy_prf()
            v
+----------------------------+
|        zk_verifier         |
|     (one instance per      |
|    circuit: age_proof,     |
|      kyc_tier_proof,       |
|    membership_proof --     |
|     real BN254 pairing     |
|           check)           |
+----------------------------+
```

## What's built

Status of each piece, so anyone reading knows exactly what's real, what's tested, and what's classical vs. genuinely zero-knowledge.

<details open>
<summary><strong><code>contracts/did_registry</code> — real</strong></summary>

Full DID document CRUD: register, add verification keys, update, deactivate, resolve. No shortcuts.

</details>

<details open>
<summary><strong><code>contracts/reputation_nft</code> — real, gated by a verified credential</strong></summary>

`mint()` used to accept an admin's say-so alone — it stored `credential_verifier`'s address at `initialize()` but never called it, so reputation could be minted for anyone regardless of whether they'd ever verified anything. It now calls `credential_verifier.has_credential(subject, credential_type)` for real (via a raw `env.invoke_contract`, not a crate dependency — see `credential_verifier`'s own note on why) and rejects the mint if that comes back false. `get_reputation()` and a soulbound `transfer()` that correctly always reverts (non-transferable by design) are unchanged.

</details>

<details open>
<summary><strong><code>contracts/asp_registry</code> — real</strong></summary>

Stores a Merkle root per registered Attestation Service Provider, plus `get_merkle_root()` for other contracts to read it. Actually consumed by `credential_verifier` — see below.

</details>

<details open>
<summary><strong><code>contracts/credential_verifier</code> — real Merkle membership verification, not a ZK proof</strong></summary>

`verify_proof()` performs genuine cryptographic verification: it computes a leaf as `sha256(b"zkident:credential-leaf:v1:" || strkey(user) || credential_type)`, walks a caller-supplied sibling path up to a root, and rejects unless that root matches the ASP's *currently registered* root — fetched directly from `asp_registry` (stored at `initialize()`, never taken as caller input, so a proof can't be checked against an attacker-controlled fake registry). This proves on-chain that `user` is one of the leaves a specific ASP committed to. It is **not** zero-knowledge: the leaf is derived from the caller's real address, so membership is not hidden. What matters is that `has_credential()` can no longer be made `true` by submitting an arbitrary string; it now requires a real path to the ASP's real root. Genuine zero-knowledge verification is a separate, real capability — see `contracts/zk_verifier` below.

</details>

<details open>
<summary><strong><code>contracts/zk_verifier</code> — real, genuine zero-knowledge verification, deployed three times over</strong></summary>

Three real Groth16 BN254 verifier instances (reusing `stellar-zkstream`'s already-proven verifier contract unmodified), one per real circuit in `circuits/`: proving age ≥ 18, KYC tier ≥ N, and Merkle-tree membership — each **without revealing the underlying private data** (birth date, actual tier, or which leaf/path). This is a real, complete Groth16 trusted-setup pipeline (circom → snarkjs powers-of-tau → phase 2 → contribution → export), not hand-crafted bytes — see [`circuits/README.md`](circuits/README.md). 12 tests pass, including 3 that feed real generated proofs for these exact circuits through the real contract's `vrfy_prf()` and 2 that confirm a tampered public input is correctly rejected.

</details>

<details open>
<summary><strong><code>circuits/</code> — real Circom circuits, rebuilt from the original Noir source</strong></summary>

Originally written in Noir, which defaults to the UltraHonk proving system — that needs a BN254+Grumpkin curve cycle Soroban has no native support for (there's an active, unfinished official proposal to build this; see [`circuits/README.md`](circuits/README.md)). Rebuilt in Circom/Groth16 instead, which only needs the BN254 pairing checks Soroban already supports natively — the same approach `stellar-zkstream` already proved works end to end.

</details>

## Deployment

All seven contracts are live on Stellar testnet (core four deployed/redeployed 2026-09-05, the three ZK verifiers deployed 2026-09-05 — see [`deployments/testnet.json`](deployments/testnet.json) — independently checkable on [stellar.expert](https://stellar.expert/explorer/testnet)):

| Contract | Address |
|---|---|
| `asp_registry` | `CACMQJV7SRSRKQDBXJWORGYKVWMXL6LDGJZXGY6A3DYPV3RXFHF4AR52` |
| `credential_verifier` | `CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF` |
| `did_registry` | `CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6` |
| `reputation_nft` | `CDA34SUCSQDOCCY5B6HJJH4CQ5PUDWII6CY3BDONGKT5E3KTEWZJ47GD` |
| `zk_verifier` (age_proof) | `CCILFFFLU6UKPXU3QD47IJULLGSPPFDS3PIUPV2MBOR5QA6OREI22NUV` |
| `zk_verifier` (kyc_tier_proof) | `CCLKJTSGJ6WJR76TKS4H4FWTH472WILJ7SGC4OWUYUAMCUGCK2E7NCYC` |
| `zk_verifier` (membership_proof) | `CCHJVP2UCG6KIOPLYIIEJ5KYESA4LGEP2QSF3JZNRFCWSK66RDIVOHTW` |

`credential_verifier` is initialized with `asp_registry`'s real deployed address above, and `reputation_nft` with `credential_verifier`'s — these aren't independently deployed instances that merely coexist, they're actually wired to each other on-chain. Each `zk_verifier` instance is initialized with its own real Groth16 verification key from `circuits/build/` — three separate instances, not one contract juggling three keys. `scripts/deploy.sh` and `scripts/deploy_zk_verifiers.sh` reproduce this from scratch — see [`docs/DEPLOYMENT_GUIDE.md`](docs/DEPLOYMENT_GUIDE.md).

## Usage

```typescript
import { StellarZkIdentClient } from '@stellar-zklab/zkident-sdk';
import freighter from '@stellar/freighter-api';

const zkident = new StellarZkIdentClient({
  didRegistryId: 'CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6', // live on testnet, see Deployment above
  credentialVerifierId: 'CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF',
  ageProofVerifierId: 'CCILFFFLU6UKPXU3QD47IJULLGSPPFDS3PIUPV2MBOR5QA6OREI22NUV',
  signTransaction: async (xdr, opts) => {
    const { signedTxXdr } = await freighter.signTransaction(xdr, opts);
    return signedTxXdr;
  },
});

// Self-service DID registration — no admin key needed.
await zkident.registerDid(userAddress, didDocumentJson);

// Real Groth16 verification: proves age >= 18 on-chain without revealing birth date.
// proof/publicInputs come from circuits/gen_inputs.mjs + snarkjs — see circuits/README.md.
const isOver18 = await zkident.verifyAgeProof(proof, publicInputs);
```

See [`sdk/README.md`](sdk/README.md) for the full API and [`circuits/README.md`](circuits/README.md) for how to generate a real proof for any of the three circuits.

## 🚀 Quick start

**Prerequisites:**
- Rust with the `wasm32v1-none` target
- Node.js 20+
- `circom` + `snarkjs` — only needed to regenerate circuits, see [`circuits/README.md`](circuits/README.md)

```bash
# Run the real contract test suite (12 tests for zk_verifier alone — see What's built above)
cargo test --all --features testutils

# Run the frontend against the real deployed contracts (connects Freighter, real did_registry calls)
cd frontend && npm install && npm run dev
```

## Ecosystem

Part of **stellar-zklab**'s Soroban Protocol 25 project suite, alongside:
- [`soroban-yield-vault`](https://github.com/stellar-zklab/soroban-yield-vault) — real Blend Protocol V2 yield vault with Yearn V3 share math ([live demo](https://soroban-yield-vault.vercel.app/))
- [`stellar-zkstream`](https://github.com/stellar-zklab/stellar-zkstream) — privacy-preserving payment streaming; this repo's `zk_verifier` contract is reused from there unmodified ([live demo](https://stellar-zkstream.vercel.app/))

All three share the same "real vs. not" documentation discipline and the same Protocol 25 BN254/testnet deployment conventions.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the phased roadmap covering contracts, circuits, SDK, and frontend work. Check the [issue tracker](https://github.com/stellar-zklab/stellar-zkident/issues) for known gaps before starting something new.

## License

Apache 2.0 — see [`LICENSE`](LICENSE).
