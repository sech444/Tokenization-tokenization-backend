// ============================================================================

// contracts/interfaces/platform/ITokenizationPlatform.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface ITokenizationPlatform {
    function getAuditTrail() external view returns (address);
    function getComplianceManager() external view returns (address);
    function getFeeManager() external view returns (address);
    function getTokenFactory() external view returns (address);
    function getAssetTokenizer() external view returns (address);
    function getMarketplaceCore() external view returns (address);
    function getRewardSystem() external view returns (address);
    function getAdminGovernance() external view returns (address);
    
    function isContractActive(address contractAddress) external view returns (bool);
    function getPlatformVersion() external view returns (string memory);
}
