// Merkle tree verification
// Tree depth of 20 supports ~1M accounts

use crate::poseidon_utils::hash2;

// Tree depth constant
pub global TREE_DEPTH: u32 = 20;

// Verify a Merkle proof
// - leaf: the leaf value being proven
// - index: leaf position in tree (used to determine left/right at each level)
// - path: sibling hashes from leaf to root
// - root: expected Merkle root
pub fn verify_merkle_proof(
    leaf: Field,
    index: Field,
    path: [Field; TREE_DEPTH],
    root: Field,
) -> bool {
    let computed_root = compute_root_from_path(leaf, index, path);
    computed_root == root
}

// Compute root from leaf and path
pub fn compute_root_from_path(leaf: Field, index: Field, path: [Field; TREE_DEPTH]) -> Field {
    let mut current = leaf;
    let index_bits = index.to_be_bits::<TREE_DEPTH>();

    for i in 0..TREE_DEPTH {
        let sibling = path[i];
        // Bit at position (TREE_DEPTH - 1 - i) determines left/right
        // We use reverse order because to_be_bits gives big-endian
        let bit = index_bits[TREE_DEPTH - 1 - i];

        if bit == 0 {
            // Current node is on the left
            current = hash2(current, sibling);
        } else {
            // Current node is on the right
            current = hash2(sibling, current);
        }
    }

    current
}

// Assert that a Merkle proof is valid
pub fn assert_merkle_proof(leaf: Field, index: Field, path: [Field; TREE_DEPTH], root: Field) {
    let computed_root = compute_root_from_path(leaf, index, path);
    assert(computed_root == root, "Invalid Merkle proof");
}

// Compute the root after updating a leaf
// Returns the new root
pub fn compute_root_after_update(
    new_leaf: Field,
    index: Field,
    path: [Field; TREE_DEPTH],
) -> Field {
    compute_root_from_path(new_leaf, index, path)
}

#[test]
fn test_single_leaf_tree() {
    // With a single leaf at index 0, all siblings are empty hashes
    let leaf: Field = 12345;
    let empty = hash2(0, 0);

    // Create path of empty hashes
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    // Compute root
    let root = compute_root_from_path(leaf, 0, path);

    // Verify the proof
    assert(verify_merkle_proof(leaf, 0, path, root));
}

#[test]
fn test_different_indices_different_roots() {
    let leaf: Field = 12345;
    let empty = hash2(0, 0);

    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    let root0 = compute_root_from_path(leaf, 0, path);
    let root1 = compute_root_from_path(leaf, 1, path);

    // Different indices should produce different roots
    assert(root0 != root1);
}
