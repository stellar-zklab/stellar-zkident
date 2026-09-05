pragma circom 2.1.6;

include "node_modules/circomlib/circuits/poseidon.circom";

/*
 * MembershipProof
 *
 * Proves: the prover knows (leaf_data, merkle_path, path_indices) such that
 *   1. Poseidon(leaf_data) == leaf_commitment
 *   2. Walking merkle_path from leaf_commitment, picking (current, sibling) order per
 *      path_indices at each level, reaches merkle_root
 * without revealing which leaf, its data, or the sibling path.
 *
 * Private: leaf_data, merkle_path[DEPTH], path_indices[DEPTH]
 * Public:  merkle_root, leaf_commitment
 *
 * Rebuilt from this repo's original Noir circuit (same depth-20 tree, same Poseidon
 * pair-hashing) — see age_proof.circom's header comment for why Circom/Groth16 instead of
 * Noir/UltraHonk.
 *
 * path_indices[i] must be constrained boolean explicitly — Circom signals aren't
 * range-limited by declaration the way Noir's u1 type is, so an unconstrained
 * "index" could otherwise be any field element, not actually a binary left/right choice.
 */
template MembershipProof(depth) {
    signal input leaf_data;
    signal input merkle_path[depth];
    signal input path_indices[depth];
    signal input merkle_root;
    signal input leaf_commitment;
    signal output valid;

    component leafHasher = Poseidon(1);
    leafHasher.inputs[0] <== leaf_data;
    leaf_commitment === leafHasher.out;

    signal levels[depth + 1];
    levels[0] <== leaf_commitment;

    signal left[depth];
    signal right[depth];
    component hashers[depth];

    for (var i = 0; i < depth; i++) {
        path_indices[i] * (1 - path_indices[i]) === 0;

        // path_indices[i] == 0: hash(current, sibling); == 1: hash(sibling, current).
        left[i] <== levels[i] + path_indices[i] * (merkle_path[i] - levels[i]);
        right[i] <== merkle_path[i] + path_indices[i] * (levels[i] - merkle_path[i]);

        hashers[i] = Poseidon(2);
        hashers[i].inputs[0] <== left[i];
        hashers[i].inputs[1] <== right[i];
        levels[i + 1] <== hashers[i].out;
    }

    merkle_root === levels[depth];
    valid <== 1;
}

component main {public [merkle_root, leaf_commitment]} = MembershipProof(20);
