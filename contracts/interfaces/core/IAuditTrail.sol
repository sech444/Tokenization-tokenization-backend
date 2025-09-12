// contracts/interfaces/core/IAuditTrail.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IAuditTrail {
    function logTransaction(bytes32 txType, address user, uint256 amount, bytes calldata data) external;
    function logCompliance(address user, string calldata action) external;
    function getAuditLog(bytes32 hash) external view returns (
        uint256 timestamp,
        bytes32 txType,
        address user,
        uint256 amount,
        bytes memory data,
        bytes32 hashlogHash
    );
    function getTotalLogs() external view returns (uint256);
}

