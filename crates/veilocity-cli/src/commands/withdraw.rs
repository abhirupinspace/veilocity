//! Withdraw command - withdraw funds from Veilocity to Mantle

use crate::config::Config;
use crate::wallet::{format_eth, parse_eth, WalletManager};
use alloy::primitives::{Address, B256, U256};
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use tracing::info;
use veilocity_contracts::create_vault_client;
use veilocity_core::poseidon::{field_to_bytes, u128_to_field, u64_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;
use veilocity_prover::{NoirProver, WithdrawWitness};

/// Run the withdraw command
pub async fn run(config: &Config, amount: f64, recipient: Option<String>) -> Result<()> {
    let wallet_manager = WalletManager::new(config.clone());

    // Load wallet
    let wallet = wallet_manager.load_wallet()?;

    // Get password
    let password = rpassword::prompt_password("Enter wallet password: ")
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

    println!("\n=== Withdrawal Details ===");
    println!("Amount: {} ({} wei)", format_eth(amount_wei), amount_wei);
    println!("Recipient: {:?}", recipient_address);

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

    println!("Current private balance: {}", format_eth(account.balance));

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

    println!("\nGenerating withdrawal proof... (this may take a moment)");

    // Generate proof
    let prover = NoirProver::new(PathBuf::from("circuits"));

    if !prover.is_compiled() {
        println!("Circuits not compiled. Compiling...");
        prover.compile().await?;
    }

    let proof = prover.prove_withdraw(&witness).await?;

    println!("Proof generated ({} bytes)", proof.len());

    // Create vault client
    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer).await?;

    // Check if nullifier is already used on-chain
    if vault.is_nullifier_used(B256::from(nullifier_bytes)).await? {
        return Err(anyhow!("This withdrawal has already been processed"));
    }

    // Check if root is valid on-chain
    if !vault.is_valid_root(B256::from(state_root_bytes)).await? {
        println!("Warning: State root may be outdated. Syncing...");
        // In production, trigger a sync here
    }

    println!("\nSubmitting withdrawal transaction...");

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

    // Update local state
    let mut account_updated = account.clone();
    account_updated.balance -= amount_wei;
    account_updated.nonce += 1;
    state.update_account(&account_updated)?;
    state.mark_nullifier_used(&nullifier_bytes)?;

    println!("\nWithdrawal successful!");
    println!("Transaction hash: 0x{}", hex::encode(tx_hash));

    if let Some(explorer) = &config.network.explorer_url {
        println!("View on explorer: {}/tx/0x{}", explorer, hex::encode(tx_hash));
    }

    println!("\n{} has been sent to {:?}", format_eth(amount_wei), recipient_address);
    println!("New private balance: {}", format_eth(account_updated.balance));

    info!("Withdrawal of {} wei completed", amount_wei);

    Ok(())
}
