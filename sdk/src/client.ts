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
 * Note on naming: `credential_verifier.verify_proof()` checks classical Merkle inclusion,
 * not a zero-knowledge proof — the leaf is derived directly from the caller's real
 * address, so membership isn't hidden. See the main repo README's "Current Status"
 * section. Real zero-knowledge verification lives separately, in the three zk_verifier
 * instances this client's verifyAgeProof/verifyKycTierProof/verifyMembershipProof methods
 * call — genuine Groth16 BN254 proofs for age/KYC-tier/Merkle-membership claims, verified
 * on-chain without revealing the underlying private data. See ../circuits/README.md.
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
  /** Real zk_verifier instance addresses — one per circuit, see circuits/README.md. Each
   * is optional independently; only configure the ones your app actually uses. */
  ageProofVerifierId?: string;
  kycTierVerifierId?: string;
  membershipVerifierId?: string;
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
  private ageProofVerifierId?: string;
  private kycTierVerifierId?: string;
  private membershipVerifierId?: string;
  private rpcUrl: string;
  private networkPassphrase: string;
  private signTransaction: SignTransaction;

  constructor(config: StellarZkIdentConfig) {
    this.didRegistryId = config.didRegistryId;
    this.credentialVerifierId = config.credentialVerifierId;
    this.reputationNftId = config.reputationNftId;
    this.ageProofVerifierId = config.ageProofVerifierId;
    this.kycTierVerifierId = config.kycTierVerifierId;
    this.membershipVerifierId = config.membershipVerifierId;
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

  /** Read-only: verifies a real Groth16 BN254 proof against a deployed zk_verifier
   * instance. `proof` and `publicInputs` must be in the exact byte layout
   * circuits/convert_to_soroban.mjs produces — see circuits/README.md for how a real
   * proof for one of these circuits is generated. No signature needed; verification
   * itself reveals nothing about the private witness. */
  private async verifyZkProof(verifierId: string, proof: Uint8Array, publicInputs: Uint8Array[]): Promise<boolean> {
    const client = await this.getClient(verifierId);
    const tx = await (client as any).vrfy_prf({
      proof: Buffer.from(proof),
      public_inputs: publicInputs.map((b) => Buffer.from(b)),
    });
    return tx.result as boolean;
  }

  /** Verifies a real zero-knowledge proof that the prover is at least 18, without
   * revealing their birth date. Requires `ageProofVerifierId` in the client config. */
  async verifyAgeProof(proof: Uint8Array, publicInputs: Uint8Array[]): Promise<boolean> {
    if (!this.ageProofVerifierId) {
      throw new Error('verifyAgeProof() requires ageProofVerifierId in the client config');
    }
    return this.verifyZkProof(this.ageProofVerifierId, proof, publicInputs);
  }

  /** Verifies a real zero-knowledge proof that the prover holds at least a given KYC
   * tier, without revealing their actual tier. Requires `kycTierVerifierId` in the client
   * config. */
  async verifyKycTierProof(proof: Uint8Array, publicInputs: Uint8Array[]): Promise<boolean> {
    if (!this.kycTierVerifierId) {
      throw new Error('verifyKycTierProof() requires kycTierVerifierId in the client config');
    }
    return this.verifyZkProof(this.kycTierVerifierId, proof, publicInputs);
  }

  /** Verifies a real zero-knowledge proof of membership in a depth-20 Merkle tree,
   * without revealing which leaf, its value, or the sibling path. Requires
   * `membershipVerifierId` in the client config. */
  async verifyMembershipProof(proof: Uint8Array, publicInputs: Uint8Array[]): Promise<boolean> {
    if (!this.membershipVerifierId) {
      throw new Error('verifyMembershipProof() requires membershipVerifierId in the client config');
    }
    return this.verifyZkProof(this.membershipVerifierId, proof, publicInputs);
  }
}
