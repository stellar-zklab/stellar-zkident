# Circuits

`age_proof/age_proof.circom`, `kyc_tier_proof/kyc_tier_proof.circom`, and
`membership_proof/membership_proof.circom` are real Circom circuits using Poseidon (via
`circomlib`) — not placeholders, and not the Noir circuits this repo originally shipped.

## Why Circom, not Noir

This repo originally had these three circuits written in Noir. Noir's default proving
backend (Barretenberg) produces UltraHonk proofs, which need a "cycle of curves" — BN254
*and* Grumpkin — to verify. Soroban has real native BN254 support (Protocol 25's
`env.crypto().bn254()`, the same host functions `stellar-zkstream`'s real Groth16 verifier
uses), but no Grumpkin support at all, and there's no way around that from within a
contract: implementing Grumpkin arithmetic from scratch in-contract is exactly the kind of
thing Stellar's own core team has said isn't worth pursuing without a protocol-level
change (see the open `noir-lang/discussions#8560` proposal to build this properly — an
active, unfinished, ~28-week effort at time of writing, not something a single contract
can route around).

Groth16 over BN254 has no such gap — it needs pairing checks Soroban already supports
natively, and `stellar-zkstream` already proved the pattern works end to end on this exact
host. So these three circuits were rebuilt in Circom rather than staying in Noir, to get
real on-chain verification today instead of a correct-but-unverifiable proof format.

## What each circuit proves

- **`age_proof`**: knowledge of a birth date + salt whose Poseidon commitment matches a
  public `credential_commitment`, and that the birth year is at least 18 years before a
  public `current_year` — without revealing the birth date itself.
- **`kyc_tier_proof`**: knowledge of a KYC tier + salt whose Poseidon commitment matches a
  public `credential_commitment`, and that the tier is at least a public `required_tier` —
  without revealing the actual tier.
- **`membership_proof`**: knowledge of a leaf value and a real depth-20 Merkle path from it
  to a public `merkle_root` — without revealing which leaf, its value, or the sibling path.

## What's here

- `age_proof/age_proof.circom`, `kyc_tier_proof/kyc_tier_proof.circom`,
  `membership_proof/membership_proof.circom` — the circuit source.
- `build/{age_proof,kyc_tier_proof,membership_proof}/` — outputs of an actual Groth16
  trusted-setup pipeline run against these circuits (see below). Committed: `*_vk.json`/
  `*_vk.hex` (the real verification keys — these are what each deployed `zk_verifier`
  instance is actually initialized with), `*_final.zkey` (the real proving key, needed to
  generate a *new* proof without repeating the ceremony), `input.json`/`proof.json`/
  `public.json` (one real worked example per circuit). Not committed (regenerable, see
  `.gitignore`): the `.r1cs`/`.sym`/witness/`.wasm` compiler output and the Powers of Tau
  `.ptau` files.
- `gen_inputs.mjs` — computes real Poseidon commitments/Merkle roots off-circuit (via
  `circomlibjs`) so each example witness actually satisfies its circuit's constraints,
  rather than using invented numbers that would fail witness generation.
- `convert_to_soroban.mjs` — converts a snarkjs VK/proof/public-signals triple into the
  exact byte layout `contracts/zk_verifier/src/groth16.rs` expects (same conversion
  `stellar-zkstream` uses, since this reuses that same contract unmodified).

## Reproducing the pipeline

```bash
npm install
circom age_proof/age_proof.circom -l . --r1cs --wasm --sym -o build/age_proof
circom kyc_tier_proof/kyc_tier_proof.circom -l . --r1cs --wasm --sym -o build/kyc_tier_proof
circom membership_proof/membership_proof.circom -l . --r1cs --wasm --sym -o build/membership_proof

# Phase 1 (shared, universal — not circuit-specific). Power 14 (16384 constraints) because
# membership_proof's depth-20 Merkle path alone needs ~5,100 constraints — comfortably over
# stellar-zkstream's power-12 setup, which was sized for much smaller circuits.
cd build
npx snarkjs powersoftau new bn128 14 pot14_0000.ptau -v
npx snarkjs powersoftau contribute pot14_0000.ptau pot14_0001.ptau -v -e="$(openssl rand -hex 32)"
npx snarkjs powersoftau prepare phase2 pot14_0001.ptau pot14_final.ptau -v

# Phase 2, per circuit
npx snarkjs groth16 setup age_proof/age_proof.r1cs pot14_final.ptau age_proof/age_proof_0000.zkey
npx snarkjs zkey contribute age_proof/age_proof_0000.zkey age_proof/age_proof_final.zkey -v -e="$(openssl rand -hex 32)"
npx snarkjs zkey export verificationkey age_proof/age_proof_final.zkey age_proof/age_proof_vk.json
# repeat groth16 setup / zkey contribute / export verificationkey for kyc_tier_proof/ and membership_proof/

cd ..
node gen_inputs.mjs
node build/age_proof/age_proof_js/generate_witness.js build/age_proof/age_proof_js/age_proof.wasm build/age_proof/input.json build/age_proof/witness.wtns
node build/kyc_tier_proof/kyc_tier_proof_js/generate_witness.js build/kyc_tier_proof/kyc_tier_proof_js/kyc_tier_proof.wasm build/kyc_tier_proof/input.json build/kyc_tier_proof/witness.wtns
node build/membership_proof/membership_proof_js/generate_witness.js build/membership_proof/membership_proof_js/membership_proof.wasm build/membership_proof/input.json build/membership_proof/witness.wtns
npx snarkjs groth16 prove build/age_proof/age_proof_final.zkey build/age_proof/witness.wtns build/age_proof/proof.json build/age_proof/public.json
npx snarkjs groth16 prove build/kyc_tier_proof/kyc_tier_proof_final.zkey build/kyc_tier_proof/witness.wtns build/kyc_tier_proof/proof.json build/kyc_tier_proof/public.json
npx snarkjs groth16 prove build/membership_proof/membership_proof_final.zkey build/membership_proof/witness.wtns build/membership_proof/proof.json build/membership_proof/public.json
npx snarkjs groth16 verify build/age_proof/age_proof_vk.json build/age_proof/public.json build/age_proof/proof.json
npx snarkjs groth16 verify build/kyc_tier_proof/kyc_tier_proof_vk.json build/kyc_tier_proof/public.json build/kyc_tier_proof/proof.json
npx snarkjs groth16 verify build/membership_proof/membership_proof_vk.json build/membership_proof/public.json build/membership_proof/proof.json

node convert_to_soroban.mjs
```

`contracts/zk_verifier/src/test.rs`'s `real_zkident_circuits` module hardcodes the
resulting VK/proof/public-input bytes and calls the actual contract's `vrfy_prf()` with
them — a passing test there is evidence this whole pipeline, including the byte
conversion, is correct end to end, not just internally self-consistent.

## Important caveat

This is a genuine, correctly-executed Groth16 setup — the math is real and the resulting
VKs are real. It is **not** a production-grade multi-party ceremony: phase 1 and each
circuit's phase 2 had a single contributor (this pipeline run), not an independent
multi-party ceremony where no single party could have retained the toxic waste. Treat
these as real testnet verification keys, not something to point real funds or real
identity claims at without a proper multi-party ceremony first.
