// contracts/oracles/RealEstatePriceOracle.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";

contract RealEstatePriceOracle {
    
    struct MarketData {
        uint256 avgPricePerSqFt;
        uint256 lastUpdated;
        int256 monthlyChange; // Stored as basis points, e.g., 100 = 1%
    }
    
    // ZIP code => Market data
    mapping(string => MarketData) public marketPrices;
    
    // Chainlink price feeds for real estate indices
    mapping(string => AggregatorV3Interface) public priceFeeds;
    
    constructor() {
        // Initialize with real estate index price feeds
        // These would be custom Chainlink nodes providing real estate data
        priceFeeds["US_NATIONAL"] = AggregatorV3Interface(
           address(0) // Address of Chainlink real estate index feed
        );
    }
    
    function getPropertyValuationEstimate(
        string calldata zipCode,
        uint256 squareFootage
    ) external view returns (uint256 estimatedValue) {
        MarketData memory market = marketPrices[zipCode];
        require(market.avgPricePerSqFt > 0, "No market data");
        
        estimatedValue = market.avgPricePerSqFt * squareFootage;
        
        // Apply market trend adjustment
        // Note: This logic only applies positive changes. A more robust implementation
        // would handle negative changes by decreasing the value.
        if (market.monthlyChange > 0) {
            uint256 monthsSinceUpdate = (block.timestamp - market.lastUpdated) / 30 days;
            uint256 adjustment = (uint256(market.monthlyChange) * monthsSinceUpdate) / 100;
            estimatedValue = estimatedValue + (estimatedValue * adjustment / 10000);
        }
    }
    
    function updateMarketData(
        string calldata zipCode
    ) external {
        // Get latest price from Chainlink oracle
        (
            /*uint80 roundId*/,
            int256 price,
            /*uint256 startedAt*/,
            uint256 updatedAt,
            /*uint80 answeredInRound*/
        ) = priceFeeds["US_NATIONAL"].latestRoundData();
        
        require(updatedAt > 0, "Round not complete");
        
        // Update local market data
        marketPrices[zipCode] = MarketData({
            avgPricePerSqFt: uint256(price),
            lastUpdated: updatedAt,
            monthlyChange: _calculateMonthlyChange(zipCode, price)
        });
    }

    // ===== FIX: Added the missing _calculateMonthlyChange function =====
    /**
     * @dev Calculates the percentage change from the last stored price.
     * @param zipCode The ZIP code to check against.
     * @param newPrice The new price from the oracle.
     * @return The change in basis points (e.g., 100 = 1%, -50 = -0.5%).
     */
    function _calculateMonthlyChange(
        string calldata zipCode,
        int256 newPrice
    ) private view returns (int256) {
        MarketData memory oldData = marketPrices[zipCode];
        int256 oldPrice = int256(oldData.avgPricePerSqFt);

        // If there's no old price, there's no change to calculate.
        if (oldPrice == 0) {
            return 0;
        }

        // Calculate change: ((new - old) / old) * 10000 for basis points
        // Multiply first to maintain precision before dividing.
        int256 priceDifference = newPrice - oldPrice;
        return (priceDifference * 10000) / oldPrice;
    }
}