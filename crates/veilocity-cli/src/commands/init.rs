//! Initialize command - create a new Veilocity wallet

use crate::config::{get_data_dir, Config};
use crate::wallet::WalletManager;
use anyhow::{Context, Result};
use tracing::info;

/// Run the init command
pub async fn run(recover: bool) -> Result<()> {
    let data_dir = get_data_dir();
    let config = Config {
        data_dir: data_dir.clone(),
        ..Config::default()
    };

    let wallet_manager = WalletManager::new(config.clone());

    // Check if wallet already exists
    if wallet_manager.wallet_exists() {
        println!("Wallet already exists at {:?}", wallet_manager.wallet_path());
        println!("To create a new wallet, delete the existing one first.");
        return Ok(());
    }

    if recover {
        println!("Recovery from seed phrase is not yet implemented.");
        println!("Please create a new wallet for now.");
        return Ok(());
    }

    // Get password from user
    let password = rpassword::prompt_password("Enter password for new wallet: ")
        .context("Failed to read password")?;

    let password_confirm = rpassword::prompt_password("Confirm password: ")
        .context("Failed to read password confirmation")?;

    if password != password_confirm {
        println!("Passwords do not match!");
        return Ok(());
    }

    if password.len() < 8 {
        println!("Password must be at least 8 characters!");
        return Ok(());
    }

    println!("\nCreating new Veilocity wallet...\n");

    // Generate wallet
    let (wallet, _signer, _secret) = wallet_manager.generate(&password)?;

    // Save wallet
    wallet_manager.save_wallet(&wallet)?;

    // Save config
    config.save()?;

    println!("Wallet created successfully!\n");
    println!("=== Wallet Information ===");
    println!("Ethereum Address: {}", wallet.address);
    println!("Veilocity Public Key: {}", wallet.veilocity_pubkey);
    println!("\nData directory: {:?}", data_dir);
    println!("\nIMPORTANT: Keep your password safe. There is no way to recover it.");
    println!("\nNext steps:");
    println!("  1. Fund your Ethereum address with MNT (Mantle native token)");
    println!("  2. Use 'veilocity deposit <amount>' to deposit into Veilocity");
    println!("  3. Use 'veilocity balance' to check your private balance");

    info!("Wallet initialized successfully");

    Ok(())
}
