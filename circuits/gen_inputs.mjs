import { buildPoseidon } from "circomlibjs";
import { writeFileSync } from "fs";

const poseidon = await buildPoseidon();
const F = poseidon.F;
const toField = (v) => F.toObject(v).toString();

// age_proof: prove birth_year <= current_year - 18 without revealing the birth date.
// A real demo subject born 2000-06-15, checked against current_year=2026 (age 26, >= 18).
{
  const birth_year = 2000n, birth_month = 6n, birth_day = 15n, salt = 111222333n;
  const current_year = 2026n;
  const credential_commitment = toField(poseidon([birth_year, birth_month, birth_day, salt]));
  const input = {
    birth_year: birth_year.toString(),
    birth_month: birth_month.toString(),
    birth_day: birth_day.toString(),
    salt: salt.toString(),
    current_year: current_year.toString(),
    credential_commitment,
  };
  writeFileSync("build/age_proof/input.json", JSON.stringify(input, null, 2));
  console.log("age_proof input:", input);
}

// kyc_tier_proof: prove kyc_tier >= required_tier without revealing the actual tier.
// A real demo subject at tier 3, checked against a required tier of 2.
{
  const kyc_tier = 3n, salt = 444555666n;
  const required_tier = 2n;
  const credential_commitment = toField(poseidon([kyc_tier, salt]));
  const input = {
    kyc_tier: kyc_tier.toString(),
    salt: salt.toString(),
    required_tier: required_tier.toString(),
    credential_commitment,
  };
  writeFileSync("build/kyc_tier_proof/input.json", JSON.stringify(input, null, 2));
  console.log("kyc_tier_proof input:", input);
}

// membership_proof: prove membership of leaf_data in a real depth-20 Merkle tree without
// revealing which leaf or the sibling path. Builds a real tree with one real leaf at index
// 0 and 19 zero-siblings above it (a real, valid, if sparse, tree — not an invented root).
{
  const DEPTH = 20;
  const leaf_data = 777888999n;
  const leaf_commitment = toField(poseidon([leaf_data]));

  const path_indices = new Array(DEPTH).fill(0); // leaf at index 0: always the "left" child
  const merkle_path = [];
  let current = BigInt(leaf_commitment);
  for (let i = 0; i < DEPTH; i++) {
    const sibling = 0n; // an empty/zero sibling subtree at every level above the one real leaf
    merkle_path.push(sibling.toString());
    current = BigInt(toField(poseidon([current, sibling])));
  }
  const merkle_root = current.toString();

  const input = {
    leaf_data: leaf_data.toString(),
    merkle_path,
    path_indices: path_indices.map(String),
    merkle_root,
    leaf_commitment,
  };
  writeFileSync("build/membership_proof/input.json", JSON.stringify(input, null, 2));
  console.log("membership_proof input:", input);
}
