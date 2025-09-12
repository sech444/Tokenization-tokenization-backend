// contracts/core/TokenFactory.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts/proxy/Clones.sol";
import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IAuditTrail.sol";
import "../interfaces/core/IFeeManager.sol";
import "../tokens/AssetToken.sol";

/**
 * @title TokenFactory
 * @dev Factory contract for creating compliant ERC20 tokens
 * @notice Uses minimal proxy pattern for gas-efficient token deployment
 */
contract TokenFactory is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable
{
    bytes32 public constant TOKEN_CREATOR_ROLE =
        keccak256("TOKEN_CREATOR_ROLE");
    bytes32 public constant SYSTEM_ROLE = keccak256("SYSTEM_ROLE");

    enum TokenType {
        ASSET,
        UTILITY,
        SECURITY,
        GOVERNANCE
    }

    struct TokenInfo {
        address tokenAddress;
        string name;
        string symbol;
        uint256 totalSupply;
        uint8 decimals;
        TokenType tokenType;
        address creator;
        uint256 createdAt;
        bool isActive;
        bool isCompliant;
        string metadataURI;
    }

    struct TokenStats {
        uint256 totalHolders;
        uint256 totalTransfers;
        uint256 totalVolume;
        uint256 lastActivity;
    }

    mapping(address => TokenInfo) public tokens;
    mapping(address => TokenStats) public tokenStats;
    mapping(address => address[]) public userTokens;
    mapping(TokenType => address[]) public tokensByType;
    address[] public allTokens;

    address public tokenImplementation;
    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;
    IFeeManager public feeManager;

    uint256 public totalTokensCreated;
    uint256 public minNameLength = 3;
    uint256 public maxNameLength = 50;
    uint256 public minSymbolLength = 2;
    uint256 public maxSymbolLength = 10;

    event TokenCreated(
        address indexed tokenAddress,
        address indexed creator,
        string name,
        string symbol,
        uint256 totalSupply,
        TokenType tokenType
    );

    event TokenUpdated(address indexed tokenAddress, string metadataURI);
    event TokenDeactivated(address indexed tokenAddress, string reason);
    event ImplementationUpdated(
        address oldImplementation,
        address newImplementation
    );

    function initialize(
        address admin,
        address _complianceManager,
        address _auditTrail,
        address _feeManager
    ) public initializer {
        __AccessControl_init();
        __Pausable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(TOKEN_CREATOR_ROLE, admin);
        _grantRole(SYSTEM_ROLE, admin);

        complianceManager = IComplianceManager(_complianceManager);
        auditTrail = IAuditTrail(_auditTrail);
        feeManager = IFeeManager(_feeManager);

        // Deploy token implementation
        tokenImplementation = address(new AssetToken());
    }

    function createToken(
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        TokenType tokenType,
        string calldata metadataURI
    ) external payable whenNotPaused returns (address) {
        require(
            hasRole(TOKEN_CREATOR_ROLE, msg.sender) ||
                complianceManager.isKYCVerified(msg.sender),
            "Creator role or KYC required"
        );
        require(_isValidName(name), "Invalid token name");
        require(_isValidSymbol(symbol), "Invalid token symbol");
        require(totalSupply > 0, "Total supply must be positive");
        require(decimals <= 18, "Decimals too high");

        // Calculate and collect creation fee
        uint256 fee = feeManager.calculateFees(
            totalSupply,
            feeManager.TOKEN_CREATION_FEE()
        );
        require(msg.value >= fee, "Insufficient fee");

        if (fee > 0) {
            feeManager.collectFees{value: fee}(
                feeManager.TOKEN_CREATION_FEE(),
                msg.sender
            );
        }

        // Deploy token using minimal proxy
        address tokenAddress = Clones.clone(tokenImplementation);
        AssetToken(payable(tokenAddress)).initialize(
            name,
            symbol,
            totalSupply,
            decimals,
            msg.sender, // Fixed: Changed from 'creator' to 'msg.sender'
            address(complianceManager),
            address(auditTrail)
        );

        // Store token information
        tokens[tokenAddress] = TokenInfo({
            tokenAddress: tokenAddress,
            name: name,
            symbol: symbol,
            totalSupply: totalSupply,
            decimals: decimals,
            tokenType: tokenType,
            creator: msg.sender,
            createdAt: block.timestamp,
            isActive: true,
            isCompliant: true,
            metadataURI: metadataURI
        });

        // Initialize token statistics
        tokenStats[tokenAddress] = TokenStats({
            totalHolders: 1, // Creator is the first holder
            totalTransfers: 0,
            totalVolume: 0,
            lastActivity: block.timestamp
        });

        // Update tracking arrays
        userTokens[msg.sender].push(tokenAddress);
        tokensByType[tokenType].push(tokenAddress);
        allTokens.push(tokenAddress);
        totalTokensCreated += 1;

        // Log creation
        auditTrail.logTransaction(
            keccak256("TOKEN_CREATED"),
            msg.sender,
            totalSupply,
            abi.encodePacked(name, symbol, uint256(tokenType))
        );

        emit TokenCreated(
            tokenAddress,
            msg.sender,
            name,
            symbol,
            totalSupply,
            tokenType
        );

        // Refund excess payment
        if (msg.value > fee) {
            payable(msg.sender).transfer(msg.value - fee);
        }

        return tokenAddress;
    }

    function createTokenBatch(
        string[] calldata names,
        string[] calldata symbols,
        uint256[] calldata totalSupplies,
        uint8[] calldata decimalsArray,
        TokenType[] calldata tokenTypes,
        string[] calldata metadataURIs
    )
        external
        payable
        onlyRole(TOKEN_CREATOR_ROLE)
        whenNotPaused
        returns (address[] memory)
    {
        require(
            names.length == symbols.length &&
                symbols.length == totalSupplies.length &&
                totalSupplies.length == decimalsArray.length &&
                decimalsArray.length == tokenTypes.length &&
                tokenTypes.length == metadataURIs.length,
            "Array length mismatch"
        );
        require(names.length <= 10, "Too many tokens");

        address[] memory createdTokens = new address[](names.length);
        uint256 totalFeeRequired = 0;

        // Calculate total fee required
        for (uint256 i = 0; i < names.length; i++) {
            uint256 fee = feeManager.calculateFees(
                totalSupplies[i],
                feeManager.TOKEN_CREATION_FEE()
            );
            totalFeeRequired += fee;
        }

        require(msg.value >= totalFeeRequired, "Insufficient total fee");

        // Create tokens
        for (uint256 i = 0; i < names.length; i++) {
            createdTokens[i] = _createTokenInternal(
                names[i],
                symbols[i],
                totalSupplies[i],
                decimalsArray[i],
                tokenTypes[i],
                metadataURIs[i]
            );
        }

        // Collect batch fee
        if (totalFeeRequired > 0) {
            feeManager.collectFees{value: totalFeeRequired}(
                feeManager.TOKEN_CREATION_FEE(),
                msg.sender
            );
        }

        // Refund excess
        if (msg.value > totalFeeRequired) {
            payable(msg.sender).transfer(msg.value - totalFeeRequired);
        }

        return createdTokens;
    }

    function updateTokenMetadata(
        address tokenAddress,
        string calldata metadataURI
    ) external {
        require(
            tokens[tokenAddress].creator == msg.sender,
            "Only creator can update"
        );
        require(tokens[tokenAddress].isActive, "Token not active");

        tokens[tokenAddress].metadataURI = metadataURI;

        auditTrail.logTransaction(
            keccak256("TOKEN_METADATA_UPDATED"),
            msg.sender,
            0,
            abi.encodePacked(tokenAddress, metadataURI)
        );

        emit TokenUpdated(tokenAddress, metadataURI);
    }

    function deactivateToken(
        address tokenAddress,
        string calldata reason
    ) external {
        require(
            tokens[tokenAddress].creator == msg.sender ||
                hasRole(DEFAULT_ADMIN_ROLE, msg.sender),
            "Unauthorized"
        );
        require(tokens[tokenAddress].isActive, "Already inactive");

        tokens[tokenAddress].isActive = false;

        auditTrail.logTransaction(
            keccak256("TOKEN_DEACTIVATED"),
            msg.sender,
            0,
            abi.encodePacked(tokenAddress, reason)
        );

        emit TokenDeactivated(tokenAddress, reason);
    }

    function updateTokenStats(
        address tokenAddress,
        uint256 transferAmount,
        bool isNewHolder
    ) external onlyRole(SYSTEM_ROLE) {
        TokenStats storage stats = tokenStats[tokenAddress];

        stats.totalTransfers += 1;
        stats.totalVolume += transferAmount;
        stats.lastActivity = block.timestamp;

        if (isNewHolder) {
            stats.totalHolders += 1;
        }
    }

    function updateImplementation(
        address newImplementation
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newImplementation != address(0), "Invalid implementation");

        address oldImplementation = tokenImplementation;
        tokenImplementation = newImplementation;

        emit ImplementationUpdated(oldImplementation, newImplementation);
    }

    function _createTokenInternal(
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        TokenType tokenType,
        string calldata metadataURI
    ) internal returns (address) {
        require(_isValidName(name), "Invalid token name");
        require(_isValidSymbol(symbol), "Invalid token symbol");
        require(totalSupply > 0, "Total supply must be positive");
        require(decimals <= 18, "Decimals too high");

        // Deploy token using minimal proxy
        address tokenAddress = Clones.clone(tokenImplementation);
        AssetToken(payable(tokenAddress)).initialize(
            name,
            symbol,
            totalSupply,
            decimals,
            msg.sender, // Fixed: Changed from 'creator' to 'msg.sender'
            address(complianceManager),
            address(auditTrail)
        );

        // Store token information
        tokens[tokenAddress] = TokenInfo({
            tokenAddress: tokenAddress,
            name: name,
            symbol: symbol,
            totalSupply: totalSupply,
            decimals: decimals,
            tokenType: tokenType,
            creator: msg.sender,
            createdAt: block.timestamp,
            isActive: true,
            isCompliant: true,
            metadataURI: metadataURI
        });

        // Initialize token statistics
        tokenStats[tokenAddress] = TokenStats({
            totalHolders: 1,
            totalTransfers: 0,
            totalVolume: 0,
            lastActivity: block.timestamp
        });

        // Update tracking arrays
        userTokens[msg.sender].push(tokenAddress);
        tokensByType[tokenType].push(tokenAddress);
        allTokens.push(tokenAddress);
        totalTokensCreated += 1;

        // Log creation
        auditTrail.logTransaction(
            keccak256("TOKEN_CREATED"),
            msg.sender,
            totalSupply,
            abi.encodePacked(name, symbol, uint256(tokenType))
        );

        emit TokenCreated(
            tokenAddress,
            msg.sender,
            name,
            symbol,
            totalSupply,
            tokenType
        );

        return tokenAddress;
    }

    function _isValidName(string calldata name) internal view returns (bool) {
        bytes memory nameBytes = bytes(name);
        return
            nameBytes.length >= minNameLength &&
            nameBytes.length <= maxNameLength;
    }

    function _isValidSymbol(
        string calldata symbol
    ) internal view returns (bool) {
        bytes memory symbolBytes = bytes(symbol);
        return
            symbolBytes.length >= minSymbolLength &&
            symbolBytes.length <= maxSymbolLength;
    }

    // View functions
    function getTokenInfo(
        address tokenAddress
    ) external view returns (TokenInfo memory) {
        return tokens[tokenAddress];
    }

    function getTokenStats(
        address tokenAddress
    ) external view returns (TokenStats memory) {
        return tokenStats[tokenAddress];
    }

    function getUserTokens(
        address user
    ) external view returns (address[] memory) {
        return userTokens[user];
    }

    function getTokensByType(
        TokenType tokenType
    ) external view returns (address[] memory) {
        return tokensByType[tokenType];
    }

    function getAllTokens() external view returns (address[] memory) {
        return allTokens;
    }

    function getActiveTokens() external view returns (address[] memory) {
        uint256 activeCount = 0;

        // Count active tokens
        for (uint256 i = 0; i < allTokens.length; i++) {
            if (tokens[allTokens[i]].isActive) {
                activeCount++;
            }
        }

        // Create array of active tokens
        address[] memory activeTokens = new address[](activeCount);
        uint256 index = 0;

        for (uint256 i = 0; i < allTokens.length; i++) {
            if (tokens[allTokens[i]].isActive) {
                activeTokens[index] = allTokens[i];
                index++;
            }
        }

        return activeTokens;
    }

    function isTokenValid(address tokenAddress) external view returns (bool) {
        return
            tokens[tokenAddress].tokenAddress != address(0) &&
            tokens[tokenAddress].isActive;
    }

    function getTotalStats()
        external
        view
        returns (
            uint256 totalCreated,
            uint256 totalActive,
            uint256 totalHolders,
            uint256 totalVolume
        )
    {
        totalCreated = totalTokensCreated;
        uint256 activeCount = 0;
        uint256 allHolders = 0;
        uint256 allVolume = 0;

        for (uint256 i = 0; i < allTokens.length; i++) {
            if (tokens[allTokens[i]].isActive) {
                activeCount++;
                allHolders += tokenStats[allTokens[i]].totalHolders;
                allVolume += tokenStats[allTokens[i]].totalVolume;
            }
        }

        return (totalCreated, activeCount, allHolders, allVolume);
    }
}
