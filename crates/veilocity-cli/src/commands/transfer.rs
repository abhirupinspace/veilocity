//! Transfer command - send private transfer to another user

use crate::config::Config;
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
    let password = rpassword::prompt_password("Enter wallet password: ")
        .context("Failed to read password")?;

    // Get Veilocity secret
    let veilocity_secret = wallet_manager.get_veilocity_secret(&wallet, &password)?;

    // Parse recipient public key
    let recipient_pubkey = hex_to_field(recipient)
        .context("Invalid recipient public key. Expected hex string (0x...).")?;

    // Parse amount
    let amount_wei = parse_eth(amount);

    println!();
    println!("{}", "═══ Private Transfer ═══".blue().bold());
    println!(
        "Amount:    {} {}",
        format_eth(amount_wei).bright_white().bold(),
        format!("({} wei)", amount_wei).dimmed()
    );
    println!(
        "Recipient: 0x{}...{}",
        &recipient[..8.min(recipient.len())].bright_white(),
        if recipient.len() > 8 {
            &recipient[recipient.len() - 6..]
        } else {
            ""
        }
        .bright_white()
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
        "Balance:   {} {}",
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
        println!("{}", "DRY RUN - No transfer executed".yellow().bold());
        println!(
            "{}",
            "Proof would be generated and state would be updated.".dimmed()
        );
        println!("{}", "Remove --dry-run to execute the transfer.".dimmed());
        return Ok(());
    }

    println!();
    println!("{}", "Generating ZK proof...".yellow());
    println!("{}", "(this may take a moment)".dimmed());

    // Generate proof
    let prover = NoirProver::new(PathBuf::from("circuits"));

    if !prover.is_compiled() {
        println!("{}", "Compiling circuits...".yellow());
        prover.compile().await?;
    }

    let proof = prover.prove_transfer(&witness).await?;

    println!(
        "{}",
        format!("✓ Proof generated ({} bytes)", proof.len()).green()
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

    println!();
    println!("{}", "✓ Transfer complete!".green().bold());
    println!(
        "Nullifier: 0x{}...",
        &hex::encode(&nullifier_bytes)[..16].dimmed()
    );

    println!();
    println!("{}", "─".repeat(50));
    println!(
        "{} sent privately to recipient",
        format_eth(amount_wei).blue().bold()
    );
    println!(
        "New balance: {}",
        format_eth(sender_updated.balance).bright_white()
    );

    println!();
    println!(
        "{}",
        "Note: This is an off-chain transfer. The recipient".dimmed()
    );
    println!(
        "{}",
        "will see the funds after syncing their local state.".dimmed()
    );

    info!(
        "Private transfer of {} wei to {} completed",
        amount_wei, recipient
    );

    Ok(())
}
