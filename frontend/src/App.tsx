import React, { useState } from 'react';
import './index.css';
import {
  connectWallet,
  registerRealDid,
  resolveRealDid,
  verifyRealCredentialOnChain,
  DID_REGISTRY_ID,
  CREDENTIAL_VERIFIER_ID,
  DEMO_CREDENTIAL_SUBJECT,
  DEMO_CREDENTIAL_TYPE,
  DIDRecord,
} from './soroban';

// This UI is wired to REAL, deployed Stellar testnet contracts for two things — DID
// registration and one pre-registered credential's Merkle verification — not mocked. See
// soroban.ts and ../deployments/testnet.json. The four Noir-circuit provers below (age,
// KYC tier, residency, ASP Merkle membership as zero-knowledge proofs) remain genuinely
// unverified: no Noir/UltraPlonk proof system is wired to anything on-chain yet, and
// there's real doubt Soroban's host functions support UltraPlonk verification at all
// (they target Groth16/BN254 pairing checks, which is why stellar-zkstream's circuits
// could go real and these can't yet without a redesign). That section is left as an
// honest, explicitly-labeled mockup rather than silently removed.

interface CredentialItem {
  id: string;
  name: string;
  circuit: string;
  status: 'Verified (demo only)' | 'Unverified';
}

export const App: React.FC = () => {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [walletError, setWalletError] = useState<string | null>(null);
  const [provingId, setProvingId] = useState<string | null>(null);
  const [didDocument, setDidDocument] = useState('');
  const [registeringDid, setRegisteringDid] = useState(false);
  const [resolvedDid, setResolvedDid] = useState<DIDRecord | null | undefined>(undefined);
  const [verifyingCredential, setVerifyingCredential] = useState(false);

  const [logs, setLogs] = useState<string[]>([
    `[REAL] This app talks to real deployed contracts on Stellar testnet — did_registry: ${DID_REGISTRY_ID}`,
  ]);
  const appendLog = (line: string) => setLogs((prev) => [...prev, line]);

  const [credentials, setCredentials] = useState<CredentialItem[]>([
    { id: 'age', name: 'Age Compliance (Age ≥ 18)', circuit: 'age_proof.nr', status: 'Unverified' },
    { id: 'kyc', name: 'KYC Attestation (Tier ≥ Silver)', circuit: 'kyc_tier_proof.nr', status: 'Unverified' },
    { id: 'residency', name: 'Jurisdiction Compliance', circuit: 'residency_proof.nr', status: 'Unverified' },
    { id: 'merkle', name: 'ASP Merkle Membership', circuit: 'membership_proof.nr', status: 'Unverified' }
  ]);

  const handleConnect = async () => {
    setWalletError(null);
    try {
      const address = await connectWallet();
      setWalletAddress(address);
      appendLog(`[REAL] Connected real Freighter wallet: ${address.substring(0, 8)}...`);
    } catch (err: any) {
      setWalletError(err.message ?? String(err));
      appendLog(`[REAL] Wallet connection failed: ${err.message ?? err}`);
    }
  };

  const handleRegisterDid = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!walletAddress || !didDocument) return;
    setRegisteringDid(true);
    appendLog('[REAL] Submitting a real register_did transaction — this needs your wallet signature.');
    try {
      await registerRealDid(walletAddress, didDocument);
      appendLog('[REAL] Transaction confirmed. Your DID is now really registered on testnet.');
      const record = await resolveRealDid(walletAddress);
      setResolvedDid(record);
    } catch (err: any) {
      appendLog(`[REAL] register_did failed: ${err.message ?? err}`);
    } finally {
      setRegisteringDid(false);
    }
  };

  const handleResolveDid = async () => {
    if (!walletAddress) return;
    appendLog(`[REAL] Calling resolve_did on the real deployed did_registry for ${walletAddress.substring(0, 8)}...`);
    try {
      const record = await resolveRealDid(walletAddress);
      setResolvedDid(record);
      appendLog(record ? '[REAL] A real DID record was found on-chain.' : '[REAL] No DID registered for this address yet.');
    } catch (err: any) {
      appendLog(`[REAL] resolve_did failed: ${err.message ?? err}`);
    }
  };

  const handleVerifyCredential = async () => {
    setVerifyingCredential(true);
    appendLog(`[REAL] Calling verify_proof on the real deployed verifier (${CREDENTIAL_VERIFIER_ID.substring(0, 8)}...) with a real Merkle proof for the pre-registered demo identity...`);
    try {
      const result = await verifyRealCredentialOnChain();
      appendLog(`[REAL] Testnet responded: verify_proof() = ${result}. This is a live simulateTransaction call against the one credential actually registered so far, not a mock.`);
    } catch (err: any) {
      appendLog(`[REAL] On-chain verification call failed: ${err.message ?? err}`);
    } finally {
      setVerifyingCredential(false);
    }
  };

  const handleProve = (id: string) => {
    setProvingId(id);
    appendLog(`[DEMO] Walking through the "prove ${id}" UI — no real Noir circuit runs, no proof is generated.`);

    setTimeout(() => {
      setCredentials(prev => prev.map(c => c.id === id ? { ...c, status: 'Verified (demo only)' } : c));
      setProvingId(null);
      appendLog(`[DEMO] Marked "${id}" as demo-verified in this browser tab only. No Noir/UltraPlonk proof system is wired to on-chain verification yet.`);
    }, 600);
  };

  return (
    <div style={{ minHeight: '100vh', backgroundColor: '#0b0914', color: '#e0e0e0' }}>
      <div style={{ background: 'linear-gradient(135deg, #4338ca, #3730a3)', color: '#fff', padding: '0.65rem 1.5rem', fontSize: '0.85rem', fontWeight: 600, textAlign: 'center' }}>
        ✓ DID registry &amp; one pre-registered credential are wired to real deployed contracts. The Noir circuit provers below are still an honest mockup — see banner in soroban.ts / App.tsx.
      </div>
      <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem 1.5rem', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>

        <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#131022', padding: '1rem 1.5rem', borderRadius: '10px', border: '1px solid #231d3d' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
            <h1 style={{ fontSize: '1.25rem', fontWeight: 700, margin: 0, color: '#818cf8' }}>stellar-zkident</h1>
            <span style={{ fontSize: '0.75rem', background: 'rgba(67, 56, 202, 0.2)', color: '#a5b4fc', padding: '0.2rem 0.5rem', borderRadius: '4px', border: '1px solid rgba(67, 56, 202, 0.4)', fontWeight: 600 }}>
              Testnet — Real Contracts
            </span>
          </div>

          <button
            onClick={handleConnect}
            style={{ padding: '0.5rem 1rem', background: '#1c1733', color: '#a78bfa', border: '1px solid #312952', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
          >
            {walletAddress ? `${walletAddress.substring(0, 6)}...${walletAddress.substring(walletAddress.length - 4)}` : 'Connect Freighter Wallet'}
          </button>
        </header>

        {walletError && (
          <div style={{ background: 'rgba(190, 18, 60, 0.15)', border: '1px solid rgba(190, 18, 60, 0.4)', color: '#fda4af', padding: '0.75rem 1rem', borderRadius: '8px', fontSize: '0.85rem' }}>
            {walletError}
          </div>
        )}

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>

          {/* Real DID registry */}
          <section style={{ background: '#131022', padding: '1.75rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <h2 style={{ fontSize: '1rem', fontWeight: 600, margin: 0, color: '#f8fafc' }}>Real DID Registry</h2>
            <p style={{ fontSize: '0.8rem', color: '#94a3b8', margin: 0 }}>
              Registers a real <code>did:stellar:</code> document for your connected wallet on the real deployed did_registry contract.
            </p>
            <form onSubmit={handleRegisterDid} style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              <textarea
                placeholder='{"service": "example"}'
                value={didDocument}
                onChange={(e) => setDidDocument(e.target.value)}
                rows={3}
                style={{ width: '100%', padding: '0.75rem 1rem', background: '#08060f', border: '1px solid #231d3d', color: '#f8fafc', borderRadius: '6px', fontSize: '0.85rem', outline: 'none', boxSizing: 'border-box', fontFamily: 'monospace', resize: 'vertical' }}
              />
              <div style={{ display: 'flex', gap: '0.75rem' }}>
                <button
                  type="submit"
                  disabled={!walletAddress || !didDocument || registeringDid}
                  style={{ flex: 1, padding: '0.75rem', background: registeringDid ? '#312952' : '#4f46e5', color: '#fff', border: 'none', borderRadius: '6px', cursor: registeringDid ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
                >
                  {!walletAddress ? 'Connect wallet first' : registeringDid ? 'Submitting...' : 'Register Real DID (Signs & Submits)'}
                </button>
                <button
                  type="button"
                  onClick={handleResolveDid}
                  disabled={!walletAddress}
                  style={{ padding: '0.75rem 1rem', background: '#1c1733', color: '#a78bfa', border: '1px solid #312952', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
                >
                  Resolve Mine
                </button>
              </div>
            </form>
            {resolvedDid !== undefined && (
              <div style={{ background: '#08060f', padding: '1rem', borderRadius: '8px', border: '1px solid #1c1733', fontSize: '0.8rem', fontFamily: 'monospace', color: resolvedDid ? '#5eead4' : '#fbbf24' }}>
                {resolvedDid ? JSON.stringify(resolvedDid, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2) : 'No DID registered for this address yet.'}
              </div>
            )}

            <div style={{ borderTop: '1px solid #231d3d', paddingTop: '1rem', marginTop: '0.5rem' }}>
              <h3 style={{ fontSize: '0.9rem', fontWeight: 600, margin: '0 0 0.5rem 0', color: '#f8fafc' }}>Real Credential Verification</h3>
              <p style={{ fontSize: '0.8rem', color: '#94a3b8', margin: '0 0 0.75rem 0' }}>
                Verifies a real Merkle proof against the one credential actually registered by an ASP so far — for a fixed demo identity, not your connected wallet (no ASP has attested to an arbitrary wallet's credential yet). No signature needed.
              </p>
              <button
                onClick={handleVerifyCredential}
                disabled={verifyingCredential}
                style={{ width: '100%', padding: '0.75rem', background: '#1c1733', color: '#a78bfa', border: '1px solid #312952', borderRadius: '6px', cursor: verifyingCredential ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
              >
                {verifyingCredential ? 'Verifying on-chain...' : `Verify Real "${DEMO_CREDENTIAL_TYPE}" Credential On-Chain`}
              </button>
              <p style={{ fontSize: '0.7rem', color: '#64748b', margin: '0.5rem 0 0 0', fontFamily: 'monospace' }}>
                subject: {DEMO_CREDENTIAL_SUBJECT.substring(0, 12)}...
              </p>
            </div>
          </section>

          {/* Right Column */}
          <section style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            <div style={{ background: '#131022', padding: '1.5rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              <h2 style={{ fontSize: '0.95rem', fontWeight: 600, margin: 0, color: '#f8fafc' }}>
                Noir Circuit Provers (Still Not Real)
              </h2>
              {credentials.map(c => (
                <div key={c.id} style={{ background: '#08060f', padding: '1rem', borderRadius: '8px', border: '1px solid #1c1733', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <h3 style={{ fontSize: '0.875rem', fontWeight: 600, margin: 0, color: '#f8fafc' }}>{c.name}</h3>
                    <small style={{ color: '#818cf8', fontFamily: 'monospace', fontSize: '0.75rem' }}>{c.circuit}</small>
                  </div>
                  {c.status === 'Verified (demo only)' ? (
                    <span style={{ background: 'rgba(180, 83, 9, 0.2)', color: '#fbbf24', padding: '0.3rem 0.6rem', borderRadius: '4px', fontSize: '0.75rem', fontWeight: 600 }}>
                      Demo Only — Not Verified
                    </span>
                  ) : (
                    <button
                      onClick={() => handleProve(c.id)}
                      disabled={provingId === c.id}
                      style={{ padding: '0.4rem 0.8rem', background: provingId === c.id ? '#312952' : '#4f46e5', color: '#ffffff', border: 'none', borderRadius: '6px', cursor: provingId === c.id ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.8rem' }}
                    >
                      {provingId === c.id ? 'Running demo...' : 'Run Demo (Not Real Proof)'}
                    </button>
                  )}
                </div>
              ))}
            </div>

            <div style={{ background: '#08060f', padding: '1.5rem', borderRadius: '10px', border: '1px solid #231d3d', display: 'flex', flexDirection: 'column', flex: 1 }}>
              <h2 style={{ fontSize: '0.95rem', fontWeight: 600, color: '#94a3b8', margin: '0 0 1rem 0' }}>
                Activity Log
              </h2>
              <div style={{ background: '#030208', padding: '1.25rem', borderRadius: '8px', border: '1px solid #131022', fontFamily: 'Fira Code, monospace', fontSize: '0.8rem', flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
                {logs.map((log, idx) => (
                  <div key={idx} style={{ color: log.startsWith('[REAL]') ? '#5eead4' : '#fbbf24' }}>
                    {log}
                  </div>
                ))}
              </div>
            </div>
          </section>

        </div>

      </div>
    </div>
  );
};
export default App;
