//! Incremental Merkle Tree implementation
//!
//! Depth 20 supports ~1M accounts. Uses Poseidon hash.

use crate::error::CoreError;
use crate::poseidon::{FieldElement, PoseidonHasher};
use std::collections::HashMap;

/// Tree depth (supports 2^20 = ~1M leaves)
pub const TREE_DEPTH: usize = 20;

/// Maximum number of leaves
pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH;

/// Merkle tree with incremental updates
pub struct MerkleTree {
    /// Current number of leaves
    leaf_count: u64,

    /// Precomputed empty subtree hashes at each level
    empty_hashes: [FieldElement; TREE_DEPTH + 1],

    /// Sparse storage: level -> index -> hash
    /// Only stores non-empty nodes
    nodes: Vec<HashMap<u64, FieldElement>>,

    /// Current root
    root: FieldElement,

    /// Hasher instance
    hasher: PoseidonHasher,
}

impl MerkleTree {
    /// Create a new empty Merkle tree
    pub fn new() -> Self {
        let mut hasher = PoseidonHasher::new();

        // Precomputed empty subtree hashes
        // empty_hashes[0] = hash of empty leaf (hash2(0, 0))
        // empty_hashes[i] = hash(empty_hashes[i-1], empty_hashes[i-1])
        let mut empty_hashes = [FieldElement::from(0u64); TREE_DEPTH + 1];
        empty_hashes[0] = hasher.hash2(&FieldElement::from(0u64), &FieldElement::from(0u64));

        for i in 1..=TREE_DEPTH {
            empty_hashes[i] = hasher.hash2(&empty_hashes[i - 1], &empty_hashes[i - 1]);
        }

        let root = empty_hashes[TREE_DEPTH];

        // Initialize node storage for each level
        let nodes = (0..=TREE_DEPTH).map(|_| HashMap::new()).collect();

        Self {
            leaf_count: 0,
            empty_hashes,
            nodes,
            root,
            hasher,
        }
    }

    /// Get the current root
    pub fn root(&self) -> FieldElement {
        self.root
    }

    /// Get the current leaf count
    pub fn leaf_count(&self) -> u64 {
        self.leaf_count
    }

    /// Get the empty hash at a given level
    pub fn empty_hash(&self, level: usize) -> FieldElement {
        self.empty_hashes[level]
    }

    /// Insert a new leaf and return its index
    pub fn insert(&mut self, leaf: FieldElement) -> Result<u64, CoreError> {
        if self.leaf_count >= MAX_LEAVES {
            return Err(CoreError::TreeFull);
        }

        let index = self.leaf_count;
        self.update_leaf(index, leaf)?;
        self.leaf_count += 1;

        Ok(index)
    }

    /// Update a leaf at the given index
    pub fn update_leaf(&mut self, index: u64, leaf: FieldElement) -> Result<(), CoreError> {
        // Store the leaf at level 0
        self.nodes[0].insert(index, leaf);

        // Update path from leaf to root
        let mut current_index = index;
        let mut current_hash = leaf;

        for level in 0..TREE_DEPTH {
            // Determine sibling index
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            // Get sibling hash (or empty hash if not present)
            let sibling_hash = self.nodes[level]
                .get(&sibling_index)
                .copied()
                .unwrap_or(self.empty_hashes[level]);

            // Compute parent hash
            let parent_hash = if current_index % 2 == 0 {
                self.hasher.hash2(&current_hash, &sibling_hash)
            } else {
                self.hasher.hash2(&sibling_hash, &current_hash)
            };

            // Move to parent level
            let parent_index = current_index / 2;
            self.nodes[level + 1].insert(parent_index, parent_hash);

            current_index = parent_index;
            current_hash = parent_hash;
        }

        self.root = current_hash;
        Ok(())
    }

    /// Get the Merkle proof (path) for a leaf at the given index
    pub fn get_proof(&self, index: u64) -> Vec<FieldElement> {
        let mut proof = Vec::with_capacity(TREE_DEPTH);
        let mut current_index = index;

        for level in 0..TREE_DEPTH {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            let sibling_hash = self.nodes[level]
                .get(&sibling_index)
                .copied()
                .unwrap_or(self.empty_hashes[level]);

            proof.push(sibling_hash);
            current_index /= 2;
        }

        proof
    }

    /// Get the leaf value at the given index
    pub fn get_leaf(&self, index: u64) -> Option<FieldElement> {
        self.nodes[0].get(&index).copied()
    }

    /// Verify a Merkle proof
    pub fn verify_proof(
        &mut self,
        leaf: FieldElement,
        index: u64,
        proof: &[FieldElement],
        root: FieldElement,
    ) -> bool {
        if proof.len() != TREE_DEPTH {
            return false;
        }

        let mut current_hash = leaf;
        let mut current_index = index;

        for sibling in proof.iter() {
            current_hash = if current_index % 2 == 0 {
                self.hasher.hash2(&current_hash, sibling)
            } else {
                self.hasher.hash2(sibling, &current_hash)
            };
            current_index /= 2;
        }

        current_hash == root
    }

    /// Static verification (without mutating hasher)
    pub fn verify_proof_static(
        hasher: &mut PoseidonHasher,
        leaf: FieldElement,
        index: u64,
        proof: &[FieldElement],
        root: FieldElement,
    ) -> bool {
        if proof.len() != TREE_DEPTH {
            return false;
        }

        let mut current_hash = leaf;
        let mut current_index = index;

        for sibling in proof.iter() {
            current_hash = if current_index % 2 == 0 {
                hasher.hash2(&current_hash, sibling)
            } else {
                hasher.hash2(sibling, &current_hash)
            };
            current_index /= 2;
        }

        current_hash == root
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new();
        assert_eq!(tree.leaf_count(), 0);
        assert_ne!(tree.root(), FieldElement::from(0u64));
    }

    #[test]
    fn test_insert_and_proof() {
        let mut tree = MerkleTree::new();

        let leaf = FieldElement::from(12345u64);
        let index = tree.insert(leaf).unwrap();

        assert_eq!(index, 0);
        assert_eq!(tree.leaf_count(), 1);

        let proof = tree.get_proof(index);
        assert_eq!(proof.len(), TREE_DEPTH);

        assert!(tree.verify_proof(leaf, index, &proof, tree.root()));
    }

    #[test]
    fn test_multiple_inserts() {
        let mut tree = MerkleTree::new();

        for i in 0..10 {
            let leaf = FieldElement::from(i as u64);
            let index = tree.insert(leaf).unwrap();
            assert_eq!(index, i);
        }

        // Verify all proofs
        for i in 0..10 {
            let leaf = FieldElement::from(i as u64);
            let proof = tree.get_proof(i);
            assert!(tree.verify_proof(leaf, i, &proof, tree.root()));
        }
    }

    #[test]
    fn test_update_leaf() {
        let mut tree = MerkleTree::new();

        let leaf1 = FieldElement::from(100u64);
        let index = tree.insert(leaf1).unwrap();
        let root1 = tree.root();

        let leaf2 = FieldElement::from(200u64);
        tree.update_leaf(index, leaf2).unwrap();
        let root2 = tree.root();

        assert_ne!(root1, root2);

        let proof = tree.get_proof(index);
        assert!(tree.verify_proof(leaf2, index, &proof, root2));
        assert!(!tree.verify_proof(leaf1, index, &proof, root2));
    }

    #[test]
    fn test_invalid_proof() {
        let mut tree = MerkleTree::new();

        let leaf = FieldElement::from(12345u64);
        tree.insert(leaf).unwrap();

        let proof = tree.get_proof(0);

        // Wrong leaf
        let wrong_leaf = FieldElement::from(99999u64);
        assert!(!tree.verify_proof(wrong_leaf, 0, &proof, tree.root()));

        // Wrong index
        assert!(!tree.verify_proof(leaf, 1, &proof, tree.root()));
    }

    #[test]
    fn test_root_changes_on_insert() {
        let mut tree = MerkleTree::new();
        let root0 = tree.root();

        tree.insert(FieldElement::from(1u64)).unwrap();
        let root1 = tree.root();
        assert_ne!(root0, root1);

        tree.insert(FieldElement::from(2u64)).unwrap();
        let root2 = tree.root();
        assert_ne!(root1, root2);
    }
}
