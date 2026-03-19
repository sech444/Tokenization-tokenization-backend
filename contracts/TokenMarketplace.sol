// import { Initializable } from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
// import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
// import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
// import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
// import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// import "./core/MarketplaceCore.sol";
// import "./interfaces/core/IComplianceManager.sol";
// import "./interfaces/core/IAuditTrail.sol";
// import "./interfaces/core/IFeeManager.sol";

// /**
//  * @title TokenMarketplace
//  * @dev High-level contract that connects MarketplaceCore with compliance, audit, and fees
//  * @notice Provides simplified functions for users to trade tokenized assets
//  */
// contract TokenMarketplace is
//     Initializable,
//     AccessControlUpgradeable,
//     PausableUpgradeable,
//     ReentrancyGuardUpgradeable
// {
//     bytes32 public constant MARKETPLACE_ADMIN_ROLE = keccak256("MARKETPLACE_ADMIN_ROLE");

//     MarketplaceCore public marketplaceCore;
//     IComplianceManager public complianceManager;
//     IAuditTrail public auditTrail;
//     IFeeManager public feeManager;

//     event TokenBuyOrderCreated(address indexed buyer, address indexed token, uint256 orderId);
//     event TokenSellOrderCreated(address indexed seller, address indexed token, uint256 orderId);
//     event TokenOrderFilled(address indexed filler, uint256 orderId, uint256 amount, uint256 totalCost);

//     function initialize(
//         address admin,
//         address _marketplaceCore,
//         address _complianceManager,
//         address _auditTrail,
//         address _feeManager
//     ) public initializer {
//         __AccessControl_init();
//         __Pausable_init();
//         __ReentrancyGuard_init();

//         _grantRole(DEFAULT_ADMIN_ROLE, admin);
//         _grantRole(MARKETPLACE_ADMIN_ROLE, admin);

//         marketplaceCore = MarketplaceCore(_marketplaceCore);
//         complianceManager = IComplianceManager(_complianceManager);
//         auditTrail = IAuditTrail(_auditTrail);
//         feeManager = IFeeManager(_feeManager);
//     }

//     // ========== USER FUNCTIONS ==========

//     function createSellOrder(
//         address token,
//         uint256 amount,
//         uint256 pricePerToken,
//         uint256 duration,
//         uint256 minFillAmount,
//         bool allowPartialFill
//     ) external whenNotPaused nonReentrant returns (uint256) {
//         require(complianceManager.isKYCVerified(msg.sender), "KYC required");

//         IERC20(token).transferFrom(msg.sender, address(this), amount);
//         IERC20(token).approve(address(marketplaceCore), amount);

//         uint256 orderId = marketplaceCore.createSellOrder(
//             token,
//             amount,
//             pricePerToken,
//             duration,
//             minFillAmount,
//             allowPartialFill
//         );

//         auditTrail.logTransaction(
//             keccak256("SELL_ORDER_CREATED"),
//             msg.sender,
//             amount,
//             abi.encodePacked(token, pricePerToken)
//         );

//         emit TokenSellOrderCreated(msg.sender, token, orderId);
//         return orderId;
//     }

//     function createBuyOrder(
//         address token,
//         uint256 amount,
//         uint256 pricePerToken,
//         uint256 duration,
//         uint256 minFillAmount,
//         bool allowPartialFill
//     ) external payable whenNotPaused nonReentrant returns (uint256) {
//         require(complianceManager.isKYCVerified(msg.sender), "KYC required");

//         uint256 orderId = marketplaceCore.createBuyOrder{value: msg.value}(
//             token,
//             amount,
//             pricePerToken,
//             duration,
//             minFillAmount,
//             allowPartialFill
//         );

//         auditTrail.logTransaction(
//             keccak256("BUY_ORDER_CREATED"),
//             msg.sender,
//             amount,
//             abi.encodePacked(token, pricePerToken)
//         );

//         emit TokenBuyOrderCreated(msg.sender, token, orderId);
//         return orderId;
//     }

//     function fillOrder(uint256 orderId, uint256 amount) external payable whenNotPaused nonReentrant {
//         marketplaceCore.fillOrder{value: msg.value}(orderId, amount);

//         auditTrail.logTransaction(
//             keccak256("ORDER_FILLED"),
//             msg.sender,
//             amount,
//             abi.encodePacked(orderId)
//         );

//         emit TokenOrderFilled(msg.sender, orderId, amount, msg.value);
//     }

//     function cancelOrder(uint256 orderId) external whenNotPaused nonReentrant {
//         marketplaceCore.cancelOrder(orderId);

//         auditTrail.logTransaction(
//             keccak256("ORDER_CANCELLED"),
//             msg.sender,
//             0,
//             abi.encodePacked(orderId)
//         );
//     }

//     // ========== ADMIN FUNCTIONS ==========
//     function pauseMarketplace() external onlyRole(MARKETPLACE_ADMIN_ROLE) {
//         _pause();
//     }

//     function unpauseMarketplace() external onlyRole(MARKETPLACE_ADMIN_ROLE) {
//         _unpause();
//     }

//     // ========== VIEW HELPERS ==========
//     function getUserOrders(address user) external view returns (uint256[] memory) {
//         return marketplaceCore.getUserOrders(user);
//     }

//     function getTokenOrders(address token) external view returns (uint256[] memory) {
//         return marketplaceCore.getTokenOrders(token);
//     }

//     function getMarketData(address token) external view returns (MarketplaceCore.MarketData memory) {
//         return marketplaceCore.getMarketData(token);
//     }
// }

// contracts/TokenMarketplace.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import "./core/MarketplaceCore.sol";
import "./interfaces/core/IComplianceManager.sol";
import "./interfaces/core/IAuditTrail.sol";
import "./interfaces/core/IFeeManager.sol";

/**
 * @title TokenMarketplace
 * @dev High-level contract that connects MarketplaceCore with compliance, audit, and fees
 * @notice Provides simplified functions for users to trade tokenized assets
 */
contract TokenMarketplace is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable
{
    bytes32 public constant MARKETPLACE_ADMIN_ROLE =
        keccak256("MARKETPLACE_ADMIN_ROLE");

    MarketplaceCore public marketplaceCore;
    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;
    IFeeManager public feeManager;

    event TokenBuyOrderCreated(
        address indexed buyer,
        address indexed token,
        uint256 orderId
    );
    event TokenSellOrderCreated(
        address indexed seller,
        address indexed token,
        uint256 orderId
    );
    event TokenOrderFilled(
        address indexed filler,
        uint256 orderId,
        uint256 amount,
        uint256 totalCost
    );

    function initialize(
        address admin,
        address _marketplaceCore,
        address _complianceManager,
        address _auditTrail,
        address _feeManager
    ) public initializer {
        __AccessControl_init();
        __Pausable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(MARKETPLACE_ADMIN_ROLE, admin);

        // Fixed: Use payable cast for MarketplaceCore since it has a receive function
        marketplaceCore = MarketplaceCore(payable(_marketplaceCore));
        complianceManager = IComplianceManager(_complianceManager);
        auditTrail = IAuditTrail(_auditTrail);
        feeManager = IFeeManager(_feeManager);
    }

    // ========== USER FUNCTIONS ==========

    function createSellOrder(
        address token,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) external whenNotPaused nonReentrant returns (uint256) {
        require(complianceManager.isKYCVerified(msg.sender), "KYC required");

        IERC20(token).transferFrom(msg.sender, address(this), amount);
        IERC20(token).approve(address(marketplaceCore), amount);

        uint256 orderId = marketplaceCore.createSellOrder(
            token,
            amount,
            pricePerToken,
            duration,
            minFillAmount,
            allowPartialFill
        );

        auditTrail.logTransaction(
            keccak256("SELL_ORDER_CREATED"),
            msg.sender,
            amount,
            abi.encodePacked(token, pricePerToken)
        );

        emit TokenSellOrderCreated(msg.sender, token, orderId);
        return orderId;
    }

    function createBuyOrder(
        address token,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) external payable whenNotPaused nonReentrant returns (uint256) {
        require(complianceManager.isKYCVerified(msg.sender), "KYC required");

        uint256 orderId = marketplaceCore.createBuyOrder{value: msg.value}(
            token,
            amount,
            pricePerToken,
            duration,
            minFillAmount,
            allowPartialFill
        );

        auditTrail.logTransaction(
            keccak256("BUY_ORDER_CREATED"),
            msg.sender,
            amount,
            abi.encodePacked(token, pricePerToken)
        );

        emit TokenBuyOrderCreated(msg.sender, token, orderId);
        return orderId;
    }

    function fillOrder(
        uint256 orderId,
        uint256 amount
    ) external payable whenNotPaused nonReentrant {
        marketplaceCore.fillOrder{value: msg.value}(orderId, amount);

        auditTrail.logTransaction(
            keccak256("ORDER_FILLED"),
            msg.sender,
            amount,
            abi.encodePacked(orderId)
        );

        emit TokenOrderFilled(msg.sender, orderId, amount, msg.value);
    }

    function cancelOrder(uint256 orderId) external whenNotPaused nonReentrant {
        marketplaceCore.cancelOrder(orderId);

        auditTrail.logTransaction(
            keccak256("ORDER_CANCELLED"),
            msg.sender,
            0,
            abi.encodePacked(orderId)
        );
    }

    // ========== ADMIN FUNCTIONS ==========
    function pauseMarketplace() external onlyRole(MARKETPLACE_ADMIN_ROLE) {
        _pause();
    }

    function unpauseMarketplace() external onlyRole(MARKETPLACE_ADMIN_ROLE) {
        _unpause();
    }

    // ========== VIEW HELPERS ==========
    function getUserOrders(
        address user
    ) external view returns (uint256[] memory) {
        return marketplaceCore.getUserOrders(user);
    }

    function getTokenOrders(
        address token
    ) external view returns (uint256[] memory) {
        return marketplaceCore.getTokenOrders(token);
    }

    function getMarketData(
        address token
    ) external view returns (MarketplaceCore.MarketData memory) {
        return marketplaceCore.getMarketData(token);
    }

    function getOrder(
        uint256 orderId
    ) external view returns (MarketplaceCore.Order memory) {
        return marketplaceCore.getOrder(orderId);
    }

    function getTrade(
        uint256 tradeId
    ) external view returns (MarketplaceCore.Trade memory) {
        return marketplaceCore.getTrade(tradeId);
    }

    function getActiveOrders(
        address token
    ) external view returns (uint256[] memory) {
        return marketplaceCore.getActiveOrders(token);
    }

    function getBestPrices(
        address token
    ) external view returns (uint256 bestBuyPrice, uint256 bestSellPrice) {
        return marketplaceCore.getBestPrices(token);
    }

    function getOrderBook(
        address token,
        uint256 depth
    )
        external
        view
        returns (
            uint256[] memory buyOrderIds,
            uint256[] memory buyPrices,
            uint256[] memory buyAmounts,
            uint256[] memory sellOrderIds,
            uint256[] memory sellPrices,
            uint256[] memory sellAmounts
        )
    {
        return marketplaceCore.getOrderBook(token, depth);
    }

    function canFillOrder(
        uint256 orderId,
        address filler,
        uint256 amount
    ) external view returns (bool, string memory) {
        return marketplaceCore.canFillOrder(orderId, filler, amount);
    }

    function estimateFillCost(
        uint256 orderId,
        uint256 amount
    )
        external
        view
        returns (uint256 totalCost, uint256 tradingFee, uint256 totalRequired)
    {
        return marketplaceCore.estimateFillCost(orderId, amount);
    }

    function getEscrowBalance(
        address user,
        address token
    ) external view returns (uint256) {
        return marketplaceCore.getEscrowBalance(user, token);
    }

    function getOrderStatistics(
        address token
    )
        external
        view
        returns (
            uint256 totalOrders,
            uint256 activeOrders,
            uint256 totalTrades,
            uint256 totalVolume
        )
    {
        return marketplaceCore.getOrderStatistics(token);
    }

    function getUserTrades(
        address user,
        uint256 limit
    ) external view returns (uint256[] memory) {
        return marketplaceCore.getUserTrades(user, limit);
    }

    function getRecentTrades(
        address token,
        uint256 limit
    ) external view returns (uint256[] memory) {
        return marketplaceCore.getRecentTrades(token, limit);
    }
}
