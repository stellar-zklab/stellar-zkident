fn main(
    kyc_tier: Field,
    salt: Field,
    required_tier: pub Field,
    credential_commitment: pub Field,
) {
    let computed = std::hash::poseidon::bn254::hash_2([kyc_tier, salt]);
    assert(computed == credential_commitment);
    assert(kyc_tier as u64 >= required_tier as u64);
}
