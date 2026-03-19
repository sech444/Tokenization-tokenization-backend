// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";

// Import contracts
import {TokenFactory} from "../contracts/core/TokenFactory.sol";
import {MarketplaceCore} from "../contracts/core/MarketplaceCore.sol";
import {ComplianceManager} from "../contracts/core/ComplianceManager.sol";
import {RewardSystem} from "../contracts/core/RewardSystem.sol";
import {TokenRegistry} from "../contracts/core/TokenRegistry.sol";

// OpenZeppelin upgradeable proxies
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

contract Deploy is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("BLOCKCHAIN_PRIVATE_KEY");
        vm.startBroadcast(deployerKey);

        // ---------------------------------------------------------------------
        // Deploy ProxyAdmin (controls upgrades)
        // ---------------------------------------------------------------------
        ProxyAdmin proxyAdmin = new ProxyAdmin(msg.sender);

        // ---------------------------------------------------------------------
        // Deploy logic contracts
        // ---------------------------------------------------------------------
        TokenFactory factoryImpl = new TokenFactory();
        MarketplaceCore marketplaceImpl = new MarketplaceCore();
        ComplianceManager complianceImpl = new ComplianceManager();
        RewardSystem rewardImpl = new RewardSystem();
        TokenRegistry registryImpl = new TokenRegistry();

        // ---------------------------------------------------------------------
        // Deploy proxies pointing to the implementations
        // ---------------------------------------------------------------------
        TransparentUpgradeableProxy factoryProxy = new TransparentUpgradeableProxy(
            address(factoryImpl),
            address(proxyAdmin),
            ""
        );

        TransparentUpgradeableProxy marketplaceProxy = new TransparentUpgradeableProxy(
            address(marketplaceImpl),
            address(proxyAdmin),
            ""
        );

        TransparentUpgradeableProxy complianceProxy = new TransparentUpgradeableProxy(
            address(complianceImpl),
            address(proxyAdmin),
            ""
        );

        TransparentUpgradeableProxy rewardProxy = new TransparentUpgradeableProxy(
            address(rewardImpl),
            address(proxyAdmin),
            ""
        );

        TransparentUpgradeableProxy registryProxy = new TransparentUpgradeableProxy(
            address(registryImpl),
            address(proxyAdmin),
            ""
        );

        // ---------------------------------------------------------------------
        // Initialize contracts behind proxies
        // ---------------------------------------------------------------------
        // Initialize TokenRegistry
        TokenRegistry(address(registryProxy)).initialize(msg.sender);

        // Initialize TokenFactory with registry wired in
        TokenFactory(address(factoryProxy)).initialize(
            address(factoryImpl),       // tokenImplementation (dummy placeholder)
            address(complianceProxy),   // compliance manager
            address(0),                 // auditTrail (to be replaced later)
            address(0),                 // feeManager (to be replaced later)
            address(registryProxy),     // ✅ registry proxy
            msg.sender                  // admin
        );

        // Add similar initialize() calls for Marketplace, Compliance, Reward
        // depending on their initializer signatures.

        // ---------------------------------------------------------------------
        // Log deployed contract addresses
        // ---------------------------------------------------------------------
        console.log("PROXY_ADMIN=%s", address(proxyAdmin));
        console.log("CONTRACT_TOKEN_FACTORY=%s", address(factoryProxy));
        console.log("CONTRACT_MARKETPLACE=%s", address(marketplaceProxy));
        console.log("CONTRACT_COMPLIANCE=%s", address(complianceProxy));
        console.log("CONTRACT_REWARD=%s", address(rewardProxy));
        console.log("CONTRACT_REGISTRY=%s", address(registryProxy));

        vm.stopBroadcast();
    }
}
