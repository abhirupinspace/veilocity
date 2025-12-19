//! Config command - view and update configuration

use crate::config::{get_data_dir, Config};
use crate::ui;
use anyhow::{anyhow, Result};
use colored::Colorize;

/// Config subcommands
#[derive(Debug, Clone)]
pub enum ConfigAction {
    Show,
    Set { key: String, value: String },
}

/// Run the config command
pub async fn run(action: ConfigAction) -> Result<()> {
    let data_dir = get_data_dir();
    let config_path = data_dir.join("config.toml");

    match action {
        ConfigAction::Show => show_config(&config_path).await,
        ConfigAction::Set { key, value } => set_config(&config_path, &key, &value).await,
    }
}

async fn show_config(config_path: &std::path::Path) -> Result<()> {
    println!();
    println!("{}", ui::header("Configuration"));
    println!();

    if !config_path.exists() {
        println!(
            "  {}",
            "No config file found. Run 'veilocity init' first.".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
        );
        println!();
        return Ok(());
    }

    let config = Config::load(config_path)?;

    println!("{}", ui::header("Network"));
    println!();
    println!(
        "  {} {}",
        "RPC URL:      ".truecolor(120, 120, 120),
        config.network.rpc_url.bright_white()
    );
    println!(
        "  {} {}",
        "Chain ID:     ".truecolor(120, 120, 120),
        config.network.chain_id.to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Vault Address:".truecolor(120, 120, 120),
        if config.network.vault_address.is_empty() {
            "(not set)".truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2).to_string()
        } else {
            config.network.vault_address.clone().bright_white().to_string()
        }
    );
    if let Some(explorer) = &config.network.explorer_url {
        println!(
            "  {} {}",
            "Explorer:     ".truecolor(120, 120, 120),
            explorer.dimmed()
        );
    }

    println!();
    println!("{}", ui::header("Sync"));
    println!();
    println!(
        "  {} {}s",
        "Poll Interval:   ".truecolor(120, 120, 120),
        config.sync.poll_interval_secs.to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Confirmations:   ".truecolor(120, 120, 120),
        config.sync.confirmations.to_string().bright_white()
    );
    if let Some(block) = config.sync.deployment_block {
        println!(
            "  {} {}",
            "Deployment Block:".truecolor(120, 120, 120),
            block.to_string().bright_white()
        );
    }

    println!();
    println!("{}", ui::header("Paths"));
    println!();
    println!(
        "  {} {}",
        "Config File:".truecolor(120, 120, 120),
        config_path.display().to_string().dimmed()
    );
    println!(
        "  {} {}",
        "Data Dir:   ".truecolor(120, 120, 120),
        config.data_dir.display().to_string().dimmed()
    );

    println!();
    ui::divider(55);
    println!(
        "  Use '{}' to update settings",
        ui::command("veilocity config set <key> <value>")
    );
    println!();

    Ok(())
}

async fn set_config(config_path: &std::path::Path, key: &str, value: &str) -> Result<()> {
    if !config_path.exists() {
        return Err(anyhow!("No config file found. Run 'veilocity init' first."));
    }

    let mut config = Config::load(config_path)?;

    match key.to_lowercase().as_str() {
        "vault" | "vault_address" | "vault-address" => {
            // Validate it looks like an address
            if !value.starts_with("0x") || value.len() != 42 {
                return Err(anyhow!("Invalid vault address. Expected 0x... (42 characters)"));
            }
            config.network.vault_address = value.to_string();
            config.save()?;

            println!();
            ui::print_success("Vault address updated!");
            println!();
            println!(
                "  {} {}",
                "Vault:".truecolor(120, 120, 120),
                value.truecolor(ui::ORANGE.0, ui::ORANGE.1, ui::ORANGE.2)
            );
            println!();
        }
        "rpc" | "rpc_url" | "rpc-url" => {
            config.network.rpc_url = value.to_string();
            config.save()?;

            println!();
            ui::print_success("RPC URL updated!");
            println!();
            println!(
                "  {} {}",
                "RPC:".truecolor(120, 120, 120),
                value.bright_white()
            );
            println!();
        }
        "chain_id" | "chain-id" | "chainid" => {
            let chain_id: u64 = value.parse().map_err(|_| anyhow!("Invalid chain ID"))?;
            config.network.chain_id = chain_id;
            config.save()?;

            println!();
            ui::print_success("Chain ID updated!");
            println!();
            println!(
                "  {} {}",
                "Chain ID:".truecolor(120, 120, 120),
                chain_id.to_string().bright_white()
            );
            println!();
        }
        "deployment_block" | "deployment-block" => {
            let block: u64 = value.parse().map_err(|_| anyhow!("Invalid block number"))?;
            config.sync.deployment_block = Some(block);
            config.save()?;

            println!();
            ui::print_success("Deployment block updated!");
            println!();
            println!(
                "  {} {}",
                "Deployment Block:".truecolor(120, 120, 120),
                block.to_string().bright_white()
            );
            println!();
        }
        _ => {
            return Err(anyhow!(
                "Unknown config key: '{}'\n\nAvailable keys:\n  \
                vault, vault_address  - VeilocityVault contract address\n  \
                rpc, rpc_url          - Network RPC URL\n  \
                chain_id              - Network chain ID\n  \
                deployment_block      - Block number where vault was deployed",
                key
            ));
        }
    }

    Ok(())
}
