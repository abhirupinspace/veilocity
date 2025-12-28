//! Veilocity Indexer - Fast state sync service
//!
//! This service maintains a synchronized copy of on-chain state and provides
//! instant state queries via REST API, enabling sub-second sync for the CLI.

mod indexer;
mod api;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::indexer::IndexerState;

#[derive(Parser, Debug)]
#[command(name = "veilocity-indexer")]
#[command(about = "Fast indexer service for Veilocity")]
struct Args {
    /// RPC URL for the network
    #[arg(long, env = "RPC_URL", default_value = "https://rpc.sepolia.mantle.xyz")]
    rpc_url: String,

    /// Vault contract address
    #[arg(long, env = "VAULT_ADDRESS")]
    vault_address: String,

    /// Block number where contract was deployed
    #[arg(long, env = "DEPLOYMENT_BLOCK", default_value = "0")]
    deployment_block: u64,

    /// HTTP server port
    #[arg(long, env = "PORT", default_value = "3001")]
    port: u16,

    /// Sync poll interval in seconds
    #[arg(long, default_value = "2")]
    poll_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    let args = Args::parse();

    info!("Starting Veilocity Indexer");
    info!("  RPC URL: {}", args.rpc_url);
    info!("  Vault: {}", args.vault_address);
    info!("  Port: {}", args.port);

    // Parse vault address
    let vault_address: alloy::primitives::Address = args
        .vault_address
        .parse()
        .expect("Invalid vault address");

    // Create shared indexer state
    let state = Arc::new(RwLock::new(IndexerState::new()));

    // Start background sync task
    let sync_state = state.clone();
    let rpc_url = args.rpc_url.clone();
    let deployment_block = args.deployment_block;
    let poll_interval = args.poll_interval;

    tokio::spawn(async move {
        indexer::run_sync_loop(
            sync_state,
            &rpc_url,
            vault_address,
            deployment_block,
            poll_interval,
        )
        .await
    });

    // Start HTTP server
    let addr = format!("0.0.0.0:{}", args.port);
    info!("Starting HTTP server on {}", addr);

    api::run_server(&addr, state).await?;

    Ok(())
}
