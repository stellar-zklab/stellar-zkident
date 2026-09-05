global DEPTH: u32 = 20;

fn hash_pair(left: Field, right: Field) -> Field {
    std::hash::poseidon::bn254::hash_2([left, right])
}

fn main(
    leaf_data: Field,
    merkle_path: [Field; 20],
    path_indices: [u1; 20],
    merkle_root: pub Field,
    leaf_commitment: pub Field,
) {
    let computed_leaf = std::hash::poseidon::bn254::hash_1([leaf_data]);
    assert(computed_leaf == leaf_commitment);

    let mut current = leaf_commitment;
    for i in 0..DEPTH {
        let sibling = merkle_path[i];
        current = if path_indices[i] == 0 {
            hash_pair(current, sibling)
        } else {
            hash_pair(sibling, current)
        };
    }
    assert(current == merkle_root);
}
