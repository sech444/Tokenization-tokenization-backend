// contracts/core/ComplianceManager.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "../interfaces/core/IAuditTrail.sol";

/**
 * @title ComplianceManager
 * @dev KYC/AML enforcement and regulatory compliance management
 * @notice Handles user verification, blacklisting, and transfer authorization
 */
contract ComplianceManager is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable
{
    bytes32 public constant COMPLIANCE_OFFICER_ROLE =
        keccak256("COMPLIANCE_OFFICER_ROLE");
    bytes32 public constant SYSTEM_ROLE = keccak256("SYSTEM_ROLE");

    struct KYCData {
        bool isVerified;
        uint256 verificationDate;
        uint256 expiryDate;
        string documentHash;
        address verifiedBy;
        uint8 riskLevel; // 1-5 scale
    }

    struct AMLStatus {
        bool isCleared;
        uint256 riskScore; // 0-100
        uint256 lastCheck;
        string[] flaggedReasons;
        bool requiresManualReview;
    }

    struct TransferLimits {
        uint256 dailyLimit;
        uint256 monthlyLimit;
        uint256 dailySpent;
        uint256 monthlySpent;
        uint256 lastDailyReset;
        uint256 lastMonthlyReset;
    }

    mapping(address => KYCData) public kycData;
    mapping(address => AMLStatus) public amlStatus;
    mapping(address => TransferLimits) public transferLimits;
    mapping(address => bool) public blacklistedAddresses;
    mapping(string => bool) public blacklistedCountries;
    mapping(address => string) public userCountries;

    uint256 public kycExpiryPeriod = 365 days;
    uint256 public amlRecheckPeriod = 30 days;
    uint256 public maxRiskScore = 75;
    uint256 public defaultDailyLimit = 10000 ether;
    uint256 public defaultMonthlyLimit = 100000 ether;

    IAuditTrail public auditTrail;

    event KYCVerified(
        address indexed user,
        address indexed officer,
        uint256 expiryDate
    );
    event KYCExpired(address indexed user);
    event AMLCleared(address indexed user, uint256 riskScore);
    event AMLFlagged(address indexed user, uint256 riskScore, string reason);
    event AddressBlacklisted(address indexed user, string reason);
    event CountryBlacklisted(string country);
    event TransferLimitSet(
        address indexed user,
        uint256 dailyLimit,
        uint256 monthlyLimit
    );
    event TransferBlocked(
        address indexed from,
        address indexed to,
        uint256 amount,
        string reason
    );

    function initialize(address admin, address _auditTrail) public initializer {
        __AccessControl_init();
        __Pausable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(COMPLIANCE_OFFICER_ROLE, admin);
        _grantRole(SYSTEM_ROLE, admin);

        auditTrail = IAuditTrail(_auditTrail);
    }

    function verifyKYC(
        address user,
        string calldata documentHash,
        uint8 riskLevel,
        string calldata country
    ) external onlyRole(COMPLIANCE_OFFICER_ROLE) whenNotPaused {
        require(riskLevel >= 1 && riskLevel <= 5, "Invalid risk level");
        require(!blacklistedCountries[country], "Country is blacklisted");

        uint256 expiryDate = block.timestamp + kycExpiryPeriod;

        kycData[user] = KYCData({
            isVerified: true,
            verificationDate: block.timestamp,
            expiryDate: expiryDate,
            documentHash: documentHash,
            verifiedBy: msg.sender,
            riskLevel: riskLevel
        });

        userCountries[user] = country;

        // Set default transfer limits
        _setDefaultTransferLimits(user);

        auditTrail.logCompliance(user, "KYC_VERIFIED");
        emit KYCVerified(user, msg.sender, expiryDate);
    }

    function clearAML(
        address user,
        uint256 riskScore,
        string[] calldata flaggedReasons
    ) external onlyRole(COMPLIANCE_OFFICER_ROLE) whenNotPaused {
        require(riskScore <= 100, "Invalid risk score");

        bool isCleared = riskScore <= maxRiskScore;

        amlStatus[user] = AMLStatus({
            isCleared: isCleared,
            riskScore: riskScore,
            lastCheck: block.timestamp,
            flaggedReasons: flaggedReasons,
            requiresManualReview: riskScore > maxRiskScore
        });

        auditTrail.logCompliance(
            user,
            isCleared ? "AML_CLEARED" : "AML_FLAGGED"
        );

        if (isCleared) {
            emit AMLCleared(user, riskScore);
        } else {
            emit AMLFlagged(
                user,
                riskScore,
                flaggedReasons.length > 0
                    ? flaggedReasons[0]
                    : "High risk score"
            );
        }
    }

    function blacklistAddress(
        address user,
        string calldata reason
    ) external onlyRole(COMPLIANCE_OFFICER_ROLE) {
        blacklistedAddresses[user] = true;
        auditTrail.logCompliance(user, "BLACKLISTED");
        emit AddressBlacklisted(user, reason);
    }

    function unblacklistAddress(
        address user
    ) external onlyRole(COMPLIANCE_OFFICER_ROLE) {
        blacklistedAddresses[user] = false;
        auditTrail.logCompliance(user, "UNBLACKLISTED");
    }

    function blacklistCountry(
        string calldata country
    ) external onlyRole(COMPLIANCE_OFFICER_ROLE) {
        blacklistedCountries[country] = true;
        emit CountryBlacklisted(country);
    }

    function setTransferLimits(
        address user,
        uint256 dailyLimit,
        uint256 monthlyLimit
    ) external onlyRole(COMPLIANCE_OFFICER_ROLE) {
        transferLimits[user].dailyLimit = dailyLimit;
        transferLimits[user].monthlyLimit = monthlyLimit;

        emit TransferLimitSet(user, dailyLimit, monthlyLimit);
    }

    /// @notice Allows admin to update the KYC expiry period (default 365 days)
    function setKYCExpiryPeriod(uint256 newPeriod) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newPeriod > 0, "Invalid expiry period");
        kycExpiryPeriod = newPeriod;
    }

    function isKYCVerified(address user) external view returns (bool) {
        KYCData memory kyc = kycData[user];
        return kyc.isVerified && block.timestamp <= kyc.expiryDate;
    }

    function isAMLCleared(address user) external view returns (bool) {
        AMLStatus memory aml = amlStatus[user];
        return
            aml.isCleared &&
            (block.timestamp - aml.lastCheck) <= amlRecheckPeriod;
    }

    function canTransfer(
        address from,
        address to,
        uint256 amount
    ) external view returns (bool) {
        // Check blacklist
        if (blacklistedAddresses[from] || blacklistedAddresses[to]) {
            return false;
        }

        // Check country restrictions
        if (
            blacklistedCountries[userCountries[from]] ||
            blacklistedCountries[userCountries[to]]
        ) {
            return false;
        }

        // Check KYC
        if (!this.isKYCVerified(from) || !this.isKYCVerified(to)) {
            return false;
        }

        // Check AML
        if (!this.isAMLCleared(from) || !this.isAMLCleared(to)) {
            return false;
        }

        // Check transfer limits
        return _checkTransferLimits(from, amount);
    }

    function authorizeTransfer(
        address from,
        address to,
        uint256 amount
    ) external onlyRole(SYSTEM_ROLE) returns (bool) {
        if (!this.canTransfer(from, to, amount)) {
            emit TransferBlocked(from, to, amount, "Compliance check failed");
            return false;
        }

        // Update transfer limits
        _updateTransferLimits(from, amount);

        // Log the authorized transfer
        auditTrail.logTransaction(
            keccak256("TRANSFER_AUTHORIZED"),
            from,
            amount,
            abi.encodePacked(to)
        );

        return true;
    }

    function _setDefaultTransferLimits(address user) internal {
        transferLimits[user] = TransferLimits({
            dailyLimit: defaultDailyLimit,
            monthlyLimit: defaultMonthlyLimit,
            dailySpent: 0,
            monthlySpent: 0,
            lastDailyReset: block.timestamp,
            lastMonthlyReset: block.timestamp
        });
    }

    function _checkTransferLimits(
        address user,
        uint256 amount
    ) internal view returns (bool) {
        TransferLimits memory limits = transferLimits[user];

        // Reset counters if time periods have passed
        uint256 currentDailySpent = limits.dailySpent;
        uint256 currentMonthlySpent = limits.monthlySpent;

        if (block.timestamp >= limits.lastDailyReset + 1 days) {
            currentDailySpent = 0;
        }

        if (block.timestamp >= limits.lastMonthlyReset + 30 days) {
            currentMonthlySpent = 0;
        }

        return
            (currentDailySpent + amount <= limits.dailyLimit) &&
            (currentMonthlySpent + amount <= limits.monthlyLimit);
    }

    function _updateTransferLimits(address user, uint256 amount) internal {
        TransferLimits storage limits = transferLimits[user];

        // Reset daily counter if needed
        if (block.timestamp >= limits.lastDailyReset + 1 days) {
            limits.dailySpent = 0;
            limits.lastDailyReset = block.timestamp;
        }

        // Reset monthly counter if needed
        if (block.timestamp >= limits.lastMonthlyReset + 30 days) {
            limits.monthlySpent = 0;
            limits.lastMonthlyReset = block.timestamp;
        }

        limits.dailySpent += amount;
        limits.monthlySpent += amount;
    }

    function getKYCData(address user) external view returns (KYCData memory) {
        return kycData[user];
    }

    function getAMLStatus(
        address user
    ) external view returns (AMLStatus memory) {
        return amlStatus[user];
    }

    function getTransferLimits(
        address user
    ) external view returns (TransferLimits memory) {
        return transferLimits[user];
    }

    function getRemainingDailyLimit(
        address user
    ) external view returns (uint256) {
        TransferLimits memory limits = transferLimits[user];
        uint256 currentSpent = block.timestamp >= limits.lastDailyReset + 1 days
            ? 0
            : limits.dailySpent;
        return
            limits.dailyLimit > currentSpent
                ? limits.dailyLimit - currentSpent
                : 0;
    }

    function getRemainingMonthlyLimit(
        address user
    ) external view returns (uint256) {
        TransferLimits memory limits = transferLimits[user];
        uint256 currentSpent = block.timestamp >=
            limits.lastMonthlyReset + 30 days
            ? 0
            : limits.monthlySpent;
        return
            limits.monthlyLimit > currentSpent
                ? limits.monthlyLimit - currentSpent
                : 0;
    }
}
