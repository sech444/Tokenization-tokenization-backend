// script/Deploy.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import "../contracts/core/AuditTrail.sol";
import "../contracts/core/ComplianceManager.sol";
import "../contracts/core/FeeManager.sol";
import "../contracts/core/AdminGovernance.sol";
import "../contracts/core/TokenRegistry.sol";
import "../contracts/core/RewardSystem.sol";
import "../contracts/core/HybridAssetTokenizer.sol";
import "../contracts/core/AssetVerificationGateway.sol";
import "../contracts/core/TokenFactory.sol";
import "../contracts/core/MarketplaceCore.sol";
import "../contracts/tokens/AssetToken.sol";
import "../contracts/factory/PlatformDeployer.sol";
import "../contracts/factory/TokenizationPlatformFactory.sol";
import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        address treasury = vm.envAddress("TREASURY_ADDRESS"); // Add to .env

        vm.startBroadcast(deployer);

        console.log("Deploying contracts with:", deployer);
        console.log("Treasury:", treasury);

        // Proxy admin
        ProxyAdmin proxyAdmin = new ProxyAdmin(deployer);

        // Deploy implementations
        AuditTrail auditTrailImpl = new AuditTrail();
        ComplianceManager complianceImpl = new ComplianceManager();
        FeeManager feeImpl = new FeeManager();
        AdminGovernance adminGovImpl = new AdminGovernance();
        TokenRegistry registryImpl = new TokenRegistry();
        RewardSystem rewardImpl = new RewardSystem();
        HybridAssetTokenizer hybridTokenizerImpl = new HybridAssetTokenizer();
        TokenFactory factoryImpl = new TokenFactory();
        MarketplaceCore marketplaceImpl = new MarketplaceCore();
        AssetToken assetTokenImpl = new AssetToken(); // Implementation for token clones

        // Deploy proxies
        TransparentUpgradeableProxy auditTrailProxy =
            new TransparentUpgradeableProxy(address(auditTrailImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy complianceProxy =
            new TransparentUpgradeableProxy(address(complianceImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy feeProxy =
            new TransparentUpgradeableProxy(address(feeImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy adminGovProxy =
            new TransparentUpgradeableProxy(address(adminGovImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy registryProxy =
            new TransparentUpgradeableProxy(address(registryImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy rewardProxy =
            new TransparentUpgradeableProxy(address(rewardImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy hybridTokenizerProxy =
            new TransparentUpgradeableProxy(address(hybridTokenizerImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy factoryProxy =
            new TransparentUpgradeableProxy(address(factoryImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy marketplaceProxy =
            new TransparentUpgradeableProxy(address(marketplaceImpl), address(proxyAdmin), "");

        // Cast proxies back to contracts
        AuditTrail auditTrail = AuditTrail(payable(address(auditTrailProxy)));
        ComplianceManager compliance = ComplianceManager(payable(address(complianceProxy)));
        FeeManager fees = FeeManager(payable(address(feeProxy)));
        AdminGovernance adminGov = AdminGovernance(payable(address(adminGovProxy)));
        TokenRegistry registry = TokenRegistry(address(registryProxy));
        RewardSystem rewards = RewardSystem(payable(address(rewardProxy)));
        HybridAssetTokenizer hybridTokenizer = HybridAssetTokenizer(payable(address(hybridTokenizerProxy)));
        TokenFactory factory = TokenFactory(payable(address(factoryProxy)));
        MarketplaceCore marketplace = MarketplaceCore(payable(address(marketplaceProxy)));

        // Initialize core contracts
        auditTrail.initialize(deployer);
        
        compliance.initialize(deployer, address(auditTrail));
        
        fees.initialize(deployer, treasury);
        
        adminGov.initialize(deployer, address(auditTrail));
        
        registry.initialize(deployer);
        
        rewards.initialize(
            deployer, 
            address(0), // Reward token (can be set later)
            address(auditTrail)
        );
        
        hybridTokenizer.initialize(
            deployer,
            address(factory),
            address(registry),
            address(compliance),
            address(auditTrail),
            address(fees)
        );
        
        factory.initialize(
            address(assetTokenImpl),  // Token implementation for clones
            address(compliance),
            address(auditTrail),
            address(fees),
            address(registry),
            deployer
        );
        
        marketplace.initialize(
            deployer,
            address(compliance),
            address(auditTrail),
            address(fees)
        );

        // Deploy non-upgradeable contracts
        AssetVerificationGateway verificationGateway = new AssetVerificationGateway(
            deployer,
            address(compliance), // Acts as KYC provider
            address(compliance), // Also handles compliance checks
            address(hybridTokenizer), // Now handles valuations
            address(0), // Price oracle (to be set later)
            address(hybridTokenizer)
        );

        // Set verification gateway in hybrid tokenizer
        hybridTokenizer.setVerificationGateway(address(verificationGateway));

        // Grant necessary roles
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(compliance));
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(hybridTokenizer));
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(factory));
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(marketplace));
        
        fees.grantRole(fees.FEE_ADMIN_ROLE(), deployer);
        
        compliance.grantRole(compliance.COMPLIANCE_OFFICER_ROLE(), deployer);
        compliance.grantRole(compliance.SYSTEM_ROLE(), address(hybridTokenizer));
        compliance.grantRole(compliance.SYSTEM_ROLE(), address(factory));

        // Platform factory & deployer for multi-tenant deployments
        PlatformDeployer platformDeployer = new PlatformDeployer(deployer);
        TokenizationPlatformFactory platformFactory = new TokenizationPlatformFactory(deployer);

        // Set implementations in factory
        platformFactory.setImplementations(
            address(auditTrailImpl),
            address(complianceImpl),
            address(feeImpl),
            address(factoryImpl),
            address(hybridTokenizerImpl), // Using hybrid tokenizer instead of asset tokenizer
            address(marketplaceImpl),
            address(rewardImpl),
            address(adminGovImpl)
        );

        // Register platform deployer in factory
        platformFactory.setPlatformDeployer(address(platformDeployer));

        console.log("\n=== Deployment Summary ===");
        console.log("ProxyAdmin:", address(proxyAdmin));
        console.log("\n--- Core Infrastructure ---");
        console.log("AuditTrail:", address(auditTrail));
        console.log("ComplianceManager:", address(compliance));
        console.log("FeeManager:", address(fees));
        console.log("AdminGovernance:", address(adminGov));
        console.log("\n--- Tokenization ---");
        console.log("HybridAssetTokenizer:", address(hybridTokenizer));
        console.log("AssetVerificationGateway:", address(verificationGateway));
        console.log("TokenFactory:", address(factory));
        console.log("TokenRegistry:", address(registry));
        console.log("AssetToken Implementation:", address(assetTokenImpl));
        console.log("\n--- Marketplace ---");
        console.log("MarketplaceCore:", address(marketplace));
        console.log("RewardSystem:", address(rewards));
        console.log("\n--- Factory System ---");
        console.log("PlatformDeployer:", address(platformDeployer));
        console.log("TokenizationPlatformFactory:", address(platformFactory));
        console.log("\n=========================");

        vm.stopBroadcast();
    }
}