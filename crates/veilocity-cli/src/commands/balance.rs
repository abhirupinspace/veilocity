//! Balance command - display current private balance

use crate::config::Config;
use crate::wallet::{format_eth, WalletManager};
use anyhow::{Context, Result};
use colored::Colorize;
use veilocity_core::poseidon::{field_to_bytes, PoseidonHasher};
use veilocity_core::state::StateManager;

/// Run the balance command
pub async fn run(config: &Config) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Load wallet
    let wallet = wallet_manager.load_wallet()?;

    // Get password
    let password = rpassword::prompt_password("Enter wallet password: ")
        .context("Failed to read password")?;

    // Get Veilocity secret
    let veilocity_secret = wallet_manager.get_veilocity_secret(&wallet, &password)?;

    println!();
    println!("{}", "═══ Veilocity Balance ═══".cyan().bold());

    // Try to load state
    let db_path = config.db_path();
    if !db_path.exists() {
        println!();
        println!(
            "Private Balance: {}",
            "0.000000 ETH".bright_white().bold()
        );
        println!();
        println!(
            "{}",
            "Note: No local state found.".yellow()
        );
        println!(
            "Run '{}' to sync with the network.",
            "veilocity sync".cyan()
        );
        return Ok(());
    }

    let state = StateManager::new(&db_path).context("Failed to load state")?;

    // Get account
    let mut hasher = PoseidonHasher::new();
    let pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
    let pubkey_bytes = field_to_bytes(&pubkey_field);

    let account = state.get_account(&pubkey_bytes)?;

    println!();
    if let Some(account) = account {
        println!(
            "Private Balance: {}",
            format_eth(account.balance).green().bold()
        );
        println!();
        println!("{}", "Account Details".dimmed());
        println!("  Leaf Index: {}", account.index.to_string().bright_white());
        println!("  Nonce:      {}", account.nonce.to_string().bright_white());
    } else {
        println!(
            "Private Balance: {}",
            "0.000000 ETH".bright_white().bold()
        );
        println!();
        println!("{}", "No deposits found for this wallet.".yellow());
    }

    // Show state info
    println!();
    println!("{}", "═══ State Info ═══".cyan());
    println!(
        "State Root:   0x{}...",
        &hex::encode(field_to_bytes(&state.state_root()))[..16].dimmed()
    );
    println!("Total Leaves: {}", state.leaf_count().to_string().bright_white());
    println!("Network:      {}", config.network.rpc_url.dimmed());

    println!();
    println!("{}", "─".repeat(40));
    println!(
        "Run '{}' to update your local state.",
        "veilocity sync".cyan()
    );

    Ok(())
}
