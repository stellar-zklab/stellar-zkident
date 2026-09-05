import { describe, it, expect, vi } from 'vitest';
import { StellarZkIdentClient } from '../src/client';

const dummySign = vi.fn(async (xdr: string) => xdr);

const baseConfig = {
  didRegistryId: 'CDID0000000000000000000000000000000000000000000000000000000',
  credentialVerifierId: 'CCVER00000000000000000000000000000000000000000000000000000',
  signTransaction: dummySign,
};

describe('StellarZkIdentClient config', () => {
  it('constructs without a reputationNftId', () => {
    const client = new StellarZkIdentClient(baseConfig);
    expect(client).toBeInstanceOf(StellarZkIdentClient);
  });

  it('getReputation() rejects clearly when reputationNftId was not configured', async () => {
    const client = new StellarZkIdentClient(baseConfig);
    await expect(client.getReputation('GSOMEADDRESS')).rejects.toThrow('reputationNftId');
  });

  it('verifyAgeProof() rejects clearly when ageProofVerifierId was not configured', async () => {
    const client = new StellarZkIdentClient(baseConfig);
    await expect(client.verifyAgeProof(new Uint8Array(), [])).rejects.toThrow('ageProofVerifierId');
  });

  it('verifyKycTierProof() rejects clearly when kycTierVerifierId was not configured', async () => {
    const client = new StellarZkIdentClient(baseConfig);
    await expect(client.verifyKycTierProof(new Uint8Array(), [])).rejects.toThrow('kycTierVerifierId');
  });

  it('verifyMembershipProof() rejects clearly when membershipVerifierId was not configured', async () => {
    const client = new StellarZkIdentClient(baseConfig);
    await expect(client.verifyMembershipProof(new Uint8Array(), [])).rejects.toThrow('membershipVerifierId');
  });
});
