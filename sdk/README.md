# @stellar-zklab/zkident-sdk

TypeScript SDK for `stellar-zkident`'s DID registry and credential verification on Stellar.

## Current Status — what's real

`StellarZkIdentClient` wraps `@stellar/stellar-sdk/contract`'s `Client` and makes real
simulate/sign/submit calls against a real Soroban RPC endpoint — the same integration
`../frontend/src/soroban.ts` uses, factored out into a reusable package. `registerDid()`
builds, signs, and submits a real transaction; `resolveDid()`, `isDidActive()`,
`hasCredential()`, `getReputation()`, and `verifyAgeProof()`/`verifyKycTierProof()`/
`verifyMembershipProof()` simulate against real on-chain state.

Signing is dependency-injected rather than hard-wired to Freighter, so this SDK works with
any wallet adapter that can produce a signed transaction XDR string.

```ts
import { StellarZkIdentClient } from '@stellar-zklab/zkident-sdk';
import freighter from '@stellar/freighter-api';

const zkident = new StellarZkIdentClient({
  didRegistryId: 'CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6',
  credentialVerifierId: 'CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF',
  reputationNftId: 'CDA34SUCSQDOCCY5B6HJJH4CQ5PUDWII6CY3BDONGKT5E3KTEWZJ47GD',
  signTransaction: async (xdr, opts) => {
    const { signedTxXdr } = await freighter.signTransaction(xdr, opts);
    return signedTxXdr;
  },
});

await zkident.registerDid(ownerAddress, JSON.stringify({ name: 'example' }));
const record = await zkident.resolveDid(ownerAddress);
const hasKyc = await zkident.hasCredential(ownerAddress, 'kyc_tier_2');

// Real zero-knowledge verification — proves age >= 18 without revealing the birth date.
// `proof`/`publicInputs` come from circuits/convert_to_soroban.mjs's output; see
// circuits/README.md for how to generate a real proof for your own private inputs.
const zkidentWithZk = new StellarZkIdentClient({
  didRegistryId: 'CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6',
  credentialVerifierId: 'CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF',
  ageProofVerifierId: '<age_proof zk_verifier address, see deployments/testnet.json>',
  signTransaction: async (xdr, opts) => {
    const { signedTxXdr } = await freighter.signTransaction(xdr, opts);
    return signedTxXdr;
  },
});
const isAdult = await zkidentWithZk.verifyAgeProof(proofBytes, publicInputBytes);
```

## Two, genuinely different identity-proof mechanisms

`verifyCredential()` (wrapping `credential_verifier.verify_proof()`) checks classical
Merkle inclusion, **not** a zero-knowledge proof — the leaf is derived directly from the
caller's real address, so membership isn't hidden from the verifier.

`verifyAgeProof()`, `verifyKycTierProof()`, and `verifyMembershipProof()` are genuinely
different: they verify real Groth16 zero-knowledge proofs against dedicated `zk_verifier`
instances, proving age/KYC-tier/Merkle-membership claims **without revealing the
underlying private data**. These were originally written as Noir circuits; they're now
Circom/Groth16 because Noir's default proving system needs curve support (Grumpkin)
Soroban doesn't have — see `circuits/README.md` for the full explanation and the real
trusted-setup pipeline that produced these verifiers' verification keys.
