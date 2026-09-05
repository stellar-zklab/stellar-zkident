pragma circom 2.1.6;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";

/*
 * AgeProof
 *
 * Proves: the prover knows (birth_year, birth_month, birth_day, salt) such that
 *   1. Poseidon(birth_year, birth_month, birth_day, salt) == credential_commitment
 *   2. birth_year <= current_year - 18 (i.e. the prover is at least 18)
 * without revealing the actual birth date.
 *
 * Private: birth_year, birth_month, birth_day, salt
 * Public:  current_year, credential_commitment
 *
 * Rebuilt from this repo's original Noir circuit (same commitment/comparison logic) —
 * Noir's UltraHonk proving system needs a Grumpkin+BN254 curve cycle Soroban has no native
 * support for (see docs/DEPLOYMENT_GUIDE.md), so this uses Groth16/BN254 instead, matching
 * stellar-zkstream's already-proven on-chain verifier pattern.
 */
template AgeProof(n) {
    signal input birth_year;
    signal input birth_month;
    signal input birth_day;
    signal input salt;
    signal input current_year;
    signal input credential_commitment;
    signal output valid;

    component h = Poseidon(4);
    h.inputs[0] <== birth_year;
    h.inputs[1] <== birth_month;
    h.inputs[2] <== birth_day;
    h.inputs[3] <== salt;
    credential_commitment === h.out;

    // birth_year <= current_year - 18
    component lte = LessEqThan(n);
    lte.in[0] <== birth_year;
    lte.in[1] <== current_year - 18;
    lte.out === 1;

    valid <== 1;
}

component main {public [current_year, credential_commitment]} = AgeProof(32);
