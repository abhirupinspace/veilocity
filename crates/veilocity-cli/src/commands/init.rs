//! Initialize command - create a new Veilocity wallet

use crate::config::{get_data_dir, Config};
use crate::ui;
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

    // Print logo
    println!();
    ui::print_logo();
    println!(
        "{}",
        "  Private Execution Layer for Mantle"
            .truecolor(180, 180, 180)
            .italic()
    );
    println!();

    // Check if wallet already exists
    if wallet_manager.wallet_exists() {
        ui::print_notice(
            "Wallet Already Exists",
            &format!("Location: {:?}", wallet_manager.wallet_path()),
        );
        println!();
        println!(
            "{}",
            "To create a new wallet, delete the existing one first.".dimmed()
        );
        return Ok(());
    }

    if recover {
        ui::print_notice(
            "Recovery Not Available",
            "Seed phrase recovery is not yet implemented.",
        );
        println!("{}", "Please create a new wallet for now.".dimmed());
        return Ok(());
    }

    println!("{}", ui::header("Wallet Setup"));
    println!();
    println!("{}", "Password Requirements:".bold());
    println!(
        "  {} At least 8 characters",
        "•".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {} Mix of uppercase and lowercase letters",
        "•".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {} At least one number",
        "•".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!();

    // Get password from user
    let password = rpassword::prompt_password(format!(
        "{} ",
        "Enter password for new wallet:".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    ))
    .context("Failed to read password")?;

    // Validate password strength
    if let Err(e) = validate_password(&password) {
        println!();
        println!("{} {}", ui::error("Error:"), e);
        return Ok(());
    }

    let password_confirm = rpassword::prompt_password(format!(
        "{} ",
        "Confirm password:".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    ))
    .context("Failed to read password confirmation")?;

    if password != password_confirm {
        println!();
        println!("{}", ui::error("✗ Passwords do not match!"));
        return Ok(());
    }

    println!();
    println!(
        "{} Creating new Veilocity wallet...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );

    // Generate wallet
    let (wallet, _signer, _secret) = wallet_manager.generate(&password)?;

    // Save wallet
    wallet_manager.save_wallet(&wallet)?;

    // Save config
    config.save()?;

    ui::print_success("Wallet created successfully!");
    println!();
    println!("{}", ui::header("Wallet Information"));
    println!();
    println!(
        "  {} {}",
        "Ethereum Address:    ".truecolor(150, 150, 150),
        ui::value(&wallet.address)
    );
    println!(
        "  {} {}",
        "Veilocity Public Key:".truecolor(150, 150, 150),
        ui::orange(&wallet.veilocity_pubkey)
    );
    println!(
        "  {} {}",
        "Data Directory:      ".truecolor(150, 150, 150),
        format!("{:?}", data_dir).dimmed()
    );

    println!();
    ui::divider_double(54);
    println!(
        "  {} {}",
        "⚠".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        "IMPORTANT: Keep your password safe!".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold()
    );
    println!(
        "    {}",
        "There is no way to recover your wallet without it.".dimmed()
    );
    ui::divider_double(54);

    println!();
    println!("{}", ui::header("Next Steps"));
    println!();
    println!(
        "  {}",
        ui::step(1, "Fund your Ethereum address with MNT (Mantle native token)")
    );
    println!(
        "  {}",
        ui::step(
            2,
            &format!("Use '{}' to deposit into Veilocity", ui::command("veilocity deposit <amount>"))
        )
    );
    println!(
        "  {}",
        ui::step(
            3,
            &format!("Use '{}' to check your private balance", ui::command("veilocity balance"))
        )
    );
    println!();

    info!("Wallet initialized successfully");

    Ok(())
}
