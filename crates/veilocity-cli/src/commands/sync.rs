//! Sync command - synchronize with on-chain state
//!
//! This module provides synchronization with the Veilocity contract:
//! - Fast sync via indexer API (preferred, ~100ms)
//! - Fallback to RPC event scanning if indexer unavailable

use crate::config::Config;
use crate::ui;
use crate::wallet::WalletManager;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::time::Instant;
use tracing::{debug, info, warn};
use veilocity_contracts::{create_vault_reader, DepositEvent, EventFilter, VeilocityEvent};
use veilocity_core::account::AccountSecret;
use veilocity_core::poseidon::{bytes_to_field, field_to_bytes, u128_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;

/// Maximum blocks to scan per batch (to avoid RPC timeouts)
const BLOCKS_PER_BATCH: u64 = 10000;

/// Indexer sync state response
#[derive(Debug, Deserialize)]
struct IndexerSyncState {
    state_root: String,
    leaves: Vec<String>,
    nullifiers: Vec<String>,
    last_block: u64,
    deposit_count: u64,
    tvl_wei: String,
    is_syncing: bool,
    sync_progress: u8,
}

/// Indexer deposits response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IndexerDepositsResponse {
    deposits: Vec<IndexerDeposit>,
    total: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IndexerDeposit {
    commitment: String,
    amount_wei: String,
    amount_mnt: f64,
    leaf_index: u64,
    block_number: u64,
    tx_hash: String,
}

/// Run the sync command
pub async fn run(config: &Config) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Check wallet exists
    if !wallet_manager.wallet_exists() {
        return Err(anyhow!("Wallet not found. Run 'veilocity init' first."));
    }

    // Load wallet and get secret for deposit recognition
    let wallet = wallet_manager.load_wallet()?;
    let password = rpassword::prompt_password(format!(
        "{} ",
        "Enter wallet password:".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    ))
    .context("Failed to read password")?;

    let veilocity_secret = wallet_manager.get_veilocity_secret(&wallet, &password)?;

    // Check vault address is configured
    if config.network.vault_address.is_empty() {
        println!();
        ui::print_notice(
            "Vault Not Configured",
            "Skipping on-chain sync. Update your config with the deployed vault address.",
        );
        return Ok(());
    }

    println!();
    println!("{}", ui::header("Syncing with Network"));
    println!();

    // Try indexer first, fall back to RPC
    if let Some(ref indexer_url) = config.sync.indexer_url {
        match sync_via_indexer(config, indexer_url, &veilocity_secret).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                warn!("Indexer sync failed: {}, falling back to RPC", e);
                println!(
                    "  {} Indexer unavailable, falling back to RPC sync...",
                    "⚠".yellow()
                );
                println!();
            }
        }
    }

    // Fallback to RPC sync
    sync_via_rpc(config, &veilocity_secret).await
}

/// Fast sync via indexer API
async fn sync_via_indexer(
    config: &Config,
    indexer_url: &str,
    veilocity_secret: &AccountSecret,
) -> Result<()> {
    let start = Instant::now();

    println!(
        "  {} {}",
        "Indexer:".truecolor(120, 120, 120),
        indexer_url.dimmed()
    );

    let client = reqwest::Client::new();

    // Fetch state from indexer
    let sync_url = format!("{}/sync", indexer_url);
    let state: IndexerSyncState = client
        .get(&sync_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .context("Failed to connect to indexer")?
        .json()
        .await
        .context("Failed to parse indexer response")?;

    if state.is_syncing {
        println!(
            "  {} Indexer syncing... {}%",
            "⏳".yellow(),
            state.sync_progress
        );
    }

    // Fetch deposits for ownership check
    let deposits_url = format!("{}/deposits", indexer_url);
    let deposits_resp: IndexerDepositsResponse = client
        .get(&deposits_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?
        .json()
        .await?;

    println!();
    println!("{}", ui::header("Indexer State"));
    println!();
    println!(
        "  {} {}",
        "Last Block:   ".truecolor(120, 120, 120),
        state.last_block.to_string().bright_white()
    );
    println!(
        "  {} 0x{}...",
        "State Root:   ".truecolor(120, 120, 120),
        &state.state_root[2..18].dimmed()
    );
    println!(
        "  {} {}",
        "Deposit Count:".truecolor(120, 120, 120),
        state
            .deposit_count
            .to_string()
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );

    // Parse TVL
    let tvl_wei: u128 = state.tvl_wei.parse().unwrap_or(0);
    println!(
        "  {} {} {}",
        "TVL:          ".truecolor(120, 120, 120),
        format!("{:.6} MNT", tvl_wei as f64 / 1e18).green(),
        format!("({} wei)", state.tvl_wei).dimmed()
    );

    // Initialize or update local state
    let db_path = config.db_path();
    config.ensure_data_dir()?;
    let mut local_state = StateManager::new(&db_path)?;

    // Rebuild local Merkle tree from indexer leaves
    let mut hasher = PoseidonHasher::new();
    let mut own_deposits_found = 0u64;

    println!();
    println!("{}", ui::header("Processing Deposits"));
    println!();

    for (i, leaf_hex) in state.leaves.iter().enumerate() {
        // Parse leaf bytes
        let leaf_bytes = hex::decode(leaf_hex.trim_start_matches("0x"))
            .context("Invalid leaf hex from indexer")?;
        let leaf_arr: [u8; 32] = leaf_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid leaf size"))?;
        let leaf_field = bytes_to_field(&leaf_arr);

        // Check if already in local tree
        if (local_state.leaf_count() as usize) <= i {
            local_state.insert_leaf(leaf_field)?;
        }

        // Check if this is our deposit
        if let Some(deposit) = deposits_resp.deposits.iter().find(|d| d.leaf_index == i as u64) {
            let amount_wei: u128 = deposit.amount_wei.parse().unwrap_or(0);
            let amount_field = u128_to_field(amount_wei);
            let expected_commitment =
                hasher.compute_deposit_commitment(veilocity_secret.secret(), &amount_field);
            let expected_bytes = field_to_bytes(&expected_commitment);

            if expected_bytes == leaf_arr {
                own_deposits_found += 1;

                // Create/update account
                let pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
                let pubkey_bytes = field_to_bytes(&pubkey_field);

                if let Some(mut existing) = local_state.get_account(&pubkey_bytes)? {
                    // Check if we already counted this deposit
                    let expected_balance = own_deposits_found as u128 * amount_wei;
                    if existing.balance < expected_balance {
                        existing.balance = expected_balance;
                        local_state.update_account(&existing)?;
                    }
                } else {
                    local_state.create_account(veilocity_secret, amount_wei)?;
                }

                println!(
                    "    {} Deposit #{}: {} {}",
                    "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
                        .bold(),
                    deposit.leaf_index,
                    format!("{:.6} MNT", deposit.amount_mnt)
                        .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
                        .bold(),
                    "[YOUR DEPOSIT]".green().bold()
                );
            }
        }
    }

    // Mark nullifiers as used
    for nullifier_hex in &state.nullifiers {
        let nullifier_bytes = hex::decode(nullifier_hex.trim_start_matches("0x"))
            .context("Invalid nullifier hex")?;
        let nullifier_arr: [u8; 32] = nullifier_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid nullifier size"))?;

        if !local_state.is_nullifier_used(&nullifier_arr) {
            local_state.mark_nullifier_used(&nullifier_arr)?;
        }
    }

    // Update sync checkpoint
    local_state.set_sync_checkpoint(state.last_block)?;

    let elapsed = start.elapsed();

    println!();
    println!("{}", ui::header("Sync Complete"));
    println!();
    println!(
        "  {} {}",
        "Deposits synced:  ".truecolor(120, 120, 120),
        state
            .leaves
            .len()
            .to_string()
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    if own_deposits_found > 0 {
        println!(
            "  {} {}",
            "Your deposits:    ".truecolor(120, 120, 120),
            own_deposits_found.to_string().green().bold()
        );
    }
    println!(
        "  {} {}",
        "Nullifiers synced:".truecolor(120, 120, 120),
        state.nullifiers.len().to_string().red()
    );
    println!(
        "  {} 0x{}...",
        "Local root:       ".truecolor(120, 120, 120),
        &hex::encode(field_to_bytes(&local_state.state_root()))[..16].dimmed()
    );
    println!(
        "  {} {}",
        "Sync time:        ".truecolor(120, 120, 120),
        format!("{:.2}ms", elapsed.as_secs_f64() * 1000.0)
            .green()
            .bold()
    );

    // Verify roots match
    let local_root = field_to_bytes(&local_state.state_root());
    let indexer_root =
        hex::decode(state.state_root.trim_start_matches("0x")).unwrap_or_else(|_| vec![0; 32]);

    if local_root == indexer_root.as_slice() {
        println!();
        ui::print_success("Successfully synced with indexer!");
    } else {
        println!();
        ui::print_notice(
            "Root Mismatch",
            "Local root differs from indexer. State may be stale.",
        );
    }

    println!();
    info!(
        "Indexer sync completed in {:.2}ms: {} deposits",
        elapsed.as_secs_f64() * 1000.0,
        state.leaves.len()
    );

    Ok(())
}

/// Fallback RPC sync (slower)
async fn sync_via_rpc(config: &Config, veilocity_secret: &AccountSecret) -> Result<()> {
    let vault_address = config
        .network
        .vault_address
        .parse()
        .context("Invalid vault address")?;

    println!(
        "  {} {}",
        "Network:".truecolor(120, 120, 120),
        config.network.rpc_url.dimmed()
    );
    println!(
        "  {} RPC sync is slower - consider running the indexer",
        "⚠".yellow()
    );

    // Create read-only vault client
    let vault = create_vault_reader(&config.network.rpc_url, vault_address)?;

    // Get current chain state
    let current_block = vault.get_block_number().await?;
    let on_chain_root = vault.current_root().await?;
    let deposit_count = vault.deposit_count().await?;
    let tvl = vault.total_value_locked().await?;

    println!();
    println!("{}", ui::header("On-chain State"));
    println!();
    println!(
        "  {} {}",
        "Block Number: ".truecolor(120, 120, 120),
        current_block.to_string().bright_white()
    );
    println!(
        "  {} 0x{}...",
        "State Root:   ".truecolor(120, 120, 120),
        &hex::encode(on_chain_root)[..16].dimmed()
    );
    println!(
        "  {} {}",
        "Deposit Count:".truecolor(120, 120, 120),
        deposit_count
            .to_string()
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {} {} {}",
        "TVL:          ".truecolor(120, 120, 120),
        format!("{:.6} MNT", tvl.to::<u128>() as f64 / 1e18).green(),
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
    println!("{}", ui::header("Local State"));
    println!();
    println!(
        "  {} 0x{}...",
        "Local Root:     ".truecolor(120, 120, 120),
        &hex::encode(field_to_bytes(&state.state_root()))[..16].dimmed()
    );
    println!(
        "  {} {}",
        "Local Leaves:   ".truecolor(120, 120, 120),
        state.leaf_count().to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Last Sync Block:".truecolor(120, 120, 120),
        last_synced_block.to_string().bright_white()
    );

    // Check if we need to sync
    let local_root_bytes = field_to_bytes(&state.state_root());
    if local_root_bytes == on_chain_root.0 {
        println!();
        ui::print_success("Local state is in sync with on-chain state.");
        return Ok(());
    }

    println!();
    ui::print_notice(
        "State Mismatch",
        "Local state differs from on-chain. Fetching new events...",
    );

    // Progress bar for RPC sync
    let total_blocks = current_block.saturating_sub(last_synced_block);
    let pb = ProgressBar::new(total_blocks);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.orange} [{bar:40.orange/dim}] {pos}/{len} blocks ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );

    // Fetch and process events in batches
    let mut from_block = last_synced_block + 1;
    let mut total_deposits_processed = 0u64;
    let mut own_deposits_found = 0u64;
    let mut total_withdrawals_processed = 0u64;

    // Create hasher for commitment verification
    let mut hasher = PoseidonHasher::new();

    while from_block <= current_block {
        let to_block = std::cmp::min(from_block + BLOCKS_PER_BATCH - 1, current_block);

        debug!("Fetching events from block {} to {}", from_block, to_block);

        let filter = EventFilter::block_range(from_block, to_block);
        let events = vault.get_all_events(&filter).await?;

        for event in events {
            match event {
                VeilocityEvent::Deposit(deposit) => {
                    // Check if this deposit belongs to us
                    let is_ours =
                        check_deposit_ownership(veilocity_secret, &deposit, &mut hasher);

                    process_deposit(&mut state, &deposit)?;
                    total_deposits_processed += 1;

                    if is_ours {
                        own_deposits_found += 1;
                        create_account_from_deposit(
                            &mut state,
                            veilocity_secret,
                            &deposit,
                            &mut hasher,
                        )?;
                    }
                }
                VeilocityEvent::Withdrawal(withdrawal) => {
                    let nullifier_bytes: [u8; 32] = withdrawal.nullifier.0;
                    if !state.is_nullifier_used(&nullifier_bytes) {
                        state.mark_nullifier_used(&nullifier_bytes)?;
                        total_withdrawals_processed += 1;
                    }
                }
                VeilocityEvent::StateRootUpdated(_) => {}
            }
        }

        // Update sync checkpoint
        state.set_sync_checkpoint(to_block)?;
        pb.set_position(to_block - last_synced_block);
        from_block = to_block + 1;
    }

    pb.finish_and_clear();

    // Final status
    println!();
    println!("{}", ui::header("Sync Complete"));
    println!();
    println!(
        "  {} {}",
        "Deposits processed:   ".truecolor(120, 120, 120),
        total_deposits_processed
            .to_string()
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    if own_deposits_found > 0 {
        println!(
            "  {} {} {}",
            "Your deposits found:  ".truecolor(120, 120, 120),
            own_deposits_found.to_string().green().bold(),
            "(account created/updated)".dimmed()
        );
    }
    println!(
        "  {} {}",
        "Withdrawals processed:".truecolor(120, 120, 120),
        total_withdrawals_processed.to_string().red()
    );
    println!(
        "  {} 0x{}...",
        "New local root:       ".truecolor(120, 120, 120),
        &hex::encode(field_to_bytes(&state.state_root()))[..16].dimmed()
    );
    println!(
        "  {} {}",
        "Synced to block:      ".truecolor(120, 120, 120),
        current_block.to_string().bright_white()
    );

    // Verify sync
    let new_local_root = field_to_bytes(&state.state_root());
    if new_local_root == on_chain_root.0 {
        println!();
        ui::print_success("Successfully synced with on-chain state!");
    } else {
        println!();
        ui::print_notice(
            "Root Mismatch",
            "Local root still differs from on-chain. May indicate pending updates.",
        );
    }

    println!();

    info!(
        "RPC sync completed: {} deposits, {} withdrawals, block {}",
        total_deposits_processed, total_withdrawals_processed, current_block
    );

    Ok(())
}

/// Check if a deposit belongs to the user by verifying the commitment
fn check_deposit_ownership(
    secret: &AccountSecret,
    deposit: &DepositEvent,
    hasher: &mut PoseidonHasher,
) -> bool {
    let amount_field = u128_to_field(deposit.amount.to::<u128>());
    let expected_commitment = hasher.compute_deposit_commitment(secret.secret(), &amount_field);
    let expected_bytes = field_to_bytes(&expected_commitment);
    expected_bytes == deposit.commitment.0
}

/// Create or update an account from a recognized deposit
fn create_account_from_deposit(
    state: &mut StateManager,
    secret: &AccountSecret,
    deposit: &DepositEvent,
    hasher: &mut PoseidonHasher,
) -> Result<()> {
    let pubkey_field = secret.derive_pubkey(hasher);
    let pubkey_bytes = field_to_bytes(&pubkey_field);
    let deposit_amount = deposit.amount.to::<u128>();

    if let Some(mut existing) = state.get_account(&pubkey_bytes)? {
        existing.balance += deposit_amount;
        state.update_account(&existing)?;
        debug!(
            "Updated existing account: balance now {} wei",
            existing.balance
        );
    } else {
        let account = state.create_account(secret, deposit_amount)?;
        debug!(
            "Created new account at index {} with balance {} wei",
            account.index, account.balance
        );
    }

    Ok(())
}

/// Process a deposit event and update local state
fn process_deposit(state: &mut StateManager, deposit: &DepositEvent) -> Result<()> {
    let expected_index = state.leaf_count();
    let deposit_index: u64 = deposit.leaf_index.try_into().unwrap_or(0);

    if deposit_index != expected_index {
        if deposit_index > expected_index {
            debug!(
                "Gap detected: expected index {}, got {}. Filling with empty leaves.",
                expected_index, deposit_index
            );
            let mut hasher = PoseidonHasher::new();
            for _ in expected_index..deposit_index {
                let empty_leaf = hasher.hash2(
                    &veilocity_core::poseidon::FieldElement::from(0u64),
                    &veilocity_core::poseidon::FieldElement::from(0u64),
                );
                state.insert_leaf(empty_leaf)?;
            }
        } else {
            debug!("Deposit {} already processed, skipping", deposit_index);
            return Ok(());
        }
    }

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

    println!(
        "{} Watching for new events...",
        "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!("{}", "Press Ctrl+C to stop.".dimmed());
    println!();

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
                            "  {} [Block {}] Deposit: {} MNT (index {})",
                            "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
                            deposit.block_number,
                            deposit.amount_mnt(),
                            deposit.leaf_index
                        );
                        process_deposit(&mut state, &deposit)?;
                    }
                    VeilocityEvent::Withdrawal(withdrawal) => {
                        println!(
                            "  {} [Block {}] Withdrawal: {} MNT to {:?}",
                            "↑".red(),
                            withdrawal.block_number,
                            withdrawal.amount_mnt(),
                            withdrawal.recipient
                        );
                        let nullifier_bytes: [u8; 32] = withdrawal.nullifier.0;
                        if !state.is_nullifier_used(&nullifier_bytes) {
                            state.mark_nullifier_used(&nullifier_bytes)?;
                        }
                    }
                    VeilocityEvent::StateRootUpdated(update) => {
                        println!(
                            "  {} [Block {}] State root updated (batch {})",
                            "⟲".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
                            update.block_number,
                            update.batch_index
                        );
                    }
                }
            }

            state.set_sync_checkpoint(current_block)?;
            last_block = current_block;
        }
    }
}
