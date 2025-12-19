//! Sync command - synchronize with on-chain state
//!
//! This module provides real-time synchronization with the Veilocity contract:
//! - Fetches deposit events from chain
//! - Processes deposits and updates local Merkle tree
//! - Tracks sync checkpoint for incremental updates

use crate::config::Config;
use crate::wallet::WalletManager;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use tracing::{debug, info};
use veilocity_contracts::{create_vault_reader, DepositEvent, EventFilter, VeilocityEvent};
use veilocity_core::poseidon::{bytes_to_field, field_to_bytes, PoseidonHasher};
use veilocity_core::state::StateManager;

/// Maximum blocks to scan per batch (to avoid RPC timeouts)
const BLOCKS_PER_BATCH: u64 = 10000;

/// Run the sync command
pub async fn run(config: &Config) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Check wallet exists
    if !wallet_manager.wallet_exists() {
        return Err(anyhow!("Wallet not found. Run 'veilocity init' first."));
    }

    // Check vault address is configured
    if config.network.vault_address.is_empty() {
        println!();
        println!("{}", "⚠ Vault address not configured.".yellow());
        println!(
            "{}",
            "Skipping on-chain sync. Update your config with the deployed vault address.".dimmed()
        );
        return Ok(());
    }

    let vault_address = config
        .network
        .vault_address
        .parse()
        .context("Invalid vault address")?;

    println!();
    println!("{}", "═══ Syncing with Network ═══".cyan().bold());
    println!("Network: {}", config.network.rpc_url.dimmed());

    // Create read-only vault client
    let vault = create_vault_reader(&config.network.rpc_url, vault_address)?;

    // Get current chain state
    let current_block = vault.get_block_number().await?;
    let on_chain_root = vault.current_root().await?;
    let deposit_count = vault.deposit_count().await?;
    let tvl = vault.total_value_locked().await?;

    println!();
    println!("{}", "═══ On-chain State ═══".cyan());
    println!("Block Number:  {}", current_block.to_string().bright_white());
    println!(
        "State Root:    0x{}...",
        &hex::encode(on_chain_root)[..16].dimmed()
    );
    println!("Deposit Count: {}", deposit_count.to_string().bright_white());
    println!(
        "TVL:           {} {}",
        format!("{:.6} ETH", tvl.to::<u128>() as f64 / 1e18).green(),
        format!("({} wei)", tvl).dimmed()
    );

    // Initialize or load local state
    let db_path = config.db_path();
    config.ensure_data_dir()?;

    let mut state = StateManager::new(&db_path)?;

    // Get last synced block from database (default to 0 if not set)
    let deployment_block = config.sync.deployment_block.unwrap_or(0);
    let last_synced_block = state.get_sync_checkpoint().unwrap_or(deployment_block);

    println!();
    println!("{}", "═══ Local State ═══".cyan());
    println!(
        "Local Root:      0x{}...",
        &hex::encode(field_to_bytes(&state.state_root()))[..16].dimmed()
    );
    println!("Local Leaves:    {}", state.leaf_count().to_string().bright_white());
    println!("Last Sync Block: {}", last_synced_block.to_string().bright_white());

    // Check if we need to sync
    let local_root_bytes = field_to_bytes(&state.state_root());
    if local_root_bytes == on_chain_root.0 {
        println!();
        println!("{}", "✓ Local state is in sync with on-chain state.".green().bold());
        return Ok(());
    }

    println!();
    println!(
        "{}",
        "⚠ Local state differs from on-chain state.".yellow()
    );
    println!("{}", "  Fetching new events...".dimmed());

    // Fetch and process events in batches
    let mut from_block = last_synced_block + 1;
    let mut total_deposits_processed = 0u64;
    let mut total_withdrawals_processed = 0u64;

    while from_block <= current_block {
        let to_block = std::cmp::min(from_block + BLOCKS_PER_BATCH - 1, current_block);

        debug!("Fetching events from block {} to {}", from_block, to_block);

        let filter = EventFilter::block_range(from_block, to_block);
        let events = vault.get_all_events(&filter).await?;

        if !events.is_empty() {
            println!(
                "Processing {} events from blocks {}-{}...",
                events.len().to_string().bright_white(),
                from_block,
                to_block
            );
        }

        for event in events {
            match event {
                VeilocityEvent::Deposit(deposit) => {
                    process_deposit(&mut state, &deposit)?;
                    total_deposits_processed += 1;
                    println!(
                        "  {} Deposit #{}: {} (tx: 0x{}...)",
                        "✓".green(),
                        deposit.leaf_index,
                        format!("{} ETH", deposit.amount_eth()).green(),
                        &hex::encode(deposit.tx_hash)[..8].dimmed()
                    );
                }
                VeilocityEvent::Withdrawal(withdrawal) => {
                    // Mark nullifier as used
                    let nullifier_bytes: [u8; 32] = withdrawal.nullifier.0;
                    if !state.is_nullifier_used(&nullifier_bytes) {
                        state.mark_nullifier_used(&nullifier_bytes)?;
                        total_withdrawals_processed += 1;
                        println!(
                            "  {} Withdrawal: {} to {:?} (tx: 0x{}...)",
                            "✓".red(),
                            format!("{} ETH", withdrawal.amount_eth()).red(),
                            withdrawal.recipient,
                            &hex::encode(withdrawal.tx_hash)[..8].dimmed()
                        );
                    }
                }
                VeilocityEvent::StateRootUpdated(update) => {
                    debug!(
                        "State root updated: 0x{}... -> 0x{}...",
                        &hex::encode(update.old_root)[..8],
                        &hex::encode(update.new_root)[..8]
                    );
                }
            }
        }

        // Update sync checkpoint
        state.set_sync_checkpoint(to_block)?;
        from_block = to_block + 1;
    }

    // Final status
    println!();
    println!("{}", "═══ Sync Complete ═══".cyan().bold());
    println!(
        "Deposits processed:    {}",
        total_deposits_processed.to_string().green()
    );
    println!(
        "Withdrawals processed: {}",
        total_withdrawals_processed.to_string().red()
    );
    println!(
        "New local root:        0x{}...",
        &hex::encode(field_to_bytes(&state.state_root()))[..16].dimmed()
    );
    println!(
        "Synced to block:       {}",
        current_block.to_string().bright_white()
    );

    // Verify sync
    let new_local_root = field_to_bytes(&state.state_root());
    if new_local_root == on_chain_root.0 {
        println!();
        println!(
            "{}",
            "✓ Successfully synced with on-chain state!".green().bold()
        );
    } else {
        println!();
        println!(
            "{}",
            "⚠ Warning: Local root still differs from on-chain.".yellow()
        );
        println!(
            "{}",
            "  This may indicate pending state updates or missing events.".dimmed()
        );
        println!("  On-chain: 0x{}...", &hex::encode(on_chain_root)[..16]);
        println!("  Local:    0x{}...", &hex::encode(new_local_root)[..16]);
    }

    info!(
        "Sync completed: {} deposits, {} withdrawals, block {}",
        total_deposits_processed, total_withdrawals_processed, current_block
    );

    Ok(())
}

/// Process a deposit event and update local state
fn process_deposit(state: &mut StateManager, deposit: &DepositEvent) -> Result<()> {
    let expected_index = state.leaf_count();
    let deposit_index: u64 = deposit.leaf_index.try_into().unwrap_or(0);

    // Verify this is the next expected deposit
    if deposit_index != expected_index {
        // If we're behind, we may have missed deposits
        if deposit_index > expected_index {
            debug!(
                "Gap detected: expected index {}, got {}. Filling with empty leaves.",
                expected_index, deposit_index
            );
            // Fill gaps with empty leaves (shouldn't happen in normal operation)
            let mut hasher = PoseidonHasher::new();
            for _ in expected_index..deposit_index {
                let empty_leaf = hasher.hash2(
                    &veilocity_core::poseidon::FieldElement::from(0u64),
                    &veilocity_core::poseidon::FieldElement::from(0u64),
                );
                state.insert_leaf(empty_leaf)?;
            }
        } else {
            // Already processed this deposit
            debug!("Deposit {} already processed, skipping", deposit_index);
            return Ok(());
        }
    }

    // Insert the deposit commitment as a leaf
    // Note: The commitment is hash(secret, amount), not the full account leaf
    // The account leaf will be created when the user claims/activates the deposit
    let commitment_field = bytes_to_field(&deposit.commitment.0);
    state.insert_leaf(commitment_field)?;

    debug!(
        "Inserted deposit {} with commitment 0x{}",
        deposit_index,
        &hex::encode(deposit.commitment)[..16]
    );

    Ok(())
}

/// Watch for new events in real-time (for background sync daemon)
pub async fn watch(config: &Config) -> Result<()> {
    let vault_address = config
        .network
        .vault_address
        .parse()
        .context("Invalid vault address")?;

    let vault = create_vault_reader(&config.network.rpc_url, vault_address)?;

    println!("Watching for new events...");
    println!("Press Ctrl+C to stop.\n");

    let db_path = config.db_path();
    let mut state = StateManager::new(&db_path)?;
    let mut last_block = vault.get_block_number().await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let current_block = vault.get_block_number().await?;
        if current_block > last_block {
            let filter = EventFilter::block_range(last_block + 1, current_block);
            let events = vault.get_all_events(&filter).await?;

            for event in events {
                match event {
                    VeilocityEvent::Deposit(deposit) => {
                        println!(
                            "[Block {}] New deposit: {} ETH (index {})",
                            deposit.block_number,
                            deposit.amount_eth(),
                            deposit.leaf_index
                        );
                        process_deposit(&mut state, &deposit)?;
                    }
                    VeilocityEvent::Withdrawal(withdrawal) => {
                        println!(
                            "[Block {}] Withdrawal: {} ETH to {:?}",
                            withdrawal.block_number,
                            withdrawal.amount_eth(),
                            withdrawal.recipient
                        );
                        let nullifier_bytes: [u8; 32] = withdrawal.nullifier.0;
                        if !state.is_nullifier_used(&nullifier_bytes) {
                            state.mark_nullifier_used(&nullifier_bytes)?;
                        }
                    }
                    VeilocityEvent::StateRootUpdated(update) => {
                        println!(
                            "[Block {}] State root updated (batch {})",
                            update.block_number, update.batch_index
                        );
                    }
                }
            }

            state.set_sync_checkpoint(current_block)?;
            last_block = current_block;
        }
    }
}
