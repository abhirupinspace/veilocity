//! Deposit command - deposit funds from Mantle into Veilocity

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_mnt, parse_mnt, WalletManager};
use alloy::primitives::{B256, U256};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{self, Write};
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
    let password = rpassword::prompt_password(format!(
        "{} ",
        "Enter wallet password:".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    ))
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
    let amount_wei = parse_mnt(amount);
    let amount_u256 = U256::from(amount_wei);

    println!();
    println!("{}", ui::header("Shield Deposit"));
    println!();

    // Amount display
    println!(
        "  {} {}  {}",
        "↓".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        format_mnt(amount_wei).truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        "(entering shielded pool)".dimmed()
    );
    println!();

    ui::divider(55);
    println!();
    println!(
        "  {} {}",
        "From:   ".truecolor(120, 120, 120),
        wallet.address.bright_white()
    );
    println!(
        "  {} {}",
        "To:     ".truecolor(120, 120, 120),
        "Veilocity Shielded Pool".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {} {}",
        "Network:".truecolor(120, 120, 120),
        config.network.rpc_url.dimmed()
    );

    println!();
    ui::divider(55);

    // =========================================================================
    // CRYPTOGRAPHIC COMMITMENT GENERATION
    // =========================================================================

    println!();
    println!(
        "  {}",
        "╔═══════════════════════════════════════════════════════╗"
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {}  {}  {}",
        "║".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        "CRYPTOGRAPHIC COMMITMENT GENERATION"
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
            .bold(),
        "            ║".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {}",
        "╚═══════════════════════════════════════════════════════╝"
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!();

    // Stage 1: Derive public key
    print!(
        "  {} Deriving shielded public key...",
        "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let mut hasher = PoseidonHasher::new();
    let pubkey = veilocity_secret.derive_pubkey(&mut hasher);
    let pubkey_bytes = field_to_bytes(&pubkey);
    let pubkey_hex = hex::encode(&pubkey_bytes);

    println!(
        "\r  {} Public key derived                         ",
        "✓".green().bold()
    );
    println!(
        "    {} 0x{}...",
        "└".truecolor(60, 60, 60),
        &pubkey_hex[..16].truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 2: Compute Poseidon hash commitment
    print!(
        "  {} Computing Poseidon commitment...",
        "◇".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let commitment = veilocity_secret.compute_deposit_commitment(&mut hasher, amount_wei);
    let commitment_bytes = field_to_bytes(&commitment);
    let commitment_b256 = B256::from(commitment_bytes);
    let commitment_hex = hex::encode(&commitment_bytes);

    println!(
        "\r  {} Commitment computed                        ",
        "✓".green().bold()
    );
    println!(
        "    {} Hash: Poseidon(secret, amount, nonce)",
        "├".truecolor(60, 60, 60),
    );
    println!(
        "    {} 0x{}...",
        "└".truecolor(60, 60, 60),
        &commitment_hex[..16].bright_white()
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 3: Show commitment details
    println!(
        "  {} {}",
        "◆".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        "Commitment Structure".truecolor(200, 200, 200)
    );
    println!(
        "    {} secret_key: {}",
        "├".truecolor(60, 60, 60),
        "████████████████".truecolor(80, 80, 80)
    );
    println!(
        "    {} amount: {} wei",
        "├".truecolor(60, 60, 60),
        amount_wei.to_string().bright_white()
    );
    println!(
        "    {} {}",
        "└".truecolor(60, 60, 60),
        "(only you can spend this note)".truecolor(100, 100, 100).italic()
    );

    if dry_run {
        println!();
        ui::print_notice(
            "DRY RUN",
            "No transaction submitted. Remove --dry-run to execute.",
        );
        return Ok(());
    }

    // =========================================================================
    // ON-CHAIN TRANSACTION
    // =========================================================================

    println!();
    ui::divider(55);
    println!();
    println!(
        "  {} {}",
        "▶".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        "Submitting On-Chain Transaction".bright_white().bold()
    );
    println!();

    // Create vault client
    print!(
        "  {} Connecting to Mantle network...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer).await?;

    println!(
        "\r  {} Connected to Mantle                        ",
        "✓".green().bold()
    );

    // Submit deposit
    print!(
        "  {} Submitting deposit transaction...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let tx_hash = vault.deposit(commitment_b256, amount_u256).await?;
    let tx_hash_hex = hex::encode(tx_hash);

    println!(
        "\r  {} Transaction confirmed!                     ",
        "✓".green().bold()
    );

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

    // =========================================================================
    // SUCCESS SUMMARY
    // =========================================================================

    println!();
    ui::divider_double(55);
    println!();
    println!(
        "  {} {}",
        "✓".green().bold(),
        "DEPOSIT SUCCESSFUL".green().bold()
    );
    println!();

    println!(
        "  {} 0x{}",
        "Transaction:".truecolor(120, 120, 120),
        tx_hash_hex.bright_white()
    );
    println!(
        "  {} 0x{}...",
        "Commitment:".truecolor(120, 120, 120),
        &commitment_hex[..16].dimmed()
    );

    if let Some(explorer) = &config.network.explorer_url {
        println!(
            "  {} {}",
            "Explorer:  ".truecolor(120, 120, 120),
            format!("{}/tx/0x{}", explorer, tx_hash_hex)
                .truecolor(100, 149, 237)
                .underline()
        );
    }

    println!();
    println!(
        "  {} {} is now in the shielded pool",
        "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        format_mnt(amount_wei).truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold()
    );

    // Privacy summary
    println!();
    println!(
        "  {}",
        "┌─ Privacy Status ─────────────────────────────────────┐"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {} Funds status: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Shielded".green().bold()
    );
    println!(
        "  {} {} Spendable by: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Only you".green().bold()
    );
    println!(
        "  {} {} Amount visible: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "On-chain (deposit only)".yellow()
    );
    println!(
        "  {} {} Future txns: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Fully private".green().bold()
    );
    println!(
        "  {}",
        "└───────────────────────────────────────────────────────┘"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );

    ui::divider_double(55);

    println!();
    println!(
        "  {} Run '{}' to update your local state.",
        "ℹ".truecolor(100, 149, 237),
        "veilocity sync".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!();

    info!("Deposit of {} wei completed", amount_wei);

    Ok(())
}
