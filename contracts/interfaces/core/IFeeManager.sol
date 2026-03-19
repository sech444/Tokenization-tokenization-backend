// contracts/interfaces/core/IFeeManager.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IFeeManager
 * @dev The complete interface for the FeeManager contract.
 */
interface IFeeManager {
    /**
     * @dev Initializes the contract.
     */
    function initialize(address admin, address treasury) external;

    /**
     * @dev Calculates the fee for a given amount and fee type.
     */
    function calculateFees(uint256 amount, bytes32 feeType) external view returns (uint256);

    /**
     * @dev Collects a fee from a payer.
     */
    function collectFees(bytes32 feeType, address payer) external payable returns (uint256);

    /**
     * @dev Distributes collected fees.
     */
    function distributeFees(bytes32 feeType) external;

    /**
     * @dev Configures a fee structure for a given fee type.
     */
    function setFeeStructure(
        bytes32 feeType,
        uint256 percentage,
        uint256 fixedMin,
        uint256 fixedMax,
        uint256 cap,
        bool isActive
    ) external;

    // --- Fee Type Constants ---
    function TOKEN_CREATION_FEE() external view returns (bytes32);
    function TOKENIZATION_FEE() external view returns (bytes32);
    function TRADING_FEE() external view returns (bytes32);
    function WITHDRAWAL_FEE() external view returns (bytes32);
    function LISTING_FEE() external view returns (bytes32);
}