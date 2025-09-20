// contracts/library/PlatformConfigurator.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";

// Correct imports
import "../AuditTrail.sol";
import "../ComplianceManager.sol";
import "../FeeManager.sol";
import "../TokenFactory.sol";
import "../AssetTokenizer.sol";
import "../MarketplaceCore.sol";
import "../RewardSystem.sol";
import "../AdminGovernance.sol";

library PlatformConfigurator {
    struct DeploymentConfig {
        address treasury;
        address rewardToken;
        uint256 tradingFeePercentage;
        uint256 tokenizationFeePercentage;
        uint256 minAssetValue;
        uint256 kycExpiryPeriod;
        bool useUpgradeableProxies;
        // Required for TokenFactory.initialize
        address tokenImplementation;
        address tokenRegistry;
    }

    struct PlatformContracts {
        address auditTrail;
        address complianceManager;
        address feeManager;
        address tokenFactory;
        address assetTokenizer;
        address marketplaceCore;
        address rewardSystem;
        address adminGovernance;
        address proxyAdmin;
    }

    function deployAndInit(
        address auditTrailImpl,
        address complianceManagerImpl,
        address feeManagerImpl,
        address tokenFactoryImpl,
        address assetTokenizerImpl,
        address marketplaceCoreImpl,
        address rewardSystemImpl,
        address adminGovernanceImpl,
        address proxyAdmin,
        DeploymentConfig calldata config
    ) internal returns (PlatformContracts memory pc) {
        if (config.useUpgradeableProxies) {
            pc.auditTrail = _deployProxy(auditTrailImpl, proxyAdmin);
            pc.complianceManager = _deployProxy(complianceManagerImpl, proxyAdmin);
            pc.feeManager = _deployProxy(feeManagerImpl, proxyAdmin);
            pc.tokenFactory = _deployProxy(tokenFactoryImpl, proxyAdmin);
            pc.assetTokenizer = _deployProxy(assetTokenizerImpl, proxyAdmin);
            pc.marketplaceCore = _deployProxy(marketplaceCoreImpl, proxyAdmin);
            pc.rewardSystem = _deployProxy(rewardSystemImpl, proxyAdmin);
            pc.adminGovernance = _deployProxy(adminGovernanceImpl, proxyAdmin);
            pc.proxyAdmin = proxyAdmin;
        } else {
            // Deploy contracts without constructor args
            pc.auditTrail = address(new AuditTrail());
            pc.complianceManager = address(new ComplianceManager());
            pc.feeManager = address(new FeeManager());
            pc.tokenFactory = address(new TokenFactory());
            pc.assetTokenizer = address(new AssetTokenizer());
            pc.marketplaceCore = address(new MarketplaceCore());
            pc.rewardSystem = address(new RewardSystem());
            pc.adminGovernance = address(new AdminGovernance());
            pc.proxyAdmin = address(0);
        }

        _initializeContracts(pc, config);
    }

    function _deployProxy(address implementation, address admin)
        private
        returns (address)
    {
        return address(new TransparentUpgradeableProxy(implementation, admin, ""));
    }

    function _initializeContracts(
        PlatformContracts memory pc,
        DeploymentConfig calldata config
    ) private {
        // Initialize AuditTrail
        AuditTrail(payable(pc.auditTrail)).initialize(msg.sender);

        // Initialize ComplianceManager
        ComplianceManager(payable(pc.complianceManager)).initialize(msg.sender, pc.auditTrail);

        // Initialize FeeManager
        FeeManager(payable(pc.feeManager)).initialize(msg.sender, config.treasury);

        // Initialize TokenFactory
        TokenFactory(payable(pc.tokenFactory)).initialize(
            config.tokenImplementation,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager,
            config.tokenRegistry,
            msg.sender
        );

        // Initialize AssetTokenizer
        AssetTokenizer(payable(pc.assetTokenizer)).initialize(
            msg.sender,
            pc.tokenFactory,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager
        );

        // Initialize MarketplaceCore
        MarketplaceCore(payable(pc.marketplaceCore)).initialize(
            msg.sender,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager
        );

        // Initialize RewardSystem
        RewardSystem(payable(pc.rewardSystem)).initialize(
            msg.sender,
            config.rewardToken,
            pc.auditTrail
        );

        // Initialize AdminGovernance
        AdminGovernance(payable(pc.adminGovernance)).initialize(msg.sender, pc.auditTrail);

        // Apply configuration
        _configurePlatform(pc, config);
    }

    function _configurePlatform(
        PlatformContracts memory pc,
        DeploymentConfig calldata config
    ) private {
        if (config.kycExpiryPeriod > 0) {
            ComplianceManager(payable(pc.complianceManager)).setKYCExpiryPeriod(config.kycExpiryPeriod);
        }

        if (config.minAssetValue > 0) {
            AssetTokenizer(payable(pc.assetTokenizer)).setMinAssetValue(config.minAssetValue);
        }

        if (config.tradingFeePercentage > 0) {
            FeeManager(payable(pc.feeManager)).setFeeStructure(
                keccak256("TRADING"),
                config.tradingFeePercentage,
                0,
                0,
                0,
                true
            );
        }

        if (config.tokenizationFeePercentage > 0) {
            FeeManager(payable(pc.feeManager)).setFeeStructure(
                keccak256("TOKENIZATION"),
                config.tokenizationFeePercentage,
                0,
                0.1 ether,
                10 ether,
                true
            );
        }
    }
}
