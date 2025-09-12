// ============================================================================

// contracts/interfaces/core/IRewardSystem.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IRewardSystem {
    function stakeTokens(address tokenAddress, uint256 amount) external;
    function unstakeTokens(address tokenAddress, uint256 amount) external;
    function claimRewards() external;
    function registerReferral(address referrer) external;
    function awardTradingRewards(address trader, uint256 tradingVolume) external;
    
    function earnedRewards(address tokenAddress, address user) external view returns (uint256);
    function getUserRewards(address user) external view returns (
        uint256 totalEarned,
        uint256 totalClaimed,
        uint256 stakingRewards,
        uint256 tradingRewards,
        uint256 referralRewards,
        uint256 loyaltyPoints,
        uint256 lastClaimTimestamp,
        uint256 lastActivityTimestamp
    );
}
