// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title IVerifier
/// @notice Interface for the Noir UltraPlonk verifier
/// @dev This interface matches the auto-generated verifier from Noir/Barretenberg
interface IVerifier {
    /// @notice Verify a proof against public inputs
    /// @param _proof The serialized proof bytes
    /// @param _publicInputs Array of public input field elements
    /// @return True if the proof is valid, false otherwise
    function verify(bytes calldata _proof, bytes32[] calldata _publicInputs) external view returns (bool);
}
