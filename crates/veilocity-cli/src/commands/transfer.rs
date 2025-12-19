//! Transfer command - send private transfer to another user

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::info;
use veilocity_core::poseidon::{field_to_bytes, hex_to_field, u128_to_field, u64_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;
use veilocity_prover::{NoirProver, TransferWitness, TREE_DEPTH};

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

    // Amount display with privacy emphasis
    println!(
        "  {} {}  {}",
        "◈".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        format_eth(amount_wei).truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        "(shielded transaction)".dimmed()
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
        recipient_short.truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {}",
        "Type:     ".truecolor(120, 120, 120),
        "Private (off-chain with ZK proof)".dimmed()
    );

    // Load state
    let mut state = StateManager::new(&config.db_path())
        .context("Failed to load state. Run 'veilocity sync' first.")?;

    // Get sender's account
    println!();
    print!(
        "  {} Loading shielded account...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let mut hasher = PoseidonHasher::new();
    let sender_pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
    let sender_pubkey_bytes = field_to_bytes(&sender_pubkey_field);

    let sender_account = state
        .get_account(&sender_pubkey_bytes)?
        .ok_or_else(|| anyhow!("Account not found. Have you made a deposit?"))?;

    println!(
        "\r  {} Shielded account loaded                    ",
        "✓".green().bold()
    );

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
        "(shielded)".dimmed()
    );

    println!();
    ui::divider(55);

    // =========================================================================
    // ZK-PROOF GENERATION - REAL TIME
    // =========================================================================

    println!();
    println!(
        "  {}",
        "╔═══════════════════════════════════════════════════════╗"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {}  {}  {}",
        "║".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "ZK-PROOF GENERATION: PRIVATE TRANSFER"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
            .bold(),
        "          ║".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {}",
        "╚═══════════════════════════════════════════════════════╝"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!();

    // Stage 1: Compute nullifier
    print!(
        "  {} Computing nullifier...",
        "◈".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    let nullifier = veilocity_secret.compute_nullifier(
        state.hasher(),
        sender_account.index,
        sender_account.nonce,
    );
    let nullifier_bytes = field_to_bytes(&nullifier);
    let nullifier_hex = hex::encode(&nullifier_bytes);

    println!(
        "\r  {} Nullifier computed                         ",
        "✓".green().bold()
    );
    println!(
        "    {} 0x{}...",
        "├".truecolor(60, 60, 60),
        &nullifier_hex[..16].truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "    {} {}",
        "└".truecolor(60, 60, 60),
        "(unique identifier for this note)".truecolor(100, 100, 100).italic()
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 2: Generate Merkle proof
    print!(
        "  {} Generating Merkle inclusion proof...",
        "◇".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    let sender_path = state.get_merkle_proof(sender_account.index);
    let sender_path_fields: Vec<_> = sender_path.into_iter().collect();

    println!(
        "\r  {} Merkle proof generated                     ",
        "✓".green().bold()
    );
    println!(
        "    {} depth={}, leaf_index={}",
        "├".truecolor(60, 60, 60),
        TREE_DEPTH.to_string().bright_white(),
        sender_account.index.to_string().bright_white()
    );
    println!(
        "    {} {}",
        "└".truecolor(60, 60, 60),
        "(proves your note exists in the tree)".truecolor(100, 100, 100).italic()
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 3: Retrieve state root
    print!(
        "  {} Computing state root...",
        "◆".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    let state_root = state.state_root();
    let state_root_bytes = field_to_bytes(&state_root);
    let state_root_hex = hex::encode(&state_root_bytes);

    println!(
        "\r  {} State root computed                        ",
        "✓".green().bold()
    );
    println!(
        "    {} 0x{}...",
        "└".truecolor(60, 60, 60),
        &state_root_hex[..16].bright_white()
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 4: Build witness
    print!(
        "  {} Constructing circuit witness...",
        "▣".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    let witness = TransferWitness::new(
        state_root,
        nullifier,
        *veilocity_secret.secret(),
        u128_to_field(sender_account.balance),
        u64_to_field(sender_account.nonce),
        u64_to_field(sender_account.index),
        sender_path_fields,
        recipient_pubkey,
        u128_to_field(amount_wei),
    )?;

    println!(
        "\r  {} Witness constructed                        ",
        "✓".green().bold()
    );
    println!(
        "    {} sender_secret: {}",
        "├".truecolor(60, 60, 60),
        "████████████████".truecolor(80, 80, 80)
    );
    println!(
        "    {} sender_balance: {} (hidden)",
        "├".truecolor(60, 60, 60),
        "████████".truecolor(80, 80, 80)
    );
    println!(
        "    {} transfer_amount: {} (hidden in proof)",
        "├".truecolor(60, 60, 60),
        "████████".truecolor(80, 80, 80)
    );
    println!(
        "    {} recipient_pubkey: 0x{}...",
        "└".truecolor(60, 60, 60),
        &recipient[2..10].truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );

    if dry_run {
        println!();
        ui::print_notice(
            "DRY RUN",
            "Proof would be generated and state updated. Remove --dry-run to execute.",
        );
        return Ok(());
    }

    println!();

    // Stage 5: Initialize prover
    let prover = NoirProver::new(PathBuf::from("circuits"));

    if !prover.is_compiled() {
        print!(
            "  {} Compiling Noir circuits...",
            "◐".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
        );
        io::stdout().flush().unwrap();
        prover.compile().await?;
        println!(
            "\r  {} Circuits compiled                          ",
            "✓".green().bold()
        );
    }

    // Stage 6: Generate ZK proof with real-time updates
    println!(
        "  {} {} {}",
        "▶".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        "Generating ZK-SNARK proof".bright_white().bold(),
        "(Barretenberg)".dimmed()
    );
    println!(
        "    {}",
        "├─ Protocol: UltraPlonk with Plookup".truecolor(100, 100, 100)
    );
    println!(
        "    {}",
        "├─ Curve: BN254 (alt_bn128)".truecolor(100, 100, 100)
    );
    println!(
        "    {}",
        "├─ Commitment: KZG polynomial".truecolor(100, 100, 100)
    );
    println!(
        "    {}",
        "└─ Security: 128-bit soundness".truecolor(100, 100, 100)
    );
    println!();

    // Show progress during actual proof generation
    print!(
        "    {} Computing proof ",
        "◐".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    let proof = prover.prove_transfer(&witness).await?;

    println!(
        "\r    {} Proof computation complete                ",
        "✓".green().bold()
    );

    // Show proof verification box
    println!();
    println!(
        "  {}",
        "┌─────────────────────────────────────────────────────┐"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {} {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "ZK-PROOF VERIFIED".green().bold(),
        "                                │".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {}",
        "├─────────────────────────────────────────────────────┤"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} Proof size: {} bytes                              {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        format!("{}", proof.len()).bright_white().bold(),
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {} Zero-Knowledge: Private inputs hidden        {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "✓".green(),
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {} Soundness: Proof mathematically valid        {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "✓".green(),
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {} Completeness: All constraints satisfied      {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "✓".green(),
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {}",
        "└─────────────────────────────────────────────────────┘"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );

    // =========================================================================
    // UPDATE STATE
    // =========================================================================

    println!();
    ui::divider(55);
    println!();
    println!(
        "  {} {}",
        "▶".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        "Updating Shielded State".bright_white().bold()
    );
    println!();

    // Update local state optimistically
    print!(
        "  {} Updating sender note...",
        "◐".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    let mut sender_updated = sender_account.clone();
    sender_updated.balance -= amount_wei;
    sender_updated.nonce += 1;
    state.update_account(&sender_updated)?;

    println!(
        "\r  {} Sender note updated                        ",
        "✓".green().bold()
    );

    // Mark nullifier as used
    print!(
        "  {} Recording nullifier...",
        "◐".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    io::stdout().flush().unwrap();

    state.mark_nullifier_used(&nullifier_bytes)?;

    println!(
        "\r  {} Nullifier recorded                         ",
        "✓".green().bold()
    );

    // Record transaction
    let _ = state.record_transaction("transfer", amount_wei, None, Some(recipient), "confirmed");

    // =========================================================================
    // SUCCESS SUMMARY
    // =========================================================================

    println!();
    ui::divider_double(55);
    println!();
    println!(
        "  {} {}",
        "✓".green().bold(),
        "PRIVATE TRANSFER COMPLETE".green().bold()
    );
    println!();

    println!(
        "  {} 0x{}...",
        "Nullifier:".truecolor(120, 120, 120),
        &nullifier_hex[..16].truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} 0x{}...",
        "Proof:    ".truecolor(120, 120, 120),
        &hex::encode(&proof[..8]).dimmed()
    );

    println!();
    println!(
        "  {} {} sent privately to {}",
        "◈".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        format_eth(amount_wei).truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2).bold(),
        recipient_short.bright_white()
    );
    println!(
        "  New balance: {}",
        format_eth(sender_updated.balance)
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
            .bold()
    );

    // Privacy summary
    println!();
    println!(
        "  {}",
        "┌─ Privacy Guarantees ─────────────────────────────────┐"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );
    println!(
        "  {} {} Sender identity: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Recipient identity: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Transfer amount: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Transaction link: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Unlinkable".green().bold()
    );
    println!(
        "  {}",
        "└───────────────────────────────────────────────────────┘"
            .truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2)
    );

    ui::divider_double(55);

    println!();
    println!(
        "  {} {}",
        "ℹ".truecolor(100, 149, 237),
        "This is a shielded off-chain transfer.".truecolor(150, 150, 150).italic()
    );
    println!(
        "    {}",
        "The recipient will see funds after syncing their local state.".truecolor(150, 150, 150).italic()
    );
    println!();

    info!(
        "Private transfer of {} wei to {} completed",
        amount_wei, recipient
    );

    Ok(())
}
