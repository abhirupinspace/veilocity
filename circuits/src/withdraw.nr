// Withdrawal circuit
// Proves ownership of funds and generates a nullifier for withdrawal
//
// This circuit (~8,000 constraints) verifies:
// 1. User owns an account in the Merkle tree (via secret -> pubkey)
// 2. Account has sufficient balance for withdrawal
// 3. Nullifier is correctly derived (prevents double-spend)
// 4. Recipient is bound to the proof (prevents front-running)
//
// Public inputs:
// - state_root: Current state root of the Merkle tree
// - nullifier: Unique identifier preventing double-withdrawal
// - amount: Withdrawal amount
// - recipient: On-chain address receiving funds (as Field)
//
// Private inputs:
// - secret: Account secret key
// - balance: Current account balance
// - nonce: Account nonce
// - index: Leaf index in tree
// - path: Merkle proof path

use crate::poseidon_utils::{derive_pubkey, compute_leaf, compute_nullifier};
use crate::merkle::{assert_merkle_proof, TREE_DEPTH};

// Main withdrawal circuit function
pub fn verify_withdrawal(
    // Public inputs
    state_root: pub Field,
    nullifier: pub Field,
    amount: pub Field,
    recipient: pub Field,
    // Private inputs
    secret: Field,
    balance: Field,
    nonce: Field,
    index: Field,
    path: [Field; TREE_DEPTH]
) {
    // 1. Derive public key from secret (proves ownership)
    let pubkey = derive_pubkey(secret);

    // 2. Reconstruct the leaf commitment
    let leaf = compute_leaf(pubkey, balance, nonce);

    // 3. Verify leaf exists in the Merkle tree
    assert_merkle_proof(leaf, index, path, state_root);

    // 4. Verify sufficient balance
    let balance_u64 = balance as u64;
    let amount_u64 = amount as u64;
    assert(balance_u64 >= amount_u64, "Insufficient balance");

    // 5. Verify nullifier is correctly computed
    let computed_nullifier = compute_nullifier(secret, index, nonce);
    assert(computed_nullifier == nullifier, "Invalid nullifier");

    // 6. Bind recipient to the proof (prevents front-running)
    // The recipient is a public input, so it's bound to this specific proof
    // The verifier contract will use this to send funds to the correct address
    // We just need to ensure it's not zero
    assert(recipient != 0, "Invalid recipient");
}

#[test]
fn test_valid_withdrawal() {
    use crate::poseidon_utils::hash2;

    // Setup: create an account
    let secret: Field = 123456789;
    let balance: Field = 2000000000000000000; // 2 ETH
    let nonce: Field = 0;
    let index: Field = 0;
    let amount: Field = 1000000000000000000; // Withdraw 1 ETH
    let recipient: Field = 0x1234567890abcdef; // Recipient address

    // Compute pubkey and leaf
    let pubkey = derive_pubkey(secret);
    let leaf = compute_leaf(pubkey, balance, nonce);

    // Create empty Merkle path (single leaf tree)
    let empty = hash2(0, 0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    // Compute the state root
    let state_root = crate::merkle::compute_root_from_path(leaf, index, path);

    // Compute nullifier
    let nullifier = compute_nullifier(secret, index, nonce);

    // This should pass
    verify_withdrawal(
        state_root,
        nullifier,
        amount,
        recipient,
        secret,
        balance,
        nonce,
        index,
        path
    );
}

#[test(should_fail_with = "Insufficient balance")]
fn test_insufficient_balance_fails() {
    use crate::poseidon_utils::hash2;

    let secret: Field = 123456789;
    let balance: Field = 500000000000000000; // 0.5 ETH
    let nonce: Field = 0;
    let index: Field = 0;
    let amount: Field = 1000000000000000000; // Try to withdraw 1 ETH
    let recipient: Field = 0x1234567890abcdef;

    let pubkey = derive_pubkey(secret);
    let leaf = compute_leaf(pubkey, balance, nonce);

    let empty = hash2(0, 0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    let state_root = crate::merkle::compute_root_from_path(leaf, index, path);
    let nullifier = compute_nullifier(secret, index, nonce);

    // This should fail due to insufficient balance
    verify_withdrawal(
        state_root,
        nullifier,
        amount,
        recipient,
        secret,
        balance,
        nonce,
        index,
        path
    );
}

#[test(should_fail_with = "Invalid nullifier")]
fn test_wrong_nullifier_fails() {
    use crate::poseidon_utils::hash2;

    let secret: Field = 123456789;
    let balance: Field = 2000000000000000000;
    let nonce: Field = 0;
    let index: Field = 0;
    let amount: Field = 1000000000000000000;
    let recipient: Field = 0x1234567890abcdef;

    let pubkey = derive_pubkey(secret);
    let leaf = compute_leaf(pubkey, balance, nonce);

    let empty = hash2(0, 0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    let state_root = crate::merkle::compute_root_from_path(leaf, index, path);

    // Use wrong nullifier
    let wrong_nullifier: Field = 999999;

    verify_withdrawal(
        state_root,
        wrong_nullifier,
        amount,
        recipient,
        secret,
        balance,
        nonce,
        index,
        path
    );
}
