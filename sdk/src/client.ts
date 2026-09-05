/**
 * StellarZkIdentClient — TypeScript SDK for DID registry and ZK-adjacent credential
 * verification.
 *
 * This wraps the same `@stellar/stellar-sdk/contract` Client the deployed frontend uses
 * (see ../../frontend/src/soroban.ts) — real simulate/sign/submit calls against a real
 * Soroban RPC endpoint. Signing is injected via `signTransaction` rather than hard-wired
 * to Freighter, so this SDK works with any wallet adapter that can produce a signed
 * transaction XDR string.
 *
 * Note on naming: despite the repo name, `credential_verifier.verify_proof()` checks
 * classical Merkle inclusion, not a zero-knowledge proof — the leaf is derived directly
 * from the caller's real address, so membership isn't hidden. See the main repo README's
 * "Current Status" section.
 */
import { Client as ContractClient } from '@stellar/stellar-sdk/contract';

export type SignTransaction = (
  xdr: string,
  opts?: { network?: string; networkPassphrase?: string; accountToSign?: string }
) => Promise<string>;

export interface StellarZkIdentConfig {
  didRegistryId: string;
  credentialVerifierId: string;
  reputationNftId?: string;
  rpcUrl?: string;
  networkPassphrase?: string;
  signTransaction: SignTransaction;
}

export interface DIDRecord {
  owner: string;
  document: string;
  active: boolean;
  created_at: bigint;
  updated_at: bigint;
}

export interface ReputationData {
  owner: string;
  score: bigint;
  token_id: bigint;
  minted_at: bigint;
  updated_at: bigint;
}

export interface MerkleCredentialProof {
  user: string;
  credentialType: string;
  asp: string;
  merkleProof: Uint8Array[];
  leafIndex: number;
  expirationTime?: bigint;
}

export class StellarZkIdentClient {
  private didRegistryId: string;
  private credentialVerifierId: string;
  private reputationNftId?: string;
  private rpcUrl: string;
  private networkPassphrase: string;
  private signTransaction: SignTransaction;

  constructor(config: StellarZkIdentConfig) {
    this.didRegistryId = config.didRegistryId;
    this.credentialVerifierId = config.credentialVerifierId;
    this.reputationNftId = config.reputationNftId;
    this.rpcUrl = config.rpcUrl ?? 'https://soroban-testnet.stellar.org';
    this.networkPassphrase = config.networkPassphrase ?? 'Test SDF Network ; September 2015';
    this.signTransaction = config.signTransaction;
  }

  private async getClient(contractId: string, publicKey?: string) {
    return ContractClient.from({
      contractId,
      networkPassphrase: this.networkPassphrase,
      rpcUrl: this.rpcUrl,
      publicKey,
      signTransaction: this.signTransaction,
    });
  }

  /** Real, live register_did call for `owner` — fully self-service, no admin key needed.
   * Requires `owner` to sign. */
  async registerDid(owner: string, document: string): Promise<string> {
    const client = await this.getClient(this.didRegistryId, owner);
    const tx = await (client as any).register_did({ owner, document }, { timeoutInSeconds: 1800 });
    const sent = await tx.signAndSend();
    return sent.result as string;
  }

  /** Read-only: a real DID record from on-chain state, or null if none is registered. */
  async resolveDid(address: string): Promise<DIDRecord | null> {
    const client = await this.getClient(this.didRegistryId);
    const tx = await (client as any).resolve_did({ address });
    return (tx.result as DIDRecord | null) ?? null;
  }

  /** Read-only: whether `address` currently has an active DID. */
  async isDidActive(address: string): Promise<boolean> {
    const client = await this.getClient(this.didRegistryId);
    const tx = await (client as any).is_active({ address });
    return tx.result as boolean;
  }

  /** Verifies a real Merkle inclusion proof against the credential_verifier's on-chain
   * asp_registry root — NOT a zero-knowledge proof (see class-level note above). Requires
   * `proof.user` to sign, per the contract's own require_auth(). */
  async verifyCredential(proof: MerkleCredentialProof): Promise<boolean> {
    const client = await this.getClient(this.credentialVerifierId, proof.user);
    const tx = await (client as any).verify_proof({
      user: proof.user,
      credential_type: proof.credentialType,
      asp: proof.asp,
      merkle_proof: proof.merkleProof.map((b) => Buffer.from(b)),
      leaf_index: proof.leafIndex,
      expiration_time: proof.expirationTime ?? 0n,
    });
    return tx.result as boolean;
  }

  /** Read-only: whether `user` has a currently-recorded credential of `credentialType`. */
  async hasCredential(user: string, credentialType: string): Promise<boolean> {
    const client = await this.getClient(this.credentialVerifierId);
    const tx = await (client as any).has_credential({ user, credential_type: credentialType });
    return tx.result as boolean;
  }

  /** Read-only: a subject's real reputation record, or null if none has been minted.
   * Requires `reputationNftId` in the client config. */
  async getReputation(subject: string): Promise<ReputationData | null> {
    if (!this.reputationNftId) {
      throw new Error('getReputation() requires reputationNftId in the client config');
    }
    const client = await this.getClient(this.reputationNftId);
    const tx = await (client as any).get_reputation({ subject });
    return (tx.result as ReputationData | null) ?? null;
  }
}
