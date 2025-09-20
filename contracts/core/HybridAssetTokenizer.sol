// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import { Initializable } from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

interface ITokenFactory {
    function createToken(
        string memory name,
        string memory symbol,
        uint256 totalSupply,
        uint8 decimals,
        uint8 tokenType,
        string memory metadataURI
    ) external returns (address);
}

interface ITokenRegistry {
    function registerToken(
        address token,
        string memory name,
        string memory symbol,
        uint256 supply,
        uint8 decimals,
        uint8 tokenType,
        address creator,
        string memory metadataURI
    ) external;
}

interface IComplianceManager {
    function isKYCVerified(address user) external view returns (bool);
}

interface IAuditTrail {
    function logTransaction(
        bytes32 txType,
        address user,
        uint256 amount,
        bytes calldata data
    ) external;
}

interface IFeeManager {
    function calculateFees(uint256 amount, bytes32 feeType) external view returns (uint256);
    function collectFees(bytes32 feeType, address payer) external payable returns (uint256);
}

contract HybridAssetTokenizer is
    Initializable,
    ERC721Upgradeable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable,
    UUPSUpgradeable
{
    /*//////////////////////////////////////////////////////////////
                            ACCESS ROLES
    //////////////////////////////////////////////////////////////*/
    bytes32 public constant PLATFORM_ADMIN_ROLE = keccak256("PLATFORM_ADMIN_ROLE");
    bytes32 public constant VALUATOR_ROLE = keccak256("VALUATOR_ROLE");
    bytes32 public constant ASSET_ADMIN_ROLE = keccak256("ASSET_ADMIN_ROLE");

    /*//////////////////////////////////////////////////////////////
                            ENUMS & STRUCTS
    //////////////////////////////////////////////////////////////*/
    enum AssetType {
        REAL_ESTATE,
        BUSINESS,
        COMMODITY,
        INTELLECTUAL_PROPERTY,
        ARTWORK,
        VEHICLE,
        OTHER
    }

    enum AssetStatus {
        PENDING,
        UNDER_REVIEW,
        VERIFIED,
        TOKENIZED,
        SUSPENDED,
        REJECTED
    }

    struct Asset {
        uint256 assetId;
        string name;
        string description;
        AssetType assetType;
        AssetStatus status;
        uint256 totalValue;
        uint256 totalTokens;
        address tokenAddress;
        address owner;
        string[] documentHashes;
        string location;
        uint256 createdAt;
        uint256 tokenizedAt;
        string rejectionReason;
    }

    struct Valuation {
        uint256 value;
        address valuator;
        uint256 timestamp;
        string reportHash;
        bool isVerified;
        string methodology;
        uint256 validUntil;
    }

    /*//////////////////////////////////////////////////////////////
                            CORE CONTRACTS
    //////////////////////////////////////////////////////////////*/
    ITokenFactory public tokenFactory;
    ITokenRegistry public tokenRegistry;
    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;
    IFeeManager public feeManager;

    // Only VerificationGateway can create hybrid assets
    address public verificationGateway;

    /*//////////////////////////////////////////////////////////////
                            STORAGE
    //////////////////////////////////////////////////////////////*/
    mapping(uint256 => address) public deedToToken; // assetId -> fractional ERC20
    mapping(uint256 => string) private _deedURIs;   // ERC721 metadata storage
    
    // Merged from AssetTokenizer
    mapping(uint256 => Asset) public assets;
    mapping(uint256 => Valuation[]) public assetValuations;
    mapping(address => uint256[]) public ownerAssets;
    mapping(address => bool) public authorizedValuators;
    mapping(string => bool) public usedDocumentHashes;
    
    uint256 public nextAssetId;
    uint256 public minValuations;
    uint256 public valuationValidPeriod;
    uint256 public minAssetValue;

    /*//////////////////////////////////////////////////////////////
                            EVENTS
    //////////////////////////////////////////////////////////////*/
    event AssetRegistered(uint256 indexed assetId, address indexed owner, string name, AssetType assetType);
    event AssetValued(uint256 indexed assetId, uint256 value, address indexed valuator);
    event AssetVerified(uint256 indexed assetId);
    event AssetRejected(uint256 indexed assetId, string reason);
    event HybridAssetCreated(uint256 indexed assetId, address indexed tokenAddress, uint256 totalTokens);
    event ValuatorAuthorized(address indexed valuator, bool authorized);
    event DocumentAdded(uint256 indexed assetId, string documentHash);

    /*//////////////////////////////////////////////////////////////
                            MODIFIERS
    //////////////////////////////////////////////////////////////*/
    modifier onlyGateway() {
        require(msg.sender == verificationGateway, "HybridAssetTokenizer: only gateway");
        _;
    }

    modifier onlyAssetOwner(uint256 assetId) {
        require(assets[assetId].owner == msg.sender, "Not asset owner");
        _;
    }

    /*//////////////////////////////////////////////////////////////
                                INIT
    //////////////////////////////////////////////////////////////*/
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address admin,
        address tokenFactory_,
        address tokenRegistry_,
        address complianceManager_,
        address auditTrail_,
        address feeManager_
    ) public initializer {
        __ERC721_init("Hybrid Real Estate Deed", "HRED");
        __AccessControl_init();
        __Pausable_init();
        __ReentrancyGuard_init();
        __UUPSUpgradeable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(PLATFORM_ADMIN_ROLE, admin);
        _grantRole(ASSET_ADMIN_ROLE, admin);
        _grantRole(VALUATOR_ROLE, admin);

        tokenFactory = ITokenFactory(tokenFactory_);
        tokenRegistry = ITokenRegistry(tokenRegistry_);
        complianceManager = IComplianceManager(complianceManager_);
        auditTrail = IAuditTrail(auditTrail_);
        feeManager = IFeeManager(feeManager_);

        authorizedValuators[admin] = true;
        
        nextAssetId = 1;
        minValuations = 2;
        valuationValidPeriod = 180 days;
        minAssetValue = 1000 ether;
    }

    /*//////////////////////////////////////////////////////////////
                            ASSET REGISTRATION
    //////////////////////////////////////////////////////////////*/
    function registerAsset(
        string calldata name,
        string calldata description,
        AssetType assetType,
        string[] calldata documentHashes,
        string calldata location
    ) external whenNotPaused returns (uint256) {
        require(complianceManager.isKYCVerified(msg.sender), "KYC required");
        require(bytes(name).length > 0 && bytes(name).length <= 100, "Invalid name");
        require(bytes(description).length > 0 && bytes(description).length <= 500, "Invalid description");
        require(documentHashes.length > 0 && documentHashes.length <= 10, "Invalid documents");

        // Verify unique document hashes
        for (uint256 i = 0; i < documentHashes.length; i++) {
            require(!usedDocumentHashes[documentHashes[i]], "Document already used");
            usedDocumentHashes[documentHashes[i]] = true;
        }

        uint256 assetId = nextAssetId++;

        Asset storage asset = assets[assetId];
        asset.assetId = assetId;
        asset.name = name;
        asset.description = description;
        asset.assetType = assetType;
        asset.status = AssetStatus.PENDING;
        asset.owner = msg.sender;
        asset.location = location;
        asset.createdAt = block.timestamp;

        for (uint256 i = 0; i < documentHashes.length; i++) {
            asset.documentHashes.push(documentHashes[i]);
        }

        ownerAssets[msg.sender].push(assetId);

        auditTrail.logTransaction(
            keccak256("ASSET_REGISTERED"),
            msg.sender,
            assetId,
            abi.encodePacked(name, uint256(assetType))
        );

        emit AssetRegistered(assetId, msg.sender, name, assetType);
        return assetId;
    }

    /*//////////////////////////////////////////////////////////////
                            VALUATIONS
    //////////////////////////////////////////////////////////////*/
    function addValuation(
        uint256 assetId,
        uint256 value,
        string calldata reportHash,
        string calldata methodology
    ) external whenNotPaused {
        require(authorizedValuators[msg.sender], "Not authorized valuator");
        require(assets[assetId].assetId != 0, "Asset not found");
        require(value >= minAssetValue, "Value below minimum");

        Asset storage asset = assets[assetId];
        require(
            asset.status == AssetStatus.PENDING || asset.status == AssetStatus.UNDER_REVIEW,
            "Asset not reviewable"
        );

        uint256 validUntil = block.timestamp + valuationValidPeriod;

        assetValuations[assetId].push(
            Valuation({
                value: value,
                valuator: msg.sender,
                timestamp: block.timestamp,
                reportHash: reportHash,
                isVerified: true,
                methodology: methodology,
                validUntil: validUntil
            })
        );

        if (asset.status == AssetStatus.PENDING) {
            asset.status = AssetStatus.UNDER_REVIEW;
        }

        emit AssetValued(assetId, value, msg.sender);
    }

    function getLatestValuation(uint256 assetId) external view returns (uint256) {
        return _getAverageValuation(assetId);
    }

    /*//////////////////////////////////////////////////////////////
                            ASSET VERIFICATION
    //////////////////////////////////////////////////////////////*/
    function verifyAsset(uint256 assetId) external onlyRole(ASSET_ADMIN_ROLE) {
        Asset storage asset = assets[assetId];
        require(asset.status == AssetStatus.UNDER_REVIEW, "Not under review");

        uint256 validCount = _getValidValuationsCount(assetId);
        require(validCount >= minValuations, "Insufficient valuations");

        asset.status = AssetStatus.VERIFIED;
        asset.totalValue = _getAverageValuation(assetId);

        emit AssetVerified(assetId);
    }

    /*//////////////////////////////////////////////////////////////
                        HYBRID ASSET CREATION
    //////////////////////////////////////////////////////////////*/
    function createHybridAsset(
        uint256 assetId,
        string memory name,
        string memory symbol,
        uint256 supply,
        uint8 decimals,
        uint8 tokenType,
        string memory metadataURI,
        string memory deedURI
    ) external onlyGateway whenNotPaused nonReentrant returns (address tokenAddress) {
        Asset storage asset = assets[assetId];
        
        require(asset.assetId == assetId, "Asset not found");
        require(asset.status == AssetStatus.VERIFIED, "Asset not verified");
        require(asset.tokenAddress == address(0), "Already tokenized");

        // Collect tokenization fee
        uint256 fee = feeManager.calculateFees(asset.totalValue, keccak256("TOKENIZATION"));
        if (fee > 0) {
            feeManager.collectFees{value: fee}(keccak256("TOKENIZATION"), asset.owner);
        }

        // 1. Deploy ERC20 fractional token
        tokenAddress = tokenFactory.createToken(
            name,
            symbol,
            supply,
            decimals,
            tokenType,
            metadataURI
        );

        // 2. Register in registry
        tokenRegistry.registerToken(
            tokenAddress,
            name,
            symbol,
            supply,
            decimals,
            tokenType,
            asset.owner,
            metadataURI
        );

        // 3. Mint deed NFT for the property
        _mint(asset.owner, assetId);
        _deedURIs[assetId] = deedURI;

        // 4. Update asset status
        asset.tokenAddress = tokenAddress;
        asset.totalTokens = supply;
        asset.status = AssetStatus.TOKENIZED;
        asset.tokenizedAt = block.timestamp;

        deedToToken[assetId] = tokenAddress;

        emit HybridAssetCreated(assetId, tokenAddress, supply);
    }

    /*//////////////////////////////////////////////////////////////
                            ADMIN FUNCTIONS
    //////////////////////////////////////////////////////////////*/
    function setVerificationGateway(address gateway) external onlyRole(PLATFORM_ADMIN_ROLE) {
        require(gateway != address(0), "Invalid gateway");
        verificationGateway = gateway;
    }

    function authorizeValuator(address valuator, bool authorized) external onlyRole(ASSET_ADMIN_ROLE) {
        authorizedValuators[valuator] = authorized;
        
        if (authorized) {
            _grantRole(VALUATOR_ROLE, valuator);
        } else {
            _revokeRole(VALUATOR_ROLE, valuator);
        }
        
        emit ValuatorAuthorized(valuator, authorized);
    }

    function setMinAssetValue(uint256 newValue) external onlyRole(PLATFORM_ADMIN_ROLE) {
        require(newValue > 0, "Invalid value");
        minAssetValue = newValue;
    }

    function setMinValuations(uint256 newMin) external onlyRole(PLATFORM_ADMIN_ROLE) {
        require(newMin > 0, "Invalid minimum");
        minValuations = newMin;
    }

    /*//////////////////////////////////////////////////////////////
                            VIEW FUNCTIONS
    //////////////////////////////////////////////////////////////*/
    function _getValidValuationsCount(uint256 assetId) internal view returns (uint256) {
        Valuation[] storage valuations = assetValuations[assetId];
        uint256 validCount = 0;

        for (uint256 i = 0; i < valuations.length; i++) {
            if (valuations[i].isVerified && block.timestamp <= valuations[i].validUntil) {
                validCount++;
            }
        }
        return validCount;
    }

    function _getAverageValuation(uint256 assetId) internal view returns (uint256) {
        Valuation[] storage valuations = assetValuations[assetId];
        uint256 totalValue = 0;
        uint256 validCount = 0;

        for (uint256 i = 0; i < valuations.length; i++) {
            if (valuations[i].isVerified && block.timestamp <= valuations[i].validUntil) {
                totalValue += valuations[i].value;
                validCount++;
            }
        }

        require(validCount > 0, "No valid valuations");
        return totalValue / validCount;
    }

    function getAssetsByOwner(address owner) external view returns (uint256[] memory) {
        return ownerAssets[owner];
    }

    function getAssetValuations(uint256 assetId) external view returns (Valuation[] memory) {
        return assetValuations[assetId];
    }

    /*//////////////////////////////////////////////////////////////
                            ERC721 OVERRIDES
    //////////////////////////////////////////////////////////////*/
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        require(_ownerOf(tokenId) != address(0), "Nonexistent token");
        return _deedURIs[tokenId];
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(ERC721Upgradeable, AccessControlUpgradeable)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    /*//////////////////////////////////////////////////////////////
                        UPGRADEABILITY & SAFETY
    //////////////////////////////////////////////////////////////*/
    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyRole(PLATFORM_ADMIN_ROLE)
    {}

    function pause() external onlyRole(PLATFORM_ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(PLATFORM_ADMIN_ROLE) {
        _unpause();
    }
}