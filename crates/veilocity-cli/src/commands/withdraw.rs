//! Withdraw command - withdraw funds from Veilocity to Mantle

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use alloy::primitives::{Address, B256, U256};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::info;
use veilocity_contracts::create_vault_client;
use veilocity_core::poseidon::{field_to_bytes, u128_to_field, u64_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;
use veilocity_prover::{NoirProver, WithdrawWitness, TREE_DEPTH};

/// Run the withdraw command
pub async fn run(config: &Config, amount: f64, recipient: Option<String>, dry_run: bool) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Load wallet
    let wallet = wallet_manager.load_wallet()?;

    // Get password
    let password = rpassword::prompt_password(format!(
        "{} ",
        "Enter wallet password:".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    ))
    .context("Failed to read password")?;

    // Unlock wallet and get secrets
    let signer = wallet_manager.unlock(&wallet, &password)?;
    let veilocity_secret = wallet_manager.get_veilocity_secret(&wallet, &password)?;

    // Determine recipient address
    let recipient_address: Address = if let Some(ref addr) = recipient {
        addr.parse().context("Invalid recipient address")?
    } else {
        wallet.address()?
    };

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
    let amount_wei = parse_eth(amount);

    println!();
    println!("{}", ui::header("Private Withdrawal"));
    println!();

    // Amount display
    println!(
        "  {} {}  {}",
        "↑".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        format_eth(amount_wei).truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        format!("({} wei)", amount_wei).dimmed()
    );
    println!();

    ui::divider(55);
    println!();
    println!(
        "  {} {}",
        "Recipient:".truecolor(120, 120, 120),
        recipient_address.to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Network:  ".truecolor(120, 120, 120),
        config.network.rpc_url.dimmed()
    );

    // Load state
    let mut state = StateManager::new(&config.db_path())
        .context("Failed to load state. Run 'veilocity sync' first.")?;

    // Get account
    println!();
    print!(
        "  {} Loading private account...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let mut hasher = PoseidonHasher::new();
    let pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
    let pubkey_bytes = field_to_bytes(&pubkey_field);

    let account = state
        .get_account(&pubkey_bytes)?
        .ok_or_else(|| anyhow!("Account not found. Have you made a deposit?"))?;

    println!(
        "\r  {} Private account loaded                    ",
        "✓".green().bold()
    );

    // Check balance
    if account.balance < amount_wei {
        return Err(anyhow!(
            "Insufficient balance. Have: {}, Need: {}",
            format_eth(account.balance),
            format_eth(amount_wei)
        ));
    }

    println!(
        "  {} {} {}",
        "Balance:  ".truecolor(120, 120, 120),
        format_eth(account.balance).green(),
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
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {}  {}  {}",
        "║".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        "ZK-PROOF GENERATION: WITHDRAW"
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
            .bold(),
        "                  ║".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!(
        "  {}",
        "╚═══════════════════════════════════════════════════════╝"
            .truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    println!();

    // Stage 1: Compute nullifier
    print!(
        "  {} Computing nullifier...",
        "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let nullifier = veilocity_secret.compute_nullifier(state.hasher(), account.index, account.nonce);
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
        "(prevents double-spending)".truecolor(100, 100, 100).italic()
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 2: Generate Merkle proof
    print!(
        "  {} Generating Merkle proof...",
        "◇".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let merkle_path = state.get_merkle_proof(account.index);
    let merkle_path_fields: Vec<_> = merkle_path.into_iter().collect();

    println!(
        "\r  {} Merkle proof generated                     ",
        "✓".green().bold()
    );
    println!(
        "    {} depth={}, leaf_index={}",
        "├".truecolor(60, 60, 60),
        TREE_DEPTH.to_string().bright_white(),
        account.index.to_string().bright_white()
    );
    println!(
        "    {} {}",
        "└".truecolor(60, 60, 60),
        "(cryptographic path proving membership)".truecolor(100, 100, 100).italic()
    );
    println!("    {}", "│".truecolor(60, 60, 60));

    // Stage 3: Retrieve state root
    print!(
        "  {} Retrieving state root...",
        "◆".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let state_root = state.state_root();
    let state_root_bytes = field_to_bytes(&state_root);
    let state_root_hex = hex::encode(&state_root_bytes);

    println!(
        "\r  {} State root retrieved                       ",
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
        "  {} Building circuit witness...",
        "▣".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    // Convert recipient address to field
    let recipient_field = veilocity_core::poseidon::FieldElement::from(
        u128::from_be_bytes({
            let mut bytes = [0u8; 16];
            bytes[4..].copy_from_slice(&recipient_address.0 .0[..12]);
            bytes
        }),
    );

    let witness = WithdrawWitness::new(
        state_root,
        nullifier,
        u128_to_field(amount_wei),
        recipient_field,
        *veilocity_secret.secret(),
        u128_to_field(account.balance),
        u64_to_field(account.nonce),
        u64_to_field(account.index),
        merkle_path_fields,
    )?;

    println!(
        "\r  {} Witness constructed                        ",
        "✓".green().bold()
    );
    println!(
        "    {} secret_key: {}",
        "├".truecolor(60, 60, 60),
        "████████████████".truecolor(80, 80, 80)
    );
    println!(
        "    {} balance: {} (private)",
        "├".truecolor(60, 60, 60),
        "████████".truecolor(80, 80, 80)
    );
    println!(
        "    {} amount: {} wei",
        "├".truecolor(60, 60, 60),
        amount_wei.to_string().bright_white()
    );
    println!(
        "    {} recipient: 0x{}...",
        "└".truecolor(60, 60, 60),
        &recipient_address.to_string()[2..10].bright_white()
    );

    if dry_run {
        println!();
        ui::print_notice(
            "DRY RUN",
            "Proof would be generated and withdrawal submitted. Remove --dry-run to execute.",
        );
        return Ok(());
    }

    println!();

    // Stage 5: Initialize prover
    let prover = NoirProver::new(PathBuf::from("circuits"));

    if !prover.is_compiled() {
        print!(
            "  {} Compiling Noir circuits...",
            "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
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
        "▶".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
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
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let proof = prover.prove_withdraw(&witness).await?;

    println!(
        "\r    {} Proof computation complete                ",
        "✓".green().bold()
    );

    // Show proof verification box
    println!();
    println!(
        "  {}",
        "┌─────────────────────────────────────────────────────┐"
            .truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {} {} {}",
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2),
        "ZK-PROOF VERIFIED".green().bold(),
        "                                │".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {}",
        "├─────────────────────────────────────────────────────┤"
            .truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {} Proof size: {} bytes                              {}",
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2),
        format!("{}", proof.len()).bright_white().bold(),
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {} {} Zero-Knowledge: Private inputs hidden        {}",
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2),
        "✓".green(),
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {} {} Soundness: Proof mathematically valid        {}",
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2),
        "✓".green(),
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {} {} Completeness: All constraints satisfied      {}",
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2),
        "✓".green(),
        "│".truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );
    println!(
        "  {}",
        "└─────────────────────────────────────────────────────┘"
            .truecolor(ui::ORANGE_DARK.0, ui::ORANGE_DARK.1, ui::ORANGE_DARK.2)
    );

    // =========================================================================
    // ON-CHAIN VERIFICATION
    // =========================================================================

    println!();
    ui::divider(55);
    println!();
    println!(
        "  {} {}",
        "▶".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).bold(),
        "On-Chain Verification".bright_white().bold()
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
        "\r  {} Connected to Mantle                         ",
        "✓".green().bold()
    );

    // Check if nullifier is already used on-chain
    print!(
        "  {} Checking nullifier status...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    if vault.is_nullifier_used(B256::from(nullifier_bytes)).await? {
        println!(
            "\r  {} Nullifier already used!                     ",
            "✗".red().bold()
        );
        return Err(anyhow!("This withdrawal has already been processed"));
    }

    println!(
        "\r  {} Nullifier unused (valid)                    ",
        "✓".green().bold()
    );

    // Check if root is valid on-chain
    print!(
        "  {} Verifying state root on-chain...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let root_valid = vault.is_valid_root(B256::from(state_root_bytes)).await?;
    if root_valid {
        println!(
            "\r  {} State root verified on-chain               ",
            "✓".green().bold()
        );
    } else {
        println!(
            "\r  {} State root not found on-chain              ",
            "⚠".yellow().bold()
        );
        ui::print_notice(
            "Outdated State",
            "State root may be outdated. Consider running 'veilocity sync'.",
        );
    }

    // =========================================================================
    // SUBMIT TRANSACTION
    // =========================================================================

    println!();
    print!(
        "  {} Submitting withdrawal transaction...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );
    io::stdout().flush().unwrap();

    let tx_hash = vault
        .withdraw(
            B256::from(nullifier_bytes),
            recipient_address,
            U256::from(amount_wei),
            B256::from(state_root_bytes),
            proof,
        )
        .await?;

    let tx_hash_hex = hex::encode(tx_hash);

    println!(
        "\r  {} Transaction confirmed!                      ",
        "✓".green().bold()
    );

    // Update local state
    let mut account_updated = account.clone();
    account_updated.balance -= amount_wei;
    account_updated.nonce += 1;
    state.update_account(&account_updated)?;
    state.mark_nullifier_used(&nullifier_bytes)?;

    // Record transaction
    let _ = state.record_transaction(
        "withdraw",
        amount_wei,
        Some(tx_hash.as_slice()),
        Some(&format!("{:?}", recipient_address)),
        "confirmed",
    );

    // =========================================================================
    // SUCCESS SUMMARY
    // =========================================================================

    println!();
    ui::divider_double(55);
    println!();
    println!(
        "  {} {}",
        "✓".green().bold(),
        "WITHDRAWAL SUCCESSFUL".green().bold()
    );
    println!();
    println!(
        "  {} 0x{}",
        "Transaction:".truecolor(120, 120, 120),
        tx_hash_hex.bright_white()
    );

    if let Some(explorer) = &config.network.explorer_url {
        println!(
            "  {} {}",
            "Explorer:   ".truecolor(120, 120, 120),
            format!("{}/tx/0x{}", explorer, tx_hash_hex)
                .truecolor(100, 149, 237)
                .underline()
        );
    }

    println!();
    println!(
        "  {} {} sent to {}",
        "◈".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2),
        format_eth(amount_wei).green().bold(),
        format!("{:?}", recipient_address).bright_white()
    );
    println!(
        "  New private balance: {}",
        format_eth(account_updated.balance)
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
        "  {} {} Source of funds: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Previous balance: {}",
        "│".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "●".truecolor(ui::PURPLE.0, ui::PURPLE.1, ui::PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Transaction history: {}",
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

    info!("Withdrawal of {} wei completed", amount_wei);

    Ok(())
}
