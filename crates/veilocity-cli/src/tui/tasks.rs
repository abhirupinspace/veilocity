//! TUI Background Tasks
//!
//! Async tasks for polling network state and events.

use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;

use alloy::providers::{Provider, ProviderBuilder};

use veilocity_contracts::vault::VaultReader;

use crate::config::Config;

use super::app::{AppEvent, ConnectionStatus, DepositEventInfo, WithdrawalEventInfo};

/// Polling interval for background tasks
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Spawn background polling tasks
pub fn spawn_background_tasks(config: Config, event_tx: mpsc::Sender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        let mut last_block: u64 = 0;
        let mut consecutive_failures = 0;

        loop {
            interval.tick().await;

            match poll_network_state(&config, last_block, &event_tx).await {
                Ok(new_block) => {
                    if consecutive_failures > 0 {
                        // Connection restored
                        let _ = event_tx
                            .send(AppEvent::ConnectionStatusChanged(ConnectionStatus::Connected))
                            .await;
                        consecutive_failures = 0;
                    }
                    last_block = new_block;
                }
                Err(e) => {
                    consecutive_failures += 1;
                    warn!("Background poll failed (attempt {}): {}", consecutive_failures, e);

                    if consecutive_failures == 1 {
                        let _ = event_tx
                            .send(AppEvent::ConnectionStatusChanged(
                                ConnectionStatus::Reconnecting,
                            ))
                            .await;
                    } else if consecutive_failures >= 3 {
                        let _ = event_tx
                            .send(AppEvent::ConnectionStatusChanged(
                                ConnectionStatus::Disconnected,
                            ))
                            .await;
                    }
                }
            }
        }
    })
}

/// Poll network state and send events
async fn poll_network_state(
    config: &Config,
    last_block: u64,
    event_tx: &mpsc::Sender<AppEvent>,
) -> anyhow::Result<u64> {
    // Connect to RPC
    let provider = ProviderBuilder::new()
        .connect_http(config.network.rpc_url.parse()?);

    // Get current block number
    let current_block = provider.get_block_number().await?;

    // Send block update
    event_tx
        .send(AppEvent::BlockNumberUpdated(current_block))
        .await
        .ok();

    // If this is first poll or we have new blocks, check for events
    if last_block > 0 && current_block > last_block {
        // Create vault reader
        let vault_address: alloy::primitives::Address = config.network.vault_address.parse()?;
        let vault = VaultReader::with_provider(provider.clone(), vault_address);

        // Fetch events from last_block to current
        let filter = veilocity_contracts::events::EventFilter::block_range(
            last_block + 1,
            current_block,
        );

        // Get deposit events
        if let Ok(deposits) = vault.get_deposit_events(&filter).await {
            for deposit in deposits {
                let amount_wei = deposit.amount.try_into().unwrap_or(0u128);
                let leaf_index = deposit.leaf_index.try_into().unwrap_or(0u64);

                event_tx
                    .send(AppEvent::NewDeposit(DepositEventInfo {
                        amount_wei,
                        leaf_index,
                        is_own: false, // Would need to check against user's commitments
                    }))
                    .await
                    .ok();
            }
        }

        // Get withdrawal events
        if let Ok(withdrawals) = vault.get_withdrawal_events(&filter).await {
            for withdrawal in withdrawals {
                let amount_wei = withdrawal.amount.try_into().unwrap_or(0u128);

                event_tx
                    .send(AppEvent::NewWithdrawal(WithdrawalEventInfo {
                        amount_wei,
                        recipient: format!("{:?}", withdrawal.recipient),
                        is_own: false, // Would need to check against user's address
                    }))
                    .await
                    .ok();
            }
        }

        // Get state root updates
        if let Ok(root_updates) = vault.get_state_root_events(&filter).await {
            for update in root_updates {
                let batch_index = update.batch_index.try_into().unwrap_or(0u64);
                event_tx
                    .send(AppEvent::StateRootUpdated { batch_index })
                    .await
                    .ok();
            }
        }
    }

    // Send sync progress if connected to vault
    event_tx
        .send(AppEvent::SyncProgress {
            current: current_block,
            target: current_block,
        })
        .await
        .ok();

    // Send connected status on first successful poll
    if last_block == 0 {
        event_tx
            .send(AppEvent::ConnectionStatusChanged(ConnectionStatus::Connected))
            .await
            .ok();
    }

    Ok(current_block)
}

/// Poll balance for a specific account
pub async fn poll_balance(
    config: &Config,
    pubkey: &[u8; 32],
) -> anyhow::Result<Option<u128>> {
    // Load state from SQLite
    let state = veilocity_core::state::StateManager::new(&config.db_path())?;

    // Get account balance
    if let Some(account) = state.get_account(pubkey)? {
        Ok(Some(account.balance))
    } else {
        Ok(None)
    }
}

/// Trigger a sync operation
pub async fn trigger_sync(
    _config: &Config,
    event_tx: mpsc::Sender<AppEvent>,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    // TODO: Implement actual sync using commands::sync logic
    // For now, just simulate

    let duration_ms = start.elapsed().as_millis() as u64;
    event_tx
        .send(AppEvent::SyncCompleted { duration_ms })
        .await
        .ok();

    Ok(())
}
