// Poseidon hash wrappers for consistent interface
// These functions must produce identical outputs to the Rust implementation

use poseidon::poseidon::bn254::{hash_1, hash_2, hash_3};

// Hash 2 field elements (for Merkle tree nodes)
pub fn hash2(left: Field, right: Field) -> Field {
    hash_2([left, right])
}

// Hash 3 field elements (for account leaves)
pub fn hash3(a: Field, b: Field, c: Field) -> Field {
    hash_3([a, b, c])
}

// Hash 1 field element
pub fn hash1(a: Field) -> Field {
    hash_1([a])
}

// Compute public key from secret
pub fn derive_pubkey(secret: Field) -> Field {
    hash1(secret)
}

// Compute nullifier for spending
// nullifier = hash(secret, leaf_index, nonce)
pub fn compute_nullifier(secret: Field, leaf_index: Field, nonce: Field) -> Field {
    hash3(secret, leaf_index, nonce)
}

// Compute account leaf commitment
// leaf = hash(pubkey, balance, nonce)
pub fn compute_leaf(pubkey: Field, balance: Field, nonce: Field) -> Field {
    hash3(pubkey, balance, nonce)
}

// Compute deposit commitment
// commitment = hash(secret, amount)
pub fn compute_deposit_commitment(secret: Field, amount: Field) -> Field {
    hash2(secret, amount)
}

// Empty leaf value (for empty tree nodes)
pub fn empty_leaf() -> Field {
    hash2(0, 0)
}

#[test]
fn test_hash2_deterministic() {
    let h1 = hash2(1, 2);
    let h2 = hash2(1, 2);
    assert(h1 == h2);
}

#[test]
fn test_hash2_different_inputs() {
    let h1 = hash2(1, 2);
    let h2 = hash2(1, 3);
    assert(h1 != h2);
}

#[test]
fn test_derive_pubkey() {
    let secret: Field = 12345;
    let pubkey = derive_pubkey(secret);
    assert(pubkey != secret);
    assert(pubkey != 0);
}

#[test]
fn test_compute_leaf() {
    let pubkey: Field = 100;
    let balance: Field = 1000000000000000000; // 1 MNT in wei
    let nonce: Field = 0;

    let leaf = compute_leaf(pubkey, balance, nonce);
    assert(leaf != 0);
}

#[test]
fn test_nullifier_uniqueness() {
    let secret: Field = 12345;

    let null1 = compute_nullifier(secret, 0, 0);
    let null2 = compute_nullifier(secret, 0, 1);
    let null3 = compute_nullifier(secret, 1, 0);

    assert(null1 != null2);
    assert(null1 != null3);
    assert(null2 != null3);
}

#[test]
fn test_deposit_commitment() {
    let secret: Field = 54321;
    let amount: Field = 1000000000000000000;

    let commitment = compute_deposit_commitment(secret, amount);
    assert(commitment != 0);
    assert(commitment != secret);
    assert(commitment != amount);
}
