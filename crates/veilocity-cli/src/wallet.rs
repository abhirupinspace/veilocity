//! Wallet management for Veilocity CLI
//!
//! Handles key generation, storage, and account management.
//! Uses AES-256-GCM for encryption with Argon2id for key derivation.

use crate::config::Config;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use alloy::primitives::{Address, B256};
use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Context, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fs;
use veilocity_core::account::AccountSecret;
use veilocity_core::poseidon::{field_to_hex, PoseidonHasher};
use zeroize::{Zeroize, ZeroizeOnDrop};

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

/// Encrypted data format: salt (22 bytes base64) + nonce (12 bytes) + ciphertext
/// Salt is stored as base64 PHC string prefix, nonce and ciphertext as hex
#[derive(Zeroize, ZeroizeOnDrop)]
struct DerivedKey([u8; 32]);

/// Derive encryption key from password using Argon2id
fn derive_key(password: &str, salt: &SaltString) -> Result<DerivedKey> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

    let hash_output = password_hash
        .hash
        .ok_or_else(|| anyhow!("No hash output from Argon2"))?;

    let hash_bytes = hash_output.as_bytes();
    if hash_bytes.len() < 32 {
        return Err(anyhow!("Hash output too short"));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);
    Ok(DerivedKey(key))
}

/// Encrypt key using AES-256-GCM with Argon2id key derivation
fn encrypt_key(key: &[u8], password: &str) -> Result<String> {
    // Generate random salt and nonce
    let salt = SaltString::generate(&mut OsRng);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Derive encryption key
    let derived_key = derive_key(password, &salt)?;

    // Encrypt with AES-GCM
    let cipher = Aes256Gcm::new_from_slice(&derived_key.0)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, key)
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Format: salt$nonce$ciphertext (all hex except salt which is already base64)
    Ok(format!(
        "{}${}${}",
        salt.as_str(),
        hex::encode(nonce_bytes),
        hex::encode(ciphertext)
    ))
}

/// Decrypt key using AES-256-GCM with Argon2id key derivation
fn decrypt_key(encrypted: &str, password: &str) -> Result<Vec<u8>> {
    let parts: Vec<&str> = encrypted.split('$').collect();
    if parts.len() != 3 {
        return Err(anyhow!("Invalid encrypted key format"));
    }

    let salt = SaltString::from_b64(parts[0])
        .map_err(|e| anyhow!("Invalid salt: {}", e))?;
    let nonce_bytes = hex::decode(parts[1]).context("Invalid nonce")?;
    let ciphertext = hex::decode(parts[2]).context("Invalid ciphertext")?;

    if nonce_bytes.len() != 12 {
        return Err(anyhow!("Invalid nonce length"));
    }

    // Derive encryption key
    let derived_key = derive_key(password, &salt)?;

    // Decrypt with AES-GCM
    let cipher = Aes256Gcm::new_from_slice(&derived_key.0)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Decryption failed - wrong password or corrupted data"))?;

    Ok(plaintext)
}

/// Format MNT amount for display
pub fn format_mnt(wei: u128) -> String {
    let mnt = wei as f64 / 1e18;
    if mnt < 0.0001 {
        format!("{} wei", wei)
    } else {
        format!("{:.6} MNT", mnt)
    }
}

/// Parse MNT amount to wei
pub fn parse_mnt(amount: f64) -> u128 {
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
    fn test_wrong_password_fails() {
        let key = [1u8; 32];
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_key(&key, password).unwrap();
        let result = decrypt_key(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_format_mnt() {
        assert_eq!(format_mnt(1_000_000_000_000_000_000), "1.000000 MNT");
        assert_eq!(format_mnt(500_000_000_000_000_000), "0.500000 MNT");
        assert!(format_mnt(100).contains("wei"));
    }

    #[test]
    fn test_parse_mnt() {
        assert_eq!(parse_mnt(1.0), 1_000_000_000_000_000_000);
        assert_eq!(parse_mnt(0.5), 500_000_000_000_000_000);
    }
}
