//! Balance command - display current private balance

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_mnt, WalletManager};
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
    let password = rpassword::prompt_password(format!(
        "{} ",
        "Enter wallet password:".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    ))
    .context("Failed to read password")?;

    // Get Veilocity secret
    let veilocity_secret = wallet_manager.get_veilocity_secret(&wallet, &password)?;

    println!();
    println!("{}", ui::header("Balance"));

    // Try to load state
    let db_path = config.db_path();
    if !db_path.exists() {
        println!();
        println!(
            "  {} {}",
            "Private Balance:".truecolor(150, 150, 150),
            "0.000000 MNT".bright_white().bold()
        );
        println!();
        ui::print_notice(
            "No Local State",
            &format!("Run '{}' to sync with the network.", ui::command("veilocity sync")),
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
        // Main balance display
        let balance_str = format_mnt(account.balance);
        println!(
            "  {} {}",
            "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
            "Private Balance".truecolor(150, 150, 150)
        );
        println!();
        println!(
            "    {}",
            balance_str.truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold()
        );
        println!();

        ui::divider(45);
        println!();
        println!(
            "  {} {}",
            "Leaf Index:".truecolor(120, 120, 120),
            account.index.to_string().bright_white()
        );
        println!(
            "  {} {}",
            "Nonce:     ".truecolor(120, 120, 120),
            account.nonce.to_string().bright_white()
        );
    } else {
        println!(
            "  {} {}",
            "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
            "Private Balance".truecolor(150, 150, 150)
        );
        println!();
        println!("    {}", "0.000000 MNT".bright_white().bold());
        println!();
        println!(
            "  {}",
            "No deposits found for this wallet.".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
        );
    }

    // Show state info
    println!();
    println!("{}", ui::header("State Info"));
    println!();
    println!(
        "  {} 0x{}...",
        "State Root:  ".truecolor(120, 120, 120),
        &hex::encode(field_to_bytes(&state.state_root()))[..16].dimmed()
    );
    println!(
        "  {} {}",
        "Total Leaves:".truecolor(120, 120, 120),
        state.leaf_count().to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Network:     ".truecolor(120, 120, 120),
        config.network.rpc_url.dimmed()
    );

    println!();
    ui::divider(45);
    println!(
        "  Run '{}' to update your local state.",
        ui::command("veilocity sync")
    );
    println!();

    Ok(())
}
