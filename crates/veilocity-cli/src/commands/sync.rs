//! Sync command - synchronize with on-chain state

use crate::config::Config;
use crate::wallet::WalletManager;
use anyhow::{anyhow, Context, Result};
use tracing::info;
use veilocity_contracts::create_vault_reader;
use veilocity_core::poseidon::field_to_bytes;
use veilocity_core::state::StateManager;

/// Run the sync command
pub async fn run(config: &Config) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Check wallet exists
    if !wallet_manager.wallet_exists() {
        return Err(anyhow!("Wallet not found. Run 'veilocity init' first."));
    }

    // Check vault address is configured
    if config.network.vault_address.is_empty() {
        println!("Warning: Vault address not configured.");
        println!("Skipping on-chain sync. Run with a configured vault address to sync.");
        return Ok(());
    }

    let vault_address = config
        .network
        .vault_address
        .parse()
        .context("Invalid vault address")?;

    println!("Syncing with {}...\n", config.network.rpc_url);

    // Create read-only vault client
    let vault = create_vault_reader(&config.network.rpc_url, vault_address)?;

    // Get on-chain state
    let current_root = vault.current_root().await?;
    let deposit_count = vault.deposit_count().await?;
    let tvl = vault.total_value_locked().await?;
    let block_number = vault.get_block_number().await?;

    println!("=== On-chain State ===");
    println!("Block Number:    {}", block_number);
    println!("State Root:      0x{}", hex::encode(current_root));
    println!("Deposit Count:   {}", deposit_count);
    println!("TVL:             {} wei", tvl);

    // Initialize or load local state
    let db_path = config.db_path();
    config.ensure_data_dir()?;

    let state = StateManager::new(&db_path)?;

    println!("\n=== Local State ===");
    println!("Local Root:      0x{}", hex::encode(field_to_bytes(&state.state_root())));
    println!("Local Leaves:    {}", state.leaf_count());

    // Compare roots
    let local_root_bytes = field_to_bytes(&state.state_root());
    if local_root_bytes == current_root.0 {
        println!("\n✓ Local state is in sync with on-chain state.");
    } else {
        println!("\n⚠ Local state differs from on-chain state.");
        println!("  This may happen if there are new deposits or state updates.");
        println!("  In a full implementation, we would fetch and process new events here.");
    }

    // In a full implementation, we would:
    // 1. Query deposit events from the last synced block
    // 2. Process each deposit and update local state
    // 3. Verify state roots match
    // 4. Store sync checkpoint

    println!("\nSync complete.");

    info!(
        "Sync completed at block {}, deposit count: {}",
        block_number, deposit_count
    );

    Ok(())
}
