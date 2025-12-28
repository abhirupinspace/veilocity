// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IVerifier} from "./interfaces/IVerifier.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title VeilocityVault
/// @notice Main contract for Veilocity private execution layer on Mantle
/// @dev Handles deposits, withdrawals, and state root anchoring with ZK proof verification
contract VeilocityVault is ReentrancyGuard, Pausable, Ownable {
    // ============ Constants ============

    /// @notice Number of historical roots to keep (for withdrawal flexibility)
    uint256 public constant ROOT_HISTORY_SIZE = 100;

    /// @notice Minimum deposit amount (to prevent dust attacks)
    uint256 public constant MIN_DEPOSIT = 0.001 ether;

    // ============ State Variables ============

    /// @notice The ZK verifier contract
    IVerifier public immutable verifier;

    /// @notice Current state root of the Merkle tree
    bytes32 public currentRoot;

    /// @notice Mapping of historical roots (root => isValid)
    mapping(bytes32 => bool) public rootHistory;

    /// @notice Array of root history for iteration
    bytes32[ROOT_HISTORY_SIZE] public roots;

    /// @notice Current index in root history array
    uint256 public currentRootIndex;

    /// @notice Number of deposits (also serves as leaf index counter)
    uint256 public depositCount;

    /// @notice Mapping of used nullifiers (nullifier => isUsed)
    mapping(bytes32 => bool) public nullifiers;

    /// @notice Total value locked in the vault
    uint256 public totalValueLocked;

    // ============ Events ============

    /// @notice Emitted when a deposit is made
    /// @param commitment The deposit commitment (hash of secret and amount)
    /// @param amount The deposited amount in wei
    /// @param leafIndex The assigned leaf index in the Merkle tree
    /// @param timestamp Block timestamp of the deposit
    event Deposit(
        bytes32 indexed commitment,
        uint256 amount,
        uint256 leafIndex,
        uint256 timestamp
    );

    /// @notice Emitted when a withdrawal is made
    /// @param nullifier The nullifier preventing double-spend
    /// @param recipient The address receiving the funds
    /// @param amount The withdrawn amount in wei
    event Withdrawal(
        bytes32 indexed nullifier,
        address indexed recipient,
        uint256 amount
    );

    /// @notice Emitted when the state root is updated
    /// @param oldRoot The previous state root
    /// @param newRoot The new state root
    /// @param batchIndex The batch number
    /// @param timestamp Block timestamp
    event StateRootUpdated(
        bytes32 indexed oldRoot,
        bytes32 indexed newRoot,
        uint256 batchIndex,
        uint256 timestamp
    );

    // ============ Errors ============

    error InvalidVerifier();
    error DepositTooSmall();
    error InvalidCommitment();
    error InvalidProof();
    error InvalidRoot();
    error NullifierAlreadyUsed();
    error InvalidRecipient();
    error InvalidAmount();
    error TransferFailed();
    error InvalidNullifier();

    // ============ Constructor ============

    /// @notice Initialize the vault with verifier and initial empty root
    /// @param _verifier Address of the ZK verifier contract
    /// @param _initialRoot Initial empty Merkle tree root
    constructor(
        address _verifier,
        bytes32 _initialRoot
    ) Ownable(msg.sender) {
        if (_verifier == address(0)) revert InvalidVerifier();

        verifier = IVerifier(_verifier);
        currentRoot = _initialRoot;
        rootHistory[_initialRoot] = true;
        roots[0] = _initialRoot;
    }

    // ============ External Functions ============

    /// @notice Deposit funds into the private execution layer
    /// @param commitment The deposit commitment = hash(secret, amount)
    /// @dev The commitment binds the deposit to a secret known only to the depositor
    function deposit(bytes32 commitment) external payable nonReentrant whenNotPaused {
        if (msg.value < MIN_DEPOSIT) revert DepositTooSmall();
        if (commitment == bytes32(0)) revert InvalidCommitment();

        uint256 leafIndex = depositCount;
        depositCount++;
        totalValueLocked += msg.value;

        emit Deposit(commitment, msg.value, leafIndex, block.timestamp);
    }

    /// @notice Withdraw funds from the private execution layer
    /// @param nullifier Unique identifier preventing double-withdrawal
    /// @param recipient Address to receive the funds
    /// @param amount Amount to withdraw in wei
    /// @param root The state root used in the proof
    /// @param proof The ZK proof of valid withdrawal
    function withdraw(
        bytes32 nullifier,
        address recipient,
        uint256 amount,
        bytes32 root,
        bytes calldata proof
    ) external nonReentrant whenNotPaused {
        // Validate inputs
        if (nullifier == bytes32(0)) revert InvalidNullifier();
        if (recipient == address(0)) revert InvalidRecipient();
        if (amount == 0) revert InvalidAmount();
        if (!isValidRoot(root)) revert InvalidRoot();
        if (nullifiers[nullifier]) revert NullifierAlreadyUsed();

        // Verify the ZK proof
        bytes32[] memory publicInputs = new bytes32[](4);
        publicInputs[0] = root;                              // state_root
        publicInputs[1] = nullifier;                         // nullifier
        publicInputs[2] = bytes32(amount);                   // amount
        publicInputs[3] = bytes32(uint256(uint160(recipient))); // recipient as field

        if (!verifier.verify(proof, publicInputs)) revert InvalidProof();

        // Mark nullifier as used
        nullifiers[nullifier] = true;

        // Update TVL
        totalValueLocked -= amount;

        // Transfer funds
        (bool success, ) = recipient.call{value: amount}("");
        if (!success) revert TransferFailed();

        emit Withdrawal(nullifier, recipient, amount);
    }

    /// @notice Update the state root with a validity proof
    /// @param newRoot The new state root
    /// @param proof ZK proof of valid state transition
    /// @dev Only callable by authorized operators (owner for MVP)
    function updateStateRoot(
        bytes32 newRoot,
        bytes calldata proof
    ) external onlyOwner whenNotPaused {
        if (newRoot == bytes32(0)) revert InvalidRoot();

        // For MVP, we trust the owner to submit valid state roots
        // In production, this would verify a state transition proof
        // bytes32[] memory publicInputs = new bytes32[](2);
        // publicInputs[0] = currentRoot;
        // publicInputs[1] = newRoot;
        // if (!verifier.verify(proof, publicInputs)) revert InvalidProof();

        bytes32 oldRoot = currentRoot;
        currentRoot = newRoot;

        // Add to history
        uint256 newIndex = (currentRootIndex + 1) % ROOT_HISTORY_SIZE;
        currentRootIndex = newIndex;
        roots[newIndex] = newRoot;
        rootHistory[newRoot] = true;

        emit StateRootUpdated(oldRoot, newRoot, newIndex, block.timestamp);
    }

    // ============ View Functions ============

    /// @notice Check if a root is valid (current or in history)
    /// @param root The root to check
    /// @return True if the root is valid
    function isValidRoot(bytes32 root) public view returns (bool) {
        return rootHistory[root];
    }

    /// @notice Check if a nullifier has been used
    /// @param nullifier The nullifier to check
    /// @return True if the nullifier has been used
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }

    /// @notice Get the current deposit count (next leaf index)
    /// @return The number of deposits made
    function getDepositCount() external view returns (uint256) {
        return depositCount;
    }

    /// @notice Get the total value locked in the vault
    /// @return The TVL in wei
    function getTotalValueLocked() external view returns (uint256) {
        return totalValueLocked;
    }

    // ============ Admin Functions ============

    /// @notice Pause the contract in case of emergency
    function pause() external onlyOwner {
        _pause();
    }

    /// @notice Unpause the contract
    function unpause() external onlyOwner {
        _unpause();
    }

    /// @notice Emergency withdrawal (only when paused)
    /// @param recipient Address to receive all funds
    /// @dev This is a last resort for recovering funds in case of critical issues
    function emergencyWithdraw(address recipient) external onlyOwner whenPaused {
        if (recipient == address(0)) revert InvalidRecipient();

        uint256 balance = address(this).balance;
        totalValueLocked = 0;

        (bool success, ) = recipient.call{value: balance}("");
        if (!success) revert TransferFailed();
    }

    // ============ Receive ============

    /// @notice Receive function to accept plain MNT transfers
    receive() external payable {
        revert("Use deposit() instead");
    }
}
