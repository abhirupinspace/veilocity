//! Veilocity CLI - Private execution layer for Mantle
//!
//! This CLI provides commands for interacting with the Veilocity private execution layer:
//! - Initialize wallets
//! - Deposit funds from Mantle
//! - Make private transfers
//! - Withdraw funds to Mantle
//! - Check balances and sync state

use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing_subscriber::EnvFilter;

pub mod commands;
pub mod config;
pub mod ui;
pub mod wallet;

#[derive(Parser)]
#[command(name = "veilocity")]
#[command(about = "Private execution layer CLI for Mantle")]
#[command(version)]
#[command(before_help = "\x1b[1;38;2;255;140;0m██╗   ██╗███████╗██╗██╗      ██████╗  ██████╗██╗████████╗██╗   ██╗
██║   ██║██╔════╝██║██║     ██╔═══██╗██╔════╝██║╚══██╔══╝╚██╗ ██╔╝
██║   ██║█████╗  ██║██║     ██║   ██║██║     ██║   ██║    ╚████╔╝
╚██╗ ██╔╝██╔══╝  ██║██║     ██║   ██║██║     ██║   ██║     ╚██╔╝
 ╚████╔╝ ███████╗██║███████╗╚██████╔╝╚██████╗██║   ██║      ██║
  ╚═══╝  ╚══════╝╚═╝╚══════╝ ╚═════╝  ╚═════╝╚═╝   ╚═╝      ╚═╝\x1b[0m
")]
#[command(after_help = "Examples:
  veilocity init                    Create a new wallet
  veilocity deposit 0.1             Deposit 0.1 ETH
  veilocity transfer <pubkey> 0.05  Send 0.05 ETH privately
  veilocity withdraw 0.1            Withdraw 0.1 ETH
  veilocity balance                 Check your balance
  veilocity sync                    Sync with network
  veilocity history                 View transaction history")]
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
    #[command(alias = "i")]
    Init {
        /// Recover from seed phrase
        #[arg(long)]
        recover: bool,
    },

    /// Deposit funds from Mantle into Veilocity
    #[command(alias = "d", alias = "dep")]
    Deposit {
        /// Amount to deposit in ETH
        amount: f64,
        /// Preview the deposit without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Send private transfer to another user
    #[command(alias = "t", alias = "send")]
    Transfer {
        /// Recipient's Veilocity public key
        recipient: String,
        /// Amount to transfer
        amount: f64,
        /// Preview the transfer without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Withdraw funds from Veilocity to Mantle
    #[command(alias = "w")]
    Withdraw {
        /// Amount to withdraw
        amount: f64,
        /// Recipient address (default: connected wallet)
        #[arg(short, long)]
        recipient: Option<String>,
        /// Preview the withdrawal without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Display current private balance
    #[command(alias = "b", alias = "bal")]
    Balance,

    /// Synchronize with on-chain state
    #[command(alias = "s")]
    Sync,

    /// Show transaction history
    #[command(alias = "h", alias = "hist")]
    History,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging - only show logs in verbose mode
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("debug"))
            .init();
    }

    // Load config
    let config = config::load_config(&cli.config, &cli.network)?;

    let result = match cli.command {
        Commands::Init { recover } => {
            commands::init::run(recover).await
        }
        Commands::Deposit { amount, dry_run } => {
            commands::deposit::run(&config, amount, dry_run).await
        }
        Commands::Transfer { recipient, amount, dry_run } => {
            commands::transfer::run(&config, &recipient, amount, dry_run).await
        }
        Commands::Withdraw { amount, recipient, dry_run } => {
            commands::withdraw::run(&config, amount, recipient, dry_run).await
        }
        Commands::Balance => {
            commands::balance::run(&config).await
        }
        Commands::Sync => {
            commands::sync::run(&config).await
        }
        Commands::History => {
            commands::history::run(&config).await
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}
