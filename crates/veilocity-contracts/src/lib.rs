//! Veilocity Contracts
//!
//! ABI bindings and interaction with Mantle smart contracts.
//!
//! This crate provides:
//! - ABI bindings for VeilocityVault and Verifier contracts
//! - High-level client for contract interactions
//! - Event parsing and indexing utilities
//!
//! # Usage
//!
//! ```ignore
//! use veilocity_contracts::{VaultClient, ContractError};
//! use alloy::primitives::{Address, B256, U256};
//!
//! let client = VaultClient::new(rpc_url, contract_address, signer).await?;
//!
//! // Deposit
//! let tx_hash = client.deposit(commitment, amount).await?;
//!
//! // Withdraw
//! let tx_hash = client.withdraw(nullifier, recipient, amount, root, proof).await?;
//! ```

pub mod anchor;
pub mod bindings;
pub mod error;
pub mod events;
pub mod vault;

pub use anchor::{StateRootEntry, StateRootHistory};
pub use bindings::{IVeilocityVault, IVerifier};
pub use error::ContractError;
pub use events::{DepositEvent, EventFilter, StateRootUpdatedEvent, VeilocityEvent, WithdrawalEvent};
pub use vault::{create_vault_client, create_vault_reader, VaultClient, VaultReader};
