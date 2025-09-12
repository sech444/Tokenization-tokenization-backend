// ============================================================================

// contracts/interfaces/external/IPriceOracle.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IPriceOracle {
    function getPrice(address token) external view returns (uint256 price, uint256 timestamp);
    function getPriceInUSD(address token) external view returns (uint256 priceUSD, uint256 timestamp);
    function updatePrice(address token, uint256 price) external;
    function isPriceStale(address token, uint256 maxAge) external view returns (bool);
}


