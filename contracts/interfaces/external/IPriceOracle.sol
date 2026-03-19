// contracts/interfaces/external/IPriceOracle.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IPriceOracle
 * @dev A simplified interface for a Chainlink Aggregator to get the latest price.
 */
interface IPriceOracle {
    /**
     * @dev Returns the latest round data from the aggregator.
     */
    function latestRoundData()
        external
        view
        returns (
            uint80 roundId,
            int256 answer,
            uint256 startedAt,
            uint256 updatedAt,
            uint80 answeredInRound
        );
}