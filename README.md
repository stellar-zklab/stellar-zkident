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
<summary><strong><code>contracts/sybil_resistant_faucet</code> — real, reusable Sybil-resistance primitive</strong></summary>

The first real *external consumer* of `credential_verifier` — not another first-party demo of it. Pays a fixed amount of a real token to an address, at most once, but only if that address genuinely holds a verified credential — checked live via the exact same cross-contract `has_credential()` call `reputation_nft::mint()` already uses, not assumed or trusted from caller input. This is the standard fix for the "bots drain a giveaway with infinite addresses" problem faucets, airdrops, and sponsored-transaction budgets all face, and it's deliberately written as a standalone, general-purpose contract rather than baked into any one product: any Soroban project can point this same pattern at their own (or this) `credential_verifier` instance and get Sybil resistance without writing any proof logic themselves. 3 tests pass, covering a real successful payout, a real rejection of an unverified address, and a real rejection of a second claim from an already-paid address — see [Deployment](#deployment) for the same three outcomes exercised live on testnet, not just in the test suite.

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
| `sybil_resistant_faucet` | `CBNQ6BHR45SV5JMSKQTLIULDZFOR3DPAD4VIXGZSROVSLXXTKXIUM524` |

`credential_verifier` is initialized with `asp_registry`'s real deployed address above, and `reputation_nft` with `credential_verifier`'s — these aren't independently deployed instances that merely coexist, they're actually wired to each other on-chain. Each `zk_verifier` instance is initialized with its own real Groth16 verification key from `circuits/build/` — three separate instances, not one contract juggling three keys. `scripts/deploy.sh` and `scripts/deploy_zk_verifiers.sh` reproduce this from scratch — see [`docs/DEPLOYMENT_GUIDE.md`](docs/DEPLOYMENT_GUIDE.md).

**A real reputation score is minted on this deployment (2026-09-10).** Subject `GAUZ4T6UT7XMGOL6WYPWWSYPZQ7ZLILCAS2ROYCH5ILHHOWQYUGVRTAB` (the deployer, acting as its own demo identity) holds a real, on-chain `kyc_tier_2` credential record on `credential_verifier` and a real minted reputation NFT (token #0, score 72) on `reputation_nft` — query either yourself:
```bash
stellar contract invoke --id CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF --source deployer --network testnet -- has_credential --user GAUZ4T6UT7XMGOL6WYPWWSYPZQ7ZLILCAS2ROYCH5ILHHOWQYUGVRTAB --credential_type kyc_tier_2
stellar contract invoke --id CDA34SUCSQDOCCY5B6HJJH4CQ5PUDWII6CY3BDONGKT5E3KTEWZJ47GD --source deployer --network testnet -- get_reputation --subject GAUZ4T6UT7XMGOL6WYPWWSYPZQ7ZLILCAS2ROYCH5ILHHOWQYUGVRTAB
```

**`sybil_resistant_faucet` had all three of its real code paths exercised live on this deployment (2026-09-12), not just in the test suite.** Funded with 50,000,000 real testnet stroops from the deployer, then:

1. `claim()` from the deployer (the same subject with the real verified `kyc_tier_2` credential above) **succeeded** — a real 5,000,000-stroop native XLM transfer ([tx `843763ed...`](https://stellar.expert/explorer/testnet/tx/843763ed1448a6fc08e834d016729def77821dfc6ea52a1dcfd3ae5d23dc3219)).
2. `claim()` from a freshly generated address that has never verified anything **genuinely reverted on-chain** — the diagnostic log shows the real cross-contract `has_credential` call returning `false` before the trap, not a hardcoded rejection.
3. A second `claim()` from the deployer **also genuinely reverted** — the already-claimed check.

```bash
stellar contract invoke --id CBNQ6BHR45SV5JMSKQTLIULDZFOR3DPAD4VIXGZSROVSLXXTKXIUM524 --source deployer --network testnet -- has_claimed --user GAUZ4T6UT7XMGOL6WYPWWSYPZQ7ZLILCAS2ROYCH5ILHHOWQYUGVRTAB
```

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

**Real zero-knowledge proof verification (added 2026-09-13).** The [live demo](https://stellar-zkident.vercel.app/)'s "Zero-Knowledge Provers" panel used to be an admitted mockup — a `setTimeout` that flipped a local label with no on-chain call at all. It now submits each circuit's one real precomputed Groth16 proof (age, KYC tier, ASP Merkle membership — see `frontend/src/soroban.ts`) to its own deployed `zk_verifier` instance and shows the real result, the same pattern `stellar-zkstream`'s frontend already used for its one real proof. A fourth card ("Jurisdiction Compliance") was removed along with the mockup rather than carried forward — no circuit ever backed it.

**Ecosystem Portfolio tab (added 2026-09-12).** The [live demo](https://stellar-zkident.vercel.app/) now has a "Portfolio" tab alongside "Identity" — enter any real testnet address and see its actual state pulled live from **four contracts across three repos**, two of them a different GitHub org: this repo's `did_registry`/`credential_verifier`/`reputation_nft`/`sybil_resistant_faucet`, `stellar-zkstream`'s `stream` contract (real address-indexed `get_streams_by_sender`/`get_streams_by_recipient`, not a guess), and `soroban-yield-vault`'s `vault` contract (`balance_of`/`convert_to_assets`). All of it is a public on-chain read — no wallet signature needed for any of it. A fourth piece, a `soroban-gasless-contracts` smart wallet's `get_owner`/`get_recovery_signer`, needs a wallet contract ID typed in manually rather than being auto-discovered — there is no on-chain registry anywhere in this ecosystem mapping an owner address to the wallet contracts they've deployed, and this page is honest about that gap rather than pretending it's seamless. Every cross-repo contract address here was verified fresh against each repo's own `deployments/testnet.json` before being hardcoded — see `frontend/src/soroban.ts` for the same warning about what to update if any of them is ever redeployed.

**Sybil-Resistant Faucet panel (added 2026-09-12).** The [live demo](https://stellar-zkident.vercel.app/) now has a "Sybil-Resistant Faucet" claim button next to credential verification. It's honestly scoped the same way credential verification is: only the pre-registered demo subject holds a real verified credential right now, so claiming with any other connected wallet will genuinely revert on-chain with "claim requires a verified credential" — that's the gate working correctly, not a bug. See [Deployment](#deployment) for the real payout/rejection/already-claimed outcomes this was exercised against.

**Reputation score card (added 2026-09-10).** The [live demo](https://stellar-zkident.vercel.app/) now shows a minimalist card — address, one bold score number, mint date — reading `reputation_nft`'s real `get_reputation`, modeled on Human Passport's single-score-card pattern rather than a raw JSON dump. No wallet needed to view it; enter any address to check. The demo subject's score wasn't real before this: getting a non-empty card required actually exercising the full real pipeline for the first time — a real `verify_proof` call (persisting a genuine credential record for `kyc_tier_2`), then a real `mint()` gated on that record via `credential_verifier.has_credential()` — not a fixture inserted directly into storage. See [Deployment](#deployment) below.

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
