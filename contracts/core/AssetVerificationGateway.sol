// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";

interface IKYCProvider {
    function isKYCApproved(address user) external view returns (bool);
}

interface IComplianceManager {
    function checkCompliance(uint256 assetId, address user) external view returns (bool);
}

interface IHybridAssetTokenizer {
    function getLatestValuation(uint256 assetId) external view returns (uint256);
    function createHybridAsset(
        uint256 assetId,
        string memory name,
        string memory symbol,
        uint256 supply,
        uint8 decimals,
        uint8 tokenType,
        string memory metadataURI,
        string memory deedURI
    ) external returns (address);
}

interface IPriceOracle {
    function getPrice(string calldata symbol) external view returns (uint256);
}

contract AssetVerificationGateway is AccessControl {
    bytes32 public constant VERIFIER_ROLE = keccak256("VERIFIER_ROLE");

    IKYCProvider public kycProvider;
    IComplianceManager public complianceManager;
    IHybridAssetTokenizer public hybridTokenizer; // Now handles valuations internally
    IPriceOracle public priceOracle;

    // Events
    event DebugKYC(address indexed user, bool passed);
    event DebugCompliance(uint256 indexed assetId, address indexed user, bool passed);
    event DebugValuation(uint256 indexed assetId, uint256 valuation, bool passed);
    event DebugOracle(string symbol, uint256 price, bool passed);
    event HybridAssetCreation(uint256 indexed assetId, address tokenAddress);

    constructor(
        address admin,
        address kyc_,
        address compliance_,
        address tokenizer_,
        address oracle_
    ) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(VERIFIER_ROLE, admin);

        kycProvider = IKYCProvider(kyc_);
        complianceManager = IComplianceManager(compliance_);
        hybridTokenizer = IHybridAssetTokenizer(tokenizer_);
        priceOracle = IPriceOracle(oracle_);
    }

    function verifyAndCreateHybridAsset(
        uint256 assetId,
        address user,
        string memory name,
        string memory symbol,
        uint256 supply,
        uint8 decimals,
        uint8 tokenType,
        string memory metadataURI,
        string memory deedURI
    ) external onlyRole(VERIFIER_ROLE) returns (address tokenAddress) {
        // 1. Check KYC
        bool kycPassed = kycProvider.isKYCApproved(user);
        emit DebugKYC(user, kycPassed);
        require(kycPassed, "KYC check failed");

        // 2. Check Compliance
        bool compliancePassed = complianceManager.checkCompliance(assetId, user);
        emit DebugCompliance(assetId, user, compliancePassed);
        require(compliancePassed, "Compliance check failed");

        // 3. Check Valuation (now from HybridAssetTokenizer)
        uint256 valuation = hybridTokenizer.getLatestValuation(assetId);
        bool valuationPassed = (valuation > 0);
        emit DebugValuation(assetId, valuation, valuationPassed);
        require(valuationPassed, "Valuation check failed");

        // 4. Oracle Sanity Check
        uint256 price = priceOracle.getPrice("USD");
        bool oraclePassed = (price > 0);
        emit DebugOracle("USD", price, oraclePassed);
        require(oraclePassed, "Oracle price check failed");

        // 5. Create hybrid asset
        tokenAddress = hybridTokenizer.createHybridAsset(
            assetId,
            name,
            symbol,
            supply,
            decimals,
            tokenType,
            metadataURI,
            deedURI
        );

        emit HybridAssetCreation(assetId, tokenAddress);
    }
}