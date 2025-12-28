// Veilocity Withdrawal Circuit
// Proves ownership of funds and generates a nullifier for withdrawal
//
// This circuit verifies:
// 1. User owns an account in the Merkle tree (via secret -> pubkey)
// 2. Account has sufficient balance for withdrawal
// 3. Nullifier is correctly derived (prevents double-spend)
// 4. Recipient is bound to the proof (prevents front-running)
//
// Public inputs (exposed to Solidity verifier):
// - state_root: Current state root of the Merkle tree
// - nullifier: Unique identifier preventing double-withdrawal
// - amount: Withdrawal amount
// - recipient: On-chain address receiving funds (as Field)
//
// Private inputs (hidden from verifier):
// - secret: Account secret key
// - balance: Current account balan`ce
// - nonce: Account nonce
// - index: Leaf index in tree
// - path: Merkle proof path

use poseidon::poseidon::bn254::{hash_1, hash_2, hash_3};

// Tree depth constant (supports ~1M accounts)
global TREE_DEPTH: u32 = 20;

// Hash 2 field elements (for Merkle tree nodes)
fn hash2(left: Field, right: Field) -> Field {
    hash_2([left, right])
}

// Hash 3 field elements (for account leaves)
fn hash3(a: Field, b: Field, c: Field) -> Field {
    hash_3([a, b, c])
}

// Hash 1 field element
fn hash1(a: Field) -> Field {
    hash_1([a])
}

// Compute public key from secret
fn derive_pubkey(secret: Field) -> Field {
    hash1(secret)
}

// Compute nullifier for spending
// nullifier = hash(secret, leaf_index, nonce)
fn compute_nullifier(secret: Field, leaf_index: Field, nonce: Field) -> Field {
    hash3(secret, leaf_index, nonce)
}

// Compute account leaf commitment
// leaf = hash(pubkey, balance, nonce)
fn compute_leaf(pubkey: Field, balance: Field, nonce: Field) -> Field {
    hash3(pubkey, balance, nonce)
}

// Compute root from leaf and path
fn compute_root_from_path(leaf: Field, index: Field, path: [Field; TREE_DEPTH]) -> Field {
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
fn assert_merkle_proof(leaf: Field, index: Field, path: [Field; TREE_DEPTH], root: Field) {
    let computed_root = compute_root_from_path(leaf, index, path);
    assert(computed_root == root, "Invalid Merkle proof");
}

// ============================================================================
// MAIN WITHDRAWAL CIRCUIT
// ============================================================================
fn main(
    // Public inputs - will be verified on-chain
    state_root: pub Field,
    nullifier: pub Field,
    amount: pub Field,
    recipient: pub Field,
    // Private inputs - hidden from verifier
    secret: Field,
    balance: Field,
    nonce: Field,
    index: Field,
    path: [Field; TREE_DEPTH],
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
    assert(recipient != 0, "Invalid recipient");
}

#[test]
fn test_valid_withdrawal() {
    // Setup: create an account
    let secret: Field = 123456789;
    let balance: Field = 2000000000000000000; // 2 MNT
    let nonce: Field = 0;
    let index: Field = 0;
    let amount: Field = 1000000000000000000; // Withdraw 1 MNT
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
    let state_root = compute_root_from_path(leaf, index, path);

    // Compute nullifier
    let nullifier = compute_nullifier(secret, index, nonce);

    // This should pass
    main(
        state_root,
        nullifier,
        amount,
        recipient,
        secret,
        balance,
        nonce,
        index,
        path,
    );
}

#[test(should_fail_with = "Insufficient balance")]
fn test_insufficient_balance_fails() {
    let secret: Field = 123456789;
    let balance: Field = 500000000000000000; // 0.5 MNT
    let nonce: Field = 0;
    let index: Field = 0;
    let amount: Field = 1000000000000000000; // Try to withdraw 1 MNT
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

    let state_root = compute_root_from_path(leaf, index, path);
    let nullifier = compute_nullifier(secret, index, nonce);

    // This should fail due to insufficient balance
    main(
        state_root,
        nullifier,
        amount,
        recipient,
        secret,
        balance,
        nonce,
        index,
        path,
    );
}
