// contracts/interfaces/external/IChainlinkAssetVerifier.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IChainlinkAssetVerifier
 * @dev Interface for the ChainlinkAssetVerifier contract.
 */
interface IChainlinkAssetVerifier {
    /**
     * @dev Initiates an asset verification request through Chainlink Functions.
     */
    function requestAssetVerification(
        uint256 assetId,
        string calldata propertyAddress,
        string calldata propertyType,
        uint256 squareFootage,
        string calldata encryptedApiKeys
    ) external returns (bytes32 requestId);
}