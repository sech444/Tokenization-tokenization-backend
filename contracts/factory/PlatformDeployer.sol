// contracts/factory/PlatformDeployer.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/proxy/Clones.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

// FIX 1: Replace all implementation imports with their corresponding interfaces.
// This is the primary solution to the "Identifier already declared" error.
import "../interfaces/core/IAuditTrail.sol";
import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IFeeManager.sol";
import "../interfaces/core/ITokenFactory.sol";
import "../interfaces/core/IAssetTokenizer.sol"; // Assumed interface for HybridAssetTokenizer
import "../interfaces/core/IMarketplaceCore.sol";
import "../interfaces/core/IRewardSystem.sol";
import "../interfaces/core/IAdminGovernance.sol";

/**
 * @title PlatformDeployer
 * @dev Deploys a platform by creating minimal proxies (Clones) for each implementation and
 *      initializing them.
 */
contract PlatformDeployer is AccessControl {
    using Clones for address;

    bytes32 public constant DEPLOYER_ROLE = keccak256("DEPLOYER_ROLE");

    event PlatformDeployed(address indexed platformId, address indexed deployer);
    event ComponentCloned(address indexed implementation, address indexed clone);

    struct DeploymentConfig {
        address treasury;
        address rewardToken;
        uint256 tradingFeePercentage;
        uint256 tokenizationFeePercentage;
        uint256 minAssetValue;
        uint256 kycExpiryPeriod;
        bool useUpgradeable;
        address tokenImplementation;
        address tokenRegistry;
    }

    // FIX 2: Use valid, consistent variable names in the struct.
    struct PlatformContracts {
        address auditTrail;
        address complianceManager;
        address feeManager;
        address tokenFactory;
        address assetTokenizer; // Corrected from 'HybridAssetTokenizer'
        address marketplaceCore;
        address rewardSystem;
        address adminGovernance;
    }

    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(DEPLOYER_ROLE, admin);
    }

    // FIX 3: Use consistent camelCase for function parameters.
    function deployPlatformClones(
        address auditTrailImpl,
        address complianceImpl,
        address feeManagerImpl,
        address tokenFactoryImpl,
        address hybridAssetTokenizerImpl, // Corrected from 'HybridAssetTokenizerImpl'
        address marketplaceImpl,
        address rewardSystemImpl,
        address adminGovernanceImpl,
        DeploymentConfig calldata cfg
    ) external onlyRole(DEPLOYER_ROLE) returns (PlatformContracts memory pc) {
        // Clone implementations
        pc.auditTrail = auditTrailImpl.clone();
        emit ComponentCloned(auditTrailImpl, pc.auditTrail);

        pc.complianceManager = complianceImpl.clone();
        emit ComponentCloned(complianceImpl, pc.complianceManager);

        pc.feeManager = feeManagerImpl.clone();
        emit ComponentCloned(feeManagerImpl, pc.feeManager);

        pc.tokenFactory = tokenFactoryImpl.clone();
        emit ComponentCloned(tokenFactoryImpl, pc.tokenFactory);

        // FIX 4: Use the corrected struct field and parameter name.
        pc.assetTokenizer = hybridAssetTokenizerImpl.clone();
        emit ComponentCloned(hybridAssetTokenizerImpl, pc.assetTokenizer);

        pc.marketplaceCore = marketplaceImpl.clone();
        emit ComponentCloned(marketplaceImpl, pc.marketplaceCore);

        pc.rewardSystem = rewardSystemImpl.clone();
        emit ComponentCloned(rewardSystemImpl, pc.rewardSystem);

        pc.adminGovernance = adminGovernanceImpl.clone();
        emit ComponentCloned(adminGovernanceImpl, pc.adminGovernance);

        // FIX 5: Use interface types for all initialization and configuration calls.
        IAuditTrail(payable(pc.auditTrail)).initialize(msg.sender);

        IComplianceManager(payable(pc.complianceManager)).initialize(
            msg.sender,
            pc.auditTrail
        );

        IFeeManager(payable(pc.feeManager)).initialize(
            msg.sender,
            cfg.treasury
        );

        ITokenFactory(payable(pc.tokenFactory)).initialize(
            cfg.tokenImplementation,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager,
            cfg.tokenRegistry,
            msg.sender
        );

        // Use the correct interface and struct field
        // IAssetTokenizer(payable(pc.assetTokenizer)).initialize(
        //     msg.sender,
        //     pc.tokenFactory,
        //     pc.complianceManager,
        //     pc.auditTrail,
        //     pc.feeManager
        // );

        // AFTER (This is the fix)
        IAssetTokenizer(payable(pc.assetTokenizer)).initialize(
            msg.sender,
            pc.tokenFactory,
            cfg.tokenRegistry, // <-- ADD THIS MISSING ARGUMENT
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
            cfg.rewardToken,
            pc.auditTrail
        );

        IAdminGovernance(payable(pc.adminGovernance)).initialize(
            msg.sender,
            pc.auditTrail
        );

        // Apply configuration using interfaces
        if (cfg.kycExpiryPeriod > 0) {
            IComplianceManager(payable(pc.complianceManager)).setKYCExpiryPeriod(
                cfg.kycExpiryPeriod
            );
        }

        if (cfg.minAssetValue > 0) {
            // Ensure IAssetTokenizer interface has this function
            IAssetTokenizer(payable(pc.assetTokenizer)).setMinAssetValue(
                cfg.minAssetValue
            );
        }

        if (cfg.tradingFeePercentage > 0) {
            IFeeManager(payable(pc.feeManager)).setFeeStructure(
                keccak256("TRADING"),
                cfg.tradingFeePercentage,
                0,
                0,
                0,
                true
            );
        }

        if (cfg.tokenizationFeePercentage > 0) {
            IFeeManager(payable(pc.feeManager)).setFeeStructure(
                keccak256("TOKENIZATION"),
                cfg.tokenizationFeePercentage,
                0,
                0.1 ether,
                10 ether,
                true
            );
        }

        emit PlatformDeployed(pc.auditTrail, msg.sender);
        return pc;
    }
}