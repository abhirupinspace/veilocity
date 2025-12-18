//! Wallet management for Veilocity CLI
//!
//! Handles key generation, storage, and account management.

use crate::config::Config;
use alloy::primitives::{Address, B256};
use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Context, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use veilocity_core::account::AccountSecret;
use veilocity_core::poseidon::{field_to_hex, PoseidonHasher};

/// Wallet data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Ethereum address (for on-chain transactions)
    pub address: String,
    /// Veilocity public key (for private transfers)
    pub veilocity_pubkey: String,
    /// Encrypted private key (Ethereum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_key: Option<String>,
    /// Encrypted Veilocity secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_secret: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
}

impl Wallet {
    /// Create a new wallet
    pub fn new(address: Address, veilocity_pubkey: String) -> Self {
        Self {
            address: format!("{:?}", address),
            veilocity_pubkey,
            encrypted_key: None,
            encrypted_secret: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Get address as Address type
    pub fn address(&self) -> Result<Address> {
        self.address
            .parse()
            .context("Failed to parse wallet address")
    }
}

/// Wallet manager for creating and loading wallets
pub struct WalletManager {
    config: Config,
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Generate a new wallet
    pub fn generate(&self, password: &str) -> Result<(Wallet, PrivateKeySigner, AccountSecret)> {
        // Generate Ethereum private key
        let mut rng = rand::thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);

        let signer = PrivateKeySigner::from_bytes(&B256::from(key_bytes))
            .context("Failed to create signer")?;
        let address = signer.address();

        // Generate Veilocity secret
        let veilocity_secret = AccountSecret::generate();
        let mut hasher = PoseidonHasher::new();
        let veilocity_pubkey = veilocity_secret.derive_pubkey(&mut hasher);
        let veilocity_pubkey_hex = field_to_hex(&veilocity_pubkey);

        // Create wallet
        let mut wallet = Wallet::new(address, veilocity_pubkey_hex);

        // Encrypt and store keys
        wallet.encrypted_key = Some(encrypt_key(&key_bytes, password)?);
        wallet.encrypted_secret = Some(encrypt_key(veilocity_secret.as_bytes(), password)?);

        Ok((wallet, signer, veilocity_secret))
    }

    /// Save wallet to file
    pub fn save_wallet(&self, wallet: &Wallet) -> Result<()> {
        self.config.ensure_data_dir()?;

        let wallet_json = serde_json::to_string_pretty(wallet)
            .context("Failed to serialize wallet")?;

        fs::write(self.config.wallet_path(), wallet_json)
            .context("Failed to write wallet file")?;

        Ok(())
    }

    /// Load wallet from file
    pub fn load_wallet(&self) -> Result<Wallet> {
        let wallet_path = self.config.wallet_path();

        if !wallet_path.exists() {
            return Err(anyhow!("Wallet not found. Run 'veilocity init' first."));
        }

        let wallet_json = fs::read_to_string(&wallet_path)
            .context("Failed to read wallet file")?;

        let wallet: Wallet = serde_json::from_str(&wallet_json)
            .context("Failed to parse wallet file")?;

        Ok(wallet)
    }

    /// Unlock wallet and get signer
    pub fn unlock(&self, wallet: &Wallet, password: &str) -> Result<PrivateKeySigner> {
        let encrypted_key = wallet
            .encrypted_key
            .as_ref()
            .ok_or_else(|| anyhow!("No encrypted key in wallet"))?;

        let key_bytes = decrypt_key(encrypted_key, password)?;

        if key_bytes.len() != 32 {
            return Err(anyhow!("Invalid key length"));
        }

        let mut key_arr = [0u8; 32];
        key_arr.copy_from_slice(&key_bytes);

        PrivateKeySigner::from_bytes(&B256::from(key_arr))
            .context("Failed to create signer from key")
    }

    /// Get Veilocity secret from wallet
    pub fn get_veilocity_secret(&self, wallet: &Wallet, password: &str) -> Result<AccountSecret> {
        let encrypted_secret = wallet
            .encrypted_secret
            .as_ref()
            .ok_or_else(|| anyhow!("No encrypted secret in wallet"))?;

        let secret_bytes = decrypt_key(encrypted_secret, password)?;

        if secret_bytes.len() != 32 {
            return Err(anyhow!("Invalid secret length"));
        }

        let mut secret_arr = [0u8; 32];
        secret_arr.copy_from_slice(&secret_bytes);

        Ok(AccountSecret::from_bytes(&secret_arr))
    }

    /// Check if wallet exists
    pub fn wallet_exists(&self) -> bool {
        self.config.wallet_path().exists()
    }

    /// Get wallet path
    pub fn wallet_path(&self) -> std::path::PathBuf {
        self.config.wallet_path()
    }
}

/// Simple key encryption (for demo purposes)
/// In production, use proper key derivation and encryption (e.g., scrypt + AES-GCM)
fn encrypt_key(key: &[u8], password: &str) -> Result<String> {
    // Simple XOR encryption with password hash (NOT SECURE - demo only)
    // In production, use proper encryption like AES-GCM with scrypt-derived key
    let password_hash = simple_hash(password.as_bytes());
    let encrypted: Vec<u8> = key
        .iter()
        .zip(password_hash.iter().cycle())
        .map(|(k, p)| k ^ p)
        .collect();

    Ok(hex::encode(encrypted))
}

/// Simple key decryption
fn decrypt_key(encrypted: &str, password: &str) -> Result<Vec<u8>> {
    let encrypted_bytes = hex::decode(encrypted)
        .context("Failed to decode encrypted key")?;

    let password_hash = simple_hash(password.as_bytes());
    let decrypted: Vec<u8> = encrypted_bytes
        .iter()
        .zip(password_hash.iter().cycle())
        .map(|(e, p)| e ^ p)
        .collect();

    Ok(decrypted)
}

/// Simple hash function (for demo purposes)
fn simple_hash(data: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let h1 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    h1.hash(&mut hasher);
    let h2 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    h2.hash(&mut hasher);
    let h3 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    h3.hash(&mut hasher);
    let h4 = hasher.finish();

    let mut result = [0u8; 32];
    result[..8].copy_from_slice(&h1.to_le_bytes());
    result[8..16].copy_from_slice(&h2.to_le_bytes());
    result[16..24].copy_from_slice(&h3.to_le_bytes());
    result[24..32].copy_from_slice(&h4.to_le_bytes());

    result
}

/// Format ETH amount for display
pub fn format_eth(wei: u128) -> String {
    let eth = wei as f64 / 1e18;
    if eth < 0.0001 {
        format!("{} wei", wei)
    } else {
        format!("{:.6} ETH", eth)
    }
}

/// Parse ETH amount to wei
pub fn parse_eth(amount: f64) -> u128 {
    (amount * 1e18) as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [1u8; 32];
        let password = "test_password";

        let encrypted = encrypt_key(&key, password).unwrap();
        let decrypted = decrypt_key(&encrypted, password).unwrap();

        assert_eq!(decrypted, key);
    }

    #[test]
    fn test_format_eth() {
        assert_eq!(format_eth(1_000_000_000_000_000_000), "1.000000 ETH");
        assert_eq!(format_eth(500_000_000_000_000_000), "0.500000 ETH");
        assert!(format_eth(100).contains("wei"));
    }

    #[test]
    fn test_parse_eth() {
        assert_eq!(parse_eth(1.0), 1_000_000_000_000_000_000);
        assert_eq!(parse_eth(0.5), 500_000_000_000_000_000);
    }
}
