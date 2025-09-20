// contracts/oracles/RealEstatePriceOracle.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";

contract RealEstatePriceOracle {
    
    struct MarketData {
        uint256 avgPricePerSqFt;
        uint256 lastUpdated;
        int256 monthlyChange;
    }
    
    // ZIP code => Market data
    mapping(string => MarketData) public marketPrices;
    
    // Chainlink price feeds for real estate indices
    mapping(string => AggregatorV3Interface) public priceFeeds;
    
    constructor() {
        // Initialize with real estate index price feeds
        // These would be custom Chainlink nodes providing real estate data
        priceFeeds["US_NATIONAL"] = AggregatorV3Interface(
            0x... // Address of Chainlink real estate index feed
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
            uint80 roundId,
            int256 price,
            uint256 startedAt,
            uint256 updatedAt,
            uint80 answeredInRound
        ) = priceFeeds["US_NATIONAL"].latestRoundData();
        
        require(updatedAt > 0, "Round not complete");
        
        // Update local market data
        marketPrices[zipCode] = MarketData({
            avgPricePerSqFt: uint256(price),
            lastUpdated: updatedAt,
            monthlyChange: _calculateMonthlyChange(zipCode, price)
        });
    }
}