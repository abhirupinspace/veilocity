//! Balance command - display current private balance

use crate::config::Config;
use crate::wallet::{format_eth, WalletManager};
use anyhow::{Context, Result};
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

    // Try to load state
    let db_path = config.db_path();
    if !db_path.exists() {
        println!("\n=== Veilocity Balance ===");
        println!("Private:  0.000000 ETH");
        println!("\nNote: No local state found. Run 'veilocity sync' to sync with the network.");
        return Ok(());
    }

    let state = StateManager::new(&db_path)
        .context("Failed to load state")?;

    // Get account
    let mut hasher = PoseidonHasher::new();
    let pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
    let pubkey_bytes = field_to_bytes(&pubkey_field);

    let account = state.get_account(&pubkey_bytes)?;

    println!("\n=== Veilocity Balance ===");
    println!("────────────────────────");

    if let Some(account) = account {
        println!("Private:  {}", format_eth(account.balance));
        println!("\nAccount Details:");
        println!("  Leaf Index: {}", account.index);
        println!("  Nonce:      {}", account.nonce);
    } else {
        println!("Private:  0.000000 ETH");
        println!("\nNo deposits found for this wallet.");
    }

    // Show state info
    println!("\n=== State Info ===");
    println!("State Root: 0x{}", hex::encode(field_to_bytes(&state.state_root())));
    println!("Total Leaves: {}", state.leaf_count());

    // Check if vault is configured
    if !config.network.vault_address.is_empty() {
        // Optionally show on-chain balance
        // This would require an RPC call
    }

    println!("\nLast synced: (run 'veilocity sync' to update)");

    Ok(())
}
