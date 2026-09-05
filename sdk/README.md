# @stellar-zklab/zkident-sdk

TypeScript SDK for `stellar-zkident`'s DID registry and credential verification on Stellar.

## Current Status — what's real

`StellarZkIdentClient` wraps `@stellar/stellar-sdk/contract`'s `Client` and makes real
simulate/sign/submit calls against a real Soroban RPC endpoint — the same integration
`../frontend/src/soroban.ts` uses, factored out into a reusable package. `registerDid()`
builds, signs, and submits a real transaction; `resolveDid()`, `isDidActive()`,
`hasCredential()`, and `getReputation()` simulate against real on-chain state.

Signing is dependency-injected rather than hard-wired to Freighter, so this SDK works with
any wallet adapter that can produce a signed transaction XDR string.

```ts
import { StellarZkIdentClient } from '@stellar-zklab/zkident-sdk';
import freighter from '@stellar/freighter-api';

const zkident = new StellarZkIdentClient({
  didRegistryId: 'CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6',
  credentialVerifierId: 'CAHGQ3OH2MSJBVQ4DGDPBX7VPROCKVB2YA32QCLIPYX2766WZJPJCWTA',
  reputationNftId: 'CC3LKAQ4LOXZYIWSQBGDO7GF747QMIOZFJQEUZP6DWZXZWZ2UNJMNJ27',
  signTransaction: async (xdr, opts) => {
    const { signedTxXdr } = await freighter.signTransaction(xdr, opts);
    return signedTxXdr;
  },
});

await zkident.registerDid(ownerAddress, JSON.stringify({ name: 'example' }));
const record = await zkident.resolveDid(ownerAddress);
const hasKyc = await zkident.hasCredential(ownerAddress, 'kyc_tier_2');
```

## Important naming caveat

Despite the repo and package name, `verifyCredential()` (which wraps
`credential_verifier.verify_proof()`) checks classical Merkle inclusion, **not** a
zero-knowledge proof — the leaf is derived directly from the caller's real address, so
membership isn't hidden from the verifier. The repo's actual Noir ZK circuits
(`circuits/age_proof`, `circuits/kyc_tier_proof`, `circuits/membership_proof`) exist but
are not wired to any deployed contract yet — see the main repo README's "Current Status".
