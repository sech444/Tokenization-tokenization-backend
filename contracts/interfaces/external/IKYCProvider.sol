// ============================================================================

// contracts/interfaces/external/IKYCProvider.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IKYCProvider {
    function verifyIdentity(address user, bytes calldata kycData) external returns (bool);
    function getKYCStatus(address user) external view returns (bool isVerified, uint256 expiryDate);
    function revokeKYC(address user) external;
    function updateKYCData(address user, bytes calldata newData) external;
}
