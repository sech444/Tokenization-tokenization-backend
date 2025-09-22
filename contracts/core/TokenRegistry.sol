// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

// ===== FIX: Import the interface which now contains the TokenType enum =====
import "../interfaces/core/ITokenRegistry.sol";

// ===== FIX: Explicitly implement the ITokenRegistry interface =====
contract TokenRegistry is Initializable, AccessControlUpgradeable, ITokenRegistry {
    bytes32 public constant FACTORY_ROLE = keccak256("FACTORY_ROLE");
    bytes32 public constant SYSTEM_ROLE = keccak256("SYSTEM_ROLE");

    // ===== FIX: REMOVE the inline enum definition =====
    // enum TokenType { ASSET, UTILITY, SECURITY, GOVERNANCE }

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
        string metadataURI;
    }

    struct TokenStats {
        uint256 totalHolders;
        uint256 totalTransfers;
        uint256 totalVolume;
        uint256 lastActivity;
    }

    mapping(address => TokenInfo) public tokens;
    mapping(address => TokenStats) public stats;
    mapping(address => address[]) public userTokens;
    address[] public allTokens;

    event TokenRegistered(address indexed tokenAddress, address indexed creator, string name, string symbol, uint256 totalSupply);
    event TokenMetadataUpdated(address indexed tokenAddress, string metadataURI);
    event TokenDeactivated(address indexed tokenAddress, string reason);
    event TokenStatsUpdated(address indexed tokenAddress, uint256 totalTransfers, uint256 totalVolume);

    /// @notice Upgradeable initializer instead of constructor
    // ===== FIX: Add 'override' keyword =====
    function initialize(address admin) public override initializer {
        __AccessControl_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(SYSTEM_ROLE, admin);
    }

    // ===== FIX: Add 'override' keyword =====
    function registerToken(
        address tokenAddress,
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        TokenType tokenType,
        address creator,
        string calldata metadataURI
    ) external override onlyRole(FACTORY_ROLE) {
        require(tokenAddress != address(0), "invalid token");
        require(tokens[tokenAddress].tokenAddress == address(0), "already registered");

        tokens[tokenAddress] = TokenInfo({
            tokenAddress: tokenAddress,
            name: name,
            symbol: symbol,
            totalSupply: totalSupply,
            decimals: decimals,
            tokenType: tokenType,
            creator: creator,
            createdAt: block.timestamp,
            isActive: true,
            metadataURI: metadataURI
        });

        stats[tokenAddress] = TokenStats({
            totalHolders: 1,
            totalTransfers: 0,
            totalVolume: 0,
            lastActivity: block.timestamp
        });

        userTokens[creator].push(tokenAddress);
        allTokens.push(tokenAddress);

        emit TokenRegistered(tokenAddress, creator, name, symbol, totalSupply);
    }

    function updateTokenMetadata(address tokenAddress, string calldata metadataURI) external {
        TokenInfo storage info = tokens[tokenAddress];
        require(info.tokenAddress != address(0), "unknown token");
        require(info.creator == msg.sender || hasRole(DEFAULT_ADMIN_ROLE, msg.sender), "unauthorized");
        info.metadataURI = metadataURI;
        emit TokenMetadataUpdated(tokenAddress, metadataURI);
    }

    function deactivateToken(address tokenAddress, string calldata reason) external {
        TokenInfo storage info = tokens[tokenAddress];
        require(info.tokenAddress != address(0), "unknown token");
        require(info.creator == msg.sender || hasRole(DEFAULT_ADMIN_ROLE, msg.sender), "unauthorized");
        info.isActive = false;
        emit TokenDeactivated(tokenAddress, reason);
    }

    function updateStats(address tokenAddress, uint256 transferAmount, bool isNewHolder) external onlyRole(SYSTEM_ROLE) {
        TokenStats storage s = stats[tokenAddress];
        require(s.lastActivity != 0, "unknown token stats");

        s.totalTransfers += 1;
        s.totalVolume += transferAmount;
        s.lastActivity = block.timestamp;
        if (isNewHolder) s.totalHolders += 1;

        emit TokenStatsUpdated(tokenAddress, s.totalTransfers, s.totalVolume);
    }

    // Read helpers
    function getTokenInfo(address tokenAddress) external view returns (TokenInfo memory) {
        return tokens[tokenAddress];
    }

    function getTokenStats(address tokenAddress) external view returns (TokenStats memory) {
        return stats[tokenAddress];
    }

    function getUserTokens(address user) external view returns (address[] memory) {
        return userTokens[user];
    }

    function getAllTokens() external view returns (address[] memory) {
        return allTokens;
    }

    // Storage gap for upgrade safety
    uint256[50] private __gap;
}