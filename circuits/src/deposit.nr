// Deposit circuit
// Proves that a deposit commitment is correctly formed
//
// This is a simple circuit (~500 constraints) that verifies:
// 1. The commitment equals hash(secret, amount)
// 2. The amount is positive
//
// Public inputs:
// - commitment: The deposit commitment to verify
// - amount: The deposit amount (public so contract can verify value sent)
//
// Private inputs:
// - secret: User's secret for this deposit

use crate::poseidon_utils::compute_deposit_commitment;

// Main deposit circuit function
pub fn verify_deposit(
    commitment: pub Field,
    amount: pub Field,
    secret: Field
) {
    // Constraint 1: Commitment is correctly formed
    let computed_commitment = compute_deposit_commitment(secret, amount);
    assert(computed_commitment == commitment, "Invalid commitment");

    // Constraint 2: Amount must be positive
    // We check that amount fits in u64 and is > 0
    let amount_u64 = amount as u64;
    assert(amount_u64 > 0, "Amount must be positive");

    // Also verify amount hasn't overflowed (is same when cast back)
    assert(amount == amount_u64 as Field, "Amount overflow");
}

#[test]
fn test_valid_deposit() {
    let secret: Field = 123456789;
    let amount: Field = 1000000000000000000; // 1 ETH

    let commitment = compute_deposit_commitment(secret, amount);

    // This should not panic
    verify_deposit(commitment, amount, secret);
}

#[test]
fn test_deposit_commitment_deterministic() {
    let secret: Field = 123456789;
    let amount: Field = 1000000000000000000;

    let c1 = compute_deposit_commitment(secret, amount);
    let c2 = compute_deposit_commitment(secret, amount);

    assert(c1 == c2);
}

#[test(should_fail_with = "Invalid commitment")]
fn test_wrong_secret_fails() {
    let secret: Field = 123456789;
    let wrong_secret: Field = 987654321;
    let amount: Field = 1000000000000000000;

    let commitment = compute_deposit_commitment(secret, amount);

    // This should fail because wrong secret
    verify_deposit(commitment, amount, wrong_secret);
}

#[test(should_fail_with = "Invalid commitment")]
fn test_wrong_amount_fails() {
    let secret: Field = 123456789;
    let amount: Field = 1000000000000000000;
    let wrong_amount: Field = 2000000000000000000;

    let commitment = compute_deposit_commitment(secret, amount);

    // This should fail because wrong amount
    verify_deposit(commitment, wrong_amount, secret);
}
