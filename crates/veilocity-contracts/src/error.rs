//! Error types for contract interactions

use thiserror::Error;

/// Errors that can occur during contract interactions
#[derive(Error, Debug)]
pub enum ContractError {
    /// RPC error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Contract call failed
    #[error("Contract call failed: {0}")]
    ContractCall(String),

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Transaction reverted
    #[error("Transaction reverted: {0}")]
    TransactionReverted(String),

    /// Invalid address
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid proof
    #[error("Invalid proof")]
    InvalidProof,

    /// Nullifier already used
    #[error("Nullifier already used: {0}")]
    NullifierUsed(String),

    /// Invalid root
    #[error("Invalid state root")]
    InvalidRoot,

    /// Insufficient balance
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    /// Contract not deployed
    #[error("Contract not deployed at {0}")]
    ContractNotDeployed(String),

    /// Event parsing error
    #[error("Failed to parse event: {0}")]
    EventParsing(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Timeout waiting for transaction
    #[error("Transaction timeout")]
    Timeout,

    /// Hex decoding error
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}
