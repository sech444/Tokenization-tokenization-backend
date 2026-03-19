// contracts/interfaces/core/IAssetTokenizer.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

// Import ITokenRegistry to use its TokenType enum
import "./ITokenRegistry.sol";

/**
 * @title IAssetTokenizer
 * @dev The complete interface for the HybridAssetTokenizer contract.
 */
interface IAssetTokenizer {
    // --- Enums ---
    enum AssetType { REAL_ESTATE, BUSINESS, COMMODITY, INTELLECTUAL_PROPERTY, ARTWORK, VEHICLE, OTHER }
    enum AssetStatus { PENDING, UNDER_REVIEW, VERIFIED, TOKENIZED, SUSPENDED, REJECTED }

    // --- Functions ---
    function initialize(
        address admin,
        address tokenFactory,
        address tokenRegistry,
        address complianceManager,
        address auditTrail,
        address feeManager
    ) external;

    // Add this function to the interface
    function addValuation(
        uint256 assetId,
        uint256 value,
        string calldata reportHash,
        string calldata methodology
    ) external;

    function registerAsset(
        string calldata name,
        string calldata description,
        AssetType assetType,
        string[] calldata documentHashes,
        string calldata location
    ) external returns (uint256);

    function getAsset(uint256 assetId) external view returns (
        uint256 assetID,
        string memory name,
        string memory description,
        AssetType assetType,
        AssetStatus status,
        uint256 totalValue,
        uint256 totalTokens,
        address tokenAddress,
        address owner,
        string[] memory documentHashes,
        string memory location,
        uint256 createdAt,
        uint256 tokenizedAt,
        string memory rejectionReason
    );

    function canTokenizeAsset(uint256 assetId) external view returns (bool, string memory);

    // ===== THIS IS THE FIX: The missing function declaration =====
    function getLatestValuation(uint256 assetId) external view returns (uint256);

    function createHybridAsset(
        uint256 assetId,
        string memory name,
        string memory symbol,
        uint256 supply,
        uint8 decimals,
        ITokenRegistry.TokenType tokenType,
        string memory metadataURI,
        string memory deedURI
    ) external returns (address tokenAddress);

    function setVerificationGateway(address gateway) external;

    function setMinAssetValue(uint256 newValue) external;
}