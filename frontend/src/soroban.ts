// Real integration against the actual deployed Stellar testnet contracts — no mocking
// here. See ../deployments/testnet.json for where these addresses come from and how to
// verify them independently on stellar.expert.
import { Client as ContractClient } from '@stellar/stellar-sdk/contract';
import freighter from '@stellar/freighter-api';

export const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015';
export const RPC_URL = 'https://soroban-testnet.stellar.org';

export const DID_REGISTRY_ID = 'CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6';
export const CREDENTIAL_VERIFIER_ID = 'CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF';
export const REPUTATION_NFT_ID = 'CDA34SUCSQDOCCY5B6HJJH4CQ5PUDWII6CY3BDONGKT5E3KTEWZJ47GD';

function hexToBytes(hex: string): Uint8Array {
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
  }
  return out;
}

// A real, already-registered demo credential — NOT tied to whichever wallet connects.
// asp_registry has exactly one real ASP registered so far: the deployer address acting as
// its own demo attestation service, vouching for its own "kyc_tier_2" credential via a
// real 4-leaf Merkle tree. See ../deployments/testnet.json's notes and
// docs/DEPLOYMENT_GUIDE.md for how this was registered (a real register_asp call, not a
// fixture). Verifying an arbitrary connected wallet's own credential would need that
// wallet's address baked into a leaf an ASP has actually attested to — nothing does that
// yet, so this demo can only show a pre-registered identity's real credential, the same
// honest scoping stellar-zkstream's frontend uses for its one real proof.
export const DEMO_CREDENTIAL_SUBJECT = 'GAUZ4T6UT7XMGOL6WYPWWSYPZQ7ZLILCAS2ROYCH5ILHHOWQYUGVRTAB';
export const DEMO_CREDENTIAL_ASP = DEMO_CREDENTIAL_SUBJECT;
export const DEMO_CREDENTIAL_TYPE = 'kyc_tier_2';
const DEMO_MERKLE_PROOF_HEX = [
  '0101010101010101010101010101010101010101010101010101010101010101',
  '27f32fbbfac2fbbbce58b10752144b5a7446d4b91e4ba90ffdee305e915980e8',
];
export const DEMO_LEAF_INDEX = 0;

export class FreighterNotDetectedError extends Error {}

export async function connectWallet(): Promise<string> {
  const { isConnected, error: connErr } = await freighter.isConnected();
  if (connErr || !isConnected) {
    throw new FreighterNotDetectedError(
      'Freighter wallet extension not detected. Install it from freighter.app to use real wallet features.'
    );
  }
  const { address, error } = await freighter.requestAccess();
  if (error || !address) {
    throw new Error(error?.message ?? 'Wallet access was not granted.');
  }
  const { network, error: netErr } = await freighter.getNetwork();
  if (netErr) throw new Error(netErr.message ?? 'Could not read wallet network.');
  if (network !== 'TESTNET') {
    throw new Error(`Freighter is set to ${network}, but this app talks to Stellar testnet. Switch networks in Freighter.`);
  }
  return address;
}

async function getClient(contractId: string, publicKey?: string) {
  return ContractClient.from({
    contractId,
    networkPassphrase: NETWORK_PASSPHRASE,
    rpcUrl: RPC_URL,
    publicKey,
    signTransaction: freighter.signTransaction,
  });
}

export interface DIDRecord {
  owner: string;
  document: string;
  active: boolean;
  created_at: bigint;
  updated_at: bigint;
}

/** Real, live register_did call for whichever wallet is connected — fully self-service,
 * no admin key or precomputed fixture needed, unlike the credential-verification demo
 * below. Requires a connected wallet and a Freighter signature. */
export async function registerRealDid(ownerPublicKey: string, document: string): Promise<string> {
  const client = await getClient(DID_REGISTRY_ID, ownerPublicKey);
  const tx = await (client as any).register_did(
    { owner: ownerPublicKey, document },
    // Generous window for real human Freighter review time — did_registry's own calls
    // have no timestamp arguments to get wrong, but the signed transaction ENVELOPE still
    // needs enough time to actually get signed. See stellar-zkstream's soroban.ts for the
    // measured clock-drift bug this guards against in general.
    { timeoutInSeconds: 1800 }
  );
  const sent = await tx.signAndSend();
  return sent.result as string;
}

export async function resolveRealDid(address: string): Promise<DIDRecord | null> {
  const client = await getClient(DID_REGISTRY_ID);
  const tx = await (client as any).resolve_did({ address });
  return (tx.result as DIDRecord | null) ?? null;
}

/** Read-only: simulates a real call against the real deployed credential_verifier with a
 * real Merkle proof for the one credential an ASP has actually attested to so far. No
 * wallet needed. */
export async function verifyRealCredentialOnChain(): Promise<boolean> {
  const client = await getClient(CREDENTIAL_VERIFIER_ID);
  const tx = await (client as any).verify_proof({
    user: DEMO_CREDENTIAL_SUBJECT,
    credential_type: DEMO_CREDENTIAL_TYPE,
    asp: DEMO_CREDENTIAL_ASP,
    merkle_proof: DEMO_MERKLE_PROOF_HEX.map((h) => Buffer.from(hexToBytes(h))),
    leaf_index: DEMO_LEAF_INDEX,
    expiration_time: 0n,
  });
  return tx.result as boolean;
}

export async function hasRealCredential(user: string, credentialType: string): Promise<boolean> {
  const client = await getClient(CREDENTIAL_VERIFIER_ID);
  const tx = await (client as any).has_credential({ user, credential_type: credentialType });
  return tx.result as boolean;
}

export interface ReputationData {
  owner: string;
  score: bigint;
  token_id: bigint;
  minted_at: bigint;
  updated_at: bigint;
}

export async function getRealReputation(subject: string): Promise<ReputationData | null> {
  const client = await getClient(REPUTATION_NFT_ID);
  const tx = await (client as any).get_reputation({ subject });
  return (tx.result as ReputationData | null) ?? null;
}
