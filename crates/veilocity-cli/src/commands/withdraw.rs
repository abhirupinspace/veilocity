//! Withdraw command - withdraw funds from Veilocity to Mantle

use crate::config::Config;
use crate::ui;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use alloy::primitives::{Address, B256, U256};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use tracing::info;
use veilocity_contracts::create_vault_client;
use veilocity_core::poseidon::{field_to_bytes, u128_to_field, u64_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;
use veilocity_prover::{NoirProver, WithdrawWitness};

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
    println!("{}", ui::header("Withdrawal"));
    println!();

    // Amount display
    println!(
        "  {} {}  {}",
        "↑".red().bold(),
        format_eth(amount_wei).red().bold(),
        format!("({} wei)", amount_wei).dimmed()
    );
    println!();

    ui::divider(55);
    println!();
    println!(
        "  {} {:?}",
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
    let mut hasher = PoseidonHasher::new();
    let pubkey_field = veilocity_secret.derive_pubkey(&mut hasher);
    let pubkey_bytes = field_to_bytes(&pubkey_field);

    let account = state
        .get_account(&pubkey_bytes)?
        .ok_or_else(|| anyhow!("Account not found. Have you made a deposit?"))?;

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
        "(private)".dimmed()
    );

    // Get Merkle proof
    let merkle_path = state.get_merkle_proof(account.index);
    let merkle_path_fields: Vec<_> = merkle_path.into_iter().collect();

    // Compute nullifier
    let nullifier = veilocity_secret.compute_nullifier(state.hasher(), account.index, account.nonce);
    let nullifier_bytes = field_to_bytes(&nullifier);

    // Get state root
    let state_root = state.state_root();
    let state_root_bytes = field_to_bytes(&state_root);

    // Convert recipient address to field
    let recipient_field = veilocity_core::poseidon::FieldElement::from(
        u128::from_be_bytes({
            let mut bytes = [0u8; 16];
            bytes[4..].copy_from_slice(&recipient_address.0 .0[..12]);
            bytes
        }),
    );

    // Create witness
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

    if dry_run {
        println!();
        ui::print_notice(
            "DRY RUN",
            "Proof would be generated and withdrawal submitted. Remove --dry-run to execute.",
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Generating withdrawal proof...",
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

    let proof = prover.prove_withdraw(&witness).await?;

    println!(
        "  {} Proof generated ({} bytes)",
        "✓".green().bold(),
        proof.len()
    );

    // Create vault client
    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer).await?;

    // Check if nullifier is already used on-chain
    if vault.is_nullifier_used(B256::from(nullifier_bytes)).await? {
        return Err(anyhow!("This withdrawal has already been processed"));
    }

    // Check if root is valid on-chain
    if !vault.is_valid_root(B256::from(state_root_bytes)).await? {
        ui::print_notice(
            "Outdated State",
            "State root may be outdated. Consider running 'veilocity sync'.",
        );
    }

    println!();
    println!(
        "  {} Submitting withdrawal transaction...",
        "◐".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
    );

    // Submit withdrawal
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

    ui::print_success("Withdrawal successful!");
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
    ui::divider_double(55);
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
    ui::divider_double(55);
    println!();

    info!("Withdrawal of {} wei completed", amount_wei);

    Ok(())
}
