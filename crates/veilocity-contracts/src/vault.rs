//! VeilocityVault contract interactions
//!
//! This module provides high-level functions for interacting with the VeilocityVault contract.

use crate::bindings::IVeilocityVault;
use crate::error::ContractError;
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use std::sync::Arc;
use tracing::{debug, info};

/// VeilocityVault client for contract interactions
pub struct VaultClient<P> {
    /// Contract address
    address: Address,
    /// Provider
    provider: Arc<P>,
}

impl<P: Provider + Clone> VaultClient<P> {
    /// Create a new vault client with a provider
    pub fn with_provider(provider: P, contract_address: Address) -> Self {
        Self {
            address: contract_address,
            provider: Arc::new(provider),
        }
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the current state root
    pub async fn current_root(&self) -> Result<B256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .currentRoot()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Get the deposit count
    pub async fn deposit_count(&self) -> Result<U256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .depositCount()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Get total value locked
    pub async fn total_value_locked(&self) -> Result<U256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .totalValueLocked()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Check if a root is valid
    pub async fn is_valid_root(&self, root: B256) -> Result<bool, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .isValidRoot(root)
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Check if a nullifier has been used
    pub async fn is_nullifier_used(&self, nullifier: B256) -> Result<bool, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .isNullifierUsed(nullifier)
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Get minimum deposit amount
    pub async fn min_deposit(&self) -> Result<U256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .MIN_DEPOSIT()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Deposit funds into Veilocity
    pub async fn deposit(&self, commitment: B256, amount: U256) -> Result<B256, ContractError> {
        info!("Depositing {} wei with commitment {:?}", amount, commitment);

        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let tx = contract
            .deposit(commitment)
            .value(amount)
            .send()
            .await
            .map_err(|e| ContractError::TransactionFailed(e.to_string()))?;

        debug!("Transaction sent, waiting for confirmation...");

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| ContractError::TransactionFailed(e.to_string()))?;

        if !receipt.status() {
            return Err(ContractError::TransactionReverted(
                "Deposit transaction reverted".to_string(),
            ));
        }

        info!("Deposit confirmed in tx {:?}", receipt.transaction_hash);
        Ok(receipt.transaction_hash)
    }

    /// Withdraw funds from Veilocity
    pub async fn withdraw(
        &self,
        nullifier: B256,
        recipient: Address,
        amount: U256,
        root: B256,
        proof: Vec<u8>,
    ) -> Result<B256, ContractError> {
        info!(
            "Withdrawing {} wei to {:?} with nullifier {:?}",
            amount, recipient, nullifier
        );

        // Check nullifier hasn't been used
        if self.is_nullifier_used(nullifier).await? {
            return Err(ContractError::NullifierUsed(format!("{:?}", nullifier)));
        }

        // Check root is valid
        if !self.is_valid_root(root).await? {
            return Err(ContractError::InvalidRoot);
        }

        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let tx = contract
            .withdraw(nullifier, recipient, amount, root, Bytes::from(proof))
            .send()
            .await
            .map_err(|e| ContractError::TransactionFailed(e.to_string()))?;

        debug!("Transaction sent, waiting for confirmation...");

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| ContractError::TransactionFailed(e.to_string()))?;

        if !receipt.status() {
            return Err(ContractError::TransactionReverted(
                "Withdrawal transaction reverted".to_string(),
            ));
        }

        info!("Withdrawal confirmed in tx {:?}", receipt.transaction_hash);
        Ok(receipt.transaction_hash)
    }

    /// Update state root (admin only)
    pub async fn update_state_root(
        &self,
        new_root: B256,
        proof: Vec<u8>,
    ) -> Result<B256, ContractError> {
        info!("Updating state root to {:?}", new_root);

        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let tx = contract
            .updateStateRoot(new_root, Bytes::from(proof))
            .send()
            .await
            .map_err(|e| ContractError::TransactionFailed(e.to_string()))?;

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| ContractError::TransactionFailed(e.to_string()))?;

        if !receipt.status() {
            return Err(ContractError::TransactionReverted(
                "State root update reverted".to_string(),
            ));
        }

        info!("State root updated in tx {:?}", receipt.transaction_hash);
        Ok(receipt.transaction_hash)
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64, ContractError> {
        let block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| ContractError::Rpc(e.to_string()))?;

        Ok(block)
    }

    /// Get balance of an address
    pub async fn get_balance(&self, address: Address) -> Result<U256, ContractError> {
        let balance = self
            .provider
            .get_balance(address)
            .await
            .map_err(|e| ContractError::Rpc(e.to_string()))?;

        Ok(balance)
    }
}

/// Create a vault client with HTTP provider and wallet
pub async fn create_vault_client(
    rpc_url: &str,
    contract_address: Address,
    signer: PrivateKeySigner,
) -> Result<VaultClient<impl Provider + Clone>, ContractError> {
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(rpc_url.parse().map_err(|e| ContractError::Rpc(format!("{}", e)))?);

    Ok(VaultClient::with_provider(provider, contract_address))
}

/// Read-only vault client (no signer required)
pub struct VaultReader<P> {
    /// Provider
    provider: Arc<P>,
    /// Contract address
    address: Address,
}

impl<P: Provider + Clone> VaultReader<P> {
    /// Create with a provider
    pub fn with_provider(provider: P, contract_address: Address) -> Self {
        Self {
            provider: Arc::new(provider),
            address: contract_address,
        }
    }

    /// Get the current state root
    pub async fn current_root(&self) -> Result<B256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .currentRoot()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Get the deposit count
    pub async fn deposit_count(&self) -> Result<U256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .depositCount()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Get total value locked
    pub async fn total_value_locked(&self) -> Result<U256, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .totalValueLocked()
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Check if a root is valid
    pub async fn is_valid_root(&self, root: B256) -> Result<bool, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .isValidRoot(root)
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Check if a nullifier has been used
    pub async fn is_nullifier_used(&self, nullifier: B256) -> Result<bool, ContractError> {
        let contract = IVeilocityVault::new(self.address, &*self.provider);
        let result = contract
            .isNullifierUsed(nullifier)
            .call()
            .await
            .map_err(|e| ContractError::ContractCall(e.to_string()))?;

        Ok(result)
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64, ContractError> {
        let block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| ContractError::Rpc(e.to_string()))?;

        Ok(block)
    }
}

/// Create a read-only vault client with HTTP provider
pub fn create_vault_reader(
    rpc_url: &str,
    contract_address: Address,
) -> Result<VaultReader<impl Provider + Clone>, ContractError> {
    let provider = ProviderBuilder::new()
        .connect_http(rpc_url.parse().map_err(|e| ContractError::Rpc(format!("{}", e)))?);

    Ok(VaultReader::with_provider(provider, contract_address))
}
