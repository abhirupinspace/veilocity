//! Event parsing and indexing
//!
//! This module handles parsing and processing events from Veilocity contracts.

use crate::bindings::IVeilocityVault;
use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

/// Parsed deposit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositEvent {
    /// The deposit commitment
    pub commitment: B256,
    /// Amount deposited in wei
    pub amount: U256,
    /// Assigned leaf index
    pub leaf_index: U256,
    /// Block timestamp
    pub timestamp: U256,
    /// Block number where the event was emitted
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: B256,
}

impl DepositEvent {
    /// Create from raw log data
    pub fn from_log(
        log: &IVeilocityVault::Deposit,
        block_number: u64,
        tx_hash: B256,
    ) -> Self {
        Self {
            commitment: log.commitment,
            amount: log.amount,
            leaf_index: log.leafIndex,
            timestamp: log.timestamp,
            block_number,
            tx_hash,
        }
    }

    /// Get commitment as hex string
    pub fn commitment_hex(&self) -> String {
        format!("0x{}", hex::encode(self.commitment))
    }

    /// Get amount in MNT
    pub fn amount_mnt(&self) -> f64 {
        let wei: u128 = self.amount.try_into().unwrap_or(0);
        wei as f64 / 1e18
    }
}

/// Parsed withdrawal event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalEvent {
    /// The nullifier used
    pub nullifier: B256,
    /// Recipient address
    pub recipient: Address,
    /// Amount withdrawn in wei
    pub amount: U256,
    /// Block number
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: B256,
}

impl WithdrawalEvent {
    /// Create from raw log data
    pub fn from_log(
        log: &IVeilocityVault::Withdrawal,
        block_number: u64,
        tx_hash: B256,
    ) -> Self {
        Self {
            nullifier: log.nullifier,
            recipient: log.recipient,
            amount: log.amount,
            block_number,
            tx_hash,
        }
    }

    /// Get nullifier as hex string
    pub fn nullifier_hex(&self) -> String {
        format!("0x{}", hex::encode(self.nullifier))
    }

    /// Get amount in MNT
    pub fn amount_mnt(&self) -> f64 {
        let wei: u128 = self.amount.try_into().unwrap_or(0);
        wei as f64 / 1e18
    }
}

/// Parsed state root update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRootUpdatedEvent {
    /// Previous state root
    pub old_root: B256,
    /// New state root
    pub new_root: B256,
    /// Batch index
    pub batch_index: U256,
    /// Block timestamp
    pub timestamp: U256,
    /// Block number
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: B256,
}

impl StateRootUpdatedEvent {
    /// Create from raw log data
    pub fn from_log(
        log: &IVeilocityVault::StateRootUpdated,
        block_number: u64,
        tx_hash: B256,
    ) -> Self {
        Self {
            old_root: log.oldRoot,
            new_root: log.newRoot,
            batch_index: log.batchIndex,
            timestamp: log.timestamp,
            block_number,
            tx_hash,
        }
    }
}

/// Union type for all Veilocity events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VeilocityEvent {
    Deposit(DepositEvent),
    Withdrawal(WithdrawalEvent),
    StateRootUpdated(StateRootUpdatedEvent),
}

impl VeilocityEvent {
    /// Get the block number of the event
    pub fn block_number(&self) -> u64 {
        match self {
            VeilocityEvent::Deposit(e) => e.block_number,
            VeilocityEvent::Withdrawal(e) => e.block_number,
            VeilocityEvent::StateRootUpdated(e) => e.block_number,
        }
    }

    /// Get the transaction hash of the event
    pub fn tx_hash(&self) -> B256 {
        match self {
            VeilocityEvent::Deposit(e) => e.tx_hash,
            VeilocityEvent::Withdrawal(e) => e.tx_hash,
            VeilocityEvent::StateRootUpdated(e) => e.tx_hash,
        }
    }
}

/// Event filter configuration
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Start block (inclusive)
    pub from_block: Option<u64>,
    /// End block (inclusive)
    pub to_block: Option<u64>,
    /// Filter by commitment (for deposits)
    pub commitment: Option<B256>,
    /// Filter by nullifier (for withdrawals)
    pub nullifier: Option<B256>,
}

impl EventFilter {
    /// Create a new filter starting from a specific block
    pub fn from_block(block: u64) -> Self {
        Self {
            from_block: Some(block),
            ..Default::default()
        }
    }

    /// Create a filter for a specific block range
    pub fn block_range(from: u64, to: u64) -> Self {
        Self {
            from_block: Some(from),
            to_block: Some(to),
            ..Default::default()
        }
    }
}
