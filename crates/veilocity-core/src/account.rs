//! Private account management

use crate::poseidon::{
    bytes_to_field, field_to_bytes, u128_to_field, u64_to_field, FieldElement, PoseidonHasher,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// A private account in the Veilocity execution layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateAccount {
    /// Public key (hash of secret)
    pub pubkey: [u8; 32],

    /// Current balance in wei
    pub balance: u128,

    /// Transaction nonce (increments on each spend)
    pub nonce: u64,

    /// Leaf index in Merkle tree
    pub index: u64,
}

impl PrivateAccount {
    /// Create a new account from a secret
    pub fn new(hasher: &mut PoseidonHasher, secret: &FieldElement, index: u64) -> Self {
        let pubkey_field = hasher.derive_pubkey(secret);

        Self {
            pubkey: field_to_bytes(&pubkey_field),
            balance: 0,
            nonce: 0,
            index,
        }
    }

    /// Create an account with initial balance (for deposits)
    pub fn with_balance(
        hasher: &mut PoseidonHasher,
        secret: &FieldElement,
        index: u64,
        balance: u128,
    ) -> Self {
        let pubkey_field = hasher.derive_pubkey(secret);

        Self {
            pubkey: field_to_bytes(&pubkey_field),
            balance,
            nonce: 0,
            index,
        }
    }

    /// Get pubkey as field element
    pub fn pubkey_field(&self) -> FieldElement {
        bytes_to_field(&self.pubkey)
    }

    /// Get balance as field element
    pub fn balance_field(&self) -> FieldElement {
        u128_to_field(self.balance)
    }

    /// Get nonce as field element
    pub fn nonce_field(&self) -> FieldElement {
        u64_to_field(self.nonce)
    }

    /// Get index as field element
    pub fn index_field(&self) -> FieldElement {
        u64_to_field(self.index)
    }

    /// Compute the leaf commitment for this account
    pub fn compute_leaf(&self, hasher: &mut PoseidonHasher) -> FieldElement {
        hasher.compute_leaf(&self.pubkey_field(), &self.balance_field(), &self.nonce_field())
    }

    /// Credit balance (for deposits)
    pub fn credit(&mut self, amount: u128) {
        self.balance = self.balance.saturating_add(amount);
    }

    /// Debit balance (for transfers/withdrawals)
    /// Returns true if successful, false if insufficient balance
    pub fn debit(&mut self, amount: u128) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.nonce += 1;
            true
        } else {
            false
        }
    }

    /// Check if account has sufficient balance
    pub fn has_balance(&self, amount: u128) -> bool {
        self.balance >= amount
    }
}

/// Secret key for an account
#[derive(Clone)]
pub struct AccountSecret {
    secret: FieldElement,
    raw_bytes: [u8; 32],
}

impl AccountSecret {
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            secret: bytes_to_field(bytes),
            raw_bytes: *bytes,
        }
    }

    /// Generate a new random secret
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self::from_bytes(&bytes)
    }

    /// Get the secret field element
    pub fn secret(&self) -> &FieldElement {
        &self.secret
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.raw_bytes
    }

    /// Derive public key
    pub fn derive_pubkey(&self, hasher: &mut PoseidonHasher) -> FieldElement {
        hasher.derive_pubkey(&self.secret)
    }

    /// Compute nullifier for spending
    pub fn compute_nullifier(
        &self,
        hasher: &mut PoseidonHasher,
        index: u64,
        nonce: u64,
    ) -> FieldElement {
        hasher.compute_nullifier(&self.secret, &u64_to_field(index), &u64_to_field(nonce))
    }

    /// Compute deposit commitment
    pub fn compute_deposit_commitment(
        &self,
        hasher: &mut PoseidonHasher,
        amount: u128,
    ) -> FieldElement {
        hasher.compute_deposit_commitment(&self.secret, &u128_to_field(amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let mut hasher = PoseidonHasher::new();
        let secret = AccountSecret::generate();
        let account = PrivateAccount::new(&mut hasher, secret.secret(), 0);

        assert_eq!(account.balance, 0);
        assert_eq!(account.nonce, 0);
        assert_eq!(account.index, 0);
        assert_ne!(account.pubkey, [0u8; 32]);
    }

    #[test]
    fn test_credit_debit() {
        let mut hasher = PoseidonHasher::new();
        let secret = AccountSecret::generate();
        let mut account = PrivateAccount::new(&mut hasher, secret.secret(), 0);

        // Credit
        account.credit(1_000_000_000_000_000_000); // 1 MNT
        assert_eq!(account.balance, 1_000_000_000_000_000_000);
        assert_eq!(account.nonce, 0); // Credit doesn't increment nonce

        // Debit
        assert!(account.debit(500_000_000_000_000_000)); // 0.5 MNT
        assert_eq!(account.balance, 500_000_000_000_000_000);
        assert_eq!(account.nonce, 1); // Debit increments nonce

        // Insufficient balance
        assert!(!account.debit(1_000_000_000_000_000_000)); // Can't debit 1 MNT
        assert_eq!(account.balance, 500_000_000_000_000_000); // Balance unchanged
        assert_eq!(account.nonce, 1); // Nonce unchanged
    }

    #[test]
    fn test_leaf_computation() {
        let mut hasher = PoseidonHasher::new();
        let secret = AccountSecret::generate();
        let mut account = PrivateAccount::new(&mut hasher, secret.secret(), 0);

        let leaf1 = account.compute_leaf(&mut hasher);

        account.credit(1_000_000_000_000_000_000);
        let leaf2 = account.compute_leaf(&mut hasher);

        assert_ne!(leaf1, leaf2); // Leaf changes when balance changes
    }

    #[test]
    fn test_nullifier_uniqueness() {
        let mut hasher = PoseidonHasher::new();
        let secret = AccountSecret::generate();

        let null1 = secret.compute_nullifier(&mut hasher, 0, 0);
        let null2 = secret.compute_nullifier(&mut hasher, 0, 1);
        let null3 = secret.compute_nullifier(&mut hasher, 1, 0);

        assert_ne!(null1, null2);
        assert_ne!(null1, null3);
        assert_ne!(null2, null3);
    }

    #[test]
    fn test_secret_pubkey_consistency() {
        let mut hasher = PoseidonHasher::new();
        let secret = AccountSecret::generate();

        let pubkey1 = secret.derive_pubkey(&mut hasher);
        let pubkey2 = secret.derive_pubkey(&mut hasher);

        assert_eq!(pubkey1, pubkey2);

        let account = PrivateAccount::new(&mut hasher, secret.secret(), 0);
        assert_eq!(pubkey1, account.pubkey_field());
    }
}
