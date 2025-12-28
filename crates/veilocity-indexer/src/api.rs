//! REST API for serving indexed state

use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::indexer::IndexerState;

type SharedState = Arc<RwLock<IndexerState>>;

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    is_syncing: bool,
    sync_progress: u8,
    last_block: u64,
}

/// Sync state response - contains everything the CLI needs
#[derive(Serialize)]
pub struct SyncStateResponse {
    /// Current Merkle root (hex)
    pub state_root: String,
    /// All leaves (hex array)
    pub leaves: Vec<String>,
    /// Used nullifiers (hex array)
    pub nullifiers: Vec<String>,
    /// Last synced block
    pub last_block: u64,
    /// Deposit count
    pub deposit_count: u64,
    /// TVL in wei
    pub tvl_wei: String,
    /// Is currently syncing
    pub is_syncing: bool,
    /// Sync progress (0-100)
    pub sync_progress: u8,
}

/// Deposits response
#[derive(Serialize)]
pub struct DepositsResponse {
    pub deposits: Vec<DepositInfo>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct DepositInfo {
    pub commitment: String,
    pub amount_wei: String,
    pub amount_mnt: f64,
    pub leaf_index: u64,
    pub block_number: u64,
    pub tx_hash: String,
}

/// Withdrawals response
#[derive(Serialize)]
pub struct WithdrawalsResponse {
    pub withdrawals: Vec<WithdrawalInfo>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct WithdrawalInfo {
    pub nullifier: String,
    pub recipient: String,
    pub amount_wei: String,
    pub amount_mnt: f64,
    pub block_number: u64,
    pub tx_hash: String,
}

/// Health check endpoint
async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    let s = state.read().await;
    Json(HealthResponse {
        status: "ok",
        is_syncing: s.is_syncing,
        sync_progress: s.sync_progress,
        last_block: s.last_block,
    })
}

/// Get sync state - main endpoint for CLI
async fn get_sync_state(State(state): State<SharedState>) -> Json<SyncStateResponse> {
    let s = state.read().await;

    Json(SyncStateResponse {
        state_root: format!("0x{}", hex::encode(s.state_root)),
        leaves: s.leaves.iter().map(|l| format!("0x{}", hex::encode(l))).collect(),
        nullifiers: s.nullifiers.iter().map(|n| format!("0x{}", hex::encode(n))).collect(),
        last_block: s.last_block,
        deposit_count: s.deposit_count,
        tvl_wei: s.tvl_wei.clone(),
        is_syncing: s.is_syncing,
        sync_progress: s.sync_progress,
    })
}

/// Get all deposits
async fn get_deposits(State(state): State<SharedState>) -> Json<DepositsResponse> {
    let s = state.read().await;

    let deposits: Vec<DepositInfo> = s
        .deposits
        .iter()
        .map(|d| {
            let amount: u128 = d.amount.try_into().unwrap_or(0);
            DepositInfo {
                commitment: format!("0x{}", hex::encode(d.commitment)),
                amount_wei: d.amount.to_string(),
                amount_mnt: amount as f64 / 1e18,
                leaf_index: d.leaf_index,
                block_number: d.block_number,
                tx_hash: format!("0x{}", hex::encode(d.tx_hash)),
            }
        })
        .collect();

    let total = deposits.len();
    Json(DepositsResponse { deposits, total })
}

/// Get all withdrawals
async fn get_withdrawals(State(state): State<SharedState>) -> Json<WithdrawalsResponse> {
    let s = state.read().await;

    let withdrawals: Vec<WithdrawalInfo> = s
        .withdrawals
        .iter()
        .map(|w| {
            let amount: u128 = w.amount.try_into().unwrap_or(0);
            WithdrawalInfo {
                nullifier: format!("0x{}", hex::encode(w.nullifier)),
                recipient: format!("{:?}", w.recipient),
                amount_wei: w.amount.to_string(),
                amount_mnt: amount as f64 / 1e18,
                block_number: w.block_number,
                tx_hash: format!("0x{}", hex::encode(w.tx_hash)),
            }
        })
        .collect();

    let total = withdrawals.len();
    Json(WithdrawalsResponse { withdrawals, total })
}

/// Run the HTTP server
pub async fn run_server(addr: &str, state: SharedState) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/sync", get(get_sync_state))
        .route("/deposits", get(get_deposits))
        .route("/withdrawals", get(get_withdrawals))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Indexer API listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
