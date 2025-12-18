//! Transaction types for the private execution layer

use crate::poseidon::{field_to_bytes, FieldElement};
use serde::{Deserialize, Serialize};

/// Transaction type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Transfer,
    Withdraw,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction created but not yet proven
    Pending,
    /// Proof generated
    Proven,
    /// Submitted to chain (for deposits/withdrawals)
    Submitted,
    /// Confirmed on chain
    Confirmed,
    /// Failed
    Failed,
}

/// A deposit transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositTransaction {
    /// Deposit commitment (hash of secret and amount)
    pub commitment: [u8; 32],
    /// Amount in wei
    pub amount: u128,
    /// Status
    pub status: TransactionStatus,
    /// On-chain transaction hash (if submitted)
    pub tx_hash: Option<String>,
    /// Block number (if confirmed)
    pub block_number: Option<u64>,
    /// Leaf index assigned (if confirmed)
    pub leaf_index: Option<u64>,
    /// Timestamp
    pub timestamp: u64,
}

impl DepositTransaction {
    pub fn new(commitment: FieldElement, amount: u128) -> Self {
        Self {
            commitment: field_to_bytes(&commitment),
            amount,
            status: TransactionStatus::Pending,
            tx_hash: None,
            block_number: None,
            leaf_index: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn commitment_hex(&self) -> String {
        format!("0x{}", hex::encode(self.commitment))
    }
}

/// A private transfer transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTransaction {
    /// Nullifier for sender's spend
    pub nullifier: [u8; 32],
    /// Recipient's public key
    pub recipient_pubkey: [u8; 32],
    /// Amount transferred
    pub amount: u128,
    /// Old state root
    pub old_state_root: [u8; 32],
    /// New state root
    pub new_state_root: [u8; 32],
    /// ZK proof (if generated)
    pub proof: Option<Vec<u8>>,
    /// Status
    pub status: TransactionStatus,
    /// Timestamp
    pub timestamp: u64,
}

impl TransferTransaction {
    pub fn new(
        nullifier: FieldElement,
        recipient_pubkey: FieldElement,
        amount: u128,
        old_state_root: FieldElement,
        new_state_root: FieldElement,
    ) -> Self {
        Self {
            nullifier: field_to_bytes(&nullifier),
            recipient_pubkey: field_to_bytes(&recipient_pubkey),
            amount,
            old_state_root: field_to_bytes(&old_state_root),
            new_state_root: field_to_bytes(&new_state_root),
            proof: None,
            status: TransactionStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn nullifier_hex(&self) -> String {
        format!("0x{}", hex::encode(self.nullifier))
    }
}

/// A withdrawal transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawTransaction {
    /// Nullifier for this withdrawal
    pub nullifier: [u8; 32],
    /// Recipient address on Mantle
    pub recipient: String,
    /// Amount to withdraw
    pub amount: u128,
    /// State root used in proof
    pub state_root: [u8; 32],
    /// ZK proof
    pub proof: Option<Vec<u8>>,
    /// Status
    pub status: TransactionStatus,
    /// On-chain transaction hash
    pub tx_hash: Option<String>,
    /// Block number
    pub block_number: Option<u64>,
    /// Timestamp
    pub timestamp: u64,
}

impl WithdrawTransaction {
    pub fn new(
        nullifier: FieldElement,
        recipient: String,
        amount: u128,
        state_root: FieldElement,
    ) -> Self {
        Self {
            nullifier: field_to_bytes(&nullifier),
            recipient,
            amount,
            state_root: field_to_bytes(&state_root),
            proof: None,
            status: TransactionStatus::Pending,
            tx_hash: None,
            block_number: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn nullifier_hex(&self) -> String {
        format!("0x{}", hex::encode(self.nullifier))
    }

    pub fn state_root_hex(&self) -> String {
        format!("0x{}", hex::encode(self.state_root))
    }
}

/// Union type for any transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    Deposit(DepositTransaction),
    Transfer(TransferTransaction),
    Withdraw(WithdrawTransaction),
}

impl Transaction {
    pub fn tx_type(&self) -> TransactionType {
        match self {
            Transaction::Deposit(_) => TransactionType::Deposit,
            Transaction::Transfer(_) => TransactionType::Transfer,
            Transaction::Withdraw(_) => TransactionType::Withdraw,
        }
    }

    pub fn status(&self) -> TransactionStatus {
        match self {
            Transaction::Deposit(tx) => tx.status,
            Transaction::Transfer(tx) => tx.status,
            Transaction::Withdraw(tx) => tx.status,
        }
    }

    pub fn amount(&self) -> u128 {
        match self {
            Transaction::Deposit(tx) => tx.amount,
            Transaction::Transfer(tx) => tx.amount,
            Transaction::Withdraw(tx) => tx.amount,
        }
    }

    pub fn timestamp(&self) -> u64 {
        match self {
            Transaction::Deposit(tx) => tx.timestamp,
            Transaction::Transfer(tx) => tx.timestamp,
            Transaction::Withdraw(tx) => tx.timestamp,
        }
    }
}
