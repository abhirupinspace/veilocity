//! History command - show transaction history

use crate::config::Config;
use crate::wallet::WalletManager;
use anyhow::{anyhow, Result};

/// Run the history command
pub async fn run(config: &Config) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Check wallet exists
    if !wallet_manager.wallet_exists() {
        return Err(anyhow!("Wallet not found. Run 'veilocity init' first."));
    }

    // Load wallet
    let wallet = wallet_manager.load_wallet()?;

    println!("\n=== Transaction History ===");
    println!("Wallet: {}", wallet.address);
    println!("Veilocity PubKey: {}...", &wallet.veilocity_pubkey[..20]);
    println!("────────────────────────────────────────────────\n");

    // In a full implementation, we would:
    // 1. Query local database for stored transactions
    // 2. Query on-chain events for deposits/withdrawals
    // 3. Display formatted transaction history

    println!("Type        | Amount          | Status      | Time");
    println!("────────────|─────────────────|─────────────|────────────");

    // For now, show placeholder
    println!("(No transactions found)");
    println!("\nNote: Transaction history is stored locally.");
    println!("Make deposits, transfers, or withdrawals to see them here.");

    // Show tips
    println!("\n=== Available Commands ===");
    println!("  veilocity deposit <amount>              - Deposit funds");
    println!("  veilocity transfer <recipient> <amount> - Private transfer");
    println!("  veilocity withdraw <amount>             - Withdraw funds");
    println!("  veilocity balance                       - Check balance");
    println!("  veilocity sync                          - Sync with network");

    Ok(())
}
