// contracts/factory/TokenizationPlatformFactory.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";

// Import all core contracts
import "../core/AuditTrail.sol";
import "../core/ComplianceManager.sol";
import "../core/FeeManager.sol";
import "../core/TokenFactory.sol";
import "../core/AssetTokenizer.sol";
import "../core/MarketplaceCore.sol";
import "../core/RewardSystem.sol";
import "../core/AdminGovernance.sol";

/**
 * @title TokenizationPlatformFactory
 * @dev Factory contract for deploying the complete tokenization platform
 * @notice One-click deployment and management of the entire platform ecosystem
 */
contract TokenizationPlatformFactory is
    Initializable,
    AccessControlUpgradeable
{
    bytes32 public constant PLATFORM_ADMIN_ROLE =
        keccak256("PLATFORM_ADMIN_ROLE");
    bytes32 public constant DEPLOYER_ROLE = keccak256("DEPLOYER_ROLE");

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
        bool isDeployed;
        uint256 deployedAt;
        string version;
    }

    struct DeploymentConfig {
        address treasury;
        address rewardToken;
        uint256 tradingFeePercentage;
        uint256 tokenizationFeePercentage;
        uint256 minAssetValue;
        uint256 kycExpiryPeriod;
        bool useUpgradeableProxies;
    }

    mapping(address => PlatformContracts) public platformDeployments;
    mapping(string => address) public platformsByName;
    address[] public allPlatforms;

    address public platformAdmin;
    string public constant CURRENT_VERSION = "1.0.0";
    uint256 public totalDeployments = 0;
    uint256 public deploymentFee = 0.1 ether;

    // Implementation contract addresses for proxy deployment
    address public auditTrailImplementation;
    address public complianceManagerImplementation;
    address public feeManagerImplementation;
    address public tokenFactoryImplementation;
    address public assetTokenizerImplementation;
    address public marketplaceCoreImplementation;
    address public rewardSystemImplementation;
    address public adminGovernanceImplementation;

    event PlatformDeployed(
        address indexed admin,
        address indexed platformAddress,
        string platformName,
        uint256 timestamp
    );
    event ContractUpgraded(
        address indexed platform,
        string contractName,
        address newImplementation
    );
    event ImplementationUpdated(string contractName, address newImplementation);
    event DeploymentFeeUpdated(uint256 oldFee, uint256 newFee);

    function initialize(address admin) public initializer {
        __AccessControl_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(PLATFORM_ADMIN_ROLE, admin);
        _grantRole(DEPLOYER_ROLE, admin);

        platformAdmin = admin;
        _deployImplementations();
    }

    function deployPlatform(
        string calldata platformName,
        DeploymentConfig calldata config
    ) external payable returns (address) {
        require(hasRole(DEPLOYER_ROLE, msg.sender), "Not authorized deployer");
        require(bytes(platformName).length > 0, "Platform name required");
        require(
            platformsByName[platformName] == address(0),
            "Platform name taken"
        );
        require(msg.value >= deploymentFee, "Insufficient deployment fee");

        // Validate config
        require(config.treasury != address(0), "Invalid treasury address");
        require(config.tradingFeePercentage <= 1000, "Trading fee too high"); // Max 10%
        require(
            config.tokenizationFeePercentage <= 500,
            "Tokenization fee too high"
        ); // Max 5%

        address platformAddress = _deployPlatformContracts(config);

        platformDeployments[platformAddress] = PlatformContracts({
            auditTrail: address(0), // Will be set in _deployPlatformContracts
            complianceManager: address(0),
            feeManager: address(0),
            tokenFactory: address(0),
            assetTokenizer: address(0),
            marketplaceCore: address(0),
            rewardSystem: address(0),
            adminGovernance: address(0),
            proxyAdmin: address(0),
            isDeployed: true,
            deployedAt: block.timestamp,
            version: CURRENT_VERSION
        });

        platformsByName[platformName] = platformAddress;
        allPlatforms.push(platformAddress);
        totalDeployments += 1;

        // Refund excess payment
        if (msg.value > deploymentFee) {
            payable(msg.sender).transfer(msg.value - deploymentFee);
        }

        emit PlatformDeployed(
            msg.sender,
            platformAddress,
            platformName,
            block.timestamp
        );
        return platformAddress;
    }

    function _deployPlatformContracts(
        DeploymentConfig memory config
    ) internal returns (address) {
        address auditTrail;
        address complianceManager;
        address payable feeManager;
        address tokenFactory;
        address payable assetTokenizer;
        address payable marketplaceCore;
        address payable rewardSystem;
        address payable adminGovernance;
        address proxyAdmin;

        if (config.useUpgradeableProxies) {
            // Deploy ProxyAdmin
            proxyAdmin = address(new ProxyAdmin(msg.sender));
            ProxyAdmin(proxyAdmin).transferOwnership(msg.sender);

            // Deploy proxies
            auditTrail = _deployProxy(auditTrailImplementation, proxyAdmin);
            complianceManager = _deployProxy(
                complianceManagerImplementation,
                proxyAdmin
            );
            feeManager = payable(
                _deployProxy(feeManagerImplementation, proxyAdmin)
            );
            tokenFactory = _deployProxy(tokenFactoryImplementation, proxyAdmin);
            assetTokenizer = payable(
                _deployProxy(assetTokenizerImplementation, proxyAdmin)
            );
            marketplaceCore = payable(
                _deployProxy(marketplaceCoreImplementation, proxyAdmin)
            );
            rewardSystem = payable(
                _deployProxy(rewardSystemImplementation, proxyAdmin)
            );
            adminGovernance = payable(
                _deployProxy(adminGovernanceImplementation, proxyAdmin)
            );
        } else {
            // Deploy directly without proxies
            auditTrail = address(new AuditTrail());
            complianceManager = address(new ComplianceManager());
            feeManager = payable(address(new FeeManager()));
            tokenFactory = address(new TokenFactory());
            assetTokenizer = payable(address(new AssetTokenizer()));
            marketplaceCore = payable(address(new MarketplaceCore()));
            rewardSystem = payable(address(new RewardSystem()));
            adminGovernance = payable(address(new AdminGovernance()));
        }

        // Initialize contracts
        _initializeContracts(
            auditTrail,
            complianceManager,
            feeManager,
            tokenFactory,
            assetTokenizer,
            marketplaceCore,
            rewardSystem,
            adminGovernance,
            config
        );

        // Update platform deployment record
        PlatformContracts storage platform = platformDeployments[auditTrail]; // Use auditTrail as platform ID
        platform.auditTrail = auditTrail;
        platform.complianceManager = complianceManager;
        platform.feeManager = feeManager;
        platform.tokenFactory = tokenFactory;
        platform.assetTokenizer = assetTokenizer;
        platform.marketplaceCore = marketplaceCore;
        platform.rewardSystem = rewardSystem;
        platform.adminGovernance = adminGovernance;
        platform.proxyAdmin = proxyAdmin;

        return auditTrail; // Return auditTrail address as platform identifier
    }

    function _deployProxy(
        address implementation,
        address admin
    ) internal returns (address) {
        bytes memory initData = ""; // Empty init data, will initialize separately
        return
            address(
                new TransparentUpgradeableProxy(implementation, admin, initData)
            );
    }

    function _initializeContracts(
        address auditTrail,
        address complianceManager,
        address payable feeManager,
        address tokenFactory,
        address payable assetTokenizer,
        address payable marketplaceCore,
        address payable rewardSystem,
        address payable adminGovernance,
        DeploymentConfig memory config
    ) internal {
        // Initialize AuditTrail
        AuditTrail(auditTrail).initialize(msg.sender);

        // Initialize ComplianceManager
        ComplianceManager(complianceManager).initialize(msg.sender, auditTrail);

        // Initialize FeeManager
        FeeManager(feeManager).initialize(msg.sender, config.treasury);

        // Initialize TokenFactory
        TokenFactory(tokenFactory).initialize(
            msg.sender,
            complianceManager,
            auditTrail,
            feeManager
        );

        // Initialize AssetTokenizer
        AssetTokenizer(assetTokenizer).initialize(
            msg.sender,
            tokenFactory,
            complianceManager,
            auditTrail,
            feeManager
        );

        // Initialize MarketplaceCore
        MarketplaceCore(marketplaceCore).initialize(
            msg.sender,
            complianceManager,
            auditTrail,
            feeManager
        );

        // Initialize RewardSystem
        RewardSystem(rewardSystem).initialize(
            msg.sender,
            config.rewardToken,
            auditTrail
        );

        // Initialize AdminGovernance
        AdminGovernance(adminGovernance).initialize(msg.sender, auditTrail);

        // Configure initial parameters
        _configurePlatform(
            complianceManager,
            feeManager,
            assetTokenizer,
            config
        );
    }

    function _configurePlatform(
        address complianceManager,
        address payable feeManager,
        address payable assetTokenizer,
        DeploymentConfig memory config
    ) internal {
        // Set compliance parameters
        if (config.kycExpiryPeriod > 0) {
            // ComplianceManager(complianceManager).setKYCExpiryPeriod(config.kycExpiryPeriod);
        }

        // Set asset tokenizer parameters
        if (config.minAssetValue > 0) {
            // AssetTokenizer(assetTokenizer).setMinAssetValue(config.minAssetValue);
        }

        // Configure fee structures
        if (config.tradingFeePercentage > 0) {
            FeeManager(feeManager).setFeeStructure(
                keccak256("TRADING"),
                config.tradingFeePercentage,
                0, // No flat fee
                0, // No min fee
                0, // No max fee
                true // Percentage only
            );
        }

        if (config.tokenizationFeePercentage > 0) {
            FeeManager(feeManager).setFeeStructure(
                keccak256("TOKENIZATION"),
                config.tokenizationFeePercentage,
                0,
                0.1 ether, // Min fee
                10 ether, // Max fee
                true
            );
        }
    }

    function _deployImplementations() internal {
        auditTrailImplementation = address(new AuditTrail());
        complianceManagerImplementation = address(new ComplianceManager());
        feeManagerImplementation = address(new FeeManager());
        tokenFactoryImplementation = address(new TokenFactory());
        assetTokenizerImplementation = address(new AssetTokenizer());
        marketplaceCoreImplementation = address(new MarketplaceCore());
        rewardSystemImplementation = address(new RewardSystem());
        adminGovernanceImplementation = address(new AdminGovernance());
    }

    // Platform management functions
    function upgradePlatformContract(
        address platformAddress,
        string calldata contractName,
        address newImplementation
    ) external onlyRole(PLATFORM_ADMIN_ROLE) {
        require(
            platformDeployments[platformAddress].isDeployed,
            "Platform not found"
        );
        require(newImplementation != address(0), "Invalid implementation");

        PlatformContracts storage platform = platformDeployments[
            platformAddress
        ];
        require(platform.proxyAdmin != address(0), "Not upgradeable platform");

        address proxyAddress = _getContractAddress(platform, contractName);
        require(proxyAddress != address(0), "Contract not found");

        // Use upgradeAndCall instead of upgrade
        ProxyAdmin(platform.proxyAdmin).upgradeAndCall(
            ITransparentUpgradeableProxy(proxyAddress),
            newImplementation,
            ""
        );

        emit ContractUpgraded(platformAddress, contractName, newImplementation);
    }

    function updateImplementation(
        string calldata contractName,
        address newImplementation
    ) external onlyRole(PLATFORM_ADMIN_ROLE) {
        require(newImplementation != address(0), "Invalid implementation");

        bytes32 nameHash = keccak256(bytes(contractName));

        if (nameHash == keccak256("AuditTrail")) {
            auditTrailImplementation = newImplementation;
        } else if (nameHash == keccak256("ComplianceManager")) {
            complianceManagerImplementation = newImplementation;
        } else if (nameHash == keccak256("FeeManager")) {
            feeManagerImplementation = newImplementation;
        } else if (nameHash == keccak256("TokenFactory")) {
            tokenFactoryImplementation = newImplementation;
        } else if (nameHash == keccak256("AssetTokenizer")) {
            assetTokenizerImplementation = newImplementation;
        } else if (nameHash == keccak256("MarketplaceCore")) {
            marketplaceCoreImplementation = newImplementation;
        } else if (nameHash == keccak256("RewardSystem")) {
            rewardSystemImplementation = newImplementation;
        } else if (nameHash == keccak256("AdminGovernance")) {
            adminGovernanceImplementation = newImplementation;
        } else {
            revert("Unknown contract name");
        }

        emit ImplementationUpdated(contractName, newImplementation);
    }

    function _getContractAddress(
        PlatformContracts storage platform,
        string calldata contractName
    ) internal view returns (address) {
        bytes32 nameHash = keccak256(bytes(contractName));

        if (nameHash == keccak256("AuditTrail")) return platform.auditTrail;
        if (nameHash == keccak256("ComplianceManager"))
            return platform.complianceManager;
        if (nameHash == keccak256("FeeManager")) return platform.feeManager;
        if (nameHash == keccak256("TokenFactory")) return platform.tokenFactory;
        if (nameHash == keccak256("AssetTokenizer"))
            return platform.assetTokenizer;
        if (nameHash == keccak256("MarketplaceCore"))
            return platform.marketplaceCore;
        if (nameHash == keccak256("RewardSystem")) return platform.rewardSystem;
        if (nameHash == keccak256("AdminGovernance"))
            return platform.adminGovernance;

        return address(0);
    }

    function setDeploymentFee(
        uint256 newFee
    ) external onlyRole(PLATFORM_ADMIN_ROLE) {
        uint256 oldFee = deploymentFee;
        deploymentFee = newFee;
        emit DeploymentFeeUpdated(oldFee, newFee);
    }

    function grantDeployerRole(
        address deployer
    ) external onlyRole(PLATFORM_ADMIN_ROLE) {
        _grantRole(DEPLOYER_ROLE, deployer);
    }

    function revokeDeployerRole(
        address deployer
    ) external onlyRole(PLATFORM_ADMIN_ROLE) {
        _revokeRole(DEPLOYER_ROLE, deployer);
    }

    // View functions
    function getPlatformContracts(
        address platformAddress
    ) external view returns (PlatformContracts memory) {
        return platformDeployments[platformAddress];
    }

    function getPlatformByName(
        string calldata name
    ) external view returns (address) {
        return platformsByName[name];
    }

    function getAllPlatforms() external view returns (address[] memory) {
        return allPlatforms;
    }

    function getImplementationAddresses()
        external
        view
        returns (
            address auditTrail,
            address complianceManager,
            address feeManager,
            address tokenFactory,
            address assetTokenizer,
            address marketplaceCore,
            address rewardSystem,
            address adminGovernance
        )
    {
        return (
            auditTrailImplementation,
            complianceManagerImplementation,
            feeManagerImplementation,
            tokenFactoryImplementation,
            assetTokenizerImplementation,
            marketplaceCoreImplementation,
            rewardSystemImplementation,
            adminGovernanceImplementation
        );
    }

    function isPlatformDeployed(
        address platformAddress
    ) external view returns (bool) {
        return platformDeployments[platformAddress].isDeployed;
    }

    function getTotalDeployments() external view returns (uint256) {
        return totalDeployments;
    }

    function getDeploymentStats()
        external
        view
        returns (
            uint256 totalPlatforms,
            uint256 totalFees,
            uint256 currentVersion,
            address factoryAdmin
        )
    {
        return (
            totalDeployments,
            address(this).balance,
            1, // Version as number
            platformAdmin
        );
    }

    // Emergency functions
    function withdrawFees() external onlyRole(PLATFORM_ADMIN_ROLE) {
        uint256 balance = address(this).balance;
        require(balance > 0, "No fees to withdraw");

        payable(platformAdmin).transfer(balance);
    }

    function pause() external onlyRole(PLATFORM_ADMIN_ROLE) {
        // Implementation for pausing deployments if needed
    }

    receive() external payable {
        // Accept ETH for deployment fees
    }
}
