// script/DeployAnvil.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import "./Deploy.s.sol"; // Import main deployment script

contract DeployAnvilScript is DeployScript {
    function run() external override {
        // Anvil default accounts
        address deployer = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266; // Account 0
        address treasury = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8; // Account 1
        
        // For testing, we'll use a dummy oracle address
        address dummyOracle = 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC; // Account 2

        vm.startBroadcast(vm.envUint("DEPLOYER_PRIVATE_KEY"));

        console.log("=== Deploying on Anvil Fork ===");
        console.log("Deployer:", deployer);
        console.log("Treasury:", treasury);

        // Deploy all contracts (reuse main deployment logic)
        super.run();

        // Additional setup for testing on fork
        _setupTestEnvironment(deployer, treasury, dummyOracle);

        vm.stopBroadcast();
    }

    function _setupTestEnvironment(
        address deployer,
        address treasury,
        address dummyOracle
    ) internal {
        console.log("\n=== Setting up test environment ===");
        
        // You can add test-specific setup here
        // For example, minting test tokens, setting up test users, etc.
    }
}