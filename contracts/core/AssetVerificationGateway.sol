// contracts/core/AssetVerificationGateway.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";

import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IAssetTokenizer.sol";
import "../interfaces/external/IKYCProvider.sol";
import "../interfaces/external/IPriceOracle.sol";
import "../interfaces/core/ITokenRegistry.sol";

contract AssetVerificationGateway is AccessControl {
    bytes32 public constant VERIFIER_ROLE = keccak256("VERIFIER_ROLE");

    IKYCProvider public kycProvider;
    IComplianceManager public complianceManager;
    IAssetTokenizer public hybridTokenizer;
    IPriceOracle public priceOracle;

    // ===== THIS IS THE FIX: Add the missing event declarations back =====
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

        if (kyc_ != address(0)) kycProvider = IKYCProvider(kyc_);
        if (compliance_ != address(0)) complianceManager = IComplianceManager(compliance_);
        if (tokenizer_ != address(0)) hybridTokenizer = IAssetTokenizer(tokenizer_);
        if (oracle_ != address(0)) priceOracle = IPriceOracle(oracle_);
    }

    function verifyAndCreateHybridAsset(
        uint256 assetId,
        address user,
        string memory name,
        string memory symbol,
        uint256 supply,
        uint8 decimals,
        ITokenRegistry.TokenType tokenType,
        string memory metadataURI,
        string memory deedURI
    ) external onlyRole(VERIFIER_ROLE) returns (address tokenAddress) {
        bool kycPassed;
        if (address(kycProvider) == address(0)) {
            kycPassed = true;
        } else {
            (bool isVerified, ) = kycProvider.getKYCStatus(user);
            kycPassed = isVerified;
        }
        emit DebugKYC(user, kycPassed);
        require(kycPassed, "KYC check failed");

        bool compliancePassed;
        if (address(complianceManager) == address(0)) {
            compliancePassed = true;
        } else {
            compliancePassed = complianceManager.isKYCVerified(user);
        }
        emit DebugCompliance(assetId, user, compliancePassed);
        require(compliancePassed, "Compliance check failed");

        uint256 valuation = hybridTokenizer.getLatestValuation(assetId);
        bool valuationPassed = (valuation > 0);
        emit DebugValuation(assetId, valuation, valuationPassed);
        require(valuationPassed, "Valuation check failed");

        if (address(priceOracle) != address(0)) {
            (, int256 priceInt, , , ) = priceOracle.latestRoundData();
            uint256 price = uint256(priceInt);
            bool oraclePassed = (price > 0);
            emit DebugOracle("USD", price, oraclePassed);
            require(oraclePassed, "Oracle price check failed");
        }

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