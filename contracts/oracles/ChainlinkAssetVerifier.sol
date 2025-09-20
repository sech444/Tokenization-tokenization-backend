// contracts/oracles/ChainlinkAssetVerifier.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {FunctionsClient} from "@chainlink/contracts/src/v0.8/functions/dev/v1_0_0/FunctionsClient.sol";
import {ConfirmedOwner} from "@chainlink/contracts/src/v0.8/shared/access/ConfirmedOwner.sol";
import {FunctionsRequest} from "@chainlink/contracts/src/v0.8/functions/dev/v1_0_0/libraries/FunctionsRequest.sol";

interface IAssetTokenizer {
    function addValuation(
        uint256 assetId,
        uint256 value,
        string calldata reportHash,
        string calldata methodology
    ) external;
}

contract ChainlinkAssetVerifier is FunctionsClient, ConfirmedOwner {
    using FunctionsRequest for FunctionsRequest.Request;

    // Chainlink Functions configuration
    bytes32 public donId;
    uint64 public subscriptionId;
    uint32 public gasLimit = 300000;
    
    IAssetTokenizer public assetTokenizer;
    
    struct VerificationRequest {
        uint256 assetId;
        string propertyAddress;
        string propertyType;
        uint256 squareFootage;
        address requester;
        bool fulfilled;
    }
    
    mapping(bytes32 => VerificationRequest) public requests;
    
    // JavaScript source code for Chainlink Functions
    string public source = 
        "const propertyAddress = args[0];"
        "const propertyType = args[1];"
        "const squareFootage = args[2];"
        ""
        "// Call multiple real estate APIs"
        "const zillowResponse = await Functions.makeHttpRequest({"
        "  url: `https://api.zillow.com/valuation/${propertyAddress}`,"
        "  headers: { 'X-API-KEY': secrets.zillowKey }"
        "});"
        ""
        "const realtorResponse = await Functions.makeHttpRequest({"
        "  url: `https://api.realtor.com/properties/${propertyAddress}`,"
        "  headers: { 'Authorization': `Bearer ${secrets.realtorKey}` }"
        "});"
        ""
        "// Aggregate valuations"
        "const zillowValue = zillowResponse.data.estimate;"
        "const realtorValue = realtorResponse.data.value;"
        "const averageValue = Math.floor((zillowValue + realtorValue) / 2);"
        ""
        "// Verify property details"
        "const verifiedSqFt = zillowResponse.data.squareFootage;"
        "const propertyExists = zillowResponse.data.exists && realtorResponse.data.exists;"
        ""
        "// Return encoded result"
        "return Functions.encodeUint256(averageValue);";

    event VerificationRequested(
        bytes32 indexed requestId,
        uint256 indexed assetId,
        string propertyAddress
    );
    
    event VerificationFulfilled(
        bytes32 indexed requestId,
        uint256 indexed assetId,
        uint256 valuation
    );
    
    constructor(
        address router,
        bytes32 _donId,
        uint64 _subscriptionId,
        address _assetTokenizer
    ) FunctionsClient(router) ConfirmedOwner(msg.sender) {
        donId = _donId;
        subscriptionId = _subscriptionId;
        assetTokenizer = IAssetTokenizer(_assetTokenizer);
    }
    
    function requestAssetVerification(
        uint256 assetId,
        string calldata propertyAddress,
        string calldata propertyType,
        uint256 squareFootage,
        string calldata encryptedSecrets
    ) external returns (bytes32 requestId) {
        // Build Chainlink Functions request
        FunctionsRequest.Request memory req;
        req.initializeRequestForInlineJavaScript(source);
        
        // Set arguments for the JavaScript function
        string[] memory args = new string[](3);
        args[0] = propertyAddress;
        args[1] = propertyType;
        args[2] = Strings.toString(squareFootage);
        req.setArgs(args);
        
        // Set encrypted secrets (API keys)
        req.addSecretsReference(encryptedSecrets);
        
        // Send request
        requestId = _sendRequest(
            req.encodeCBOR(),
            subscriptionId,
            gasLimit,
            donId
        );
        
        // Store request details
        requests[requestId] = VerificationRequest({
            assetId: assetId,
            propertyAddress: propertyAddress,
            propertyType: propertyType,
            squareFootage: squareFootage,
            requester: msg.sender,
            fulfilled: false
        });
        
        emit VerificationRequested(requestId, assetId, propertyAddress);
    }
    
    function fulfillRequest(
        bytes32 requestId,
        bytes memory response,
        bytes memory err
    ) internal override {
        VerificationRequest storage request = requests[requestId];
        require(!request.fulfilled, "Request already fulfilled");
        
        if (err.length > 0) {
            emit RequestError(requestId, err);
            return;
        }
        
        // Decode the valuation from response
        uint256 valuation = abi.decode(response, (uint256));
        
        // Update request status
        request.fulfilled = true;
        
        // Send valuation to AssetTokenizer
        assetTokenizer.addValuation(
            request.assetId,
            valuation,
            string(abi.encodePacked("chainlink-", requestId)),
            "Chainlink Automated Valuation"
        );
        
        emit VerificationFulfilled(requestId, request.assetId, valuation);
    }
}