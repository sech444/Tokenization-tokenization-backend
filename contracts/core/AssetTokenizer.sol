// contracts/core/AssetTokenizer.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IAuditTrail.sol";
import "../interfaces/core/IFeeManager.sol";
import "../interfaces/core/ITokenFactory.sol";

/**
 * @title AssetTokenizer
 * @dev Tokenizes real-world assets like real estate and businesses.
 * Note: SafeMath removed (Solidity >=0.8 does overflow checks natively).
 */
contract AssetTokenizer is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable
{
    bytes32 public constant TOKENIZER_ROLE = keccak256("TOKENIZER_ROLE");
    bytes32 public constant ASSET_ADMIN_ROLE = keccak256("ASSET_ADMIN_ROLE");
    bytes32 public constant VALUATOR_ROLE = keccak256("VALUATOR_ROLE");

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

    struct TokenizationRequest {
        uint256 assetId;
        string tokenName;
        string tokenSymbol;
        uint256 requestedTokens;
        uint256 pricePerToken;
        uint256 requestTimestamp;
        address requester;
        bool isProcessed;
    }

    // Storage
    mapping(uint256 => Asset) public assets;
    mapping(uint256 => Valuation[]) public assetValuations;
    mapping(uint256 => TokenizationRequest) public tokenizationRequests;
    mapping(address => uint256[]) public ownerAssets;
    mapping(address => bool) public authorizedValuators;
    mapping(string => bool) public usedDocumentHashes;
    mapping(address => mapping(uint256 => bool)) public assetApprovals; // valuator => assetId => approved

    uint256 public nextAssetId;
    uint256 public nextRequestId;
    uint256 public minValuations;
    uint256 public valuationValidPeriod;
    uint256 public minAssetValue;
    uint256 public maxTokensPerAsset;

    ITokenFactory public tokenFactory;
    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;
    IFeeManager public feeManager;

    // Events
    event AssetRegistered(
        uint256 indexed assetId,
        address indexed owner,
        string name,
        AssetType assetType
    );
    event AssetValued(
        uint256 indexed assetId,
        uint256 value,
        address indexed valuator,
        string methodology
    );
    event AssetApproved(uint256 indexed assetId, address indexed approver);
    event AssetRejected(uint256 indexed assetId, string reason);
    event TokenizationRequested(
        uint256 indexed requestId,
        uint256 indexed assetId,
        address indexed requester
    );
    event AssetTokenized(
        uint256 indexed assetId,
        address indexed tokenAddress,
        uint256 totalTokens
    );
    event ValuatorAuthorized(address indexed valuator, bool authorized);
    event DocumentAdded(uint256 indexed assetId, string documentHash);

    // ---------------------------
    // Initialization
    // ---------------------------
    function initialize(
        address admin,
        address _tokenFactory,
        address _complianceManager,
        address _auditTrail,
        address _feeManager
    ) public initializer {
        __AccessControl_init();
        __Pausable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(TOKENIZER_ROLE, admin);
        _grantRole(ASSET_ADMIN_ROLE, admin);
        _grantRole(VALUATOR_ROLE, admin);

        tokenFactory = ITokenFactory(_tokenFactory);
        complianceManager = IComplianceManager(_complianceManager);
        auditTrail = IAuditTrail(_auditTrail);
        feeManager = IFeeManager(_feeManager);

        authorizedValuators[admin] = true;

        nextAssetId = 1;
        nextRequestId = 1;
        minValuations = 2;
        valuationValidPeriod = 180 days;
        minAssetValue = 1000 ether;
        maxTokensPerAsset = 1_000_000;
    }

    // ---------------------------
    // Asset registration & docs
    // ---------------------------
    function registerAsset(
        string calldata name,
        string calldata description,
        AssetType assetType,
        string[] calldata documentHashes,
        string calldata location
    ) external whenNotPaused returns (uint256) {
        require(complianceManager.isKYCVerified(msg.sender), "KYC required");
        require(
            bytes(name).length > 0 && bytes(name).length <= 100,
            "Invalid name length"
        );
        require(
            bytes(description).length > 0 && bytes(description).length <= 500,
            "Invalid description length"
        );
        require(
            documentHashes.length > 0 && documentHashes.length <= 10,
            "Invalid document count"
        );

        // Verify unique document hashes
        for (uint256 i = 0; i < documentHashes.length; i++) {
            require(
                !usedDocumentHashes[documentHashes[i]],
                "Document hash already used"
            );
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

    function addDocument(
        uint256 assetId,
        string calldata documentHash
    ) external {
        require(assets[assetId].owner == msg.sender, "Not asset owner");
        require(
            !usedDocumentHashes[documentHash],
            "Document hash already used"
        );
        require(
            assets[assetId].documentHashes.length < 10,
            "Too many documents"
        );

        assets[assetId].documentHashes.push(documentHash);
        usedDocumentHashes[documentHash] = true;

        emit DocumentAdded(assetId, documentHash);
    }

    // ---------------------------
    // Valuations & approval
    // ---------------------------
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
            asset.status == AssetStatus.PENDING ||
                asset.status == AssetStatus.UNDER_REVIEW,
            "Asset not in reviewable state"
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

        auditTrail.logTransaction(
            keccak256("ASSET_VALUED"),
            msg.sender,
            value,
            abi.encodePacked(assetId, reportHash)
        );

        emit AssetValued(assetId, value, msg.sender, methodology);
    }

    // function approveAsset(uint256 assetId) external onlyRole(ASSET_ADMIN_ROLE) {
    //     require(assets[assetId].assetId != 0, "Asset not found");

    //     Asset storage asset = assets[assetId];
    //     require(asset.status == AssetStatus.UNDER_REVIEW, "Asset not under review");

    //     uint256 validValuations = getValidValuationsCount(assetId);
    //     require(validValuations >= minValuations, "Insufficient valuations");

    //     asset.status = AssetStatus.VERIFIED;
    //     asset.totalValue = getAverageValuation(assetId);

    //     assetApprovals[msg.sender][assetId] = true;

    //     auditTrail.logTransaction(
    //         keccak256("ASSET_APPROVED"),
    //         msg.sender,
    //         assetId,
    //         abi.encodePacked(asset.totalValue)
    //     );

    //     emit AssetApproved(assetId, msg.sender);
    // }
    function approveAsset(uint256 assetId) external onlyRole(ASSET_ADMIN_ROLE) {
        require(assets[assetId].assetId != 0, "Asset not found");

        Asset storage asset = assets[assetId];
        require(
            asset.status == AssetStatus.UNDER_REVIEW,
            "Asset not under review"
        );

        uint256 validValuations = _getValidValuationsCount(assetId);
        require(validValuations >= minValuations, "Insufficient valuations");

        asset.status = AssetStatus.VERIFIED;
        asset.totalValue = _getAverageValuation(assetId);

        assetApprovals[msg.sender][assetId] = true;

        auditTrail.logTransaction(
            keccak256("ASSET_APPROVED"),
            msg.sender,
            assetId,
            abi.encodePacked(asset.totalValue)
        );

        emit AssetApproved(assetId, msg.sender);
    }

    // ---------------------------
    // Internal valuation helpers
    // ---------------------------
    function _getValidValuationsCount(
        uint256 assetId
    ) internal view returns (uint256) {
        Valuation[] storage valuations = assetValuations[assetId];
        uint256 validCount = 0;

        for (uint256 i = 0; i < valuations.length; i++) {
            if (
                valuations[i].isVerified &&
                block.timestamp <= valuations[i].validUntil
            ) {
                validCount++;
            }
        }
        return validCount;
    }

    function _getAverageValuation(
        uint256 assetId
    ) internal view returns (uint256) {
        Valuation[] storage valuations = assetValuations[assetId];
        uint256 totalValue = 0;
        uint256 validCount = 0;

        for (uint256 i = 0; i < valuations.length; i++) {
            if (
                valuations[i].isVerified &&
                block.timestamp <= valuations[i].validUntil
            ) {
                totalValue += valuations[i].value;
                validCount++;
            }
        }

        require(validCount > 0, "No valid valuations");
        return totalValue / validCount;
    }

    // ---------------------------
    // Public wrappers for reading
    // ---------------------------
    function getValidValuationsCount(
        uint256 assetId
    ) external view returns (uint256) {
        return _getValidValuationsCount(assetId);
    }

    function getAverageValuation(
        uint256 assetId
    ) external view returns (uint256) {
        return _getAverageValuation(assetId);
    }

    function rejectAsset(
        uint256 assetId,
        string calldata reason
    ) external onlyRole(ASSET_ADMIN_ROLE) {
        require(assets[assetId].assetId != 0, "Asset not found");
        require(bytes(reason).length > 0, "Rejection reason required");

        Asset storage asset = assets[assetId];
        require(
            asset.status == AssetStatus.PENDING ||
                asset.status == AssetStatus.UNDER_REVIEW,
            "Cannot reject asset in current state"
        );

        asset.status = AssetStatus.REJECTED;
        asset.rejectionReason = reason;

        auditTrail.logTransaction(
            keccak256("ASSET_REJECTED"),
            msg.sender,
            assetId,
            abi.encodePacked(reason)
        );

        emit AssetRejected(assetId, reason);
    }

    // ---------------------------
    // Tokenization
    // ---------------------------
    function requestTokenization(
        uint256 assetId,
        string calldata tokenName,
        string calldata tokenSymbol,
        uint256 requestedTokens,
        uint256 pricePerToken
    ) external whenNotPaused returns (uint256) {
        Asset storage asset = assets[assetId];
        require(asset.owner == msg.sender, "Not asset owner");
        require(asset.status == AssetStatus.VERIFIED, "Asset not verified");
        require(asset.tokenAddress == address(0), "Already tokenized");

        uint256 requestId = nextRequestId++;
        tokenizationRequests[requestId] = TokenizationRequest({
            assetId: assetId,
            tokenName: tokenName,
            tokenSymbol: tokenSymbol,
            requestedTokens: requestedTokens,
            pricePerToken: pricePerToken,
            requestTimestamp: block.timestamp,
            requester: msg.sender,
            isProcessed: false
        });

        emit TokenizationRequested(requestId, assetId, msg.sender);
        return requestId;
    }

    function processTokenization(
        uint256 requestId
    ) external payable nonReentrant whenNotPaused {
        TokenizationRequest storage request = tokenizationRequests[requestId];
        require(request.requester == msg.sender, "Not request owner");
        require(!request.isProcessed, "Already processed");

        Asset storage asset = assets[request.assetId];
        require(asset.status == AssetStatus.VERIFIED, "Asset not verified");

        uint256 fee = feeManager.calculateFees(
            asset.totalValue,
            feeManager.TOKENIZATION_FEE()
        );
        require(msg.value >= fee, "Insufficient fee");

        address tokenAddress = tokenFactory.createToken{value: fee}(
            request.tokenName,
            request.tokenSymbol,
            request.requestedTokens,
            18,
            ITokenFactory.TokenType.ASSET,
            ""
        );

        asset.tokenAddress = tokenAddress;
        asset.totalTokens = request.requestedTokens;
        asset.status = AssetStatus.TOKENIZED;
        asset.tokenizedAt = block.timestamp;
        request.isProcessed = true;

        auditTrail.logTransaction(
            keccak256("ASSET_TOKENIZED"),
            msg.sender,
            request.assetId,
            abi.encodePacked(tokenAddress, request.requestedTokens)
        );

        emit AssetTokenized(
            request.assetId,
            tokenAddress,
            request.requestedTokens
        );

        if (msg.value > fee) {
            payable(msg.sender).transfer(msg.value - fee);
        }
    }

    // ---------------------------
    // Admin utilities
    // ---------------------------
    function authorizeValuator(
        address valuator,
        bool authorized
    ) external onlyRole(ASSET_ADMIN_ROLE) {
        authorizedValuators[valuator] = authorized;

        if (authorized) {
            _grantRole(VALUATOR_ROLE, valuator);
        } else {
            _revokeRole(VALUATOR_ROLE, valuator);
        }

        emit ValuatorAuthorized(valuator, authorized);
    }

    // ---------------------------
    // View Helpers (Refactored)
    // ---------------------------

    function _getAssetsByStatus(
        AssetStatus status
    ) internal view returns (uint256[] memory) {
        uint256 count = 0;
        for (uint256 i = 1; i < nextAssetId; i++) {
            if (assets[i].assetId != 0 && assets[i].status == status) count++;
        }
        uint256[] memory result = new uint256[](count);
        uint256 idx = 0;
        for (uint256 i = 1; i < nextAssetId; i++) {
            if (assets[i].assetId != 0 && assets[i].status == status) {
                result[idx] = i;
                idx++;
            }
        }
        return result;
    }

    function getAssetsByStatus(
        AssetStatus status
    ) public view returns (uint256[] memory) {
        return _getAssetsByStatus(status);
    }

    function getTokenizedAssets() external view returns (uint256[] memory) {
        return _getAssetsByStatus(AssetStatus.TOKENIZED);
    }

    function getPendingAssets() external view returns (uint256[] memory) {
        uint256 count = 0;
        for (uint256 i = 1; i < nextAssetId; i++) {
            if (
                assets[i].status == AssetStatus.PENDING ||
                assets[i].status == AssetStatus.UNDER_REVIEW
            ) count++;
        }
        uint256[] memory result = new uint256[](count);
        uint256 idx = 0;
        for (uint256 i = 1; i < nextAssetId; i++) {
            if (
                assets[i].status == AssetStatus.PENDING ||
                assets[i].status == AssetStatus.UNDER_REVIEW
            ) {
                result[idx] = i;
                idx++;
            }
        }
        return result;
    }

    function getAssetsByType(
        AssetType assetType
    ) external view returns (uint256[] memory) {
        uint256 count = 0;
        for (uint256 i = 1; i < nextAssetId; i++) {
            if (assets[i].assetType == assetType && assets[i].assetId != 0)
                count++;
        }
        uint256[] memory result = new uint256[](count);
        uint256 idx = 0;
        for (uint256 i = 1; i < nextAssetId; i++) {
            if (assets[i].assetType == assetType && assets[i].assetId != 0) {
                result[idx] = i;
                idx++;
            }
        }
        return result;
    }

    // Emergency / admin
    function suspendAsset(
        uint256 assetId,
        string calldata reason
    ) external onlyRole(ASSET_ADMIN_ROLE) {
        assets[assetId].status = AssetStatus.SUSPENDED;
        auditTrail.logTransaction(
            keccak256("ASSET_SUSPENDED"),
            msg.sender,
            assetId,
            abi.encodePacked(reason)
        );
    }

    receive() external payable {}
}
