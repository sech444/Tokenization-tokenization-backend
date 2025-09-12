// contracts/core/AuditTrail.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

/**
 * @title AuditTrail
 * @dev Immutable compliance and transaction logging system
 * @notice Provides cryptographic integrity for all platform operations
 */
contract AuditTrail is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable
{
    bytes32 public constant AUDITOR_ROLE = keccak256("AUDITOR_ROLE");
    bytes32 public constant SYSTEM_ROLE = keccak256("SYSTEM_ROLE");

    struct AuditLog {
        uint256 timestamp;
        bytes32 txType;
        address user;
        uint256 amount;
        bytes data;
        bytes32 hash;
    }

    struct ComplianceLog {
        uint256 timestamp;
        address user;
        string action;
        bool success;
    }

    mapping(bytes32 => AuditLog) public auditLogs;
    mapping(address => ComplianceLog[]) public userComplianceLogs;
    mapping(bytes32 => bool) public logExists;

    bytes32[] public logHashes;
    uint256 public totalLogs;

    event TransactionLogged(
        bytes32 indexed hash,
        bytes32 indexed txType,
        address indexed user,
        uint256 amount
    );
    event ComplianceLogged(address indexed user, string action, bool success);

    function initialize(address admin) public initializer {
        __AccessControl_init();
        __Pausable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(AUDITOR_ROLE, admin);
        _grantRole(SYSTEM_ROLE, admin);
    }

    function logTransaction(
        bytes32 txType,
        address user,
        uint256 amount,
        bytes calldata data
    ) external onlyRole(SYSTEM_ROLE) whenNotPaused {
        bytes32 hash = keccak256(
            abi.encodePacked(
                block.timestamp,
                txType,
                user,
                amount,
                data,
                totalLogs
            )
        );

        require(!logExists[hash], "Duplicate log hash");

        auditLogs[hash] = AuditLog({
            timestamp: block.timestamp,
            txType: txType,
            user: user,
            amount: amount,
            data: data,
            hash: hash
        });

        logExists[hash] = true;
        logHashes.push(hash);
        totalLogs = totalLogs + 1;

        emit TransactionLogged(hash, txType, user, amount);
    }

    function logCompliance(
        address user,
        string calldata action
    ) external onlyRole(SYSTEM_ROLE) whenNotPaused {
        userComplianceLogs[user].push(
            ComplianceLog({
                timestamp: block.timestamp,
                user: user,
                action: action,
                success: true
            })
        );

        emit ComplianceLogged(user, action, true);
    }

    function getAuditLog(bytes32 hash) external view returns (AuditLog memory) {
        require(logExists[hash], "Log does not exist");
        return auditLogs[hash];
    }

    function getUserComplianceLogs(
        address user
    ) external view returns (ComplianceLog[] memory) {
        return userComplianceLogs[user];
    }

    function verifyLogIntegrity(bytes32 hash) external view returns (bool) {
        if (!logExists[hash]) return false;

        AuditLog memory log = auditLogs[hash];
        bytes32 computedHash = keccak256(
            abi.encodePacked(
                log.timestamp,
                log.txType,
                log.user,
                log.amount,
                log.data,
                _findLogIndex(hash)
            )
        );

        return computedHash == hash;
    }

    function _findLogIndex(bytes32 hash) internal view returns (uint256) {
        for (uint256 i = 0; i < logHashes.length; i++) {
            if (logHashes[i] == hash) return i;
        }
        revert("Log not found");
    }

    function getLogCount() external view returns (uint256) {
        return totalLogs;
    }

    function getRecentLogs(
        uint256 count
    ) external view returns (bytes32[] memory) {
        require(count <= totalLogs, "Count exceeds total logs");

        bytes32[] memory recentLogs = new bytes32[](count);
        uint256 startIndex = totalLogs > count ? totalLogs - count : 0;

        for (uint256 i = 0; i < count; i++) {
            recentLogs[i] = logHashes[startIndex + i];
        }

        return recentLogs;
    }
}
