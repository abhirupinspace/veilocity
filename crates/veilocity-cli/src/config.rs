//! Configuration management for Veilocity CLI

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// RPC URL for the network
    pub rpc_url: String,
    /// Chain ID
    pub chain_id: u64,
    /// VeilocityVault contract address
    pub vault_address: String,
    /// Block explorer URL (optional)
    pub explorer_url: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::sepolia()
    }
}

impl NetworkConfig {
    /// Mantle Sepolia testnet configuration
    pub fn sepolia() -> Self {
        Self {
            rpc_url: "https://rpc.sepolia.mantle.xyz".to_string(),
            chain_id: 5003,
            vault_address: String::new(), // To be set after deployment
            explorer_url: Some("https://explorer.sepolia.mantle.xyz".to_string()),
        }
    }

    /// Mantle mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            rpc_url: "https://rpc.mantle.xyz".to_string(),
            chain_id: 5000,
            vault_address: String::new(),
            explorer_url: Some("https://explorer.mantle.xyz".to_string()),
        }
    }
}

/// Prover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProverConfig {
    /// Number of threads for proof generation
    pub threads: usize,
    /// Whether to cache proofs
    pub cache_proofs: bool,
    /// Path to circuits directory
    pub circuits_path: Option<String>,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            threads: 4,
            cache_proofs: true,
            circuits_path: None,
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Poll interval in seconds
    pub poll_interval_secs: u64,
    /// Number of confirmations to wait for
    pub confirmations: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 12,
            confirmations: 2,
        }
    }
}

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Network configuration
    pub network: NetworkConfig,
    /// Prover configuration
    pub prover: ProverConfig,
    /// Sync configuration
    pub sync: SyncConfig,
    /// Data directory path
    #[serde(skip)]
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            prover: ProverConfig::default(),
            sync: SyncConfig::default(),
            data_dir: get_data_dir(),
        }
    }
}

impl Config {
    /// Create config for a specific network
    pub fn for_network(network: &str) -> Self {
        let network_config = match network.to_lowercase().as_str() {
            "mainnet" => NetworkConfig::mainnet(),
            "sepolia" | _ => NetworkConfig::sepolia(),
        };

        Self {
            network: network_config,
            ..Default::default()
        }
    }

    /// Get the database path
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("state.db")
    }

    /// Get the keystore path
    pub fn keystore_path(&self) -> PathBuf {
        self.data_dir.join("keystore")
    }

    /// Get the wallet file path
    pub fn wallet_path(&self) -> PathBuf {
        self.data_dir.join("wallet.json")
    }

    /// Get the config file path
    pub fn config_path(&self) -> PathBuf {
        self.data_dir.join("config.toml")
    }

    /// Ensure data directory exists
    pub fn ensure_data_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)
            .context("Failed to create data directory")?;
        Ok(())
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        self.ensure_data_dir()?;

        let config_str = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(self.config_path(), config_str)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Load config from file
    pub fn load(path: &Path) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .context("Failed to read config file")?;

        let mut config: Config = toml::from_str(&config_str)
            .context("Failed to parse config file")?;

        // Set data_dir from the config file's parent directory
        if let Some(parent) = path.parent() {
            config.data_dir = parent.to_path_buf();
        }

        Ok(config)
    }

    /// Check if initialized (config file exists)
    pub fn is_initialized(&self) -> bool {
        self.config_path().exists()
    }
}

/// Get the default data directory
pub fn get_data_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("xyz", "veilocity", "veilocity") {
        proj_dirs.data_dir().to_path_buf()
    } else {
        // Fallback to ~/.veilocity
        dirs::home_dir()
            .map(|h| h.join(".veilocity"))
            .unwrap_or_else(|| PathBuf::from(".veilocity"))
    }
}

/// Expand ~ in paths
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Load configuration from path or use defaults
pub fn load_config(config_path: &str, network: &str) -> Result<Config> {
    let expanded_path = expand_path(config_path);

    if expanded_path.exists() {
        Config::load(&expanded_path)
    } else {
        // Use default config for the specified network
        let mut config = Config::for_network(network);
        config.data_dir = expanded_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(get_data_dir);
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network.chain_id, 5003); // Sepolia
        assert_eq!(config.sync.poll_interval_secs, 12);
    }

    #[test]
    fn test_network_configs() {
        let sepolia = NetworkConfig::sepolia();
        assert_eq!(sepolia.chain_id, 5003);

        let mainnet = NetworkConfig::mainnet();
        assert_eq!(mainnet.chain_id, 5000);
    }

    #[test]
    fn test_expand_path() {
        let path = expand_path("~/.veilocity/config.toml");
        assert!(!path.starts_with("~"));
    }
}
