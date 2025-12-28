//! Witness generation for ZK circuits
//!
//! This module generates the private and public inputs for each circuit type.

use crate::error::ProverError;
use serde::{Deserialize, Serialize};
use veilocity_core::poseidon::{field_to_hex, FieldElement};

/// Tree depth constant (must match Noir circuit)
pub const TREE_DEPTH: usize = 20;

/// Deposit witness for the deposit circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositWitness {
    /// Public: The deposit commitment
    pub commitment: String,
    /// Public: The deposit amount
    pub amount: String,
    /// Private: The user's secret
    pub secret: String,
}

impl DepositWitness {
    /// Create a new deposit witness
    pub fn new(
        commitment: FieldElement,
        amount: FieldElement,
        secret: FieldElement,
    ) -> Self {
        Self {
            commitment: field_to_hex(&commitment),
            amount: field_to_hex(&amount),
            secret: field_to_hex(&secret),
        }
    }

    /// Convert to Prover.toml format
    pub fn to_toml(&self) -> String {
        format!(
            r#"commitment = "{}"
amount = "{}"
secret = "{}""#,
            self.commitment, self.amount, self.secret
        )
    }

    /// Convert to JSON for bb prove
    pub fn to_json(&self) -> Result<String, ProverError> {
        serde_json::to_string_pretty(self).map_err(ProverError::from)
    }
}

/// Withdrawal witness for the withdrawal circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawWitness {
    /// Public: Current state root
    pub state_root: String,
    /// Public: Nullifier for this withdrawal
    pub nullifier: String,
    /// Public: Withdrawal amount
    pub amount: String,
    /// Public: Recipient address (as field)
    pub recipient: String,
    /// Private: Account secret
    pub secret: String,
    /// Private: Current balance
    pub balance: String,
    /// Private: Account nonce
    pub nonce: String,
    /// Private: Leaf index
    pub index: String,
    /// Private: Merkle path (siblings)
    pub path: Vec<String>,
}

impl WithdrawWitness {
    /// Create a new withdrawal witness
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state_root: FieldElement,
        nullifier: FieldElement,
        amount: FieldElement,
        recipient: FieldElement,
        secret: FieldElement,
        balance: FieldElement,
        nonce: FieldElement,
        index: FieldElement,
        path: Vec<FieldElement>,
    ) -> Result<Self, ProverError> {
        if path.len() != TREE_DEPTH {
            return Err(ProverError::InvalidInput(format!(
                "Merkle path must have {} elements, got {}",
                TREE_DEPTH,
                path.len()
            )));
        }

        Ok(Self {
            state_root: field_to_hex(&state_root),
            nullifier: field_to_hex(&nullifier),
            amount: field_to_hex(&amount),
            recipient: field_to_hex(&recipient),
            secret: field_to_hex(&secret),
            balance: field_to_hex(&balance),
            nonce: field_to_hex(&nonce),
            index: field_to_hex(&index),
            path: path.iter().map(field_to_hex).collect(),
        })
    }

    /// Convert to Prover.toml format
    pub fn to_toml(&self) -> String {
        let path_str = self
            .path
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"state_root = "{}"
nullifier = "{}"
amount = "{}"
recipient = "{}"
secret = "{}"
balance = "{}"
nonce = "{}"
index = "{}"
path = [{}]"#,
            self.state_root,
            self.nullifier,
            self.amount,
            self.recipient,
            self.secret,
            self.balance,
            self.nonce,
            self.index,
            path_str
        )
    }

    /// Convert to JSON for bb prove
    pub fn to_json(&self) -> Result<String, ProverError> {
        serde_json::to_string_pretty(self).map_err(ProverError::from)
    }
}

/// Transfer witness for the private transfer circuit (simplified version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferWitness {
    /// Public: Old state root
    pub old_state_root: String,
    /// Public: Nullifier for sender's spend
    pub nullifier: String,
    /// Private: Sender's secret
    pub sender_secret: String,
    /// Private: Sender's balance
    pub sender_balance: String,
    /// Private: Sender's nonce
    pub sender_nonce: String,
    /// Private: Sender's leaf index
    pub sender_index: String,
    /// Private: Sender's Merkle path
    pub sender_path: Vec<String>,
    /// Private: Recipient's public key
    pub recipient_pubkey: String,
    /// Private: Transfer amount
    pub amount: String,
}

impl TransferWitness {
    /// Create a new transfer witness (simplified version)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        old_state_root: FieldElement,
        nullifier: FieldElement,
        sender_secret: FieldElement,
        sender_balance: FieldElement,
        sender_nonce: FieldElement,
        sender_index: FieldElement,
        sender_path: Vec<FieldElement>,
        recipient_pubkey: FieldElement,
        amount: FieldElement,
    ) -> Result<Self, ProverError> {
        if sender_path.len() != TREE_DEPTH {
            return Err(ProverError::InvalidInput(format!(
                "Sender Merkle path must have {} elements, got {}",
                TREE_DEPTH,
                sender_path.len()
            )));
        }

        Ok(Self {
            old_state_root: field_to_hex(&old_state_root),
            nullifier: field_to_hex(&nullifier),
            sender_secret: field_to_hex(&sender_secret),
            sender_balance: field_to_hex(&sender_balance),
            sender_nonce: field_to_hex(&sender_nonce),
            sender_index: field_to_hex(&sender_index),
            sender_path: sender_path.iter().map(field_to_hex).collect(),
            recipient_pubkey: field_to_hex(&recipient_pubkey),
            amount: field_to_hex(&amount),
        })
    }

    /// Convert to Prover.toml format
    pub fn to_toml(&self) -> String {
        let sender_path_str = self
            .sender_path
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"old_state_root = "{}"
nullifier = "{}"
sender_secret = "{}"
sender_balance = "{}"
sender_nonce = "{}"
sender_index = "{}"
sender_path = [{}]
recipient_pubkey = "{}"
amount = "{}""#,
            self.old_state_root,
            self.nullifier,
            self.sender_secret,
            self.sender_balance,
            self.sender_nonce,
            self.sender_index,
            sender_path_str,
            self.recipient_pubkey,
            self.amount
        )
    }

    /// Convert to JSON for bb prove
    pub fn to_json(&self) -> Result<String, ProverError> {
        serde_json::to_string_pretty(self).map_err(ProverError::from)
    }
}

/// Full transfer witness with old and new paths for both sender and recipient
/// This is required for proper state transition verification:
/// old_root -> (update sender) -> intermediate_root -> (update recipient) -> new_root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTransferWitness {
    /// Public: Old state root
    pub old_state_root: String,
    /// Public: New state root
    pub new_state_root: String,
    /// Public: Nullifier
    pub nullifier: String,
    /// Sender data
    pub sender_secret: String,
    pub sender_balance: String,
    pub sender_nonce: String,
    pub sender_index: String,
    /// Sender's Merkle path in OLD state (before any updates)
    pub sender_path_old: Vec<String>,
    /// Sender's Merkle path in NEW state (after recipient update)
    pub sender_path_new: Vec<String>,
    /// Recipient data
    pub recipient_pubkey: String,
    pub recipient_balance: String,
    pub recipient_nonce: String,
    pub recipient_index: String,
    /// Recipient's Merkle path in OLD state
    pub recipient_path_old: Vec<String>,
    /// Recipient's Merkle path in INTERMEDIATE state (after sender update)
    pub recipient_path_new: Vec<String>,
    /// Transfer amount
    pub amount: String,
}

impl FullTransferWitness {
    /// Create a new full transfer witness with 4 Merkle paths
    ///
    /// The state transition is verified as:
    /// 1. Verify sender exists in old_state_root using sender_path_old
    /// 2. Verify recipient exists in old_state_root using recipient_path_old
    /// 3. Update sender leaf -> intermediate_root
    /// 4. Verify recipient's new path leads to intermediate_root using recipient_path_new
    /// 5. Update recipient leaf -> new_state_root
    /// 6. Verify sender's new path leads to new_state_root using sender_path_new
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        old_state_root: FieldElement,
        new_state_root: FieldElement,
        nullifier: FieldElement,
        sender_secret: FieldElement,
        sender_balance: FieldElement,
        sender_nonce: FieldElement,
        sender_index: FieldElement,
        sender_path_old: Vec<FieldElement>,
        sender_path_new: Vec<FieldElement>,
        recipient_pubkey: FieldElement,
        recipient_balance: FieldElement,
        recipient_nonce: FieldElement,
        recipient_index: FieldElement,
        recipient_path_old: Vec<FieldElement>,
        recipient_path_new: Vec<FieldElement>,
        amount: FieldElement,
    ) -> Result<Self, ProverError> {
        // Validate all paths have correct length
        for (name, path) in [
            ("sender_path_old", &sender_path_old),
            ("sender_path_new", &sender_path_new),
            ("recipient_path_old", &recipient_path_old),
            ("recipient_path_new", &recipient_path_new),
        ] {
            if path.len() != TREE_DEPTH {
                return Err(ProverError::InvalidInput(format!(
                    "{} must have {} elements, got {}",
                    name,
                    TREE_DEPTH,
                    path.len()
                )));
            }
        }

        Ok(Self {
            old_state_root: field_to_hex(&old_state_root),
            new_state_root: field_to_hex(&new_state_root),
            nullifier: field_to_hex(&nullifier),
            sender_secret: field_to_hex(&sender_secret),
            sender_balance: field_to_hex(&sender_balance),
            sender_nonce: field_to_hex(&sender_nonce),
            sender_index: field_to_hex(&sender_index),
            sender_path_old: sender_path_old.iter().map(field_to_hex).collect(),
            sender_path_new: sender_path_new.iter().map(field_to_hex).collect(),
            recipient_pubkey: field_to_hex(&recipient_pubkey),
            recipient_balance: field_to_hex(&recipient_balance),
            recipient_nonce: field_to_hex(&recipient_nonce),
            recipient_index: field_to_hex(&recipient_index),
            recipient_path_old: recipient_path_old.iter().map(field_to_hex).collect(),
            recipient_path_new: recipient_path_new.iter().map(field_to_hex).collect(),
            amount: field_to_hex(&amount),
        })
    }

    /// Convert to Prover.toml format matching the Noir circuit signature
    pub fn to_toml(&self) -> String {
        let sender_path_old_str = self
            .sender_path_old
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");
        let sender_path_new_str = self
            .sender_path_new
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");
        let recipient_path_old_str = self
            .recipient_path_old
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");
        let recipient_path_new_str = self
            .recipient_path_new
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"old_state_root = "{}"
new_state_root = "{}"
nullifier = "{}"
sender_secret = "{}"
sender_balance = "{}"
sender_nonce = "{}"
sender_index = "{}"
sender_path_old = [{}]
sender_path_new = [{}]
recipient_pubkey = "{}"
recipient_balance = "{}"
recipient_nonce = "{}"
recipient_index = "{}"
recipient_path_old = [{}]
recipient_path_new = [{}]
amount = "{}""#,
            self.old_state_root,
            self.new_state_root,
            self.nullifier,
            self.sender_secret,
            self.sender_balance,
            self.sender_nonce,
            self.sender_index,
            sender_path_old_str,
            sender_path_new_str,
            self.recipient_pubkey,
            self.recipient_balance,
            self.recipient_nonce,
            self.recipient_index,
            recipient_path_old_str,
            recipient_path_new_str,
            self.amount
        )
    }

    /// Convert to JSON for bb prove
    pub fn to_json(&self) -> Result<String, ProverError> {
        serde_json::to_string_pretty(self).map_err(ProverError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veilocity_core::poseidon::u64_to_field;

    #[test]
    fn test_deposit_witness_creation() {
        let commitment = u64_to_field(12345);
        let amount = u64_to_field(1_000_000_000);
        let secret = u64_to_field(54321);

        let witness = DepositWitness::new(commitment, amount, secret);
        assert!(!witness.commitment.is_empty());
        assert!(!witness.to_toml().is_empty());
    }

    #[test]
    fn test_withdraw_witness_creation() {
        let witness = WithdrawWitness::new(
            u64_to_field(1),
            u64_to_field(2),
            u64_to_field(3),
            u64_to_field(4),
            u64_to_field(5),
            u64_to_field(6),
            u64_to_field(7),
            u64_to_field(8),
            vec![u64_to_field(0); TREE_DEPTH],
        )
        .unwrap();

        assert_eq!(witness.path.len(), TREE_DEPTH);
    }

    #[test]
    fn test_withdraw_witness_invalid_path() {
        let result = WithdrawWitness::new(
            u64_to_field(1),
            u64_to_field(2),
            u64_to_field(3),
            u64_to_field(4),
            u64_to_field(5),
            u64_to_field(6),
            u64_to_field(7),
            u64_to_field(8),
            vec![u64_to_field(0); 10], // Wrong length
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_witness_creation() {
        let witness = TransferWitness::new(
            u64_to_field(1),
            u64_to_field(2),
            u64_to_field(3),
            u64_to_field(4),
            u64_to_field(5),
            u64_to_field(6),
            vec![u64_to_field(0); TREE_DEPTH],
            u64_to_field(7),
            u64_to_field(8),
        )
        .unwrap();

        assert_eq!(witness.sender_path.len(), TREE_DEPTH);
    }

    /// Generate a valid test withdraw witness with correct Merkle proof
    /// This test outputs the Prover.toml for debugging
    #[test]
    fn test_generate_valid_withdraw_witness() {
        use veilocity_core::poseidon::{field_to_hex, PoseidonHasher};

        let mut hasher = PoseidonHasher::new();

        // Test values matching circuit test
        let secret = u64_to_field(123456789);
        let balance = u64_to_field(2000000000000000000); // 2 MNT
        let nonce = u64_to_field(0);
        let index = u64_to_field(0);
        let amount = u64_to_field(1000000000000000000); // 1 MNT
        let recipient = u64_to_field(0x1234567890abcdef);

        // Compute derived values
        let pubkey = hasher.derive_pubkey(&secret);
        let leaf = hasher.compute_leaf(&pubkey, &balance, &nonce);
        let nullifier = hasher.compute_nullifier(&secret, &index, &nonce);

        // Build empty Merkle path (for leaf at index 0)
        let zero = u64_to_field(0);
        let mut empty = hasher.hash2(&zero, &zero);
        let mut path = Vec::new();

        for _ in 0..TREE_DEPTH {
            path.push(empty);
            empty = hasher.hash2(&empty, &empty);
        }

        // Compute state root from leaf and path
        let mut current = leaf;
        let index_val: u64 = 0;
        for i in 0..TREE_DEPTH {
            let sibling = path[i];
            let bit = (index_val >> i) & 1;
            if bit == 0 {
                current = hasher.hash2(&current, &sibling);
            } else {
                current = hasher.hash2(&sibling, &current);
            }
        }
        let state_root = current;

        // Create witness
        let witness = WithdrawWitness::new(
            state_root,
            nullifier,
            amount,
            recipient,
            secret,
            balance,
            nonce,
            index,
            path,
        )
        .unwrap();

        // Output the Prover.toml for debugging
        println!("=== Valid Prover.toml for Withdraw Circuit ===");
        println!("{}", witness.to_toml());
        println!("==============================================");

        // Verify witness is valid
        assert_eq!(witness.path.len(), TREE_DEPTH);
        assert!(!witness.state_root.is_empty());
        assert!(!witness.nullifier.is_empty());
    }
}
