//! State management for the private execution layer

use crate::account::{AccountSecret, PrivateAccount};
use crate::error::CoreError;
use crate::merkle::MerkleTree;
use crate::poseidon::{bytes_to_field, u128_to_field, FieldElement, PoseidonHasher};
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;

/// State manager for Veilocity
pub struct StateManager {
    /// SQLite connection for persistent storage
    db: Connection,
    /// In-memory Merkle tree
    tree: MerkleTree,
    /// Poseidon hasher
    hasher: PoseidonHasher,
    /// Set of used nullifiers (in-memory cache)
    used_nullifiers: HashSet<[u8; 32]>,
}

impl StateManager {
    /// Create a new state manager with a database path
    pub fn new(db_path: &Path) -> Result<Self, CoreError> {
        let db = Connection::open(db_path)?;
        Self::init_db(&db)?;

        let mut manager = Self {
            db,
            tree: MerkleTree::new(),
            hasher: PoseidonHasher::new(),
            used_nullifiers: HashSet::new(),
        };

        // Load existing state from database
        manager.load_state()?;

        Ok(manager)
    }

    /// Create an in-memory state manager (for testing)
    pub fn in_memory() -> Result<Self, CoreError> {
        let db = Connection::open_in_memory()?;
        Self::init_db(&db)?;

        Ok(Self {
            db,
            tree: MerkleTree::new(),
            hasher: PoseidonHasher::new(),
            used_nullifiers: HashSet::new(),
        })
    }

    /// Initialize database schema
    fn init_db(db: &Connection) -> Result<(), CoreError> {
        db.execute_batch(
            "
            -- Accounts table
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY,
                pubkey BLOB NOT NULL UNIQUE,
                balance_encrypted BLOB NOT NULL,
                nonce INTEGER NOT NULL DEFAULT 0,
                leaf_index INTEGER NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Transactions table
            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                tx_type TEXT NOT NULL,
                nullifier BLOB,
                data BLOB NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Nullifiers table (for double-spend prevention)
            CREATE TABLE IF NOT EXISTS nullifiers (
                nullifier BLOB PRIMARY KEY,
                created_at INTEGER NOT NULL
            );

            -- Sync state
            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL
            );

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_accounts_leaf_index ON accounts(leaf_index);
            CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
            ",
        )?;

        Ok(())
    }

    /// Load state from database
    fn load_state(&mut self) -> Result<(), CoreError> {
        // Load accounts and rebuild Merkle tree
        let mut stmt = self
            .db
            .prepare("SELECT pubkey, balance_encrypted, nonce, leaf_index FROM accounts ORDER BY leaf_index")?;

        let accounts = stmt.query_map([], |row| {
            let pubkey: Vec<u8> = row.get(0)?;
            let balance_bytes: Vec<u8> = row.get(1)?;
            let nonce: u64 = row.get(2)?;
            let leaf_index: u64 = row.get(3)?;
            Ok((pubkey, balance_bytes, nonce, leaf_index))
        })?;

        for account in accounts {
            let (pubkey, balance_bytes, nonce, leaf_index) = account?;

            // Decode balance (stored as little-endian u128)
            let balance = if balance_bytes.len() >= 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&balance_bytes[..16]);
                u128::from_le_bytes(bytes)
            } else {
                0
            };

            // Reconstruct account and compute leaf
            let mut pubkey_arr = [0u8; 32];
            if pubkey.len() == 32 {
                pubkey_arr.copy_from_slice(&pubkey);
            }

            let pubkey_field = bytes_to_field(&pubkey_arr);
            let balance_field = u128_to_field(balance);
            let nonce_field = FieldElement::from(nonce);

            let leaf = self
                .hasher
                .compute_leaf(&pubkey_field, &balance_field, &nonce_field);

            // Insert into tree at correct index
            while self.tree.leaf_count() < leaf_index {
                // Insert empty leaves to reach the correct index
                let empty = self.hasher.hash2(&FieldElement::from(0u64), &FieldElement::from(0u64));
                self.tree.insert(empty)?;
            }
            self.tree.insert(leaf)?;
        }

        // Load nullifiers
        let mut stmt = self.db.prepare("SELECT nullifier FROM nullifiers")?;
        let nullifiers = stmt.query_map([], |row| {
            let nullifier: Vec<u8> = row.get(0)?;
            Ok(nullifier)
        })?;

        for nullifier in nullifiers {
            let nullifier = nullifier?;
            if nullifier.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&nullifier);
                self.used_nullifiers.insert(arr);
            }
        }

        Ok(())
    }

    /// Get the current state root
    pub fn state_root(&self) -> FieldElement {
        self.tree.root()
    }

    /// Get the current leaf count
    pub fn leaf_count(&self) -> u64 {
        self.tree.leaf_count()
    }

    /// Check if a nullifier has been used
    pub fn is_nullifier_used(&self, nullifier: &[u8; 32]) -> bool {
        self.used_nullifiers.contains(nullifier)
    }

    /// Mark a nullifier as used
    pub fn mark_nullifier_used(&mut self, nullifier: &[u8; 32]) -> Result<(), CoreError> {
        if self.used_nullifiers.contains(nullifier) {
            return Err(CoreError::NullifierUsed(hex::encode(nullifier)));
        }

        self.used_nullifiers.insert(*nullifier);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.db.execute(
            "INSERT INTO nullifiers (nullifier, created_at) VALUES (?1, ?2)",
            params![nullifier.as_slice(), now as i64],
        )?;

        Ok(())
    }

    /// Create a new account and insert into the tree
    pub fn create_account(
        &mut self,
        secret: &AccountSecret,
        initial_balance: u128,
    ) -> Result<PrivateAccount, CoreError> {
        let index = self.tree.leaf_count();
        let account = PrivateAccount::with_balance(&mut self.hasher, secret.secret(), index, initial_balance);

        // Compute leaf and insert into tree
        let leaf = account.compute_leaf(&mut self.hasher);
        self.tree.insert(leaf)?;

        // Store in database
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let balance_bytes = initial_balance.to_le_bytes();

        self.db.execute(
            "INSERT INTO accounts (pubkey, balance_encrypted, nonce, leaf_index, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                account.pubkey.as_slice(),
                balance_bytes.as_slice(),
                account.nonce as i64,
                account.index as i64,
                now as i64,
                now as i64,
            ],
        )?;

        Ok(account)
    }

    /// Get an account by public key
    pub fn get_account(&self, pubkey: &[u8; 32]) -> Result<Option<PrivateAccount>, CoreError> {
        let mut stmt = self.db.prepare(
            "SELECT pubkey, balance_encrypted, nonce, leaf_index FROM accounts WHERE pubkey = ?1",
        )?;

        let mut rows = stmt.query(params![pubkey.as_slice()])?;

        if let Some(row) = rows.next()? {
            let pubkey_bytes: Vec<u8> = row.get(0)?;
            let balance_bytes: Vec<u8> = row.get(1)?;
            let nonce: i64 = row.get(2)?;
            let leaf_index: i64 = row.get(3)?;

            let mut pubkey_arr = [0u8; 32];
            if pubkey_bytes.len() == 32 {
                pubkey_arr.copy_from_slice(&pubkey_bytes);
            }

            let balance = if balance_bytes.len() >= 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&balance_bytes[..16]);
                u128::from_le_bytes(bytes)
            } else {
                0
            };

            Ok(Some(PrivateAccount {
                pubkey: pubkey_arr,
                balance,
                nonce: nonce as u64,
                index: leaf_index as u64,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get an account by leaf index
    pub fn get_account_by_index(&self, index: u64) -> Result<Option<PrivateAccount>, CoreError> {
        let mut stmt = self.db.prepare(
            "SELECT pubkey, balance_encrypted, nonce, leaf_index FROM accounts WHERE leaf_index = ?1",
        )?;

        let mut rows = stmt.query(params![index as i64])?;

        if let Some(row) = rows.next()? {
            let pubkey_bytes: Vec<u8> = row.get(0)?;
            let balance_bytes: Vec<u8> = row.get(1)?;
            let nonce: i64 = row.get(2)?;
            let leaf_index: i64 = row.get(3)?;

            let mut pubkey_arr = [0u8; 32];
            if pubkey_bytes.len() == 32 {
                pubkey_arr.copy_from_slice(&pubkey_bytes);
            }

            let balance = if balance_bytes.len() >= 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&balance_bytes[..16]);
                u128::from_le_bytes(bytes)
            } else {
                0
            };

            Ok(Some(PrivateAccount {
                pubkey: pubkey_arr,
                balance,
                nonce: nonce as u64,
                index: leaf_index as u64,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update an account balance and nonce
    pub fn update_account(&mut self, account: &PrivateAccount) -> Result<(), CoreError> {
        // Update in database
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let balance_bytes = account.balance.to_le_bytes();

        self.db.execute(
            "UPDATE accounts SET balance_encrypted = ?1, nonce = ?2, updated_at = ?3 WHERE leaf_index = ?4",
            params![
                balance_bytes.as_slice(),
                account.nonce as i64,
                now as i64,
                account.index as i64,
            ],
        )?;

        // Update Merkle tree
        let leaf = account.compute_leaf(&mut self.hasher);
        self.tree.update_leaf(account.index, leaf)?;

        Ok(())
    }

    /// Get Merkle proof for an account
    pub fn get_merkle_proof(&self, index: u64) -> Vec<FieldElement> {
        self.tree.get_proof(index)
    }

    /// Get a mutable reference to the hasher
    pub fn hasher(&mut self) -> &mut PoseidonHasher {
        &mut self.hasher
    }

    /// Get a reference to the Merkle tree
    pub fn tree(&self) -> &MerkleTree {
        &self.tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::in_memory().unwrap();
        assert_eq!(manager.leaf_count(), 0);
    }

    #[test]
    fn test_create_account() {
        let mut manager = StateManager::in_memory().unwrap();
        let secret = AccountSecret::generate();

        let account = manager.create_account(&secret, 0).unwrap();

        assert_eq!(account.index, 0);
        assert_eq!(account.balance, 0);
        assert_eq!(manager.leaf_count(), 1);
    }

    #[test]
    fn test_account_persistence() {
        let mut manager = StateManager::in_memory().unwrap();
        let secret = AccountSecret::generate();

        let account = manager.create_account(&secret, 1_000_000_000).unwrap();
        let pubkey = account.pubkey;

        let retrieved = manager.get_account(&pubkey).unwrap().unwrap();
        assert_eq!(retrieved.balance, 1_000_000_000);
        assert_eq!(retrieved.index, account.index);
    }

    #[test]
    fn test_nullifier_tracking() {
        let mut manager = StateManager::in_memory().unwrap();
        let nullifier = [1u8; 32];

        assert!(!manager.is_nullifier_used(&nullifier));

        manager.mark_nullifier_used(&nullifier).unwrap();
        assert!(manager.is_nullifier_used(&nullifier));

        // Double-use should fail
        assert!(manager.mark_nullifier_used(&nullifier).is_err());
    }

    #[test]
    fn test_account_update() {
        let mut manager = StateManager::in_memory().unwrap();
        let secret = AccountSecret::generate();

        let mut account = manager.create_account(&secret, 1_000_000_000).unwrap();
        let root1 = manager.state_root();

        account.balance = 2_000_000_000;
        manager.update_account(&account).unwrap();
        let root2 = manager.state_root();

        assert_ne!(root1, root2);

        let retrieved = manager.get_account(&account.pubkey).unwrap().unwrap();
        assert_eq!(retrieved.balance, 2_000_000_000);
    }
}
