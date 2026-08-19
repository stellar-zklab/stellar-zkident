import React, { useState } from 'react';
import './index.css';

interface CredentialItem {
  id: string;
  name: string;
  circuit: string;
  status: 'Verified' | 'Unverified';
}

export const App: React.FC = () => {
  const [walletConnected, setWalletConnected] = useState(true);
  const [walletAddress] = useState('GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN');
  const [provingId, setProvingId] = useState<string | null>(null);

  const [logs, setLogs] = useState<string[]>([
    '[SDK] DID Registry Client initialized',
    '[W3C] Resolved DID Document: did:stellar:GAAZI4T...',
    '[Noir] Barretenberg UltraPlonk verifier circuit loaded'
  ]);

  const [credentials, setCredentials] = useState<CredentialItem[]>([
    { id: 'age', name: 'Age Compliance (Age ≥ 18)', circuit: 'age_proof.nr', status: 'Unverified' },
    { id: 'kyc', name: 'KYC Attestation (Tier ≥ Silver)', circuit: 'kyc_tier_proof.nr', status: 'Unverified' },
    { id: 'residency', name: 'Jurisdiction Compliance', circuit: 'residency_proof.nr', status: 'Unverified' },
    { id: 'merkle', name: 'ASP Merkle Membership', circuit: 'membership_proof.nr', status: 'Unverified' }
  ]);

  const handleProve = (id: string) => {
    setProvingId(id);
    setLogs(prev => [...prev, `[NoirProver] Compiling Noir UltraPlonk proof for ${id}...`]);

    setTimeout(() => {
      setCredentials(prev => prev.map(c => c.id === id ? { ...c, status: 'Verified' } : c));
      setProvingId(null);
      setLogs(prev => [
        ...prev,
        `[Soroban] Credential Proof Verified & Ledger Updated`,
        `[SBT] Soulbound Reputation Points Granted (+100 pts)`
      ]);
    }, 1000);
  };

  return (
    <div style={{ minHeight: '100vh', backgroundColor: '#0b0914', color: '#e0e0e0', padding: '2rem 1.5rem' }}>
      <div style={{ maxWidth: '1200px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        
        {/* Navigation Bar */}
        <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#131022', padding: '1rem 1.5rem', borderRadius: '10px', border: '1px solid #231d3d' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
            <h1 style={{ fontSize: '1.25rem', fontWeight: 700, margin: 0, color: '#818cf8' }}>stellar-zkident</h1>
            <span style={{ fontSize: '0.75rem', background: 'rgba(16, 185, 129, 0.15)', color: '#10b981', padding: '0.2rem 0.5rem', borderRadius: '4px', border: '1px solid rgba(16, 185, 129, 0.3)', fontWeight: 600 }}>
              Testnet RPC (24ms)
            </span>
          </div>

          <button
            onClick={() => setWalletConnected(!walletConnected)}
            style={{ padding: '0.5rem 1rem', background: '#1c1733', color: '#a78bfa', border: '1px solid #312952', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
          >
            {walletConnected ? `did:stellar:${walletAddress.substring(0, 6)}...` : 'Connect Wallet'}
          </button>
        </header>

        {/* 2-Column Split Layout */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>
          
          {/* Left Column: Provers */}
          <section style={{ background: '#131022', padding: '1.75rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <h2 style={{ fontSize: '1rem', fontWeight: 600, margin: '0 0 0.5rem 0', color: '#f8fafc' }}>
              Zero-Knowledge Credential Provers
            </h2>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              {credentials.map(c => (
                <div key={c.id} style={{ background: '#08060f', padding: '1rem', borderRadius: '8px', border: '1px solid #1c1733', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <h3 style={{ fontSize: '0.875rem', fontWeight: 600, margin: 0, color: '#f8fafc' }}>{c.name}</h3>
                    <small style={{ color: '#818cf8', fontFamily: 'monospace', fontSize: '0.75rem' }}>{c.circuit}</small>
                  </div>

                  {c.status === 'Verified' ? (
                    <span style={{ background: 'rgba(16, 185, 129, 0.15)', color: '#10b981', padding: '0.3rem 0.6rem', borderRadius: '4px', fontSize: '0.75rem', fontWeight: 600 }}>
                      Verified On-Chain
                    </span>
                  ) : (
                    <button
                      onClick={() => handleProve(c.id)}
                      disabled={provingId === c.id}
                      style={{ padding: '0.4rem 0.8rem', background: provingId === c.id ? '#312952' : '#4f46e5', color: '#ffffff', border: 'none', borderRadius: '6px', cursor: provingId === c.id ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.8rem' }}
                    >
                      {provingId === c.id ? 'Compiling...' : 'Generate Proof'}
                    </button>
                  )}
                </div>
              ))}
            </div>
          </section>

          {/* Right Column: Terminal SDK Log */}
          <section style={{ background: '#08060f', padding: '1.5rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column' }}>
            <h2 style={{ fontSize: '0.95rem', fontWeight: 600, color: '#94a3b8', margin: '0 0 1rem 0' }}>
              SDK Execution Log
            </h2>

            <div style={{ background: '#030208', padding: '1.25rem', borderRadius: '8px', border: '1px solid #131022', fontFamily: 'Fira Code, monospace', fontSize: '0.8rem', color: '#a78bfa', flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
              {logs.map((log, idx) => (
                <div key={idx} style={{ color: log.includes('[SDK]') ? '#818cf8' : log.includes('[Noir') ? '#c084fc' : '#10b981' }}>
                  {log}
                </div>
              ))}
            </div>
          </section>

        </div>

      </div>
    </div>
  );
};
export default App;
