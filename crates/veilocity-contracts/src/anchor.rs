//! State anchor functionality
//!
//! This module provides utilities for state root management and anchoring.
//! In the current implementation, state anchoring is handled by VeilocityVault.

use alloy::primitives::B256;
use serde::{Deserialize, Serialize};

/// State root history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRootEntry {
    /// The state root
    pub root: B256,
    /// Block number when this root was set
    pub block_number: u64,
    /// Batch index
    pub batch_index: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl StateRootEntry {
    /// Create a new state root entry
    pub fn new(root: B256, block_number: u64, batch_index: u64, timestamp: u64) -> Self {
        Self {
            root,
            block_number,
            batch_index,
            timestamp,
        }
    }

    /// Get root as hex string
    pub fn root_hex(&self) -> String {
        format!("0x{}", hex::encode(self.root))
    }
}

/// State root history manager
#[derive(Debug, Default)]
pub struct StateRootHistory {
    /// History of state roots
    entries: Vec<StateRootEntry>,
    /// Maximum history size
    max_size: usize,
}

impl StateRootHistory {
    /// Create a new history with default size (100)
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_size: 100,
        }
    }

    /// Create with custom max size
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a new state root to history
    pub fn add(&mut self, entry: StateRootEntry) {
        if self.entries.len() >= self.max_size {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Get the latest state root
    pub fn latest(&self) -> Option<&StateRootEntry> {
        self.entries.last()
    }

    /// Get the latest root hash
    pub fn latest_root(&self) -> Option<B256> {
        self.entries.last().map(|e| e.root)
    }

    /// Check if a root exists in history
    pub fn contains(&self, root: &B256) -> bool {
        self.entries.iter().any(|e| &e.root == root)
    }

    /// Get entry by root
    pub fn get(&self, root: &B256) -> Option<&StateRootEntry> {
        self.entries.iter().find(|e| &e.root == root)
    }

    /// Get all entries
    pub fn all(&self) -> &[StateRootEntry] {
        &self.entries
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_root_history() {
        let mut history = StateRootHistory::with_capacity(3);

        let root1 = B256::from([1u8; 32]);
        let root2 = B256::from([2u8; 32]);
        let root3 = B256::from([3u8; 32]);
        let root4 = B256::from([4u8; 32]);

        history.add(StateRootEntry::new(root1, 100, 0, 1000));
        history.add(StateRootEntry::new(root2, 200, 1, 2000));
        history.add(StateRootEntry::new(root3, 300, 2, 3000));

        assert_eq!(history.len(), 3);
        assert!(history.contains(&root1));
        assert!(history.contains(&root2));
        assert!(history.contains(&root3));

        // Adding one more should evict the oldest
        history.add(StateRootEntry::new(root4, 400, 3, 4000));

        assert_eq!(history.len(), 3);
        assert!(!history.contains(&root1)); // Evicted
        assert!(history.contains(&root2));
        assert!(history.contains(&root3));
        assert!(history.contains(&root4));

        assert_eq!(history.latest_root(), Some(root4));
    }
}
