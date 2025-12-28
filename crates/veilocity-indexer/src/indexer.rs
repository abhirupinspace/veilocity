//! Background indexer that syncs with the chain

use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use veilocity_contracts::{create_vault_reader, EventFilter, VeilocityEvent};
use veilocity_core::poseidon::{bytes_to_field, field_to_bytes};
use veilocity_core::MerkleTree;

/// Maximum blocks to scan per batch (Mantle RPC limits to 10k)
const BLOCKS_PER_BATCH: u64 = 9000;

/// Indexed deposit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDeposit {
    pub commitment: B256,
    pub amount: U256,
    pub leaf_index: u64,
    pub block_number: u64,
    pub tx_hash: B256,
}

/// Indexed withdrawal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedWithdrawal {
    pub nullifier: B256,
    pub recipient: Address,
    pub amount: U256,
    pub block_number: u64,
    pub tx_hash: B256,
}

/// Current indexer state - served via API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerState {
    /// Current Merkle root
    pub state_root: [u8; 32],
    /// All deposit commitments (leaves)
    pub leaves: Vec<[u8; 32]>,
    /// All deposits indexed
    pub deposits: Vec<IndexedDeposit>,
    /// All withdrawals indexed
    pub withdrawals: Vec<IndexedWithdrawal>,
    /// Used nullifiers
    pub nullifiers: Vec<[u8; 32]>,
    /// Last synced block
    pub last_block: u64,
    /// On-chain deposit count
    pub deposit_count: u64,
    /// Total value locked
    pub tvl_wei: String,
    /// Is currently syncing
    pub is_syncing: bool,
    /// Sync progress (0-100)
    pub sync_progress: u8,
}

impl IndexerState {
    pub fn new() -> Self {
        // Compute empty tree root
        let tree = MerkleTree::new();
        let root = field_to_bytes(&tree.root());

        Self {
            state_root: root,
            leaves: Vec::new(),
            deposits: Vec::new(),
            withdrawals: Vec::new(),
            nullifiers: Vec::new(),
            last_block: 0,
            deposit_count: 0,
            tvl_wei: "0".to_string(),
            is_syncing: true,
            sync_progress: 0,
        }
    }
}

impl Default for IndexerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the background sync loop
pub async fn run_sync_loop(
    state: Arc<RwLock<IndexerState>>,
    rpc_url: &str,
    vault_address: Address,
    deployment_block: u64,
    poll_interval: u64,
) {
    info!("Starting background sync from block {}", deployment_block);

    let vault = match create_vault_reader(rpc_url, vault_address) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to create vault reader: {}", e);
            return;
        }
    };

    // Initial sync - fetch all historical events
    let mut from_block = deployment_block;
    let mut tree = MerkleTree::new();
    let mut deposits = Vec::new();
    let mut withdrawals = Vec::new();
    let mut nullifiers = Vec::new();
    let mut leaves = Vec::new();

    loop {
        // Get current block
        let current_block = match vault.get_block_number().await {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to get block number: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval)).await;
                continue;
            }
        };

        // Calculate sync progress
        let total_blocks = current_block.saturating_sub(deployment_block);
        let synced_blocks = from_block.saturating_sub(deployment_block);
        let progress = if total_blocks > 0 {
            ((synced_blocks as f64 / total_blocks as f64) * 100.0) as u8
        } else {
            100
        };

        // Update progress
        {
            let mut s = state.write().await;
            s.sync_progress = progress;
            s.is_syncing = from_block < current_block;
        }

        if from_block > current_block {
            // Fully synced, wait for new blocks
            {
                let mut s = state.write().await;
                s.is_syncing = false;
                s.sync_progress = 100;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval)).await;
            continue;
        }

        // Fetch events in batches
        let to_block = std::cmp::min(from_block + BLOCKS_PER_BATCH - 1, current_block);

        debug!("Fetching events from block {} to {}", from_block, to_block);

        let filter = EventFilter::block_range(from_block, to_block);

        // Fetch all events concurrently
        let events = match vault.get_all_events(&filter).await {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to fetch events: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let events_count = events.len();

        // Process events
        for event in events {
            match event {
                VeilocityEvent::Deposit(deposit) => {
                    let leaf_index: u64 = deposit.leaf_index.try_into().unwrap_or(0);

                    // Insert into Merkle tree
                    let commitment_field = bytes_to_field(&deposit.commitment.0);

                    // Fill gaps if needed (shouldn't happen normally)
                    while (leaves.len() as u64) < leaf_index {
                        let empty = [0u8; 32];
                        leaves.push(empty);
                        let _ = tree.insert(bytes_to_field(&empty));
                    }

                    leaves.push(deposit.commitment.0);
                    let _ = tree.insert(commitment_field);

                    deposits.push(IndexedDeposit {
                        commitment: deposit.commitment,
                        amount: deposit.amount,
                        leaf_index,
                        block_number: deposit.block_number,
                        tx_hash: deposit.tx_hash,
                    });

                    debug!(
                        "Indexed deposit #{}: {} wei",
                        leaf_index,
                        deposit.amount
                    );
                }
                VeilocityEvent::Withdrawal(withdrawal) => {
                    nullifiers.push(withdrawal.nullifier.0);

                    withdrawals.push(IndexedWithdrawal {
                        nullifier: withdrawal.nullifier,
                        recipient: withdrawal.recipient,
                        amount: withdrawal.amount,
                        block_number: withdrawal.block_number,
                        tx_hash: withdrawal.tx_hash,
                    });

                    debug!(
                        "Indexed withdrawal: {} wei to {:?}",
                        withdrawal.amount,
                        withdrawal.recipient
                    );
                }
                VeilocityEvent::StateRootUpdated(_) => {
                    // We compute our own root, don't need to track these
                }
            }
        }

        // Update state
        {
            let deposit_count = vault.deposit_count().await.unwrap_or_default();
            let tvl = vault.total_value_locked().await.unwrap_or_default();

            let mut s = state.write().await;
            s.state_root = field_to_bytes(&tree.root());
            s.leaves = leaves.clone();
            s.deposits = deposits.clone();
            s.withdrawals = withdrawals.clone();
            s.nullifiers = nullifiers.clone();
            s.last_block = to_block;
            s.deposit_count = deposit_count.try_into().unwrap_or(0);
            s.tvl_wei = tvl.to_string();
        }

        from_block = to_block + 1;

        if events_count > 0 {
            info!(
                "Synced to block {}: {} deposits, {} withdrawals",
                to_block,
                deposits.len(),
                withdrawals.len()
            );
        }
    }
}
