# stellar-zkident Frontend

React application for managing DIDs and verifying credentials.

## Current status — what's real vs. not

**Wired to real deployed testnet contracts for two things, not mocked.**
`src/soroban.ts` uses `@stellar/stellar-sdk`'s `contract.Client` to talk directly to the
real, deployed `did_registry` and `credential_verifier` contracts (see
`../deployments/testnet.json`):

- **Real DID Registry** — needs a connected Freighter wallet. Registers and resolves a
  genuine `did_registry` record for whichever wallet is connected, fully self-service
  (any wallet can register its own DID — no admin key needed).
- **Real Credential Verification** — needs no wallet. Simulates a real call to the real
  deployed `credential_verifier` with a real Merkle proof for the one credential an ASP
  has actually attested to so far (a fixed demo identity, not your connected wallet — no
  ASP has attested to an arbitrary wallet's credential yet).

**What's still an honest mockup:** the four Noir-circuit provers (age, KYC tier,
residency, ASP Merkle membership as zero-knowledge proofs) remain genuinely unverified —
no Noir/UltraPlonk proof system is wired to on-chain verification, and Soroban's crypto
host functions target Groth16/BN254 pairing checks, not UltraPlonk, so it's an open
question whether that path is even viable without a circuit redesign (see
`stellar-zkstream`, which went this route with Circom/Groth16 instead). That section is
left clearly labeled rather than silently removed.

## Prerequisites to actually use it

- The [Freighter](https://freighter.app) browser extension, set to **Testnet**.
- A funded testnet account (Freighter can request testnet XLM from friendbot itself) —
  only needed for the DID registry half; credential verification needs no wallet.

## Running it

```bash
npm install
npm run dev
```
