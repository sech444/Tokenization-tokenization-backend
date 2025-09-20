// contracts/verification/EnhancedVerificationGateway.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";

contract EnhancedVerificationGateway {
    
    struct PropertyData {
        string propertyAddress;
        uint256 squareFootage;
        uint256 yearBuilt;
        string propertyType;
        string[] amenities;
        string taxId;
    }
    
    struct VerificationStatus {
        bool titleVerified;
        bool taxStatusVerified;
        bool physicalInspectionDone;
        bool valuationComplete;
        uint256 chainlinkValuation;
        uint256 manualValuation;
        uint256 finalValuation;
    }
    
    IChainlinkAssetVerifier public chainlinkVerifier;
    AggregatorV3Interface public priceOracle; // For USD conversion
    
    mapping(uint256 => PropertyData) public propertyData;
    mapping(uint256 => VerificationStatus) public verificationStatus;
    mapping(uint256 => bytes32) public pendingChainlinkRequests;
    
    event ChainlinkVerificationInitiated(
        uint256 indexed assetId,
        bytes32 requestId
    );
    
    function initiateFullVerification(
        uint256 assetId,
        PropertyData calldata data,
        string calldata encryptedApiKeys
    ) external {
        // Store property data
        propertyData[assetId] = data;
        
        // 1. Initiate Chainlink verification
        bytes32 requestId = chainlinkVerifier.requestAssetVerification(
            assetId,
            data.propertyAddress,
            data.propertyType,
            data.squareFootage,
            encryptedApiKeys
        );
        
        pendingChainlinkRequests[assetId] = requestId;
        emit ChainlinkVerificationInitiated(assetId, requestId);
        
        // 2. Check on-chain data (if property was previously tokenized)
        _checkOnChainHistory(assetId);
        
        // 3. Initiate other verification processes
        _initiateTitleVerification(assetId, data.taxId);
        _initiateTaxVerification(assetId, data.taxId);
    }
    
    function _checkOnChainHistory(uint256 assetId) internal view {
        // Check if property has been tokenized before
        // Look for previous transactions, ownership changes, etc.
    }
    
    function _initiateTitleVerification(
        uint256 assetId,
        string memory taxId
    ) internal {
        // Could integrate with another oracle for title verification
    }
    
    function _initiateTaxVerification(
        uint256 assetId,
        string memory taxId
    ) internal {
        // Could integrate with tax authority APIs
    }
}