// script/Deploy.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";

// Implementations for Deployment
import "../contracts/core/AuditTrail.sol";
import "../contracts/core/ComplianceManager.sol";
import "../contracts/core/FeeManager.sol";
import "../contracts/core/AdminGovernance.sol";
import "../contracts/core/TokenRegistry.sol";
import "../contracts/core/RewardSystem.sol";
import "../contracts/core/HybridAssetTokenizer.sol";
import "../contracts/core/TokenFactory.sol";
import "../contracts/core/MarketplaceCore.sol";
import "../contracts/core/AssetVerificationGateway.sol";
import "../contracts/tokens/AssetToken.sol";
import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";

// Interfaces for Interaction
import "../contracts/interfaces/core/IAuditTrail.sol";
import "../contracts/interfaces/core/IComplianceManager.sol";
import "../contracts/interfaces/core/IFeeManager.sol";
import "../contracts/interfaces/core/IAdminGovernance.sol";
import "../contracts/interfaces/core/ITokenRegistry.sol";
import "../contracts/interfaces/core/IRewardSystem.sol";
import "../contracts/interfaces/core/IAssetTokenizer.sol";
import "../contracts/interfaces/core/ITokenFactory.sol";
import "../contracts/interfaces/core/IMarketplaceCore.sol";


contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        address treasury = vm.envOr("TREASURY_ADDRESS", address(0x70997970C51812dc3A010C7d01b50e0d17dc79C8));

        vm.startBroadcast(deployerPrivateKey);

        console.log("Deploying contracts with:", deployer);
        console.log("Treasury:", treasury);

        // Deploy ProxyAdmin and Implementations...
        ProxyAdmin proxyAdmin = new ProxyAdmin(deployer);
        AuditTrail auditTrailImpl = new AuditTrail();
        ComplianceManager complianceImpl = new ComplianceManager();
        FeeManager feeImpl = new FeeManager();
        AdminGovernance adminGovImpl = new AdminGovernance();
        TokenRegistry registryImpl = new TokenRegistry();
        RewardSystem rewardImpl = new RewardSystem();
        HybridAssetTokenizer hybridTokenizerImpl = new HybridAssetTokenizer();
        TokenFactory factoryImpl = new TokenFactory();
        MarketplaceCore marketplaceImpl = new MarketplaceCore();
        AssetToken assetTokenImpl = new AssetToken();

        // Deploy Proxies...
        TransparentUpgradeableProxy auditTrailProxy = new TransparentUpgradeableProxy(address(auditTrailImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy complianceProxy = new TransparentUpgradeableProxy(address(complianceImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy feeProxy = new TransparentUpgradeableProxy(address(feeImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy adminGovProxy = new TransparentUpgradeableProxy(address(adminGovImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy registryProxy = new TransparentUpgradeableProxy(address(registryImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy rewardProxy = new TransparentUpgradeableProxy(address(rewardImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy hybridTokenizerProxy = new TransparentUpgradeableProxy(address(hybridTokenizerImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy factoryProxy = new TransparentUpgradeableProxy(address(factoryImpl), address(proxyAdmin), "");
        TransparentUpgradeableProxy marketplaceProxy = new TransparentUpgradeableProxy(address(marketplaceImpl), address(proxyAdmin), "");

        // Cast Proxies to Interfaces...
        IAuditTrail auditTrail = IAuditTrail(payable(address(auditTrailProxy)));
        IComplianceManager compliance = IComplianceManager(payable(address(complianceProxy)));
        IFeeManager fees = IFeeManager(payable(address(feeProxy)));
        IAdminGovernance adminGov = IAdminGovernance(payable(address(adminGovProxy)));
        ITokenRegistry registry = ITokenRegistry(address(registryProxy));
        IRewardSystem rewards = IRewardSystem(payable(address(rewardProxy)));
        IAssetTokenizer hybridTokenizer = IAssetTokenizer(payable(address(hybridTokenizerProxy)));
        ITokenFactory factory = ITokenFactory(payable(address(factoryProxy)));
        IMarketplaceCore marketplace = IMarketplaceCore(payable(address(marketplaceProxy)));

        // Initialize contracts
        auditTrail.initialize(deployer);
        compliance.initialize(deployer, address(auditTrail));
        fees.initialize(deployer, treasury);
        adminGov.initialize(deployer, address(auditTrail));
        registry.initialize(deployer);
        rewards.initialize(deployer, address(0), address(auditTrail));
        
        // ===== FIX #1: Add the missing 'address(registry)' argument =====
        hybridTokenizer.initialize(
            deployer,
            address(factory),
            address(registry), // <-- THIS ARGUMENT WAS MISSING
            address(compliance),
            address(auditTrail),
            address(fees)
        );
        
        factory.initialize(
            address(assetTokenImpl),
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

        // Deploy verification gateway
        // ===== FIX #2: Add the missing 'kycProvider' argument (using address(0) as a placeholder) =====
        AssetVerificationGateway verificationGateway = new AssetVerificationGateway(
            deployer,
            address(0), // <-- THIS ARGUMENT WAS MISSING for kyc_
            address(compliance),
            address(hybridTokenizer),
            address(0)
        );

        // Set verification gateway
        hybridTokenizer.setVerificationGateway(address(verificationGateway));

        // Grant necessary roles
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(compliance));
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(hybridTokenizer));
        auditTrail.grantRole(auditTrail.SYSTEM_ROLE(), address(factory));
        
        // ... (console logs are fine) ...
        console.log("\n=== Deployment Complete ===");
        console.log("ProxyAdmin:", address(proxyAdmin));
        console.log("AuditTrail:", address(auditTrail));
        console.log("ComplianceManager:", address(compliance));
        console.log("FeeManager:", address(fees));
        console.log("AdminGovernance:", address(adminGov));
        console.log("HybridAssetTokenizer:", address(hybridTokenizer));
        console.log("TokenFactory:", address(factory));
        console.log("TokenRegistry:", address(registry));
        console.log("MarketplaceCore:", address(marketplace));
        console.log("VerificationGateway:", address(verificationGateway));
        console.log("AssetToken Implementation:", address(assetTokenImpl));


        vm.stopBroadcast();
    }
}