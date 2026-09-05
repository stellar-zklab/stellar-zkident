fn main(
    birth_year: Field,
    birth_month: Field,
    birth_day: Field,
    salt: Field,
    current_year: pub Field,
    credential_commitment: pub Field,
) {
    let computed = std::hash::poseidon::bn254::hash_4([birth_year, birth_month, birth_day, salt]);
    assert(computed == credential_commitment);
    let min_birth_year = current_year - 18;
    assert(birth_year as u64 <= min_birth_year as u64);
}
