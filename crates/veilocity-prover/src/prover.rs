//! Noir proof generation using Barretenberg
//!
//! This module handles proof generation by invoking the `bb` (Barretenberg) CLI tool.

use crate::error::ProverError;
use crate::witness::{DepositWitness, TransferWitness, WithdrawWitness};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tracing::{debug, info};

/// Circuit types supported by the prover
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitType {
    Deposit,
    Withdraw,
    Transfer,
}

impl CircuitType {
    /// Get the circuit name for file paths
    pub fn name(&self) -> &'static str {
        match self {
            CircuitType::Deposit => "deposit",
            CircuitType::Withdraw => "withdraw",
            CircuitType::Transfer => "transfer",
        }
    }
}

/// Noir prover using Barretenberg backend
pub struct NoirProver {
    /// Path to the circuits directory
    circuits_dir: PathBuf,
    /// Path to the compiled circuit artifacts
    target_dir: PathBuf,
    /// Path for temporary witness/proof files
    work_dir: PathBuf,
}

impl NoirProver {
    /// Create a new prover instance
    pub fn new(circuits_dir: PathBuf) -> Self {
        let target_dir = circuits_dir.join("target");
        let work_dir = circuits_dir.join("work");

        Self {
            circuits_dir,
            target_dir,
            work_dir,
        }
    }

    /// Create a prover with default paths (assumes standard project layout)
    pub fn default_paths() -> Self {
        // Assume we're running from the project root or crates directory
        let circuits_dir = PathBuf::from("circuits");
        Self::new(circuits_dir)
    }

    /// Ensure work directory exists
    async fn ensure_work_dir(&self) -> Result<(), ProverError> {
        fs::create_dir_all(&self.work_dir).await?;
        Ok(())
    }

    /// Check if the circuit has been compiled
    pub fn is_compiled(&self) -> bool {
        self.target_dir.join("veilocity_circuits.json").exists()
    }

    /// Compile the Noir circuits
    pub async fn compile(&self) -> Result<(), ProverError> {
        info!("Compiling Noir circuits...");

        let output = Command::new("nargo")
            .current_dir(&self.circuits_dir)
            .arg("compile")
            .output()
            .map_err(|e| ProverError::CommandFailed(format!("Failed to run nargo: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProverError::CommandFailed(format!(
                "nargo compile failed: {}",
                stderr
            )));
        }

        info!("Circuits compiled successfully");
        Ok(())
    }

    /// Generate a proof for a deposit
    pub async fn prove_deposit(&self, witness: &DepositWitness) -> Result<Vec<u8>, ProverError> {
        self.ensure_work_dir().await?;

        // Write witness to Prover.toml
        let prover_toml = self.work_dir.join("Prover.toml");
        fs::write(&prover_toml, witness.to_toml()).await?;

        // Generate proof
        self.generate_proof(CircuitType::Deposit).await
    }

    /// Generate a proof for a withdrawal
    pub async fn prove_withdraw(&self, witness: &WithdrawWitness) -> Result<Vec<u8>, ProverError> {
        self.ensure_work_dir().await?;

        // Write witness to Prover.toml
        let prover_toml = self.work_dir.join("Prover.toml");
        fs::write(&prover_toml, witness.to_toml()).await?;

        // Generate proof
        self.generate_proof(CircuitType::Withdraw).await
    }

    /// Generate a proof for a transfer
    pub async fn prove_transfer(&self, witness: &TransferWitness) -> Result<Vec<u8>, ProverError> {
        self.ensure_work_dir().await?;

        // Write witness to Prover.toml
        let prover_toml = self.work_dir.join("Prover.toml");
        fs::write(&prover_toml, witness.to_toml()).await?;

        // Generate proof
        self.generate_proof(CircuitType::Transfer).await
    }

    /// Generate proof using bb (Barretenberg)
    async fn generate_proof(&self, circuit_type: CircuitType) -> Result<Vec<u8>, ProverError> {
        if !self.is_compiled() {
            return Err(ProverError::CircuitNotCompiled);
        }

        let circuit_path = self.target_dir.join("veilocity_circuits.json");
        let witness_path = self.work_dir.join("witness.gz");
        let proof_path = self.work_dir.join("proof");
        let prover_toml = self.work_dir.join("Prover.toml");

        debug!("Generating witness for {:?}...", circuit_type);

        // Step 1: Generate witness using nargo execute
        let output = Command::new("nargo")
            .current_dir(&self.circuits_dir)
            .arg("execute")
            .arg("--prover-toml")
            .arg(&prover_toml)
            .arg("--witness-path")
            .arg(&witness_path)
            .output()
            .map_err(|e| ProverError::CommandFailed(format!("Failed to run nargo execute: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProverError::WitnessGeneration(stderr.to_string()));
        }

        debug!("Generating proof...");

        // Step 2: Generate proof using bb
        let output = Command::new("bb")
            .arg("prove")
            .arg("-b")
            .arg(&circuit_path)
            .arg("-w")
            .arg(&witness_path)
            .arg("-o")
            .arg(&proof_path)
            .output()
            .map_err(|e| ProverError::CommandFailed(format!("Failed to run bb prove: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProverError::ProofGeneration(stderr.to_string()));
        }

        // Read the generated proof
        let proof = fs::read(&proof_path).await?;

        info!(
            "Proof generated successfully ({} bytes)",
            proof.len()
        );

        Ok(proof)
    }

    /// Verify a proof locally
    pub async fn verify_proof(
        &self,
        proof: &[u8],
        _circuit_type: CircuitType,
    ) -> Result<bool, ProverError> {
        if !self.is_compiled() {
            return Err(ProverError::CircuitNotCompiled);
        }

        self.ensure_work_dir().await?;

        let circuit_path = self.target_dir.join("veilocity_circuits.json");
        let vk_path = self.target_dir.join("vk");
        let proof_path = self.work_dir.join("proof_to_verify");

        // Write proof to file
        fs::write(&proof_path, proof).await?;

        // Verify using bb
        let output = Command::new("bb")
            .arg("verify")
            .arg("-b")
            .arg(&circuit_path)
            .arg("-k")
            .arg(&vk_path)
            .arg("-p")
            .arg(&proof_path)
            .output()
            .map_err(|e| ProverError::CommandFailed(format!("Failed to run bb verify: {}", e)))?;

        Ok(output.status.success())
    }

    /// Generate the verification key
    pub async fn generate_vk(&self) -> Result<(), ProverError> {
        if !self.is_compiled() {
            return Err(ProverError::CircuitNotCompiled);
        }

        let circuit_path = self.target_dir.join("veilocity_circuits.json");
        let vk_path = self.target_dir.join("vk");

        info!("Generating verification key...");

        let output = Command::new("bb")
            .arg("write_vk")
            .arg("-b")
            .arg(&circuit_path)
            .arg("-o")
            .arg(&vk_path)
            .arg("--oracle_hash")
            .arg("keccak")
            .output()
            .map_err(|e| ProverError::CommandFailed(format!("Failed to run bb write_vk: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProverError::CommandFailed(format!(
                "bb write_vk failed: {}",
                stderr
            )));
        }

        info!("Verification key generated");
        Ok(())
    }

    /// Generate the Solidity verifier contract
    pub async fn generate_solidity_verifier(&self, output_path: &Path) -> Result<(), ProverError> {
        let vk_path = self.target_dir.join("vk");

        if !vk_path.exists() {
            self.generate_vk().await?;
        }

        info!("Generating Solidity verifier...");

        let output = Command::new("bb")
            .arg("write_solidity_verifier")
            .arg("-k")
            .arg(&vk_path)
            .arg("-o")
            .arg(output_path)
            .output()
            .map_err(|e| {
                ProverError::CommandFailed(format!("Failed to run bb write_solidity_verifier: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProverError::CommandFailed(format!(
                "bb write_solidity_verifier failed: {}",
                stderr
            )));
        }

        info!("Solidity verifier generated at {:?}", output_path);
        Ok(())
    }
}

/// Proof data structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// The raw proof bytes
    pub proof: Vec<u8>,
    /// Public inputs
    pub public_inputs: Vec<String>,
    /// Circuit type
    pub circuit_type: String,
}

impl Proof {
    /// Create a new proof
    pub fn new(proof: Vec<u8>, public_inputs: Vec<String>, circuit_type: CircuitType) -> Self {
        Self {
            proof,
            public_inputs,
            circuit_type: circuit_type.name().to_string(),
        }
    }

    /// Get proof as hex string
    pub fn proof_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.proof))
    }
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_type_names() {
        assert_eq!(CircuitType::Deposit.name(), "deposit");
        assert_eq!(CircuitType::Withdraw.name(), "withdraw");
        assert_eq!(CircuitType::Transfer.name(), "transfer");
    }

    #[test]
    fn test_prover_default_paths() {
        let prover = NoirProver::default_paths();
        assert!(prover.circuits_dir.ends_with("circuits"));
    }

    #[test]
    fn test_proof_hex() {
        let proof = Proof::new(vec![1, 2, 3, 4], vec![], CircuitType::Deposit);
        assert_eq!(proof.proof_hex(), "0x01020304");
    }
}
