//! Deposit command - deposit funds from Mantle into Veilocity

use crate::config::Config;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use alloy::primitives::{B256, U256};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use tracing::info;
use veilocity_contracts::create_vault_client;
use veilocity_core::poseidon::{field_to_bytes, PoseidonHasher};
use veilocity_core::state::StateManager;

/// Run the deposit command
pub async fn run(config: &Config, amount: f64, dry_run: bool) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Load wallet
    let wallet = wallet_manager.load_wallet()?;

    // Get password
    let password = rpassword::prompt_password("Enter wallet password: ")
        .context("Failed to read password")?;

    // Unlock wallet
    let signer = wallet_manager.unlock(&wallet, &password)?;
    let veilocity_secret = wallet_manager.get_veilocity_secret(&wallet, &password)?;

    // Check vault address is configured
    if config.network.vault_address.is_empty() {
        return Err(anyhow!(
            "Vault address not configured. Please update your config with the deployed vault address."
        ));
    }

    let vault_address = config
        .network
        .vault_address
        .parse()
        .context("Invalid vault address")?;

    // Parse amount
    let amount_wei = parse_eth(amount);
    let amount_u256 = U256::from(amount_wei);

    println!();
    println!("{}", "═══ Deposit Details ═══".green().bold());
    println!(
        "Amount:     {} {}",
        format_eth(amount_wei).bright_white().bold(),
        format!("({} wei)", amount_wei).dimmed()
    );
    println!("From:       {}", wallet.address.bright_white());
    println!("Network:    {}", config.network.rpc_url.dimmed());

    // Generate deposit commitment
    let mut hasher = PoseidonHasher::new();
    let commitment = veilocity_secret.compute_deposit_commitment(&mut hasher, amount_wei);
    let commitment_bytes = field_to_bytes(&commitment);
    let commitment_b256 = B256::from(commitment_bytes);

    println!(
        "Commitment: 0x{}...",
        &hex::encode(commitment_bytes)[..16].dimmed()
    );

    if dry_run {
        println!();
        println!("{}", "DRY RUN - No transaction submitted".yellow().bold());
        println!("{}", "Remove --dry-run to execute the deposit.".dimmed());
        return Ok(());
    }

    // Create vault client
    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer).await?;

    println!();
    println!("{}", "Submitting deposit transaction...".yellow());

    // Send deposit transaction
    let tx_hash = vault.deposit(commitment_b256, amount_u256).await?;
    let tx_hash_hex = hex::encode(tx_hash);

    // Record transaction in local database
    config.ensure_data_dir()?;
    if let Ok(mut state) = StateManager::new(&config.db_path()) {
        let _ = state.record_transaction(
            "deposit",
            amount_wei,
            Some(tx_hash.as_slice()),
            None,
            "confirmed",
        );
    }

    println!();
    println!("{}", "✓ Deposit successful!".green().bold());
    println!("Transaction: 0x{}", tx_hash_hex.bright_white());

    if let Some(explorer) = &config.network.explorer_url {
        println!(
            "Explorer:   {}",
            format!("{}/tx/0x{}", explorer, tx_hash_hex).blue().underline()
        );
    }

    println!();
    println!("{}", "─".repeat(50));
    println!(
        "{}",
        "Your funds are now in the Veilocity private pool.".green()
    );
    println!(
        "Next: Run {} to update your local state.",
        "veilocity sync".cyan()
    );

    info!("Deposit of {} wei completed", amount_wei);

    Ok(())
}
