//! Sync command - synchronize with on-chain state
//!
//! This module provides real-time synchronization with the Veilocity contract:
//! - Fetches deposit events from chain
//! - Processes deposits and updates local Merkle tree
//! - Recognizes user's own deposits and creates accounts
//! - Tracks sync checkpoint for incremental updates

use crate::config::Config;
use crate::ui;
use crate::wallet::WalletManager;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use tracing::{debug, info};
use veilocity_contracts::{create_vault_reader, DepositEvent, EventFilter, VeilocityEvent};
use veilocity_core::account::AccountSecret;
use veilocity_core::poseidon::{bytes_to_field, field_to_bytes, u128_to_field, PoseidonHasher};
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

    let vault_address = config
        .network
        .vault_address
        .parse()
        .context("Invalid vault address")?;

    println!();
    println!("{}", ui::header("Syncing with Network"));
    println!();
    println!(
        "  {} {}",
        "Network:".truecolor(120, 120, 120),
        config.network.rpc_url.dimmed()
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
        deposit_count.to_string().truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {} {} {}",
        "TVL:          ".truecolor(120, 120, 120),
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

        if !events.is_empty() {
            println!();
            println!(
                "  {} Processing {} events from blocks {}-{}...",
                "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
                events.len().to_string().bright_white(),
                from_block,
                to_block
            );
        }

        for event in events {
            match event {
                VeilocityEvent::Deposit(deposit) => {
                    // Check if this deposit belongs to us
                    let is_ours = check_deposit_ownership(&veilocity_secret, &deposit, &mut hasher);

                    process_deposit(&mut state, &deposit)?;
                    total_deposits_processed += 1;

                    if is_ours {
                        own_deposits_found += 1;
                        // Create/update account for this deposit
                        create_account_from_deposit(&mut state, &veilocity_secret, &deposit, &mut hasher)?;
                        println!(
                            "    {} Deposit #{}: {} {} (tx: 0x{}...)",
                            "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
                            deposit.leaf_index,
                            format!("{} ETH", deposit.amount_eth())
                                .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
                                .bold(),
                            "[YOUR DEPOSIT]".green().bold(),
                            &hex::encode(deposit.tx_hash)[..8].dimmed()
                        );
                    } else {
                        println!(
                            "    {} Deposit #{}: {} (tx: 0x{}...)",
                            "✓".green(),
                            deposit.leaf_index,
                            format!("{} ETH", deposit.amount_eth()).dimmed(),
                            &hex::encode(deposit.tx_hash)[..8].dimmed()
                        );
                    }
                }
                VeilocityEvent::Withdrawal(withdrawal) => {
                    // Mark nullifier as used
                    let nullifier_bytes: [u8; 32] = withdrawal.nullifier.0;
                    if !state.is_nullifier_used(&nullifier_bytes) {
                        state.mark_nullifier_used(&nullifier_bytes)?;
                        total_withdrawals_processed += 1;
                        println!(
                            "    {} Withdrawal: {} to {} (tx: 0x{}...)",
                            "↑".red(),
                            format!("{} ETH", withdrawal.amount_eth()).red(),
                            ui::format_address(&format!("{:?}", withdrawal.recipient)),
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
    println!("{}", ui::header("Sync Complete"));
    println!();
    println!(
        "  {} {}",
        "Deposits processed:   ".truecolor(120, 120, 120),
        total_deposits_processed.to_string().truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
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
        println!(
            "    On-chain: 0x{}...",
            &hex::encode(on_chain_root)[..16]
        );
        println!(
            "    Local:    0x{}...",
            &hex::encode(new_local_root)[..16]
        );
    }

    println!();

    info!(
        "Sync completed: {} deposits, {} withdrawals, block {}",
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
    // Compute what the commitment should be if this deposit is ours
    let amount_field = u128_to_field(deposit.amount.to::<u128>());
    let expected_commitment = hasher.compute_deposit_commitment(secret.secret(), &amount_field);
    let expected_bytes = field_to_bytes(&expected_commitment);

    // Compare with on-chain commitment
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

    // Check if account already exists
    if let Some(mut existing) = state.get_account(&pubkey_bytes)? {
        // Account exists - add to balance
        existing.balance += deposit_amount;
        state.update_account(&existing)?;
        debug!(
            "Updated existing account: balance now {} wei",
            existing.balance
        );
    } else {
        // Create new account with this deposit
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
                            "  {} [Block {}] Deposit: {} ETH (index {})",
                            "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
                            deposit.block_number,
                            deposit.amount_eth(),
                            deposit.leaf_index
                        );
                        process_deposit(&mut state, &deposit)?;
                    }
                    VeilocityEvent::Withdrawal(withdrawal) => {
                        println!(
                            "  {} [Block {}] Withdrawal: {} ETH to {:?}",
                            "↑".red(),
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
