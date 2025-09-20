// contracts/factory/PlatformDeployer.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/proxy/Clones.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

// Core contracts (replace with interface imports if preferred)
import "../core/AuditTrail.sol";
import "../core/ComplianceManager.sol";
import "../core/FeeManager.sol";
import "../core/TokenFactory.sol";
import "../core/AssetTokenizer.sol";
import "../core/MarketplaceCore.sol";
import "../core/RewardSystem.sol";
import "../core/AdminGovernance.sol";

/**
 * @title PlatformDeployer
 * @dev Deploys a platform by creating minimal proxies (Clones) for each implementation and
 *      initializing them. Keeps deployment logic out of the main factory.
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
        bool useUpgradeable; // not used for clones; kept for compatibility

        // Required for TokenFactory
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
    }

    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(DEPLOYER_ROLE, admin);
    }

    /**
     * @dev Deploys clones of implementations and initializes them.
     * @param auditTrailImpl Implementation of AuditTrail
     * @param complianceImpl Implementation of ComplianceManager
     * @param feeManagerImpl Implementation of FeeManager
     * @param tokenFactoryImpl Implementation of TokenFactory
     * @param assetTokenizerImpl Implementation of AssetTokenizer
     * @param marketplaceImpl Implementation of MarketplaceCore
     * @param rewardSystemImpl Implementation of RewardSystem
     * @param adminGovernanceImpl Implementation of AdminGovernance
     * @param cfg Deployment configuration
     */
    function deployPlatformClones(
        address auditTrailImpl,
        address complianceImpl,
        address feeManagerImpl,
        address tokenFactoryImpl,
        address assetTokenizerImpl,
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

        pc.assetTokenizer = assetTokenizerImpl.clone();
        emit ComponentCloned(assetTokenizerImpl, pc.assetTokenizer);

        pc.marketplaceCore = marketplaceImpl.clone();
        emit ComponentCloned(marketplaceImpl, pc.marketplaceCore);

        pc.rewardSystem = rewardSystemImpl.clone();
        emit ComponentCloned(rewardSystemImpl, pc.rewardSystem);

        pc.adminGovernance = adminGovernanceImpl.clone();
        emit ComponentCloned(adminGovernanceImpl, pc.adminGovernance);

        // Initialize clones
        AuditTrail(payable(pc.auditTrail)).initialize(msg.sender);

        ComplianceManager(payable(pc.complianceManager)).initialize(
            msg.sender,
            pc.auditTrail
        );

        FeeManager(payable(pc.feeManager)).initialize(
            msg.sender,
            cfg.treasury
        );

        TokenFactory(payable(pc.tokenFactory)).initialize(
            cfg.tokenImplementation,   // <- Token implementation contract
            pc.complianceManager,       // <- Compliance
            pc.auditTrail,              // <- Audit trail
            pc.feeManager,              // <- Fee manager
            cfg.tokenRegistry,          // <- Token registry
            msg.sender                  // <- Platform admin
        );

        AssetTokenizer(payable(pc.assetTokenizer)).initialize(
            msg.sender,
            pc.tokenFactory,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager
        );

        MarketplaceCore(payable(pc.marketplaceCore)).initialize(
            msg.sender,
            pc.complianceManager,
            pc.auditTrail,
            pc.feeManager
        );

        RewardSystem(payable(pc.rewardSystem)).initialize(
            msg.sender,
            cfg.rewardToken,
            pc.auditTrail
        );

        AdminGovernance(payable(pc.adminGovernance)).initialize(
            msg.sender,
            pc.auditTrail
        );

        // Apply configuration
        if (cfg.kycExpiryPeriod > 0) {
            ComplianceManager(payable(pc.complianceManager)).setKYCExpiryPeriod(
                cfg.kycExpiryPeriod
            );
        }

        if (cfg.minAssetValue > 0) {
            AssetTokenizer(payable(pc.assetTokenizer)).setMinAssetValue(
                cfg.minAssetValue
            );
        }

        if (cfg.tradingFeePercentage > 0) {
            FeeManager(payable(pc.feeManager)).setFeeStructure(
                keccak256("TRADING"),
                cfg.tradingFeePercentage,
                0,
                0,
                0,
                true
            );
        }

        if (cfg.tokenizationFeePercentage > 0) {
            FeeManager(payable(pc.feeManager)).setFeeStructure(
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
