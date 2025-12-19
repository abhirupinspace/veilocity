// Veilocity ZK Circuits - Main Entry Points
// These are the circuit functions that generate proofs

mod poseidon_utils;
mod merkle;
mod deposit;
mod withdraw;
mod transfer;

use deposit::verify_deposit;
use merkle::TREE_DEPTH;
use poseidon_utils::compute_deposit_commitment;
use transfer::{verify_transfer, verify_transfer_simple};
use withdraw::verify_withdrawal;

// ============================================================================
// DEPOSIT CIRCUIT (Main Entry Point)
// ============================================================================
// Proves a deposit commitment is correctly formed
// Public inputs: amount (commitment is computed and returned)
// Private inputs: secret
// Returns: the computed commitment

fn main(amount: pub Field, secret: Field) -> pub Field {
    // Compute the commitment
    let commitment = compute_deposit_commitment(secret, amount);

    // Verify the deposit is valid
    verify_deposit(commitment, amount, secret);

    // Return the commitment as public output
    commitment
}

// Helper functions for other circuits (can be made into separate binaries)
pub fn main_deposit(commitment: Field, amount: Field, secret: Field) {
    verify_deposit(commitment, amount, secret);
}

// ============================================================================
// WITHDRAWAL CIRCUIT
// ============================================================================
// Proves ownership and sufficient balance for withdrawal
// Public inputs: state_root, nullifier, amount, recipient
// Private inputs: secret, balance, nonce, index, path

pub fn main_withdraw(
    state_root: Field,
    nullifier: Field,
    amount: Field,
    recipient: Field,
    secret: Field,
    balance: Field,
    nonce: Field,
    index: Field,
    path: [Field; TREE_DEPTH],
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
        path,
    );
}

// ============================================================================
// TRANSFER CIRCUIT (FULL)
// ============================================================================
// Proves valid transfer between two accounts with state transition
// Public inputs: old_state_root, new_state_root, nullifier
// Private inputs: sender details (with old/new paths), recipient details (with old/new paths), amount
//
// NOTE: Requires 4 Merkle paths total (2 per account) to properly verify
// the sequential state transition: old -> update sender -> update recipient -> new

pub fn main_transfer(
    old_state_root: Field,
    new_state_root: Field,
    nullifier: Field,
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path_old: [Field; TREE_DEPTH],
    sender_path_new: [Field; TREE_DEPTH],
    recipient_pubkey: Field,
    recipient_balance: Field,
    recipient_nonce: Field,
    recipient_index: Field,
    recipient_path_old: [Field; TREE_DEPTH],
    recipient_path_new: [Field; TREE_DEPTH],
    amount: Field,
) {
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

// ============================================================================
// TRANSFER CIRCUIT (SIMPLE - MVP)
// ============================================================================
// Simplified transfer that only verifies sender authorization
// Public inputs: old_state_root, nullifier
// Private inputs: sender details, recipient_pubkey, amount

pub fn main_transfer_simple(
    old_state_root: Field,
    nullifier: Field,
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path: [Field; TREE_DEPTH],
    recipient_pubkey: Field,
    amount: Field,
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
        amount,
    );
}
