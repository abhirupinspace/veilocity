//! Veilocity CLI - Private execution layer for Mantle
//!
//! This CLI provides commands for interacting with the Veilocity private execution layer:
//! - Initialize wallets
//! - Deposit funds from Mantle
//! - Make private transfers
//! - Withdraw funds to Mantle
//! - Check balances and sync state

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

pub mod commands;
pub mod config;
pub mod wallet;

#[derive(Parser)]
#[command(name = "veilocity")]
#[command(about = "Private execution layer CLI for Mantle")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Config file path
    #[arg(short, long, default_value = "~/.veilocity/config.toml")]
    config: String,

    /// Network: mainnet, sepolia
    #[arg(short, long, default_value = "sepolia")]
    network: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Veilocity wallet
    Init {
        /// Recover from seed phrase
        #[arg(long)]
        recover: bool,
    },

    /// Deposit funds from Mantle into Veilocity
    Deposit {
        /// Amount to deposit in ETH
        amount: f64,
    },

    /// Send private transfer to another user
    Transfer {
        /// Recipient's Veilocity public key
        recipient: String,
        /// Amount to transfer
        amount: f64,
    },

    /// Withdraw funds from Veilocity to Mantle
    Withdraw {
        /// Amount to withdraw
        amount: f64,
        /// Recipient address (default: connected wallet)
        #[arg(short, long)]
        recipient: Option<String>,
    },

    /// Display current private balance
    Balance,

    /// Synchronize with on-chain state
    Sync,

    /// Show transaction history
    History,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    // Load config
    let config = config::load_config(&cli.config, &cli.network)?;

    match cli.command {
        Commands::Init { recover } => {
            commands::init::run(recover).await?;
        }
        Commands::Deposit { amount } => {
            commands::deposit::run(&config, amount).await?;
        }
        Commands::Transfer { recipient, amount } => {
            commands::transfer::run(&config, &recipient, amount).await?;
        }
        Commands::Withdraw { amount, recipient } => {
            commands::withdraw::run(&config, amount, recipient).await?;
        }
        Commands::Balance => {
            commands::balance::run(&config).await?;
        }
        Commands::Sync => {
            commands::sync::run(&config).await?;
        }
        Commands::History => {
            commands::history::run(&config).await?;
        }
    }

    Ok(())
}
