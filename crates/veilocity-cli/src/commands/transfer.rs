//! Transfer command - send private transfer to another user

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use tracing::info;
use veilocity_core::poseidon::{hex_to_field, u128_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;
use veilocity_prover::{NoirProver, TransferWitness};

/// Run the transfer command
pub async fn run(config: &Config, recipient: &str, amount: f64, dry_run: bool) -> Result<()> {
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

    // Parse recipient public key
    let recipient_pubkey = hex_to_field(recipient)
        .context("Invalid recipient public key. Expected hex string (0x...).")?;

    // Parse amount
    let amount_wei = parse_eth(amount);

    println!();
    println!("{}", ui::header("Private Transfer"));
    println!();

    // Amount display
    println!(
        "  {} {}  {}",
        "↗".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        format_eth(amount_wei).truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        format!("({} wei)", amount_wei).dimmed()
    );
    println!();

    ui::divider(55);
    println!();

    // Format recipient for display
    let recipient_short = if recipient.len() > 20 {
        format!("0x{}...{}", &recipient[2..10], &recipient[recipient.len() - 6..])
    } else {
        recipient.to_string()
    };

    println!(
        "  {} {}",
        "Recipient:".truecolor(120, 120, 120),
        recipient_short.bright_white()
    );

    // Load state
    let mut state = StateManager::new(&config.db_path())
        .context("Failed to load state. Run 'veilocity sync' first.")?;

    // Get sender's account
    let mut hasher = PoseidonHasher::new();
    let sender_pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
    let sender_pubkey_bytes = veilocity_core::poseidon::field_to_bytes(&sender_pubkey_field);

    let sender_account = state
        .get_account(&sender_pubkey_bytes)?
        .ok_or_else(|| anyhow!("Account not found. Have you made a deposit?"))?;

    // Check balance
    if sender_account.balance < amount_wei {
        return Err(anyhow!(
            "Insufficient balance. Have: {}, Need: {}",
            format_eth(sender_account.balance),
            format_eth(amount_wei)
        ));
    }

    println!(
        "  {} {} {}",
        "Balance:  ".truecolor(120, 120, 120),
        format_eth(sender_account.balance).green(),
        "(private)".dimmed()
    );

    // Get Merkle proof for sender
    let sender_path = state.get_merkle_proof(sender_account.index);
    let sender_path_fields: Vec<_> = sender_path.into_iter().collect();

    // Compute nullifier
    let nullifier = veilocity_secret.compute_nullifier(
        state.hasher(),
        sender_account.index,
        sender_account.nonce,
    );

    // Create witness
    let witness = TransferWitness::new(
        state.state_root(),
        nullifier,
        *veilocity_secret.secret(),
        u128_to_field(sender_account.balance),
        veilocity_core::poseidon::u64_to_field(sender_account.nonce),
        veilocity_core::poseidon::u64_to_field(sender_account.index),
        sender_path_fields,
        recipient_pubkey,
        u128_to_field(amount_wei),
    )?;

    if dry_run {
        println!();
        ui::print_notice(
            "DRY RUN",
            "Proof would be generated and state updated. Remove --dry-run to execute.",
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Generating ZK proof...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!("    {}", "(this may take a moment)".dimmed());

    // Generate proof
    let prover = NoirProver::new(PathBuf::from("circuits"));

    if !prover.is_compiled() {
        println!(
            "  {} Compiling circuits...",
            "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
        );
        prover.compile().await?;
    }

    let proof = prover.prove_transfer(&witness).await?;

    println!(
        "  {} Proof generated ({} bytes)",
        "✓".green().bold(),
        proof.len()
    );

    // Update local state
    // In a full implementation, this would:
    // 1. Submit the proof to a sequencer/relayer
    // 2. Wait for state root update on-chain
    // 3. Update local state accordingly

    // For MVP, we update local state optimistically
    let mut sender_updated = sender_account.clone();
    sender_updated.balance -= amount_wei;
    sender_updated.nonce += 1;
    state.update_account(&sender_updated)?;

    // Mark nullifier as used
    let nullifier_bytes = veilocity_core::poseidon::field_to_bytes(&nullifier);
    state.mark_nullifier_used(&nullifier_bytes)?;

    // Record transaction
    let _ = state.record_transaction("transfer", amount_wei, None, Some(recipient), "confirmed");

    ui::print_success("Transfer complete!");
    println!();
    println!(
        "  {} 0x{}...",
        "Nullifier:".truecolor(120, 120, 120),
        &hex::encode(&nullifier_bytes)[..16].dimmed()
    );

    println!();
    ui::divider_double(55);
    println!(
        "  {} {} sent privately",
        "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        format_eth(amount_wei).truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold()
    );
    println!(
        "  New balance: {}",
        format_eth(sender_updated.balance)
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
            .bold()
    );
    ui::divider_double(55);

    println!();
    println!(
        "  {}",
        "Note: This is an off-chain transfer.".truecolor(150, 150, 150).italic()
    );
    println!(
        "  {}",
        "The recipient will see funds after syncing their local state.".truecolor(150, 150, 150).italic()
    );
    println!();

    info!(
        "Private transfer of {} wei to {} completed",
        amount_wei, recipient
    );

    Ok(())
}
