// contracts/interfaces/core/IAuditTrail.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IAuditTrail
 * @dev The complete interface for the AuditTrail contract.
 */
interface IAuditTrail {
    /**
     * @dev Initializes the contract, setting the admin.
     */
    function initialize(address admin) external;

    /**
     * @dev Logs a generic transaction.
     */
    function logTransaction(
        bytes32 txType,
        address user,
        uint256 amount,
        bytes calldata data
    ) external;

    /**
     * @dev Logs a compliance-related event.
     */
    function logCompliance(address user, string calldata eventType) external;

    /**
     * @dev Grants a role to an account.
     */
    function grantRole(bytes32 role, address account) external;

    /**
     * @dev Returns the bytes32 value for the SYSTEM_ROLE.
     */
    function SYSTEM_ROLE() external view returns (bytes32);
}