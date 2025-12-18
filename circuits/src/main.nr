// Veilocity ZK Circuits - Main Entry Points
// These are the circuit functions that generate proofs

use crate::merkle::TREE_DEPTH;
use crate::deposit::verify_deposit;
use crate::withdraw::verify_withdrawal;
use crate::transfer::{verify_transfer, verify_transfer_simple};

// ============================================================================
// DEPOSIT CIRCUIT
// ============================================================================
// Proves a deposit commitment is correctly formed
// Public inputs: commitment, amount
// Private inputs: secret

fn main_deposit(
    commitment: pub Field,
    amount: pub Field,
    secret: Field
) {
    verify_deposit(commitment, amount, secret);
}

// ============================================================================
// WITHDRAWAL CIRCUIT
// ============================================================================
// Proves ownership and sufficient balance for withdrawal
// Public inputs: state_root, nullifier, amount, recipient
// Private inputs: secret, balance, nonce, index, path

fn main_withdraw(
    state_root: pub Field,
    nullifier: pub Field,
    amount: pub Field,
    recipient: pub Field,
    secret: Field,
    balance: Field,
    nonce: Field,
    index: Field,
    path: [Field; TREE_DEPTH]
) {
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

// ============================================================================
// TRANSFER CIRCUIT (FULL)
// ============================================================================
// Proves valid transfer between two accounts with state transition
// Public inputs: old_state_root, new_state_root, nullifier
// Private inputs: sender details, recipient details, amount

fn main_transfer(
    old_state_root: pub Field,
    new_state_root: pub Field,
    nullifier: pub Field,
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path: [Field; TREE_DEPTH],
    recipient_pubkey: Field,
    recipient_balance: Field,
    recipient_nonce: Field,
    recipient_index: Field,
    recipient_path: [Field; TREE_DEPTH],
    amount: Field
) {
    verify_transfer(
        old_state_root,
        new_state_root,
        nullifier,
        sender_secret,
        sender_balance,
        sender_nonce,
        sender_index,
        sender_path,
        recipient_pubkey,
        recipient_balance,
        recipient_nonce,
        recipient_index,
        recipient_path,
        amount
    );
}

// ============================================================================
// TRANSFER CIRCUIT (SIMPLE - MVP)
// ============================================================================
// Simplified transfer that only verifies sender authorization
// Public inputs: old_state_root, nullifier
// Private inputs: sender details, recipient_pubkey, amount

fn main_transfer_simple(
    old_state_root: pub Field,
    nullifier: pub Field,
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path: [Field; TREE_DEPTH],
    recipient_pubkey: Field,
    amount: Field
) {
    verify_transfer_simple(
        old_state_root,
        nullifier,
        sender_secret,
        sender_balance,
        sender_nonce,
        sender_index,
        sender_path,
        recipient_pubkey,
        amount
    );
}
