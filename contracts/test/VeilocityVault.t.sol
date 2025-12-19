// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {VeilocityVault} from "../src/VeilocityVault.sol";
import {MockVerifier} from "../src/mocks/MockVerifier.sol";

contract VeilocityVaultTest is Test {
    VeilocityVault public vault;
    MockVerifier public verifier;

    address public owner = address(this);
    address public alice = address(0x1);
    address public bob = address(0x2);

    // Allow test contract to receive ETH for emergency withdraw test
    receive() external payable {}

    bytes32 constant INITIAL_ROOT = 0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864;
    bytes32 constant TEST_COMMITMENT = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
    bytes32 constant TEST_NULLIFIER = 0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890;

    function setUp() public {
        verifier = new MockVerifier();
        vault = new VeilocityVault(address(verifier), INITIAL_ROOT);

        // Fund test accounts
        vm.deal(alice, 100 ether);
        vm.deal(bob, 100 ether);
    }

    // ============ Constructor Tests ============

    function test_Constructor() public view {
        assertEq(vault.currentRoot(), INITIAL_ROOT);
        assertEq(vault.depositCount(), 0);
        assertEq(vault.totalValueLocked(), 0);
        assertTrue(vault.isValidRoot(INITIAL_ROOT));
    }

    function test_Constructor_RevertInvalidVerifier() public {
        vm.expectRevert(VeilocityVault.InvalidVerifier.selector);
        new VeilocityVault(address(0), INITIAL_ROOT);
    }

    // ============ Deposit Tests ============

    function test_Deposit() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        assertEq(vault.depositCount(), 1);
        assertEq(vault.totalValueLocked(), 1 ether);
        assertEq(address(vault).balance, 1 ether);
    }

    function test_Deposit_EmitsEvent() public {
        vm.prank(alice);
        vm.expectEmit(true, false, false, true);
        emit VeilocityVault.Deposit(TEST_COMMITMENT, 1 ether, 0, block.timestamp);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);
    }

    function test_Deposit_MultipleDeposits() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        bytes32 commitment2 = keccak256("commitment2");
        vm.prank(bob);
        vault.deposit{value: 2 ether}(commitment2);

        assertEq(vault.depositCount(), 2);
        assertEq(vault.totalValueLocked(), 3 ether);
    }

    function test_Deposit_RevertTooSmall() public {
        vm.prank(alice);
        vm.expectRevert(VeilocityVault.DepositTooSmall.selector);
        vault.deposit{value: 0.0001 ether}(TEST_COMMITMENT);
    }

    function test_Deposit_RevertZeroCommitment() public {
        vm.prank(alice);
        vm.expectRevert(VeilocityVault.InvalidCommitment.selector);
        vault.deposit{value: 1 ether}(bytes32(0));
    }

    function test_Deposit_RevertWhenPaused() public {
        vault.pause();
        vm.prank(alice);
        vm.expectRevert();
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);
    }

    // ============ Withdrawal Tests ============

    function test_Withdraw() public {
        // First deposit
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        // Withdraw
        bytes memory proof = hex"1234"; // Mock proof
        uint256 bobBalanceBefore = bob.balance;

        vault.withdraw(TEST_NULLIFIER, bob, 0.5 ether, INITIAL_ROOT, proof);

        assertEq(bob.balance, bobBalanceBefore + 0.5 ether);
        assertEq(vault.totalValueLocked(), 0.5 ether);
        assertTrue(vault.isNullifierUsed(TEST_NULLIFIER));
    }

    function test_Withdraw_EmitsEvent() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        bytes memory proof = hex"1234";
        vm.expectEmit(true, true, false, true);
        emit VeilocityVault.Withdrawal(TEST_NULLIFIER, bob, 0.5 ether);
        vault.withdraw(TEST_NULLIFIER, bob, 0.5 ether, INITIAL_ROOT, proof);
    }

    function test_Withdraw_RevertNullifierReuse() public {
        vm.prank(alice);
        vault.deposit{value: 2 ether}(TEST_COMMITMENT);

        bytes memory proof = hex"1234";
        vault.withdraw(TEST_NULLIFIER, bob, 0.5 ether, INITIAL_ROOT, proof);

        // Try to reuse nullifier
        vm.expectRevert(VeilocityVault.NullifierAlreadyUsed.selector);
        vault.withdraw(TEST_NULLIFIER, alice, 0.5 ether, INITIAL_ROOT, proof);
    }

    function test_Withdraw_RevertInvalidRoot() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        bytes memory proof = hex"1234";
        bytes32 invalidRoot = bytes32(uint256(1));

        vm.expectRevert(VeilocityVault.InvalidRoot.selector);
        vault.withdraw(TEST_NULLIFIER, bob, 0.5 ether, invalidRoot, proof);
    }

    function test_Withdraw_RevertInvalidProof() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        // Set verifier to reject proofs
        verifier.setVerifyResult(false);

        bytes memory proof = hex"1234";
        vm.expectRevert(VeilocityVault.InvalidProof.selector);
        vault.withdraw(TEST_NULLIFIER, bob, 0.5 ether, INITIAL_ROOT, proof);
    }

    function test_Withdraw_RevertZeroRecipient() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        bytes memory proof = hex"1234";
        vm.expectRevert(VeilocityVault.InvalidRecipient.selector);
        vault.withdraw(TEST_NULLIFIER, address(0), 0.5 ether, INITIAL_ROOT, proof);
    }

    function test_Withdraw_RevertZeroAmount() public {
        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        bytes memory proof = hex"1234";
        vm.expectRevert(VeilocityVault.InvalidAmount.selector);
        vault.withdraw(TEST_NULLIFIER, bob, 0, INITIAL_ROOT, proof);
    }

    // ============ State Root Tests ============

    function test_UpdateStateRoot() public {
        bytes32 newRoot = keccak256("new root");
        bytes memory proof = hex"1234";

        vault.updateStateRoot(newRoot, proof);

        assertEq(vault.currentRoot(), newRoot);
        assertTrue(vault.isValidRoot(newRoot));
        assertTrue(vault.isValidRoot(INITIAL_ROOT)); // Old root still valid
    }

    function test_UpdateStateRoot_EmitsEvent() public {
        bytes32 newRoot = keccak256("new root");
        bytes memory proof = hex"1234";

        vm.expectEmit(true, true, false, true);
        emit VeilocityVault.StateRootUpdated(INITIAL_ROOT, newRoot, 1, block.timestamp);
        vault.updateStateRoot(newRoot, proof);
    }

    function test_UpdateStateRoot_RevertNotOwner() public {
        bytes32 newRoot = keccak256("new root");
        bytes memory proof = hex"1234";

        vm.prank(alice);
        vm.expectRevert();
        vault.updateStateRoot(newRoot, proof);
    }

    function test_UpdateStateRoot_RootHistory() public {
        bytes memory proof = hex"1234";

        // Update root multiple times
        for (uint256 i = 0; i < 5; i++) {
            bytes32 newRoot = keccak256(abi.encodePacked("root", i));
            vault.updateStateRoot(newRoot, proof);
            assertTrue(vault.isValidRoot(newRoot));
        }
    }

    // ============ Admin Tests ============

    function test_Pause() public {
        vault.pause();
        assertTrue(vault.paused());
    }

    function test_Unpause() public {
        vault.pause();
        vault.unpause();
        assertFalse(vault.paused());
    }

    function test_Pause_RevertNotOwner() public {
        vm.prank(alice);
        vm.expectRevert();
        vault.pause();
    }

    function test_EmergencyWithdraw() public {
        // Deposit some funds
        vm.prank(alice);
        vault.deposit{value: 5 ether}(TEST_COMMITMENT);

        // Pause and emergency withdraw
        vault.pause();

        uint256 ownerBalanceBefore = owner.balance;
        vault.emergencyWithdraw(owner);

        assertEq(owner.balance, ownerBalanceBefore + 5 ether);
        assertEq(vault.totalValueLocked(), 0);
        assertEq(address(vault).balance, 0);
    }

    function test_EmergencyWithdraw_RevertNotPaused() public {
        vm.expectRevert();
        vault.emergencyWithdraw(owner);
    }

    // ============ View Function Tests ============

    function test_GetDepositCount() public {
        assertEq(vault.getDepositCount(), 0);

        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        assertEq(vault.getDepositCount(), 1);
    }

    function test_GetTotalValueLocked() public {
        assertEq(vault.getTotalValueLocked(), 0);

        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        assertEq(vault.getTotalValueLocked(), 1 ether);
    }

    function test_IsNullifierUsed() public {
        assertFalse(vault.isNullifierUsed(TEST_NULLIFIER));

        vm.prank(alice);
        vault.deposit{value: 1 ether}(TEST_COMMITMENT);

        bytes memory proof = hex"1234";
        vault.withdraw(TEST_NULLIFIER, bob, 0.5 ether, INITIAL_ROOT, proof);

        assertTrue(vault.isNullifierUsed(TEST_NULLIFIER));
    }

    // ============ Receive Tests ============

    function test_Receive_Reverts() public {
        vm.prank(alice);
        // Low-level call should fail when sending ETH directly (must use deposit())
        (bool success, ) = address(vault).call{value: 1 ether}("");
        assertFalse(success, "Direct ETH transfer should be rejected");
    }

    // ============ Fuzz Tests ============

    function testFuzz_Deposit(uint256 amount) public {
        amount = bound(amount, vault.MIN_DEPOSIT(), 100 ether);
        bytes32 commitment = keccak256(abi.encodePacked(amount));

        vm.prank(alice);
        vault.deposit{value: amount}(commitment);

        assertEq(vault.depositCount(), 1);
        assertEq(vault.totalValueLocked(), amount);
    }
}
