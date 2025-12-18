//! Contract ABI bindings
//!
//! This module contains the ABI definitions for interacting with Veilocity contracts.

use alloy::sol;

// VeilocityVault contract bindings
sol! {
    #[sol(rpc)]
    interface IVeilocityVault {
        // Events
        event Deposit(
            bytes32 indexed commitment,
            uint256 amount,
            uint256 leafIndex,
            uint256 timestamp
        );

        event Withdrawal(
            bytes32 indexed nullifier,
            address indexed recipient,
            uint256 amount
        );

        event StateRootUpdated(
            bytes32 indexed oldRoot,
            bytes32 indexed newRoot,
            uint256 batchIndex,
            uint256 timestamp
        );

        // View functions
        function currentRoot() external view returns (bytes32);
        function depositCount() external view returns (uint256);
        function totalValueLocked() external view returns (uint256);
        function isValidRoot(bytes32 root) external view returns (bool);
        function isNullifierUsed(bytes32 nullifier) external view returns (bool);
        function getDepositCount() external view returns (uint256);
        function getTotalValueLocked() external view returns (uint256);
        function MIN_DEPOSIT() external view returns (uint256);

        // State-changing functions
        function deposit(bytes32 commitment) external payable;

        function withdraw(
            bytes32 nullifier,
            address recipient,
            uint256 amount,
            bytes32 root,
            bytes calldata proof
        ) external;

        function updateStateRoot(
            bytes32 newRoot,
            bytes calldata proof
        ) external;

        // Admin functions
        function pause() external;
        function unpause() external;
        function emergencyWithdraw(address recipient) external;
    }
}

// Verifier contract bindings
sol! {
    #[sol(rpc)]
    interface IVerifier {
        function verify(
            bytes calldata proof,
            bytes32[] calldata publicInputs
        ) external view returns (bool);
    }
}
