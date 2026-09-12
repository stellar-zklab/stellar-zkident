// Real integration against the actual deployed Stellar testnet contracts — no mocking
// here. See ../deployments/testnet.json for where these addresses come from and how to
// verify them independently on stellar.expert.
import { Client as ContractClient } from '@stellar/stellar-sdk/contract';
import { StellarWalletsKit } from '@creit.tech/stellar-wallets-kit/sdk';
import { FreighterModule } from '@creit.tech/stellar-wallets-kit/modules/freighter';
import { xBullModule } from '@creit.tech/stellar-wallets-kit/modules/xbull';
import { Networks } from '@creit.tech/stellar-wallets-kit/types';

export const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015';
export const RPC_URL = 'https://soroban-testnet.stellar.org';

export const DID_REGISTRY_ID = 'CDGDZX4OGVCWEYANDRSWKSK6LLYOGFRJDZQNFNNYPTQPAKELKR4TXLB6';
export const CREDENTIAL_VERIFIER_ID = 'CDLRSLHALMX6OU5IHWY6CKTROK3SYENEA75K6OWSZCPAW4EOTR2OZGSF';
export const REPUTATION_NFT_ID = 'CDA34SUCSQDOCCY5B6HJJH4CQ5PUDWII6CY3BDONGKT5E3KTEWZJ47GD';
// Deployed 2026-09-12 via scripts/deploy_sybil_resistant_faucet.sh — see deployments/testnet.json
// for the real claim()/reject/already-claimed exercises run against this exact instance.
export const SYBIL_RESISTANT_FAUCET_ID = 'CBNQ6BHR45SV5JMSKQTLIULDZFOR3DPAD4VIXGZSROVSLXXTKXIUM524';

// The demo subject really holds a minted reputation score — 2026-09-10, real verify_proof +
// mint calls, not a fixture. See deployments/testnet.json's notes.
export const DEMO_REPUTATION_SUBJECT = 'GAUZ4T6UT7XMGOL6WYPWWSYPZQ7ZLILCAS2ROYCH5ILHHOWQYUGVRTAB';

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

// Scoped to just Freighter + xBull rather than the kit's full module list (which also
// pulls in Ledger/Trezor hardware-wallet and WalletConnect support) — this is a testnet
// demo, not a production wallet, so only the two lightest, most commonly available
// options are wired in. init() is a one-time, module-level call since StellarWalletsKit's
// methods are static.
let walletKitReady = false;
function ensureWalletKit(): void {
  if (walletKitReady) return;
  StellarWalletsKit.init({
    modules: [new FreighterModule(), new xBullModule()],
    network: Networks.TESTNET,
  });
  walletKitReady = true;
}

/** Opens the kit's real wallet-picker modal (Freighter or xBull), and returns the real
 * connected address. The modal itself handles "wallet not installed" — there's no
 * separate not-detected error to catch here the way the old Freighter-only code needed. */
export async function connectWallet(): Promise<string> {
  ensureWalletKit();
  const { address } = await StellarWalletsKit.authModal();
  return address;
}

async function getClient(contractId: string, publicKey?: string) {
  ensureWalletKit();
  return ContractClient.from({
    contractId,
    networkPassphrase: NETWORK_PASSPHRASE,
    rpcUrl: RPC_URL,
    publicKey,
    signTransaction: StellarWalletsKit.signTransaction,
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
 * below. Requires a connected wallet's signature. */
export async function registerRealDid(ownerPublicKey: string, document: string): Promise<string> {
  const client = await getClient(DID_REGISTRY_ID, ownerPublicKey);
  const tx = await (client as any).register_did(
    { owner: ownerPublicKey, document },
    // Generous window for real human wallet review time — did_registry's own calls
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

/** Read-only: whether `user` has already claimed from the real deployed
 * sybil_resistant_faucet. No wallet needed. */
export async function hasClaimedFromFaucet(user: string): Promise<boolean> {
  const client = await getClient(SYBIL_RESISTANT_FAUCET_ID);
  const tx = await (client as any).has_claimed({ user });
  return tx.result as boolean;
}

export async function getFaucetClaimAmount(): Promise<bigint> {
  const client = await getClient(SYBIL_RESISTANT_FAUCET_ID);
  const tx = await (client as any).get_claim_amount();
  return tx.result as bigint;
}

/** Real, live claim() call against sybil_resistant_faucet, signed by whichever wallet is
 * connected. The contract gates this on a real cross-contract has_credential() check against
 * credential_verifier — only DEMO_CREDENTIAL_SUBJECT actually holds a verified credential
 * right now (see the note on DEMO_CREDENTIAL_SUBJECT above), so claiming with any other
 * connected wallet will genuinely revert on-chain with "claim requires a verified
 * credential" — that's the gate working correctly, not a bug. */
export async function claimFromRealFaucet(userPublicKey: string): Promise<bigint> {
  const client = await getClient(SYBIL_RESISTANT_FAUCET_ID, userPublicKey);
  const tx = await (client as any).claim({ user: userPublicKey }, { timeoutInSeconds: 1800 });
  const sent = await tx.signAndSend();
  return sent.result as bigint;
}

// --- Portfolio: real cross-repo reads --------------------------------------------------
// These contracts are deployed and maintained by OTHER repos in this ecosystem, not this
// one — stellar-zkstream and soroban-yield-vault (same org), soroban-gasless-contracts
// (a different org). Source of truth for these addresses is each repo's own
// deployments/testnet.json; if either contract is ever redeployed there, these need
// updating too, or this page will silently read a stale, possibly-abandoned instance
// instead of erroring. Verified current as of 2026-09-12 against both repos' own records.
export const STREAM_CONTRACT_ID = 'CACRWU5VCHIGBMSJZMWDXE3L6UJNJIQ7O4FH32ER3M77AO3Z23562MPH'; // stellar-zkstream
export const VAULT_CONTRACT_ID = 'CAQ6YR3XKGS774M7ERT5DTGMMPFYZ4WLAIMOPCUBGAJLQKPLFUG6AETK'; // soroban-yield-vault

export interface StreamData {
  sender: string;
  recipient: string;
  token: string;
  total_amount: bigint;
  withdrawn_amount: bigint;
  start_time: bigint;
  cliff_time: bigint;
  end_time: bigint;
  active: boolean;
  cancelable: boolean;
}

export interface StreamWithClaimable extends StreamData {
  id: bigint;
  claimable: bigint;
}

/** Real reads against stellar-zkstream's deployed `stream` contract — a different repo in
 * this ecosystem. Finds every stream `address` is a party to (as sender or recipient) via
 * the contract's own real address index (get_streams_by_sender/get_streams_by_recipient —
 * not a guess or a fixture), then reads each one's full data and current claimable amount.
 * No wallet needed; all of this is public on-chain state. */
export async function getRealStreamsForAddress(address: string): Promise<StreamWithClaimable[]> {
  const client = await getClient(STREAM_CONTRACT_ID);
  const [sentTx, receivedTx] = await Promise.all([
    (client as any).get_streams_by_sender({ sender: address }),
    (client as any).get_streams_by_recipient({ recipient: address }),
  ]);
  const sentIds = sentTx.result as bigint[];
  const receivedIds = receivedTx.result as bigint[];
  const ids = Array.from(new Set([...sentIds, ...receivedIds].map((id) => id.toString()))).map((s) => BigInt(s));

  return Promise.all(
    ids.map(async (id) => {
      const [streamTx, claimableTx] = await Promise.all([
        (client as any).get_stream({ stream_id: id }),
        (client as any).claimable_amount({ stream_id: id }),
      ]);
      return { ...(streamTx.result as StreamData), id, claimable: claimableTx.result as bigint };
    })
  );
}

export interface VaultPosition {
  shares: bigint;
  assetValue: bigint;
}

/** Real reads against soroban-yield-vault's deployed `vault` contract — a different repo.
 * balance_of returns a real share balance; convert_to_assets translates that into real
 * underlying-asset value at the vault's current real share price (not a fixed 1:1 guess).
 * No wallet needed. */
export async function getRealVaultPosition(address: string): Promise<VaultPosition> {
  const client = await getClient(VAULT_CONTRACT_ID);
  const sharesTx = await (client as any).balance_of({ user: address });
  const shares = sharesTx.result as bigint;
  if (shares === 0n) return { shares: 0n, assetValue: 0n };
  const assetsTx = await (client as any).convert_to_assets({ shares });
  return { shares, assetValue: assetsTx.result as bigint };
}

export interface WalletDetails {
  owner: string;
  recoverySigner: string | null;
}

/** Real reads against a soroban-gasless-contracts `account-abstraction-wallet` instance —
 * cross-ORG (stellar-gasless-net), not just cross-repo. Unlike the reads above, there is no
 * on-chain registry mapping an owner address to the wallet contract IDs they've deployed —
 * that index simply doesn't exist anywhere in this ecosystem yet — so this needs the
 * wallet's own contract ID supplied directly, not discovered from an owner address alone. */
export async function getRealWalletDetails(walletContractId: string): Promise<WalletDetails> {
  const client = await getClient(walletContractId);
  const ownerTx = await (client as any).get_owner();
  const recoveryTx = await (client as any).get_recovery_signer();
  return { owner: ownerTx.result as string, recoverySigner: (recoveryTx.result as string | null) ?? null };
}
