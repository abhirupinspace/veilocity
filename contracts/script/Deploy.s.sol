// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {VeilocityVault} from "../src/VeilocityVault.sol";
import {MockVerifier} from "../src/mocks/MockVerifier.sol";

/// @title Deploy
/// @notice Deployment script for Veilocity contracts
contract Deploy is Script {
    // Initial empty Merkle tree root (precomputed for depth 20)
    // This is hash(hash(0,0), hash(0,0)) repeated up to depth 20
    bytes32 constant INITIAL_ROOT = 0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address verifierAddress = vm.envOr("VERIFIER_ADDRESS", address(0));

        vm.startBroadcast(deployerPrivateKey);

        // Deploy mock verifier if no verifier address provided
        if (verifierAddress == address(0)) {
            console.log("Deploying MockVerifier...");
            MockVerifier mockVerifier = new MockVerifier();
            verifierAddress = address(mockVerifier);
            console.log("MockVerifier deployed at:", verifierAddress);
        }

        // Deploy VeilocityVault
        console.log("Deploying VeilocityVault...");
        VeilocityVault vault = new VeilocityVault(verifierAddress, INITIAL_ROOT);
        console.log("VeilocityVault deployed at:", address(vault));

        vm.stopBroadcast();

        // Log deployment info
        console.log("\n=== Deployment Summary ===");
        console.log("Network:", block.chainid);
        console.log("Verifier:", verifierAddress);
        console.log("Vault:", address(vault));
        console.log("Initial Root:", vm.toString(INITIAL_ROOT));
    }
}

/// @title DeployTestnet
/// @notice Deployment script specifically for Mantle Sepolia testnet
contract DeployTestnet is Script {
    bytes32 constant INITIAL_ROOT = 0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        // Deploy mock verifier for testnet
        MockVerifier mockVerifier = new MockVerifier();
        console.log("MockVerifier deployed at:", address(mockVerifier));

        // Deploy vault
        VeilocityVault vault = new VeilocityVault(address(mockVerifier), INITIAL_ROOT);
        console.log("VeilocityVault deployed at:", address(vault));

        vm.stopBroadcast();
    }
}
