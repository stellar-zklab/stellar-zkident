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
});
