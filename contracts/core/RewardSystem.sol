// contracts/core/RewardSystem.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "../interfaces/core/IAuditTrail.sol";

/**
 * @title RewardSystem
 * @dev Manages platform rewards including staking, trading rewards, and referrals
 */
contract RewardSystem is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable
{
    bytes32 public constant REWARD_ADMIN_ROLE = keccak256("REWARD_ADMIN_ROLE");
    bytes32 public constant SYSTEM_ROLE = keccak256("SYSTEM_ROLE");

    struct UserRewards {
        uint256 totalEarned;
        uint256 totalClaimed;
        uint256 stakingRewards;
        uint256 tradingRewards;
        uint256 referralRewards;
        uint256 loyaltyPoints;
        uint256 lastClaimTimestamp;
        uint256 lastActivityTimestamp;
    }

    struct StakingPool {
        address tokenAddress;
        uint256 rewardRate; // Tokens per second per staked token (scaled by 1e18)
        uint256 totalStaked;
        uint256 lastUpdateTime;
        uint256 rewardPerTokenStored;
        uint256 periodFinish;
        uint256 rewardDuration;
        bool isActive;
        uint256 minStakeAmount;
        uint256 lockupPeriod;
    }

    struct UserStaking {
        uint256 stakedAmount;
        uint256 userRewardPerTokenPaid;
        uint256 rewards;
        uint256 stakingStartTime;
        uint256 lastStakeTime;
        uint256 unstakeRequestTime;
        bool hasUnstakeRequest;
    }

    struct Dividend {
        uint256 totalAmount;
        uint256 timestamp;
        uint256 claimedAmount;
        uint256 eligibleSupply;
        mapping(address => bool) claimed;
        mapping(address => uint256) entitlement;
    }

    struct ReferralTier {
        uint256 minReferrals;
        uint256 rewardPercentage; // basis points
        uint256 bonusMultiplier; // basis points (10000 = 1x)
        string tierName;
    }

    mapping(address => UserRewards) public userRewards;
    mapping(address => StakingPool) public stakingPools;
    mapping(address => mapping(address => UserStaking)) public userStaking; // user => token => staking info
    mapping(address => address) public referrals; // user => referrer
    mapping(address => address[]) public referees; // referrer => referees
    mapping(address => uint256) public referralCounts;
    mapping(uint256 => Dividend) public dividends;
    mapping(uint256 => ReferralTier) public referralTiers;

    uint256 public tradingRewardRate; // in basis points
    uint256 public referralRewardRate; // in basis points
    uint256 public loyaltyMultiplier;
    uint256 public maxLoyaltyBonus; // in basis points
    uint256 public rewardTokenDecimals;
    uint256 public nextDividendId;
    uint256 public totalReferralTiers;
    uint256 public unstakeCooldown;

    IERC20 public rewardToken;
    IAuditTrail public auditTrail;

    // Events
    event RewardsClaimed(
        address indexed user,
        uint256 amount,
        uint256 loyaltyBonus
    );
    event TokensStaked(
        address indexed user,
        address indexed token,
        uint256 amount
    );
    event TokensUnstaked(
        address indexed user,
        address indexed token,
        uint256 amount
    );
    event UnstakeRequested(
        address indexed user,
        address indexed token,
        uint256 amount
    );
    event StakingPoolCreated(
        address indexed token,
        uint256 rewardRate,
        uint256 rewardDuration
    );
    event StakingPoolUpdated(address indexed token, uint256 newRewardRate);
    event ReferralRegistered(address indexed user, address indexed referrer);
    event TradingRewardEarned(
        address indexed user,
        uint256 amount,
        uint256 volume
    );
    event ReferralRewardEarned(
        address indexed referrer,
        address indexed referee,
        uint256 amount
    );
    event DividendDistributed(uint256 indexed dividendId, uint256 totalAmount);
    event DividendClaimed(
        uint256 indexed dividendId,
        address indexed user,
        uint256 amount
    );
    event LoyaltyPointsEarned(
        address indexed user,
        uint256 points,
        string reason
    );

    // ------------------------
    // Init
    // ------------------------
    function initialize(
        address admin,
        address _rewardToken,
        address _auditTrail
    ) public initializer {
        __AccessControl_init();
        __Pausable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(REWARD_ADMIN_ROLE, admin);
        _grantRole(SYSTEM_ROLE, admin);

        rewardToken = IERC20(_rewardToken);
        auditTrail = IAuditTrail(_auditTrail);

        // Defaults
        tradingRewardRate = 100; // 1%
        referralRewardRate = 500; // 5%
        loyaltyMultiplier = 1000;
        maxLoyaltyBonus = 5000; // 50%
        rewardTokenDecimals = 18;
        nextDividendId = 1;
        unstakeCooldown = 7 days;

        _initializeReferralTiers();
    }

    // ------------------------
    // Staking
    // ------------------------
    function createStakingPool(
        address tokenAddress,
        uint256 rewardRate,
        uint256 rewardDuration,
        uint256 minStakeAmount,
        uint256 lockupPeriod
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        require(tokenAddress != address(0), "Invalid token");
        require(!stakingPools[tokenAddress].isActive, "Pool exists");
        require(rewardRate > 0, "Invalid rate");
        require(
            rewardDuration >= 7 days && rewardDuration <= 365 days,
            "Invalid duration"
        );

        stakingPools[tokenAddress] = StakingPool({
            tokenAddress: tokenAddress,
            rewardRate: rewardRate,
            totalStaked: 0,
            lastUpdateTime: block.timestamp,
            rewardPerTokenStored: 0,
            periodFinish: block.timestamp + rewardDuration,
            rewardDuration: rewardDuration,
            isActive: true,
            minStakeAmount: minStakeAmount,
            lockupPeriod: lockupPeriod
        });

        emit StakingPoolCreated(tokenAddress, rewardRate, rewardDuration);
    }

    function stakeTokens(
        address tokenAddress,
        uint256 amount
    ) external nonReentrant whenNotPaused {
        require(amount > 0, "Invalid amount");
        StakingPool storage pool = stakingPools[tokenAddress];
        require(pool.isActive, "Pool inactive");
        require(amount >= pool.minStakeAmount, "Below minimum");

        _updateReward(tokenAddress, msg.sender);

        IERC20(tokenAddress).transferFrom(msg.sender, address(this), amount);

        UserStaking storage userStake = userStaking[msg.sender][tokenAddress];
        userStake.stakedAmount += amount;
        if (userStake.stakingStartTime == 0) {
            userStake.stakingStartTime = block.timestamp;
        }
        userStake.lastStakeTime = block.timestamp;

        pool.totalStaked += amount;

        uint256 loyaltyPoints = amount / 1e18;
        _awardLoyaltyPoints(msg.sender, loyaltyPoints, "STAKING");

        emit TokensStaked(msg.sender, tokenAddress, amount);
    }

    function requestUnstake(
        address tokenAddress,
        uint256 amount
    ) external whenNotPaused {
        UserStaking storage userStake = userStaking[msg.sender][tokenAddress];
        require(userStake.stakedAmount >= amount, "Insufficient staked");

        StakingPool memory pool = stakingPools[tokenAddress];
        require(
            block.timestamp >= userStake.lastStakeTime + pool.lockupPeriod,
            "Lockup not met"
        );

        userStake.unstakeRequestTime = block.timestamp;
        userStake.hasUnstakeRequest = true;

        emit UnstakeRequested(msg.sender, tokenAddress, amount);
    }

    function unstakeTokens(
        address tokenAddress,
        uint256 amount
    ) external nonReentrant {
        require(amount > 0, "Invalid amount");
        UserStaking storage userStake = userStaking[msg.sender][tokenAddress];
        require(userStake.stakedAmount >= amount, "Insufficient");
        require(userStake.hasUnstakeRequest, "No request");
        require(
            block.timestamp >= userStake.unstakeRequestTime + unstakeCooldown,
            "Cooldown"
        );

        _updateReward(tokenAddress, msg.sender);

        userStake.stakedAmount -= amount;
        userStake.hasUnstakeRequest = false;
        stakingPools[tokenAddress].totalStaked -= amount;

        IERC20(tokenAddress).transfer(msg.sender, amount);

        emit TokensUnstaked(msg.sender, tokenAddress, amount);
    }

    // ------------------------
    // Rewards
    // ------------------------
    function claimRewards() external nonReentrant whenNotPaused {
        UserRewards storage rewards = userRewards[msg.sender];
        uint256 claimable = rewards.totalEarned - rewards.totalClaimed;
        require(claimable > 0, "No rewards");

        uint256 loyaltyBonus = _calculateLoyaltyBonus(msg.sender, claimable);
        uint256 totalClaim = claimable + loyaltyBonus;

        rewards.totalClaimed += claimable;
        rewards.lastClaimTimestamp = block.timestamp;
        rewards.lastActivityTimestamp = block.timestamp;

        _awardLoyaltyPoints(msg.sender, claimable / 1e18, "CLAIMING");

        require(
            rewardToken.balanceOf(address(this)) >= totalClaim,
            "Insufficient rewards"
        );
        rewardToken.transfer(msg.sender, totalClaim);

        if (referrals[msg.sender] != address(0)) {
            _processReferralReward(
                referrals[msg.sender],
                msg.sender,
                claimable
            );
        }

        emit RewardsClaimed(msg.sender, claimable, loyaltyBonus);
    }

    function awardTradingRewards(
        address trader,
        uint256 tradingVolume
    ) external onlyRole(SYSTEM_ROLE) whenNotPaused {
        require(tradingVolume > 0, "Invalid volume");

        uint256 baseReward = (tradingVolume * tradingRewardRate) / 10000;
        uint256 loyaltyBonus = _calculateLoyaltyBonus(trader, baseReward);
        uint256 totalReward = baseReward + loyaltyBonus;

        UserRewards storage rewards = userRewards[trader];
        rewards.tradingRewards += totalReward;
        rewards.totalEarned += totalReward;
        rewards.lastActivityTimestamp = block.timestamp;

        uint256 loyaltyPoints = tradingVolume / 1e18;
        _awardLoyaltyPoints(trader, loyaltyPoints, "TRADING");

        emit TradingRewardEarned(trader, totalReward, tradingVolume);
    }

    // ------------------------
    // Referrals
    // ------------------------
    function registerReferral(address referrer) external whenNotPaused {
        require(referrals[msg.sender] == address(0), "Already referred");
        require(
            referrer != msg.sender && referrer != address(0),
            "Invalid referrer"
        );

        referrals[msg.sender] = referrer;
        referees[referrer].push(msg.sender);
        referralCounts[referrer]++;

        _awardLoyaltyPoints(msg.sender, 10, "REFERRAL_SIGNUP");
        _awardLoyaltyPoints(referrer, 20, "REFERRAL_BONUS");

        emit ReferralRegistered(msg.sender, referrer);
    }

    function _processReferralReward(
        address referrer,
        address referee,
        uint256 refereeReward
    ) internal {
        uint256 referralReward = (refereeReward * referralRewardRate) / 10000;
        ReferralTier memory tier = _getReferralTier(referrer);
        uint256 finalReward = (referralReward * tier.bonusMultiplier) / 10000;

        UserRewards storage referrerRewards = userRewards[referrer];
        referrerRewards.referralRewards += finalReward;
        referrerRewards.totalEarned += finalReward;

        emit ReferralRewardEarned(referrer, referee, finalReward);
    }

    function _getReferralTier(
        address referrer
    ) internal view returns (ReferralTier memory) {
        uint256 referralCount = referralCounts[referrer];
        for (uint256 i = totalReferralTiers; i > 0; i--) {
            if (referralCount >= referralTiers[i].minReferrals) {
                return referralTiers[i];
            }
        }
        return referralTiers[1]; // fallback to Bronze
    }

    function _initializeReferralTiers() internal {
        referralTiers[1] = ReferralTier(0, 500, 10000, "Bronze");
        referralTiers[2] = ReferralTier(10, 750, 12500, "Silver");
        referralTiers[3] = ReferralTier(25, 1000, 15000, "Gold");
        referralTiers[4] = ReferralTier(50, 1250, 20000, "Platinum");
        totalReferralTiers = 4;
    }

    // ------------------------
    // Dividends
    // ------------------------
    function distributeDividend() external payable onlyRole(REWARD_ADMIN_ROLE) {
        require(msg.value > 0, "No ETH");

        uint256 dividendId = nextDividendId++;
        Dividend storage dividend = dividends[dividendId];
        dividend.totalAmount = msg.value;
        dividend.timestamp = block.timestamp;
        dividend.eligibleSupply = rewardToken.totalSupply();

        emit DividendDistributed(dividendId, msg.value);
    }

    function claimDividend(uint256 dividendId) external nonReentrant {
        require(dividendId < nextDividendId, "Invalid ID");
        Dividend storage dividend = dividends[dividendId];
        require(!dividend.claimed[msg.sender], "Claimed");
        require(dividend.totalAmount > 0, "Empty");

        uint256 balance = rewardToken.balanceOf(msg.sender);
        require(balance > 0, "No tokens");

        uint256 entitlement = (dividend.totalAmount * balance) /
            dividend.eligibleSupply;
        require(entitlement > 0, "Zero entitlement");

        dividend.claimed[msg.sender] = true;
        dividend.entitlement[msg.sender] = entitlement;
        dividend.claimedAmount += entitlement;

        payable(msg.sender).transfer(entitlement);

        emit DividendClaimed(dividendId, msg.sender, entitlement);
    }

    // ------------------------
    // Internals
    // ------------------------
    function _updateReward(address tokenAddress, address user) internal {
        StakingPool storage pool = stakingPools[tokenAddress];
        pool.rewardPerTokenStored = _rewardPerToken(tokenAddress);
        pool.lastUpdateTime = _lastTimeRewardApplicable(tokenAddress);

        if (user != address(0)) {
            UserStaking storage userStake = userStaking[user][tokenAddress];
            uint256 earnedNow = _earned(tokenAddress, user);
            userStake.rewards = earnedNow;
            userStake.userRewardPerTokenPaid = pool.rewardPerTokenStored;

            UserRewards storage rewards = userRewards[user];
            rewards.stakingRewards += earnedNow;
            rewards.totalEarned += earnedNow;
            rewards.lastActivityTimestamp = block.timestamp;

            userStake.rewards = 0; // reset
        }
    }

    function _rewardPerToken(
        address tokenAddress
    ) internal view returns (uint256) {
        StakingPool memory pool = stakingPools[tokenAddress];
        if (pool.totalStaked == 0) return pool.rewardPerTokenStored;

        uint256 timeElapsed = _lastTimeRewardApplicable(tokenAddress) -
            pool.lastUpdateTime;
        return
            pool.rewardPerTokenStored +
            ((timeElapsed * pool.rewardRate * 1e18) / pool.totalStaked);
    }

    function _earned(
        address tokenAddress,
        address user
    ) internal view returns (uint256) {
        UserStaking memory userStake = userStaking[user][tokenAddress];
        return
            (userStake.stakedAmount *
                (_rewardPerToken(tokenAddress) -
                    userStake.userRewardPerTokenPaid)) /
            1e18 +
            userStake.rewards;
    }

    function _lastTimeRewardApplicable(
        address tokenAddress
    ) internal view returns (uint256) {
        StakingPool memory pool = stakingPools[tokenAddress];
        return
            block.timestamp < pool.periodFinish
                ? block.timestamp
                : pool.periodFinish;
    }

    function _calculateLoyaltyBonus(
        address user,
        uint256 baseAmount
    ) internal view returns (uint256) {
        UserRewards memory rewards = userRewards[user];
        if (rewards.loyaltyPoints == 0) return 0;

        uint256 bonusPercentage = rewards.loyaltyPoints / 100;
        uint256 maxBonusPct = maxLoyaltyBonus / 100;
        if (bonusPercentage > maxBonusPct) bonusPercentage = maxBonusPct;

        return (baseAmount * bonusPercentage) / 100;
    }

    function _awardLoyaltyPoints(
        address user,
        uint256 points,
        string memory reason
    ) internal {
        if (points == 0) return;
        userRewards[user].loyaltyPoints += points;
        emit LoyaltyPointsEarned(user, points, reason);
    }

    // ------------------------
    // Admin
    // ------------------------
    function updateStakingPool(
        address tokenAddress,
        uint256 newRewardRate,
        uint256 newRewardDuration
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        StakingPool storage pool = stakingPools[tokenAddress];
        require(pool.isActive, "Inactive");

        _updateReward(tokenAddress, address(0));
        pool.rewardRate = newRewardRate;
        pool.periodFinish = block.timestamp + newRewardDuration;
        pool.rewardDuration = newRewardDuration;

        emit StakingPoolUpdated(tokenAddress, newRewardRate);
    }

    function setRewardRates(
        uint256 _tradingRewardRate,
        uint256 _referralRewardRate,
        uint256 _loyaltyMultiplier,
        uint256 _maxLoyaltyBonus
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        require(_tradingRewardRate <= 1000, "Trading >10%");
        require(_referralRewardRate <= 2000, "Referral >20%");
        require(_maxLoyaltyBonus <= 10000, "Max >100%");

        tradingRewardRate = _tradingRewardRate;
        referralRewardRate = _referralRewardRate;
        loyaltyMultiplier = _loyaltyMultiplier;
        maxLoyaltyBonus = _maxLoyaltyBonus;
    }

    function addReferralTier(
        uint256 minReferrals,
        uint256 rewardPercentage,
        uint256 bonusMultiplier,
        string calldata tierName
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        require(minReferrals > 0, "Invalid min");
        require(rewardPercentage <= 5000, "Too high");
        require(bonusMultiplier >= 10000, "Too low");

        totalReferralTiers++;
        referralTiers[totalReferralTiers] = ReferralTier(
            minReferrals,
            rewardPercentage,
            bonusMultiplier,
            tierName
        );
    }

    function pauseStakingPool(
        address tokenAddress
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        stakingPools[tokenAddress].isActive = false;
    }

    function unpauseStakingPool(
        address tokenAddress
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        stakingPools[tokenAddress].isActive = true;
    }

    function setUnstakeCooldown(
        uint256 _unstakeCooldown
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        require(
            _unstakeCooldown >= 1 days && _unstakeCooldown <= 30 days,
            "Invalid cooldown"
        );
        unstakeCooldown = _unstakeCooldown;
    }

    function depositRewardTokens(
        uint256 amount
    ) external onlyRole(REWARD_ADMIN_ROLE) {
        require(amount > 0, "Invalid");
        rewardToken.transferFrom(msg.sender, address(this), amount);
    }

    function emergencyWithdrawRewardTokens(
        uint256 amount
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(amount <= rewardToken.balanceOf(address(this)), "Insufficient");
        rewardToken.transfer(msg.sender, amount);
    }

    function emergencyWithdrawETH() external onlyRole(DEFAULT_ADMIN_ROLE) {
        uint256 balance = address(this).balance;
        require(balance > 0, "No ETH");
        payable(msg.sender).transfer(balance);
    }

    function pause() external onlyRole(REWARD_ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(REWARD_ADMIN_ROLE) {
        _unpause();
    }

    receive() external payable {}
}
