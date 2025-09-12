 // ============================================================================

// contracts/interfaces/core/IComplianceManager.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IComplianceManager {
    function isKYCVerified(address user) external view returns (bool);
    function isAMLCleared(address user) external view returns (bool);
    function canTransfer(address from, address to, uint256 amount) external view returns (bool);
    function authorizeTransfer(address from, address to, uint256 amount) external returns (bool);
    function getRemainingDailyLimit(address user) external view returns (uint256);
    function getRemainingMonthlyLimit(address user) external view returns (uint256);
}

