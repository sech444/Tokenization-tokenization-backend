// ============================================================================

// contracts/libraries/TokenizationUtils.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title TokenizationUtils
 * @dev Utility library for tokenization calculations and validations
 */
library TokenizationUtils {
    uint256 private constant BASIS_POINTS = 10000;
    uint256 private constant SECONDS_PER_YEAR = 365 days;

    /**
     * @dev Calculate token price based on asset value and total tokens
     */
    function calculateTokenPrice(
        uint256 assetValue,
        uint256 totalTokens,
        uint256 premiumBasisPoints
    ) internal pure returns (uint256) {
        require(assetValue > 0 && totalTokens > 0, "Invalid parameters");

        uint256 basePrice = assetValue / totalTokens;
        uint256 premium = (basePrice * premiumBasisPoints) / BASIS_POINTS;
        return basePrice + premium;
    }

    /**
     * @dev Validate tokenization parameters
     */
    function validateTokenization(
        uint256 assetValue,
        uint256 totalTokens,
        uint256 minTokenValue
    ) internal pure returns (bool) {
        if (assetValue == 0 || totalTokens == 0) return false;
        uint256 tokenValue = assetValue / totalTokens;
        return tokenValue >= minTokenValue;
    }

    /**
     * @dev Calculate dividend share for a token holder
     */
    function calculateDividendShare(
        uint256 totalDividend,
        uint256 userTokens,
        uint256 totalSupply
    ) internal pure returns (uint256) {
        if (totalSupply == 0 || userTokens == 0) return 0;
        return (totalDividend * userTokens) / totalSupply;
    }

    /**
     * @dev Calculate compound interest
     */
    function calculateCompoundInterest(
        uint256 principal,
        uint256 ratePerSecond,
        uint256 timeElapsed
    ) internal pure returns (uint256) {
        if (timeElapsed == 0) return principal;

        // Simple interest approximation for gas efficiency
        uint256 interest = (principal * ratePerSecond * timeElapsed) / 1e18;
        return principal + interest;
    }

    /**
     * @dev Calculate APY from rate per second
     */
    function calculateAPY(uint256 ratePerSecond) internal pure returns (uint256) {
        // Approximation: rate * seconds_per_year * 100 (for percentage)
        return (ratePerSecond * SECONDS_PER_YEAR * 100) / 1e18;
    }

    /**
     * @dev Calculate trading fee with tiers
     */
    function calculateTradingFee(
        uint256 volume,
        uint256 baseFeeRate,
        uint256 volumeThreshold,
        uint256 discountRate
    ) internal pure returns (uint256) {
        uint256 baseFee = (volume * baseFeeRate) / BASIS_POINTS;

        if (volume >= volumeThreshold) {
            uint256 discount = (baseFee * discountRate) / BASIS_POINTS;
            return baseFee - discount;
        }

        return baseFee;
    }

    /**
     * @dev Validate address array for duplicates
     */
    function validateNoDuplicates(address[] memory addresses) internal pure returns (bool) {
        for (uint256 i = 0; i < addresses.length; i++) {
            for (uint256 j = i + 1; j < addresses.length; j++) {
                if (addresses[i] == addresses[j]) {
                    return false;
                }
            }
        }
        return true;
    }

    /**
     * @dev Calculate weighted average
     */
    function calculateWeightedAverage(
        uint256[] memory values,
        uint256[] memory weights
    ) internal pure returns (uint256) {
        require(values.length == weights.length, "Array length mismatch");
        require(values.length > 0, "Empty arrays");

        uint256 weightedSum = 0;
        uint256 totalWeight = 0;

        for (uint256 i = 0; i < values.length; i++) {
            weightedSum += values[i] * weights[i];
            totalWeight += weights[i];
        }

        require(totalWeight > 0, "Zero total weight");
        return weightedSum / totalWeight;
    }

    /**
     * @dev Calculate percentage difference between two values
     */
    function calculatePercentageDifference(
        uint256 value1,
        uint256 value2
    ) internal pure returns (uint256) {
        if (value1 == value2) return 0;

        uint256 difference = value1 > value2 ? value1 - value2 : value2 - value1;
        uint256 average = (value1 + value2) / 2;

        return (difference * BASIS_POINTS) / average;
    }

    /**
     * @dev Validate token symbol format
     */
    function isValidTokenSymbol(string memory symbol) internal pure returns (bool) {
        bytes memory symbolBytes = bytes(symbol);
        if (symbolBytes.length < 2 || symbolBytes.length > 10) return false;

        for (uint256 i = 0; i < symbolBytes.length; i++) {
            bytes1 char = symbolBytes[i];
            if (
                !(char >= 0x41 && char <= 0x5A) && // A-Z
                !(char >= 0x61 && char <= 0x7A) && // a-z
                !(char >= 0x30 && char <= 0x39)    // 0-9
            ) {
                return false;
            }
        }

        return true;
    }

    /**
     * @dev Calculate time-based multiplier for loyalty rewards
     */
    function calculateLoyaltyMultiplier(
        uint256 stakingDuration,
        uint256 maxMultiplier,
        uint256 maxDuration
    ) internal pure returns (uint256) {
        if (stakingDuration >= maxDuration) {
            return maxMultiplier;
        }

        return BASIS_POINTS + ((maxMultiplier - BASIS_POINTS) * stakingDuration) / maxDuration;
    }

    /**
     * @dev Safe percentage calculation
     */
    function safePercentage(uint256 value, uint256 percentage) internal pure returns (uint256) {
        return (value * percentage) / 100;
    }

    /**
     * @dev Convert basis points to percentage
     */
    function basisPointsToPercentage(uint256 basisPoints) internal pure returns (uint256) {
        return basisPoints / 100;
    }

    /**
     * @dev Convert percentage to basis points
     */
    function percentageToBasisPoints(uint256 percentage) internal pure returns (uint256) {
        return percentage * 100;
    }
}
