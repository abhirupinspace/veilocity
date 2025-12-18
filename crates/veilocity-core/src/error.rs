use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    #[error("Nullifier already used: {0}")]
    NullifierUsed(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid field element")]
    InvalidFieldElement,

    #[error("Tree full")]
    TreeFull,

    #[error("Invalid secret key")]
    InvalidSecretKey,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
