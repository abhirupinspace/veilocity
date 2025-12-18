//! Deposit command - deposit funds from Mantle into Veilocity

use crate::config::Config;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use alloy::primitives::{B256, U256};
use anyhow::{anyhow, Context, Result};
use tracing::info;
use veilocity_contracts::create_vault_client;
use veilocity_core::poseidon::{field_to_bytes, PoseidonHasher};

/// Run the deposit command
pub async fn run(config: &Config, amount: f64) -> Result<()> {
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

    // Create vault client
    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer).await?;

    // Parse amount
    let amount_wei = parse_eth(amount);
    let amount_u256 = U256::from(amount_wei);

    println!("\n=== Deposit Details ===");
    println!("Amount: {} ({} wei)", format_eth(amount_wei), amount_wei);

    // Generate deposit commitment
    let mut hasher = PoseidonHasher::new();
    let commitment = veilocity_secret.compute_deposit_commitment(&mut hasher, amount_wei);
    let commitment_bytes = field_to_bytes(&commitment);
    let commitment_b256 = B256::from(commitment_bytes);

    println!("Commitment: 0x{}", hex::encode(commitment_bytes));

    // Confirm deposit
    println!("\nSubmitting deposit transaction...");

    // Send deposit transaction
    let tx_hash = vault.deposit(commitment_b256, amount_u256).await?;

    println!("\nDeposit successful!");
    println!("Transaction hash: 0x{}", hex::encode(tx_hash));

    if let Some(explorer) = &config.network.explorer_url {
        println!("View on explorer: {}/tx/0x{}", explorer, hex::encode(tx_hash));
    }

    println!("\nYour funds are now in the Veilocity private pool.");
    println!("Use 'veilocity sync' to update your local state.");
    println!("Use 'veilocity balance' to check your private balance.");

    info!("Deposit of {} wei completed", amount_wei);

    Ok(())
}
