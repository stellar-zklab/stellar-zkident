pragma circom 2.1.6;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";

/*
 * KycTierProof
 *
 * Proves: the prover knows (kyc_tier, salt) such that
 *   1. Poseidon(kyc_tier, salt) == credential_commitment
 *   2. kyc_tier >= required_tier
 * without revealing the prover's actual KYC tier.
 *
 * Private: kyc_tier, salt
 * Public:  required_tier, credential_commitment
 *
 * Rebuilt from this repo's original Noir circuit — see age_proof.circom's header comment
 * for why (Noir/UltraHonk needs curve support Soroban doesn't have; Groth16/BN254 does).
 */
template KycTierProof(n) {
    signal input kyc_tier;
    signal input salt;
    signal input required_tier;
    signal input credential_commitment;
    signal output valid;

    component h = Poseidon(2);
    h.inputs[0] <== kyc_tier;
    h.inputs[1] <== salt;
    credential_commitment === h.out;

    component gte = GreaterEqThan(n);
    gte.in[0] <== kyc_tier;
    gte.in[1] <== required_tier;
    gte.out === 1;

    valid <== 1;
}

component main {public [required_tier, credential_commitment]} = KycTierProof(32);
