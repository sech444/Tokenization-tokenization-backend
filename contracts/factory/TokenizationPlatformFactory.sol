
// / contracts/factory/TokenizationPlatformFactory.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";

interface IPlatformDeployer {
    struct DeploymentConfig {
        address treasury;
        address rewardToken;
        uint256 tradingFeePercentage;
        uint256 tokenizationFeePercentage;
        uint256 minAssetValue;
        uint256 kycExpiryPeriod;
        bool useUpgradeable;
    }

    function deployPlatformClones(
        address auditTrailImpl,
        address complianceImpl,
        address feeManagerImpl,
        address tokenFactoryImpl,
        address assetTokenizerImpl,
        address marketplaceImpl,
        address rewardSystemImpl,
        address adminGovernanceImpl,
        IPlatformDeployer.DeploymentConfig calldata cfg
    ) external returns (address /*auditTrail*/);
}

contract TokenizationPlatformFactory is AccessControl {
    bytes32 public constant PLATFORM_ADMIN_ROLE = keccak256("PLATFORM_ADMIN_ROLE");
    bytes32 public constant DEPLOYER_ROLE = keccak256("DEPLOYER_ROLE");

    address public platformAdmin;
    uint256 public deploymentFee = 0.1 ether;

    // Implementation addresses (set by admin)
    address public auditTrailImplementation;
    address public complianceImplementation;
    address public feeManagerImplementation;
    address public tokenFactoryImplementation;
    address public assetTokenizerImplementation;
    address public marketplaceImplementation;
    address public rewardSystemImplementation;
    address public adminGovernanceImplementation;

    address public platformDeployer; // external deployer contract

    event ImplementationUpdated(string name, address impl);
    event PlatformFactoryDeployed(address indexed platformId, address deployer);

    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(PLATFORM_ADMIN_ROLE, admin);
        _grantRole(DEPLOYER_ROLE, admin);
        platformAdmin = admin;
    }


    // Admin sets implementations (do not deploy here to keep factory small)
    function setImplementations(
        address auditTrail,
        address compliance,
        address feeMgr,
        address tokenFactory,
        address assetTokenizer,
        address marketplace,
        address rewardSystem,
        address adminGovernance
    ) external onlyRole(PLATFORM_ADMIN_ROLE) {
        auditTrailImplementation = auditTrail;
        complianceImplementation = compliance;
        feeManagerImplementation = feeMgr;
        tokenFactoryImplementation = tokenFactory;
        assetTokenizerImplementation = assetTokenizer;
        marketplaceImplementation = marketplace;
        rewardSystemImplementation = rewardSystem;
        adminGovernanceImplementation = adminGovernance;
    }

    function setPlatformDeployer(address deployerAddr) external onlyRole(PLATFORM_ADMIN_ROLE) {
        platformDeployer = deployerAddr;
    }

    function deployPlatform(string calldata name, IPlatformDeployer.DeploymentConfig calldata cfg) external payable onlyRole(DEPLOYER_ROLE) returns (address) {
        require(msg.value >= deploymentFee, "Insufficient fee");
        require(platformDeployer != address(0), "PlatformDeployer not set");

        address auditTrailAddr = IPlatformDeployer(platformDeployer).deployPlatformClones(
            auditTrailImplementation,
            complianceImplementation,
            feeManagerImplementation,
            tokenFactoryImplementation,
            assetTokenizerImplementation,
            marketplaceImplementation,
            rewardSystemImplementation,
            adminGovernanceImplementation,
            cfg
        );

        emit PlatformFactoryDeployed(auditTrailAddr, msg.sender);

        if (msg.value > deploymentFee) payable(msg.sender).transfer(msg.value - deploymentFee);
        return auditTrailAddr;
    }
}
