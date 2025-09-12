// ============================================================================

// contracts/interfaces/core/IFeeManager.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IFeeManager {
    function calculateFees(uint256 amount, bytes32 feeType) external view returns (uint256);
    function collectFees(bytes32 feeType, address payer) external payable returns (uint256);
    function distributeFees(bytes32 feeType) external;
    
    function TOKEN_CREATION_FEE() external view returns (bytes32);
    function TOKENIZATION_FEE() external view returns (bytes32);
    function TRADING_FEE() external view returns (bytes32);
    function WITHDRAWAL_FEE() external view returns (bytes32);
    function LISTING_FEE() external view returns (bytes32);
}

