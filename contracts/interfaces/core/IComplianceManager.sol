// contracts/interfaces/core/IComplianceManager.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IComplianceManager
 * @dev The complete interface for the ComplianceManager contract.
 */
interface IComplianceManager {
    /**
     * @dev Initializes the contract.
     */
    function initialize(address admin, address auditTrail) external;

    /**
     * @dev Checks if a user's KYC is currently valid.
     */
    function isKYCVerified(address user) external view returns (bool);

    /**
     * @dev Checks if a transfer between two parties is compliant.
     */
    function canTransfer(
        address from,
        address to,
        uint256 amount
    ) external view returns (bool);

    /**
     * @dev Called by a system contract to authorize and log a compliant transfer.
     */
    function authorizeTransfer(
        address from,
        address to,
        uint256 amount
    ) external returns (bool);

    /**
     * @dev Sets the duration for which a KYC verification is valid.
     */
    function setKYCExpiryPeriod(uint256 newPeriod) external;
}