//! Error types for the prover crate

use thiserror::Error;

/// Errors that can occur during proof generation
#[derive(Error, Debug)]
pub enum ProverError {
    /// Witness generation failed
    #[error("Failed to generate witness: {0}")]
    WitnessGeneration(String),

    /// Proof generation failed
    #[error("Failed to generate proof: {0}")]
    ProofGeneration(String),

    /// Proof verification failed
    #[error("Proof verification failed: {0}")]
    ProofVerification(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Circuit not found
    #[error("Circuit not found: {0}")]
    CircuitNotFound(String),

    /// Command execution failed
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    /// Insufficient balance
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    /// Invalid Merkle proof
    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,

    /// Circuit compilation required
    #[error("Circuit needs to be compiled first. Run: cd circuits && nargo compile")]
    CircuitNotCompiled,
}
