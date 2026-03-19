// contracts/core/libraries/PlatformConfigurator.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";

// FIX 1: Replace implementation imports with interface imports.
// This resolves the "Identifier already declared" errors by breaking dependency cycles.
// Note the corrected relative paths from 'contracts/core/libraries/'.
import "../../interfaces/core/IAuditTrail.sol";
import "../../interfaces/core/IComplianceManager.sol";
import "../../interfaces/core/IFeeManager.sol";
import "../../interfaces/core/ITokenFactory.sol";
import "../../interfaces/core/IAssetTokenizer.sol"; // NOTE: Assumed interface for HybridAssetTokenizer
import "../../interfaces/core/IMarketplaceCore.sol";
import "../../interfaces/core/IRewardSystem.sol";
import "../../interfaces/core/IAdminGovernance.sol";

library PlatformConfigurator {
    struct DeploymentConfig {
        address treasury;
        address rewardToken;
        uint256 tradingFeePercentage;
        uint256 tokenizationFeePercentage;
        uint256 minAssetValue;
        uint256 kycExpiryPeriod;
        bool useUpgradeableProxies;
        address tokenImplementation;
        address tokenRegistry;
    }

    // FIX 2: Correct the struct field name for consistency and validity.
    struct PlatformContracts {
        address auditTrail;
        address complianceManager;
        address feeManager;
        address tokenFactory;
        address assetTokenizer; // Renamed from 'HybridAssetTokenizer'
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
        address hybridAssetTokenizerImpl,
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
            // FIX 3: Use the corrected struct field 'assetTokenizer'.
            pc.assetTokenizer = _deployProxy(hybridAssetTokenizerImpl, proxyAdmin);
            pc.marketplaceCore = _deployProxy(marketplaceCoreImpl, proxyAdmin);
            pc.rewardSystem = _deployProxy(rewardSystemImpl, proxyAdmin);
            pc.adminGovernance = _deployProxy(adminGovernanceImpl, proxyAdmin);
            pc.proxyAdmin = proxyAdmin;
        } else {
            // This block for non-proxy deployments will fail without implementation imports.
            // The recommended pattern is to deploy these contracts in your script and
            // pass their addresses, rather than having the library deploy them.
            revert("Non-proxy deployment from PlatformConfigurator is disabled; deploy implementations in script.");
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
        // FIX 4: Use interface types for all contract interactions.
        IAuditTrail(payable(pc.auditTrail)).initialize(msg.sender);

        IComplianceManager(payable(pc.complianceManager)).initialize(msg.sender, pc.auditTrail);

        IFeeManager(payable(pc.feeManager)).initialize(msg.sender, config.treasury);

        ITokenFactory(payable(pc.tokenFactory)).initialize(
            config.tokenImplementation,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager,
            config.tokenRegistry,
            msg.sender
        );

        // IAssetTokenizer(payable(pc.assetTokenizer)).initialize(
        //     msg.sender,
        //     pc.tokenFactory,
        //     pc.complianceManager,
        //     pc.auditTrail,
        //     pc.feeManager
        // );
        // AFTER
        IAssetTokenizer(payable(pc.assetTokenizer)).initialize(
            msg.sender,
            pc.tokenFactory,
            config.tokenRegistry, // <-- ADD THIS MISSING ARGUMENT
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager
        );

        IMarketplaceCore(payable(pc.marketplaceCore)).initialize(
            msg.sender,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager
        );

        IRewardSystem(payable(pc.rewardSystem)).initialize(
            msg.sender,
            config.rewardToken,
            pc.auditTrail
        );

        IAdminGovernance(payable(pc.adminGovernance)).initialize(msg.sender, pc.auditTrail);

        _configurePlatform(pc, config);
    }

    function _configurePlatform(
        PlatformContracts memory pc,
        DeploymentConfig calldata config
    ) private {
        // FIX 5: Use interface types for all configuration calls.
        if (config.kycExpiryPeriod > 0) {
            IComplianceManager(payable(pc.complianceManager)).setKYCExpiryPeriod(config.kycExpiryPeriod);
        }

        if (config.minAssetValue > 0) {
            // Note: Ensure IAssetTokenizer interface declares setMinAssetValue(uint256).
            IAssetTokenizer(payable(pc.assetTokenizer)).setMinAssetValue(config.minAssetValue);
        }

        if (config.tradingFeePercentage > 0) {
            IFeeManager(payable(pc.feeManager)).setFeeStructure(
                keccak256("TRADING"),
                config.tradingFeePercentage,
                0,
                0,
                0,
                true
            );
        }

        if (config.tokenizationFeePercentage > 0) {
            IFeeManager(payable(pc.feeManager)).setFeeStructure(
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