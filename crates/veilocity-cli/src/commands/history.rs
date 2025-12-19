//! History command - show transaction history

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_eth, WalletManager};
use anyhow::{anyhow, Result};
use colored::Colorize;
use veilocity_core::state::StateManager;

/// Run the history command
pub async fn run(config: &Config) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Check wallet exists
    if !wallet_manager.wallet_exists() {
        return Err(anyhow!("Wallet not found. Run 'veilocity init' first."));
    }

    // Load wallet
    let wallet = wallet_manager.load_wallet()?;

    println!();
    println!("{}", ui::header("Transaction History"));
    println!();
    println!(
        "  {} {}",
        "Wallet:".truecolor(120, 120, 120),
        wallet.address.bright_white()
    );
    println!(
        "  {} {}...",
        "PubKey:".truecolor(120, 120, 120),
        &wallet.veilocity_pubkey[..24].truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );

    ui::divider(65);

    // Try to load state and get transactions
    let db_path = config.db_path();
    if !db_path.exists() {
        println!();
        println!(
            "  {}",
            "(No transactions found)".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
        );
        println!(
            "  {}",
            "Run 'veilocity sync' to synchronize with the network.".dimmed()
        );
        print_help();
        return Ok(());
    }

    let state = StateManager::new(&db_path)?;
    let transactions = state.get_transactions(50)?;

    if transactions.is_empty() {
        println!();
        println!(
            "  {}",
            "(No transactions found)".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
        );
        println!(
            "  {}",
            "Make deposits, transfers, or withdrawals to see them here.".dimmed()
        );
        print_help();
        return Ok(());
    }

    // Print header
    println!();
    println!(
        "  {:<12} {:<18} {:<12} {:<20}",
        "Type".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        "Amount".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        "Status".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        "Time".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold()
    );
    ui::divider(65);

    // Print transactions
    for tx in transactions {
        let tx_type = match tx.tx_type.as_str() {
            "deposit" => "DEPOSIT".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
            "withdraw" => "WITHDRAW".red(),
            "transfer" => "TRANSFER".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
            _ => tx.tx_type.normal(),
        };

        let amount = tx
            .amount()
            .map(format_eth)
            .unwrap_or_else(|| "N/A".to_string());

        let status = match tx.status.as_str() {
            "confirmed" => "confirmed".green(),
            "pending" => "pending".yellow(),
            "failed" => "failed".red(),
            _ => tx.status.normal(),
        };

        let time = format_timestamp(tx.created_at);

        println!(
            "  {:<12} {:<18} {:<12} {:<20}",
            tx_type, amount, status, time
        );

        // Show tx hash if available
        if let Some(hash) = tx.tx_hash() {
            println!(
                "    {} 0x{}...",
                "→".truecolor(80, 80, 80),
                &hash[..16].dimmed()
            );
        }
    }

    ui::divider(65);
    print_help();

    Ok(())
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let tx_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();

    if let Ok(duration) = now.duration_since(tx_time) {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    } else {
        "just now".to_string()
    }
}

fn print_help() {
    println!();
    println!("{}", ui::header("Quick Commands"));
    println!();
    println!(
        "  {} {} {}",
        ui::command("veilocity deposit"),
        "<amount>".dimmed(),
        "- Deposit funds".truecolor(120, 120, 120)
    );
    println!(
        "  {} {} {} {}",
        ui::command("veilocity transfer"),
        "<recipient>".dimmed(),
        "<amount>".dimmed(),
        "- Private transfer".truecolor(120, 120, 120)
    );
    println!(
        "  {} {} {}",
        ui::command("veilocity withdraw"),
        "<amount>".dimmed(),
        "- Withdraw funds".truecolor(120, 120, 120)
    );
    println!(
        "  {} {}",
        ui::command("veilocity balance"),
        "- Check balance".truecolor(120, 120, 120)
    );
    println!(
        "  {} {}",
        ui::command("veilocity sync"),
        "- Sync with network".truecolor(120, 120, 120)
    );
    println!();
}
