//! Veilocity Core Library
//!
//! State management, Merkle trees, and account logic for the private execution layer.

pub mod poseidon;
pub mod merkle;
pub mod account;
pub mod state;
pub mod transaction;
pub mod error;

pub use error::CoreError;
pub use poseidon::PoseidonHasher;
pub use merkle::MerkleTree;
pub use account::PrivateAccount;
pub use state::StateManager;
