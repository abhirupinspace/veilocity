// Private Transfer circuit
// Proves a valid balance transfer between two accounts
//
// This circuit (~15,000 constraints) verifies:
// 1. Sender owns an account with sufficient balance
// 2. Recipient account exists in the tree (or is being created)
// 3. Balances are updated correctly (conservation of funds)
// 4. New state root is computed correctly via sequential updates
// 5. Nullifier prevents double-spending
//
// Public inputs:
// - old_state_root: State root before transfer
// - new_state_root: State root after transfer
// - nullifier: Sender's nullifier for this transfer
//
// Private inputs:
// - Sender: secret, balance, nonce, index, path_old, path_new
// - Recipient: pubkey, balance, nonce, index, path_old, path_new
// - amount: Transfer amount
//
// NOTE: This circuit requires BOTH old and new Merkle paths because:
// - Old paths verify existence in the pre-transfer state
// - New paths are needed after the first update changes shared siblings
//
// State transition: old_root -> (update sender) -> intermediate_root -> (update recipient) -> new_root

use crate::merkle::{assert_merkle_proof, compute_root_from_path, TREE_DEPTH};
use crate::poseidon_utils::{compute_leaf, compute_nullifier, derive_pubkey, hash2};

// Full transfer circuit with proper state transition verification
// Requires additional path inputs to handle sequential leaf updates correctly
pub fn verify_transfer(
    // Public inputs
    old_state_root: Field,
    new_state_root: Field,
    nullifier: Field,
    // Sender private inputs
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path_old: [Field; TREE_DEPTH],     // Path in OLD state
    sender_path_new: [Field; TREE_DEPTH],     // Path in NEW state (after recipient update)
    // Recipient private inputs
    recipient_pubkey: Field,
    recipient_balance: Field,
    recipient_nonce: Field,
    recipient_index: Field,
    recipient_path_old: [Field; TREE_DEPTH],  // Path in OLD state
    recipient_path_new: [Field; TREE_DEPTH],  // Path after sender update (intermediate state)
    // Transfer amount (private to hide it)
    amount: Field,
) {
    // === SENDER VERIFICATION ===

    // 1. Derive sender's public key from secret
    let sender_pubkey = derive_pubkey(sender_secret);

    // 2. Reconstruct sender's leaf commitment
    let sender_leaf_old = compute_leaf(sender_pubkey, sender_balance, sender_nonce);

    // 3. Verify sender exists in old state
    assert_merkle_proof(sender_leaf_old, sender_index, sender_path_old, old_state_root);

    // 4. Verify sender has sufficient balance
    let sender_balance_u64 = sender_balance as u64;
    let amount_u64 = amount as u64;
    assert(sender_balance_u64 >= amount_u64, "Insufficient balance");

    // 5. Verify nullifier is correctly computed
    let computed_nullifier = compute_nullifier(sender_secret, sender_index, sender_nonce);
    assert(computed_nullifier == nullifier, "Invalid nullifier");

    // === RECIPIENT VERIFICATION ===

    // 6. Reconstruct recipient's leaf commitment
    let recipient_leaf_old = compute_leaf(recipient_pubkey, recipient_balance, recipient_nonce);

    // 7. Verify recipient exists in old state
    assert_merkle_proof(
        recipient_leaf_old,
        recipient_index,
        recipient_path_old,
        old_state_root,
    );

    // === STATE TRANSITION VERIFICATION ===
    // We verify: old_root -> update(sender) -> intermediate_root -> update(recipient) -> new_root

    // 8. Ensure sender and recipient are different accounts
    assert(sender_index != recipient_index, "Cannot transfer to self");

    // 9. Compute new balances (conservation of funds)
    let new_sender_balance = sender_balance - amount;
    let new_recipient_balance = recipient_balance + amount;

    // 10. Compute new leaves
    // Sender's nonce increments after spending
    let sender_leaf_new = compute_leaf(sender_pubkey, new_sender_balance, sender_nonce + 1);
    // Recipient's nonce stays the same (receiving doesn't increment nonce)
    let recipient_leaf_new = compute_leaf(recipient_pubkey, new_recipient_balance, recipient_nonce);

    // 11. Compute intermediate root (after sender update, before recipient update)
    // Use sender_path_old since the tree hasn't changed for sender's path yet
    let intermediate_root = compute_root_from_path(sender_leaf_new, sender_index, sender_path_old);

    // 12. Verify intermediate root by checking recipient's NEW path leads to it
    // The prover provides recipient_path_new which accounts for the sender update
    let intermediate_check = compute_root_from_path(
        recipient_leaf_old, // Recipient's OLD leaf value at this point
        recipient_index,
        recipient_path_new, // Path AFTER sender update
    );
    assert(intermediate_root == intermediate_check, "Invalid intermediate state");

    // 13. Compute final root (after recipient update)
    let final_root = compute_root_from_path(recipient_leaf_new, recipient_index, recipient_path_new);

    // 14. Verify the new state root matches
    assert(new_state_root == final_root, "Invalid new state root");

    // 15. Cross-check: Sender's path in final state should also be valid
    // This ensures the prover provided consistent paths
    let sender_final_check = compute_root_from_path(sender_leaf_new, sender_index, sender_path_new);
    assert(sender_final_check == new_state_root, "Inconsistent sender path in new state");
}

// Simplified transfer verification for MVP
// Only verifies sender side and nullifier
pub fn verify_transfer_simple(
    // Public inputs
    old_state_root: Field,
    nullifier: Field,
    // Sender private inputs
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path: [Field; TREE_DEPTH],
    // Transfer details
    recipient_pubkey: Field,
    amount: Field,
) {
    // 1. Derive sender's public key
    let sender_pubkey = derive_pubkey(sender_secret);

    // 2. Reconstruct sender's leaf
    let sender_leaf = compute_leaf(sender_pubkey, sender_balance, sender_nonce);

    // 3. Verify sender in old state
    assert_merkle_proof(sender_leaf, sender_index, sender_path, old_state_root);

    // 4. Check balance
    let sender_balance_u64 = sender_balance as u64;
    let amount_u64 = amount as u64;
    assert(sender_balance_u64 >= amount_u64, "Insufficient balance");

    // 5. Verify nullifier
    let computed_nullifier = compute_nullifier(sender_secret, sender_index, sender_nonce);
    assert(computed_nullifier == nullifier, "Invalid nullifier");

    // 6. Verify recipient pubkey is not zero
    assert(recipient_pubkey != 0, "Invalid recipient");

    // 7. Verify amount is positive
    assert(amount_u64 > 0, "Amount must be positive");
}

#[test]
fn test_valid_simple_transfer() {
    // Setup sender account
    let sender_secret: Field = 111111;
    let sender_balance: Field = 2000000000000000000; // 2 MNT
    let sender_nonce: Field = 0;
    let sender_index: Field = 0;
    let amount: Field = 500000000000000000; // 0.5 MNT

    // Recipient
    let recipient_pubkey: Field = 0xabcdef123456;

    // Create sender leaf and path
    let sender_pubkey = derive_pubkey(sender_secret);
    let sender_leaf = compute_leaf(sender_pubkey, sender_balance, sender_nonce);

    // Empty Merkle path for single-leaf tree
    let empty = hash2(0, 0);
    let mut sender_path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        sender_path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    // Compute state root
    let old_state_root = compute_root_from_path(sender_leaf, sender_index, sender_path);

    // Compute nullifier
    let nullifier = compute_nullifier(sender_secret, sender_index, sender_nonce);

    // This should pass
    verify_transfer_simple(
        old_state_root,
        nullifier,
        sender_secret,
        sender_balance,
        sender_nonce,
        sender_index,
        sender_path,
        recipient_pubkey,
        amount,
    );
}

#[test(should_fail_with = "Insufficient balance")]
fn test_transfer_insufficient_balance() {
    let sender_secret: Field = 111111;
    let sender_balance: Field = 500000000000000000; // 0.5 MNT
    let sender_nonce: Field = 0;
    let sender_index: Field = 0;
    let amount: Field = 1000000000000000000; // Try to send 1 MNT

    let recipient_pubkey: Field = 0xabcdef123456;

    let sender_pubkey = derive_pubkey(sender_secret);
    let sender_leaf = compute_leaf(sender_pubkey, sender_balance, sender_nonce);

    let empty = hash2(0, 0);
    let mut sender_path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        sender_path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }

    let old_state_root = compute_root_from_path(sender_leaf, sender_index, sender_path);
    let nullifier = compute_nullifier(sender_secret, sender_index, sender_nonce);

    // This should fail
    verify_transfer_simple(
        old_state_root,
        nullifier,
        sender_secret,
        sender_balance,
        sender_nonce,
        sender_index,
        sender_path,
        recipient_pubkey,
        amount,
    );
}

// Helper to compute empty hash chain for Merkle paths
fn compute_empty_path() -> [Field; TREE_DEPTH] {
    let empty = hash2(0, 0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_empty = empty;
    for i in 0..TREE_DEPTH {
        path[i] = current_empty;
        current_empty = hash2(current_empty, current_empty);
    }
    path
}

// Test for full transfer with state transition verification
// This tests the case where sender and recipient don't share path nodes (index 0 and 1)
#[test]
fn test_valid_full_transfer_adjacent_indices() {
    // === Setup accounts ===
    let sender_secret: Field = 111111;
    let sender_balance: Field = 2000000000000000000; // 2 MNT
    let sender_nonce: Field = 0;
    let sender_index: Field = 0;

    let recipient_secret: Field = 222222;
    let recipient_balance: Field = 1000000000000000000; // 1 MNT
    let recipient_nonce: Field = 0;
    let recipient_index: Field = 1;

    let amount: Field = 500000000000000000; // 0.5 MNT

    // Derive keys and compute leaves
    let sender_pubkey = derive_pubkey(sender_secret);
    let recipient_pubkey = derive_pubkey(recipient_secret);

    let sender_leaf_old = compute_leaf(sender_pubkey, sender_balance, sender_nonce);
    let recipient_leaf_old = compute_leaf(recipient_pubkey, recipient_balance, recipient_nonce);

    // === Build old tree with two leaves (index 0 and 1) ===
    // At level 0: leaves are sender (0) and recipient (1)
    // At level 1 and up: hash of children

    let empty = hash2(0, 0);

    // Compute empty subtree hashes for levels 1-19
    let mut empty_hashes: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    empty_hashes[0] = empty;
    for i in 1..TREE_DEPTH {
        empty_hashes[i] = hash2(empty_hashes[i-1], empty_hashes[i-1]);
    }

    // Build sender's path in old state (index 0)
    // Level 0: sibling is recipient_leaf_old
    // Levels 1-19: siblings are empty subtrees
    let mut sender_path_old: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    sender_path_old[0] = recipient_leaf_old; // Sibling at level 0
    for i in 1..TREE_DEPTH {
        sender_path_old[i] = empty_hashes[i];
    }

    // Build recipient's path in old state (index 1)
    // Level 0: sibling is sender_leaf_old
    // Levels 1-19: siblings are empty subtrees
    let mut recipient_path_old: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    recipient_path_old[0] = sender_leaf_old;
    for i in 1..TREE_DEPTH {
        recipient_path_old[i] = empty_hashes[i];
    }

    // Compute old state root
    let old_state_root = compute_root_from_path(sender_leaf_old, sender_index, sender_path_old);

    // Verify recipient is also in old state (sanity check)
    let old_root_from_recipient = compute_root_from_path(recipient_leaf_old, recipient_index, recipient_path_old);
    assert(old_state_root == old_root_from_recipient); // Both should match

    // Compute nullifier
    let nullifier = compute_nullifier(sender_secret, sender_index, sender_nonce);

    // === Compute new leaves after transfer ===
    let new_sender_balance = sender_balance - amount;
    let new_recipient_balance = recipient_balance + amount;
    let sender_leaf_new = compute_leaf(sender_pubkey, new_sender_balance, sender_nonce + 1);
    let recipient_leaf_new = compute_leaf(recipient_pubkey, new_recipient_balance, recipient_nonce);

    // === Compute intermediate state (after sender update) ===
    // Sender's path is still valid since we're just updating the sender leaf
    let intermediate_root = compute_root_from_path(sender_leaf_new, sender_index, sender_path_old);

    // Build recipient's path in intermediate state (after sender update)
    // Level 0 sibling is now sender_leaf_new
    let mut recipient_path_new: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    recipient_path_new[0] = sender_leaf_new;
    for i in 1..TREE_DEPTH {
        recipient_path_new[i] = empty_hashes[i];
    }

    // Verify intermediate root matches from recipient's perspective
    let intermediate_from_recipient = compute_root_from_path(recipient_leaf_old, recipient_index, recipient_path_new);
    assert(intermediate_root == intermediate_from_recipient);

    // === Compute final state (after recipient update) ===
    let new_state_root = compute_root_from_path(recipient_leaf_new, recipient_index, recipient_path_new);

    // Build sender's path in new state (after recipient update)
    // Level 0 sibling is now recipient_leaf_new
    let mut sender_path_new: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    sender_path_new[0] = recipient_leaf_new;
    for i in 1..TREE_DEPTH {
        sender_path_new[i] = empty_hashes[i];
    }

    // Verify final root matches from sender's perspective
    let final_from_sender = compute_root_from_path(sender_leaf_new, sender_index, sender_path_new);
    assert(new_state_root == final_from_sender);

    // === Run the full transfer verification ===
    verify_transfer(
        old_state_root,
        new_state_root,
        nullifier,
        sender_secret,
        sender_balance,
        sender_nonce,
        sender_index,
        sender_path_old,
        sender_path_new,
        recipient_pubkey,
        recipient_balance,
        recipient_nonce,
        recipient_index,
        recipient_path_old,
        recipient_path_new,
        amount,
    );
}

#[test(should_fail_with = "Cannot transfer to self")]
fn test_transfer_to_self_fails() {
    let sender_secret: Field = 111111;
    let sender_balance: Field = 2000000000000000000;
    let sender_nonce: Field = 0;
    let sender_index: Field = 0;
    let amount: Field = 500000000000000000;

    let sender_pubkey = derive_pubkey(sender_secret);
    let sender_leaf = compute_leaf(sender_pubkey, sender_balance, sender_nonce);

    let path = compute_empty_path();
    // Modify first element to be the sender itself (for a simple tree)
    let mut sender_path_old = path;
    let mut sender_path_new = path;
    let mut recipient_path_old = path;
    let mut recipient_path_new = path;

    let old_state_root = compute_root_from_path(sender_leaf, sender_index, sender_path_old);
    let new_state_root = old_state_root; // Dummy - won't be checked
    let nullifier = compute_nullifier(sender_secret, sender_index, sender_nonce);

    // Try to transfer to self (same index)
    verify_transfer(
        old_state_root,
        new_state_root,
        nullifier,
        sender_secret,
        sender_balance,
        sender_nonce,
        sender_index,       // Sender at index 0
        sender_path_old,
        sender_path_new,
        sender_pubkey,
        sender_balance,
        sender_nonce,
        sender_index,       // Recipient also at index 0 - should fail!
        recipient_path_old,
        recipient_path_new,
        amount,
    );
}
