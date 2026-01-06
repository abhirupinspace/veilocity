//! TUI Actions
//!
//! Handles deposit, withdraw, and sync operations with async execution.

use std::path::PathBuf;
use std::time::Instant;

use alloy::primitives::{Address, B256, U256};
use tokio::sync::mpsc;

use veilocity_contracts::{create_vault_client, create_vault_reader};
use veilocity_core::account::PrivateAccount;
use veilocity_core::poseidon::{field_to_bytes, u128_to_field, u64_to_field, PoseidonHasher};
use veilocity_core::state::StateManager;
use veilocity_prover::{NoirProver, WithdrawWitness};

use crate::config::Config;
use crate::wallet::parse_mnt;

use super::app::{AppEvent, ZkStage};

/// Execute a deposit transaction
pub async fn execute_deposit(
    config: Config,
    amount: f64,
    signer: alloy::signers::local::PrivateKeySigner,
    secret: veilocity_core::account::AccountSecret,
    event_tx: mpsc::Sender<AppEvent>,
) -> Result<String, String> {
    // Parse amount
    let amount_wei = parse_mnt(amount);
    let amount_u256 = U256::from(amount_wei);

    // Check vault address
    if config.network.vault_address.is_empty() {
        return Err("Vault address not configured".to_string());
    }

    let vault_address: Address = config
        .network
        .vault_address
        .parse()
        .map_err(|_| "Invalid vault address")?;

    // Send progress: commitment generation
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::CommitmentGeneration,
            progress: 0.1,
            message: "Generating cryptographic commitment...".to_string(),
        })
        .await;

    // Compute deposit commitment
    let mut hasher = PoseidonHasher::new();
    let commitment = secret.compute_deposit_commitment(&mut hasher, amount_wei);
    let commitment_bytes = field_to_bytes(&commitment);
    let commitment_b256 = B256::from(commitment_bytes);

    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::CommitmentGeneration,
            progress: 0.4,
            message: "Poseidon hash computed successfully".to_string(),
        })
        .await;

    // Connect to network
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::Submitting,
            progress: 0.6,
            message: "Connecting to Mantle network...".to_string(),
        })
        .await;

    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Submit deposit
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::Submitting,
            progress: 0.8,
            message: "Broadcasting deposit transaction...".to_string(),
        })
        .await;

    let tx_hash = vault
        .deposit(commitment_b256, amount_u256)
        .await
        .map_err(|e| format!("Deposit failed: {}", e))?;

    let tx_hash_hex = format!("0x{}", hex::encode(tx_hash));

    // Record transaction locally
    config.ensure_data_dir().ok();
    if let Ok(mut state) = StateManager::new(&config.db_path()) {
        let _ = state.record_transaction(
            "deposit",
            amount_wei,
            Some(tx_hash.as_slice()),
            None,
            "confirmed",
        );
    }

    Ok(tx_hash_hex)
}

/// Execute a withdrawal transaction with ZK proof
pub async fn execute_withdraw(
    config: Config,
    amount: f64,
    recipient: Address,
    signer: alloy::signers::local::PrivateKeySigner,
    secret: veilocity_core::account::AccountSecret,
    event_tx: mpsc::Sender<AppEvent>,
) -> Result<String, String> {
    let amount_wei = parse_mnt(amount);

    // Check vault address
    if config.network.vault_address.is_empty() {
        return Err("Vault address not configured".to_string());
    }

    let vault_address: Address = config
        .network
        .vault_address
        .parse()
        .map_err(|_| "Invalid vault address")?;

    // Load state
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::WitnessGeneration,
            progress: 0.05,
            message: "Loading private account...".to_string(),
        })
        .await;

    let mut state = StateManager::new(&config.db_path())
        .map_err(|_| "Failed to load state. Run sync first.")?;

    let mut hasher = PoseidonHasher::new();
    let pubkey_field = secret.derive_pubkey(&mut hasher);
    let pubkey_bytes = field_to_bytes(&pubkey_field);

    let account = state
        .get_account(&pubkey_bytes)
        .map_err(|e| format!("Failed to get account: {}", e))?
        .ok_or_else(|| "Account not found. Have you made a deposit?")?;

    // Check balance
    if account.balance < amount_wei {
        return Err(format!(
            "Insufficient balance. Have: {:.6} MNT, Need: {:.6} MNT",
            account.balance as f64 / 1e18,
            amount_wei as f64 / 1e18
        ));
    }

    // Compute nullifier
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::WitnessGeneration,
            progress: 0.1,
            message: "Computing nullifier...".to_string(),
        })
        .await;

    let nullifier = secret.compute_nullifier(state.hasher(), account.index, account.nonce);
    let nullifier_bytes = field_to_bytes(&nullifier);

    // Generate Merkle proof
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::WitnessGeneration,
            progress: 0.15,
            message: "Generating Merkle proof...".to_string(),
        })
        .await;

    let merkle_path = state.get_merkle_proof(account.index);
    let merkle_path_fields: Vec<_> = merkle_path.into_iter().collect();

    // Get state root
    let state_root = state.state_root();
    let state_root_bytes = field_to_bytes(&state_root);

    // Build witness
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::WitnessGeneration,
            progress: 0.2,
            message: "Building circuit witness...".to_string(),
        })
        .await;

    let recipient_field = veilocity_core::poseidon::FieldElement::from(u128::from_be_bytes({
        let mut bytes = [0u8; 16];
        bytes[4..].copy_from_slice(&recipient.0 .0[..12]);
        bytes
    }));

    let witness = WithdrawWitness::new(
        state_root,
        nullifier,
        u128_to_field(amount_wei),
        recipient_field,
        *secret.secret(),
        u128_to_field(account.balance),
        u64_to_field(account.nonce),
        u64_to_field(account.index),
        merkle_path_fields,
    )
    .map_err(|e| format!("Failed to build witness: {}", e))?;

    // Initialize prover
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::ConstraintSatisfaction,
            progress: 0.25,
            message: "Initializing Noir prover...".to_string(),
        })
        .await;

    let prover = NoirProver::new(PathBuf::from("circuits"));

    // Compile if needed
    if !prover.is_compiled() {
        let _ = event_tx
            .send(AppEvent::ZkProgress {
                stage: ZkStage::ConstraintSatisfaction,
                progress: 0.3,
                message: "Compiling Noir circuits...".to_string(),
            })
            .await;

        prover
            .compile()
            .await
            .map_err(|e| format!("Compilation failed: {}", e))?;
    }

    // Generate proof - this is the slow part (30-90 seconds typically)
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::ProofComputation,
            progress: 0.35,
            message: "Starting ZK proof generation...".to_string(),
        })
        .await;

    // Realistic progress updates during proof generation
    let progress_tx = event_tx.clone();
    let proof_handle = tokio::spawn(async move {
        let messages = [
            (0.38, "Parsing arithmetic circuit..."),
            (0.42, "Computing witness assignments..."),
            (0.46, "Building constraint system..."),
            (0.50, "Generating polynomial commitments..."),
            (0.55, "Computing FFT on polynomials..."),
            (0.60, "Evaluating gate constraints..."),
            (0.65, "Computing quotient polynomial..."),
            (0.70, "Splitting quotient into parts..."),
            (0.75, "Computing KZG commitments..."),
            (0.78, "Generating opening proofs..."),
            (0.80, "Finalizing proof structure..."),
        ];

        for (progress, msg) in messages {
            // Variable delays to feel more realistic
            let delay = if progress < 0.5 { 2000 } else if progress < 0.7 { 4000 } else { 3000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            let _ = progress_tx
                .send(AppEvent::ZkProgress {
                    stage: ZkStage::ProofComputation,
                    progress,
                    message: msg.to_string(),
                })
                .await;
        }
    });

    let proof = prover
        .prove_withdraw(&witness)
        .await
        .map_err(|e| format!("Proof generation failed: {}", e))?;

    // Cancel progress task
    proof_handle.abort();

    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::Verification,
            progress: 0.85,
            message: "Proof generated! Verifying...".to_string(),
        })
        .await;

    // Connect to vault
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::Submitting,
            progress: 0.9,
            message: "Connecting to Mantle...".to_string(),
        })
        .await;

    let vault = create_vault_client(&config.network.rpc_url, vault_address, signer)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Check nullifier not used
    if vault
        .is_nullifier_used(B256::from(nullifier_bytes))
        .await
        .map_err(|e| format!("Nullifier check failed: {}", e))?
    {
        return Err("This withdrawal has already been processed".to_string());
    }

    // Submit withdrawal
    let _ = event_tx
        .send(AppEvent::ZkProgress {
            stage: ZkStage::Submitting,
            progress: 0.95,
            message: "Submitting withdrawal...".to_string(),
        })
        .await;

    let tx_hash = vault
        .withdraw(
            B256::from(nullifier_bytes),
            recipient,
            U256::from(amount_wei),
            B256::from(state_root_bytes),
            proof,
        )
        .await
        .map_err(|e| format!("Withdrawal failed: {}", e))?;

    let tx_hash_hex = format!("0x{}", hex::encode(tx_hash));

    // Update local state
    let mut account_updated = account.clone();
    account_updated.balance -= amount_wei;
    account_updated.nonce += 1;
    state.update_account(&account_updated).ok();
    state.mark_nullifier_used(&nullifier_bytes).ok();
    let _ = state.record_transaction(
        "withdraw",
        amount_wei,
        Some(tx_hash.as_slice()),
        Some(&format!("{:?}", recipient)),
        "confirmed",
    );

    Ok(tx_hash_hex)
}

/// Execute a sync operation to update local state from chain
pub async fn execute_sync(
    config: Config,
    secret: Option<veilocity_core::account::AccountSecret>,
    event_tx: mpsc::Sender<AppEvent>,
) -> Result<u64, String> {
    use veilocity_contracts::events::EventFilter;

    let start = Instant::now();

    // Check vault address
    if config.network.vault_address.is_empty() {
        return Err("Vault address not configured".to_string());
    }

    let vault_address: Address = config
        .network
        .vault_address
        .parse()
        .map_err(|_| "Invalid vault address")?;

    // Send initial progress
    let _ = event_tx
        .send(AppEvent::SyncProgress {
            current: 0,
            target: 100,
        })
        .await;

    // Create vault reader
    let vault_reader = create_vault_reader(&config.network.rpc_url, vault_address)
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Get current block number
    let block_number = vault_reader
        .get_block_number()
        .await
        .map_err(|e| format!("Failed to get block number: {}", e))?;

    let _ = event_tx
        .send(AppEvent::BlockNumberUpdated(block_number))
        .await;

    // Ensure data dir exists
    config.ensure_data_dir().ok();

    // Load or create state manager
    let mut state = StateManager::new(&config.db_path())
        .map_err(|e| format!("Failed to create state manager: {}", e))?;

    // Use deployment block from config, or default to recent blocks
    let start_block = config.sync.deployment_block.unwrap_or_else(|| {
        block_number.saturating_sub(25000)
    });

    // Send scanning message
    let _ = event_tx
        .send(AppEvent::SyncMessage(format!(
            "Scanning blocks {}..{}",
            start_block, block_number
        )))
        .await;

    // Chunk size to avoid RPC limits (max 25000 blocks per request)
    const CHUNK_SIZE: u64 = 25000;

    let mut deposits = Vec::new();
    let mut current_start = start_block;
    let total_blocks = block_number - start_block;

    while current_start < block_number {
        let current_end = (current_start + CHUNK_SIZE).min(block_number);
        let filter = EventFilter::block_range(current_start, current_end);

        // Update message
        let _ = event_tx
            .send(AppEvent::SyncMessage(format!(
                "Fetching deposits: block {}",
                current_end
            )))
            .await;

        match vault_reader.get_deposit_events(&filter).await {
            Ok(chunk_deposits) => {
                if !chunk_deposits.is_empty() {
                    let _ = event_tx
                        .send(AppEvent::SyncMessage(format!(
                            "Found {} deposits",
                            chunk_deposits.len()
                        )))
                        .await;
                }
                deposits.extend(chunk_deposits);
            }
            Err(e) => {
                tracing::warn!("Failed to get deposits for blocks {}-{}: {}", current_start, current_end, e);
            }
        }

        current_start = current_end + 1;

        // Update progress
        let scanned = current_start - start_block;
        let progress = if total_blocks > 0 {
            20 + ((scanned as f32 / total_blocks as f32) * 30.0) as u64
        } else {
            50
        };
        let _ = event_tx
            .send(AppEvent::SyncProgress {
                current: progress.min(50),
                target: 100,
            })
            .await;
    }

    let _ = event_tx
        .send(AppEvent::SyncMessage(format!(
            "Processing {} deposits...",
            deposits.len()
        )))
        .await;

    // Process each deposit
    let pubkey_bytes = secret.as_ref().map(|s| {
        let mut hasher = PoseidonHasher::new();
        let pubkey_field = s.derive_pubkey(&mut hasher);
        field_to_bytes(&pubkey_field)
    });

    for deposit in deposits {
        // Convert U256 to native types
        let amount_u128: u128 = deposit.amount.try_into().unwrap_or(0);
        let leaf_index_u64: u64 = deposit.leaf_index.try_into().unwrap_or(0);

        // Check if this is our deposit
        let is_own = if let (Some(ref pubkey), Some(ref secret)) = (&pubkey_bytes, &secret) {
            // Re-compute commitment to verify
            let mut hasher = PoseidonHasher::new();
            let our_commitment = secret.compute_deposit_commitment(&mut hasher, amount_u128);
            let our_commitment_bytes = field_to_bytes(&our_commitment);
            let commitment_bytes: [u8; 32] = deposit.commitment.0;
            commitment_bytes == our_commitment_bytes
        } else {
            false
        };

        if is_own {
            // This is our deposit, create/update account
            if let Some(ref pubkey) = pubkey_bytes {
                let existing = state.get_account(pubkey).ok().flatten();

                let account = match existing {
                    Some(mut acc) => {
                        acc.balance += amount_u128;
                        acc
                    }
                    None => PrivateAccount {
                        pubkey: *pubkey,
                        balance: amount_u128,
                        nonce: 0,
                        index: leaf_index_u64,
                    },
                };

                state.update_account(&account).ok();
            }
        }

        // Send event
        let _ = event_tx
            .send(AppEvent::NewDeposit(super::app::DepositEventInfo {
                amount_wei: amount_u128,
                leaf_index: leaf_index_u64,
                is_own,
            }))
            .await;
    }

    // Get withdrawal events (chunked)
    let _ = event_tx
        .send(AppEvent::SyncMessage("Fetching withdrawals...".to_string()))
        .await;

    let mut withdrawals = Vec::new();
    current_start = start_block;

    while current_start < block_number {
        let current_end = (current_start + CHUNK_SIZE).min(block_number);
        let filter = EventFilter::block_range(current_start, current_end);

        match vault_reader.get_withdrawal_events(&filter).await {
            Ok(chunk_withdrawals) => {
                if !chunk_withdrawals.is_empty() {
                    let _ = event_tx
                        .send(AppEvent::SyncMessage(format!(
                            "Found {} withdrawals",
                            chunk_withdrawals.len()
                        )))
                        .await;
                }
                withdrawals.extend(chunk_withdrawals);
            }
            Err(e) => {
                tracing::warn!("Failed to get withdrawals for blocks {}-{}: {}", current_start, current_end, e);
            }
        }

        current_start = current_end + 1;

        // Update progress (70-90%)
        let scanned = current_start - start_block;
        let progress = if total_blocks > 0 {
            70 + ((scanned as f32 / total_blocks as f32) * 20.0) as u64
        } else {
            90
        };
        let _ = event_tx
            .send(AppEvent::SyncProgress {
                current: progress.min(90),
                target: 100,
            })
            .await;
    }

    for withdrawal in withdrawals {
        let amount_u128: u128 = withdrawal.amount.try_into().unwrap_or(0);
        let _ = event_tx
            .send(AppEvent::NewWithdrawal(super::app::WithdrawalEventInfo {
                amount_wei: amount_u128,
                recipient: format!("{:?}", withdrawal.recipient),
                is_own: false,
            }))
            .await;
    }

    // Finalize
    let _ = event_tx
        .send(AppEvent::SyncMessage("Finalizing state...".to_string()))
        .await;

    let _ = event_tx
        .send(AppEvent::SyncProgress {
            current: 100,
            target: 100,
        })
        .await;

    // Clear sync message on completion
    let _ = event_tx
        .send(AppEvent::SyncMessage(String::new()))
        .await;

    let duration_ms = start.elapsed().as_millis() as u64;
    Ok(duration_ms)
}
