//! Veilocity Prover
//!
//! Witness generation and proof creation using Noir/Barretenberg.
//!
//! This crate provides:
//! - Witness generation for deposit, withdrawal, and transfer circuits
//! - Proof generation using Barretenberg (`bb` CLI)
//! - Proof verification
//! - Solidity verifier generation
//!
//! # Usage
//!
//! ```ignore
//! use veilocity_prover::{NoirProver, DepositWitness};
//!
//! let prover = NoirProver::default_paths();
//!
//! // Create witness
//! let witness = DepositWitness::new(commitment, amount, secret);
//!
//! // Generate proof
//! let proof = prover.prove_deposit(&witness).await?;
//! ```

pub mod error;
pub mod prover;
pub mod witness;

pub use error::ProverError;
pub use prover::{CircuitType, NoirProver, Proof};
pub use witness::{DepositWitness, FullTransferWitness, TransferWitness, WithdrawWitness, TREE_DEPTH};
