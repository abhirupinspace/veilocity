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

/// Full transfer witness with both sender and recipient paths
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
    pub sender_path: Vec<String>,
    /// Recipient data
    pub recipient_pubkey: String,
    pub recipient_balance: String,
    pub recipient_nonce: String,
    pub recipient_index: String,
    pub recipient_path: Vec<String>,
    /// Transfer amount
    pub amount: String,
}

impl FullTransferWitness {
    /// Create a new full transfer witness
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        old_state_root: FieldElement,
        new_state_root: FieldElement,
        nullifier: FieldElement,
        sender_secret: FieldElement,
        sender_balance: FieldElement,
        sender_nonce: FieldElement,
        sender_index: FieldElement,
        sender_path: Vec<FieldElement>,
        recipient_pubkey: FieldElement,
        recipient_balance: FieldElement,
        recipient_nonce: FieldElement,
        recipient_index: FieldElement,
        recipient_path: Vec<FieldElement>,
        amount: FieldElement,
    ) -> Result<Self, ProverError> {
        if sender_path.len() != TREE_DEPTH {
            return Err(ProverError::InvalidInput(format!(
                "Sender path must have {} elements",
                TREE_DEPTH
            )));
        }
        if recipient_path.len() != TREE_DEPTH {
            return Err(ProverError::InvalidInput(format!(
                "Recipient path must have {} elements",
                TREE_DEPTH
            )));
        }

        Ok(Self {
            old_state_root: field_to_hex(&old_state_root),
            new_state_root: field_to_hex(&new_state_root),
            nullifier: field_to_hex(&nullifier),
            sender_secret: field_to_hex(&sender_secret),
            sender_balance: field_to_hex(&sender_balance),
            sender_nonce: field_to_hex(&sender_nonce),
            sender_index: field_to_hex(&sender_index),
            sender_path: sender_path.iter().map(field_to_hex).collect(),
            recipient_pubkey: field_to_hex(&recipient_pubkey),
            recipient_balance: field_to_hex(&recipient_balance),
            recipient_nonce: field_to_hex(&recipient_nonce),
            recipient_index: field_to_hex(&recipient_index),
            recipient_path: recipient_path.iter().map(field_to_hex).collect(),
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
        let recipient_path_str = self
            .recipient_path
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
sender_path = [{}]
recipient_pubkey = "{}"
recipient_balance = "{}"
recipient_nonce = "{}"
recipient_index = "{}"
recipient_path = [{}]
amount = "{}""#,
            self.old_state_root,
            self.new_state_root,
            self.nullifier,
            self.sender_secret,
            self.sender_balance,
            self.sender_nonce,
            self.sender_index,
            sender_path_str,
            self.recipient_pubkey,
            self.recipient_balance,
            self.recipient_nonce,
            self.recipient_index,
            recipient_path_str,
            self.amount
        )
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
}
