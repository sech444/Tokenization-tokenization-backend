// ============================================================================

// contracts/interfaces/core/IAssetTokenizer.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IAssetTokenizer {
    enum AssetType { REAL_ESTATE, BUSINESS, COMMODITY, INTELLECTUAL_PROPERTY, ARTWORK, VEHICLE, OTHER }
    enum AssetStatus { PENDING, UNDER_REVIEW, VERIFIED, TOKENIZED, SUSPENDED, REJECTED }
    
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
}
