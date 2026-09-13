import React, { useEffect, useState } from 'react';
import {
  resolveRealDid,
  hasRealCredential,
  getRealReputation,
  hasClaimedFromFaucet,
  getRealStreamsForAddress,
  getRealVaultPosition,
  getRealWalletDetails,
  DEMO_CREDENTIAL_TYPE,
  STREAM_CONTRACT_ID,
  VAULT_CONTRACT_ID,
  DIDRecord,
  ReputationData,
  StreamWithClaimable,
  VaultPosition,
  WalletDetails,
} from './soroban';

// Aggregates real, live reads across FOUR contracts from THREE repos, two of them in a
// different GitHub org — did_registry/credential_verifier/reputation_nft/sybil_resistant_faucet
// (this repo), stream (stellar-zkstream), vault (soroban-yield-vault, stellar-zklab), and
// account_abstraction_wallet (soroban-gasless-contracts, stellar-gasless-net). Every read here
// is public on-chain state — no wallet signature needed for any of it, and nothing here can
// write to any of these contracts. See soroban.ts for the real cross-repo contract IDs and
// why the wallet lookup needs a contract ID typed in rather than being auto-discovered.

interface PortfolioProps {
  initialAddress?: string;
}

type LoadState<T> = { loading: boolean; error: string | null; data: T | null };

const initialLoadState = <T,>(): LoadState<T> => ({ loading: false, error: null, data: null });

function short(addr: string): string {
  return `${addr.substring(0, 6)}...${addr.substring(addr.length - 4)}`;
}

function xlm(stroops: bigint): string {
  return (Number(stroops) / 10_000_000).toLocaleString(undefined, { maximumFractionDigits: 7 });
}

export const Portfolio: React.FC<PortfolioProps> = ({ initialAddress }) => {
  const [address, setAddress] = useState(initialAddress ?? '');
  const [queried, setQueried] = useState(false);

  const [did, setDid] = useState(initialLoadState<DIDRecord | null>());
  const [credential, setCredential] = useState(initialLoadState<boolean>());
  const [reputation, setReputation] = useState(initialLoadState<ReputationData | null>());
  const [faucet, setFaucet] = useState(initialLoadState<boolean>());
  const [streams, setStreams] = useState(initialLoadState<StreamWithClaimable[]>());
  const [vault, setVault] = useState(initialLoadState<VaultPosition>());

  const [walletContractId, setWalletContractId] = useState('');
  const [wallet, setWallet] = useState(initialLoadState<WalletDetails>());

  const loadPortfolio = async (addr: string) => {
    if (!addr) return;
    setQueried(true);

    setDid({ loading: true, error: null, data: null });
    resolveRealDid(addr)
      .then((data) => setDid({ loading: false, error: null, data }))
      .catch((err: any) => setDid({ loading: false, error: err.message ?? String(err), data: null }));

    setCredential({ loading: true, error: null, data: null });
    hasRealCredential(addr, DEMO_CREDENTIAL_TYPE)
      .then((data) => setCredential({ loading: false, error: null, data }))
      .catch((err: any) => setCredential({ loading: false, error: err.message ?? String(err), data: null }));

    setReputation({ loading: true, error: null, data: null });
    getRealReputation(addr)
      .then((data) => setReputation({ loading: false, error: null, data }))
      .catch((err: any) => setReputation({ loading: false, error: err.message ?? String(err), data: null }));

    setFaucet({ loading: true, error: null, data: null });
    hasClaimedFromFaucet(addr)
      .then((data) => setFaucet({ loading: false, error: null, data }))
      .catch((err: any) => setFaucet({ loading: false, error: err.message ?? String(err), data: null }));

    setStreams({ loading: true, error: null, data: null });
    getRealStreamsForAddress(addr)
      .then((data) => setStreams({ loading: false, error: null, data }))
      .catch((err: any) => setStreams({ loading: false, error: err.message ?? String(err), data: null }));

    setVault({ loading: true, error: null, data: null });
    getRealVaultPosition(addr)
      .then((data) => setVault({ loading: false, error: null, data }))
      .catch((err: any) => setVault({ loading: false, error: err.message ?? String(err), data: null }));
  };

  useEffect(() => {
    if (initialAddress) loadPortfolio(initialAddress);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialAddress]);

  const loadWallet = async () => {
    if (!walletContractId) return;
    setWallet({ loading: true, error: null, data: null });
    try {
      const data = await getRealWalletDetails(walletContractId);
      setWallet({ loading: false, error: null, data });
    } catch (err: any) {
      setWallet({ loading: false, error: err.message ?? String(err), data: null });
    }
  };

  const cardStyle: React.CSSProperties = {
    background: '#131022',
    padding: '1.5rem',
    borderRadius: '10px',
    border: '1px solid #231d3d',
    display: 'flex',
    flexDirection: 'column',
    gap: '0.6rem',
  };
  const labelStyle: React.CSSProperties = { fontSize: '0.75rem', color: '#64748b', textTransform: 'uppercase', letterSpacing: '0.05em' };
  const valueStyle: React.CSSProperties = { fontSize: '0.9rem', color: '#f8fafc' };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <section style={{ background: '#131022', padding: '1.5rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        <h2 style={{ fontSize: '1rem', fontWeight: 600, margin: 0, color: '#f8fafc' }}>Ecosystem Portfolio</h2>
        <p style={{ fontSize: '0.8rem', color: '#94a3b8', margin: 0 }}>
          One address, real state pulled live from four contracts across three repos — this one, <code>stellar-zkstream</code>, and{' '}
          <code>soroban-yield-vault</code> (all read-only, no wallet signature needed). Most addresses will show empty or zero
          unless they've actually interacted with these contracts.
        </p>
        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
          <input
            type="text"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="G... address to look up"
            style={{ flex: 1, minWidth: '280px', padding: '0.6rem 0.85rem', background: '#08060f', border: '1px solid #231d3d', color: '#f8fafc', borderRadius: '6px', fontSize: '0.8rem', fontFamily: 'monospace', outline: 'none' }}
          />
          <button
            onClick={() => loadPortfolio(address)}
            disabled={!address}
            style={{ padding: '0.6rem 1.2rem', background: '#4f46e5', color: '#fff', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.8rem' }}
          >
            Load Portfolio
          </button>
        </div>
      </section>

      {queried && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '1rem' }}>
          <div style={cardStyle}>
            <div style={labelStyle}>Identity — did_registry</div>
            {did.loading ? (
              <div style={valueStyle}>Reading...</div>
            ) : did.error ? (
              <div style={{ ...valueStyle, color: '#f87171' }}>{did.error}</div>
            ) : did.data ? (
              <>
                <div style={{ ...valueStyle, fontFamily: 'monospace', wordBreak: 'break-all' }}>{did.data.document}</div>
                <div style={{ fontSize: '0.7rem', color: '#64748b' }}>{did.data.active ? 'Active' : 'Deactivated'}</div>
              </>
            ) : (
              <div style={{ ...valueStyle, color: '#64748b' }}>No DID registered for this address.</div>
            )}
          </div>

          <div style={cardStyle}>
            <div style={labelStyle}>Verified Credential — credential_verifier</div>
            {credential.loading ? (
              <div style={valueStyle}>Reading...</div>
            ) : credential.error ? (
              <div style={{ ...valueStyle, color: '#f87171' }}>{credential.error}</div>
            ) : (
              <div style={{ ...valueStyle, color: credential.data ? '#5eead4' : '#64748b' }}>
                {credential.data ? `Verified ("${DEMO_CREDENTIAL_TYPE}")` : 'Not verified'}
              </div>
            )}
          </div>

          <div style={cardStyle}>
            <div style={labelStyle}>Reputation — reputation_nft</div>
            {reputation.loading ? (
              <div style={valueStyle}>Reading...</div>
            ) : reputation.error ? (
              <div style={{ ...valueStyle, color: '#f87171' }}>{reputation.error}</div>
            ) : reputation.data ? (
              <>
                <div style={{ fontSize: '1.75rem', fontWeight: 800, color: '#a78bfa' }}>{reputation.data.score.toString()}</div>
                <div style={{ fontSize: '0.7rem', color: '#64748b' }}>Token #{reputation.data.token_id.toString()}</div>
              </>
            ) : (
              <div style={{ ...valueStyle, color: '#64748b' }}>No score minted.</div>
            )}
          </div>

          <div style={cardStyle}>
            <div style={labelStyle}>Faucet — sybil_resistant_faucet</div>
            {faucet.loading ? (
              <div style={valueStyle}>Reading...</div>
            ) : faucet.error ? (
              <div style={{ ...valueStyle, color: '#f87171' }}>{faucet.error}</div>
            ) : (
              <div style={{ ...valueStyle, color: faucet.data ? '#5eead4' : '#64748b' }}>
                {faucet.data ? 'Already claimed' : 'Not claimed yet'}
              </div>
            )}
          </div>

          <div style={{ ...cardStyle, gridColumn: 'span 2' }}>
            <div style={labelStyle}>Streams — stellar-zkstream ({short(STREAM_CONTRACT_ID)})</div>
            {streams.loading ? (
              <div style={valueStyle}>Reading...</div>
            ) : streams.error ? (
              <div style={{ ...valueStyle, color: '#f87171' }}>{streams.error}</div>
            ) : streams.data && streams.data.length > 0 ? (
              streams.data.map((s) => (
                <div key={s.id.toString()} style={{ background: '#08060f', padding: '0.75rem', borderRadius: '6px', border: '1px solid #1c1733', fontSize: '0.75rem', display: 'flex', justifyContent: 'space-between', flexWrap: 'wrap', gap: '0.5rem' }}>
                  <span>
                    Stream #{s.id.toString()} &middot; {s.sender === address ? 'sending to' : 'receiving from'}{' '}
                    {short(s.sender === address ? s.recipient : s.sender)}
                  </span>
                  <span style={{ color: '#5eead4' }}>{xlm(s.claimable)} claimable of {xlm(s.total_amount)}</span>
                </div>
              ))
            ) : (
              <div style={{ ...valueStyle, color: '#64748b' }}>No streams for this address.</div>
            )}
          </div>

          <div style={{ ...cardStyle, gridColumn: 'span 2' }}>
            <div style={labelStyle}>Vault Position — soroban-yield-vault ({short(VAULT_CONTRACT_ID)})</div>
            {vault.loading ? (
              <div style={valueStyle}>Reading...</div>
            ) : vault.error ? (
              <div style={{ ...valueStyle, color: '#f87171' }}>{vault.error}</div>
            ) : vault.data && vault.data.shares > 0n ? (
              <div style={valueStyle}>
                {vault.data.shares.toString()} shares &middot; {xlm(vault.data.assetValue)} real underlying value
              </div>
            ) : (
              <div style={{ ...valueStyle, color: '#64748b' }}>No vault position for this address.</div>
            )}
          </div>
        </div>
      )}

      <section style={{ background: '#131022', padding: '1.5rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        <h3 style={{ fontSize: '0.9rem', fontWeight: 600, margin: 0, color: '#f8fafc' }}>Smart Wallet — soroban-gasless-contracts (cross-org)</h3>
        <p style={{ fontSize: '0.8rem', color: '#94a3b8', margin: 0 }}>
          Unlike everything above, a smart wallet can't be discovered from an owner address alone — there's no on-chain
          registry mapping owner → wallet contract IDs anywhere in this ecosystem yet. If you know a wallet's contract ID,
          paste it below.
        </p>
        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
          <input
            type="text"
            value={walletContractId}
            onChange={(e) => setWalletContractId(e.target.value)}
            placeholder="C... wallet contract ID"
            style={{ flex: 1, minWidth: '280px', padding: '0.6rem 0.85rem', background: '#08060f', border: '1px solid #231d3d', color: '#f8fafc', borderRadius: '6px', fontSize: '0.8rem', fontFamily: 'monospace', outline: 'none' }}
          />
          <button
            onClick={loadWallet}
            disabled={!walletContractId}
            style={{ padding: '0.6rem 1.2rem', background: '#1c1733', color: '#a78bfa', border: '1px solid #312952', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.8rem' }}
          >
            Look Up
          </button>
        </div>
        {wallet.loading ? (
          <div style={valueStyle}>Reading...</div>
        ) : wallet.error ? (
          <div style={{ ...valueStyle, color: '#f87171' }}>{wallet.error}</div>
        ) : wallet.data ? (
          <div style={{ fontSize: '0.8rem', color: '#f8fafc', fontFamily: 'monospace' }}>
            owner: {wallet.data.owner}
            <br />
            recovery signer: {wallet.data.recoverySigner ?? 'none set'}
          </div>
        ) : null}
      </section>
    </div>
  );
};

export default Portfolio;
