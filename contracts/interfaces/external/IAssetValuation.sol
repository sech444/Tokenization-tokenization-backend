// contracts/interfaces/external/IAssetValuation.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IAssetValuation {
    function requestValuation(uint256 assetId, string calldata assetType) external payable;
    function submitValuation(uint256 assetId, uint256 value, string calldata reportHash) external;
    function getValuation(uint256 assetId) external view returns (uint256 value, bool isVerified, uint256 timestamp);
    function getValuationHistory(uint256 assetId) external view returns (
        uint256[] memory values,
        uint256[] memory timestamps,
        address[] memory valuators
    );
}
