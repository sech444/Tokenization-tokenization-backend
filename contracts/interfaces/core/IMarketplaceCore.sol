// contracts/interfaces/core/IMarketplaceCore.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IMarketplaceCore {
    // --- Enums ---
    enum OrderType { BUY, SELL }
    enum OrderStatus { ACTIVE, COMPLETED, CANCELLED, EXPIRED, PARTIALLY_FILLED }
    
    // ===== THIS IS THE FIX: Add the missing initialize function =====
    function initialize(
        address admin,
        address complianceManager,
        address auditTrail,
        address feeManager
    ) external;

    // --- Other Functions ---
    function createSellOrder(
        address tokenAddress,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) external returns (uint256);
    
    function createBuyOrder(
        address tokenAddress,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) external payable returns (uint256);
    
    function fillOrder(uint256 orderId, uint256 amount) external payable;
    function cancelOrder(uint256 orderId) external;
    
    function getOrder(uint256 orderId) external view returns (
        uint256 id,
        OrderType orderType,
        address tokenAddress,
        uint256 amount,
        uint256 price,
        uint256 filledAmount,
        address creator,
        OrderStatus status,
        uint256 createdAt,
        uint256 expiresAt,
        uint256 minFillAmount,
        bool allowPartialFill
    );
}