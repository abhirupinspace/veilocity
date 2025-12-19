//! Initialize command - create a new Veilocity wallet

use crate::config::{get_data_dir, Config};
use crate::wallet::WalletManager;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use tracing::info;

/// Validate password strength
fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(anyhow!("Password must be at least 8 characters"));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(anyhow!(
            "Password must contain uppercase, lowercase, and a digit"
        ));
    }

    Ok(())
}

/// Run the init command
pub async fn run(recover: bool) -> Result<()> {
    let data_dir = get_data_dir();
    let config = Config {
        data_dir: data_dir.clone(),
        ..Config::default()
    };

    let wallet_manager = WalletManager::new(config.clone());

    println!();
    println!("{}", "═══ Veilocity Wallet Setup ═══".cyan().bold());

    // Check if wallet already exists
    if wallet_manager.wallet_exists() {
        println!();
        println!(
            "{} Wallet already exists at:",
            "⚠".yellow()
        );
        println!("  {:?}", wallet_manager.wallet_path());
        println!();
        println!(
            "{}",
            "To create a new wallet, delete the existing one first.".dimmed()
        );
        return Ok(());
    }

    if recover {
        println!();
        println!(
            "{}",
            "Recovery from seed phrase is not yet implemented.".yellow()
        );
        println!("{}", "Please create a new wallet for now.".dimmed());
        return Ok(());
    }

    println!();
    println!("{}", "Password Requirements:".bold());
    println!("  • At least 8 characters");
    println!("  • Mix of uppercase and lowercase letters");
    println!("  • At least one number");
    println!();

    // Get password from user
    let password = rpassword::prompt_password("Enter password for new wallet: ")
        .context("Failed to read password")?;

    // Validate password strength
    if let Err(e) = validate_password(&password) {
        println!();
        println!("{} {}", "Error:".red().bold(), e);
        return Ok(());
    }

    let password_confirm = rpassword::prompt_password("Confirm password: ")
        .context("Failed to read password confirmation")?;

    if password != password_confirm {
        println!();
        println!("{}", "✗ Passwords do not match!".red().bold());
        return Ok(());
    }

    println!();
    println!("{}", "Creating new Veilocity wallet...".yellow());

    // Generate wallet
    let (wallet, _signer, _secret) = wallet_manager.generate(&password)?;

    // Save wallet
    wallet_manager.save_wallet(&wallet)?;

    // Save config
    config.save()?;

    println!();
    println!("{}", "✓ Wallet created successfully!".green().bold());
    println!();
    println!("{}", "═══ Wallet Information ═══".cyan());
    println!(
        "Ethereum Address:     {}",
        wallet.address.bright_white().bold()
    );
    println!(
        "Veilocity Public Key: {}",
        wallet.veilocity_pubkey.bright_white()
    );
    println!("Data Directory:       {}", format!("{:?}", data_dir).dimmed());

    println!();
    println!("{}", "─".repeat(50));
    println!(
        "{}",
        "⚠ IMPORTANT: Keep your password safe!".yellow().bold()
    );
    println!(
        "{}",
        "  There is no way to recover your wallet without it.".yellow()
    );
    println!("{}", "─".repeat(50));

    println!();
    println!("{}", "═══ Next Steps ═══".cyan());
    println!(
        "  {} Fund your Ethereum address with MNT (Mantle native token)",
        "1.".bold()
    );
    println!(
        "  {} Use '{}' to deposit into Veilocity",
        "2.".bold(),
        "veilocity deposit <amount>".green()
    );
    println!(
        "  {} Use '{}' to check your private balance",
        "3.".bold(),
        "veilocity balance".green()
    );

    info!("Wallet initialized successfully");

    Ok(())
}
