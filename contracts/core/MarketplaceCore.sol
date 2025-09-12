// contracts/core/MarketplaceCore.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IAuditTrail.sol";
import "../interfaces/core/IFeeManager.sol";

/**
 * @title MarketplaceCore
 * @dev Decentralized marketplace for trading tokenized assets
 * @notice Provides order book functionality with escrow and compliance
 */
contract MarketplaceCore is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable
{
    // Using Solidity 0.8+ checked arithmetic (no SafeMath needed)

    bytes32 public constant MARKETPLACE_ADMIN_ROLE =
        keccak256("MARKETPLACE_ADMIN_ROLE");
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

    event OrderCreated(
        uint256 indexed orderId,
        address indexed creator,
        OrderType orderType,
        address tokenAddress,
        uint256 amount,
        uint256 price
    );

    event OrderFilled(
        uint256 indexed orderId,
        address indexed filler,
        uint256 amount,
        uint256 totalCost,
        bool isComplete
    );

    event TradeExecuted(
        uint256 indexed tradeId,
        address indexed buyer,
        address indexed seller,
        address tokenAddress,
        uint256 amount,
        uint256 price
    );

    event OrderCancelled(
        uint256 indexed orderId,
        address indexed creator,
        uint256 refundAmount
    );

    event MarketDataUpdated(
        address indexed tokenAddress,
        uint256 price,
        uint256 volume24h,
        uint256 totalVolume
    );

    enum OrderType {
        BUY,
        SELL
    }
    enum OrderStatus {
        ACTIVE,
        COMPLETED,
        CANCELLED,
        EXPIRED,
        PARTIALLY_FILLED
    }

    struct Order {
        uint256 orderId;
        OrderType orderType;
        address tokenAddress;
        uint256 amount;
        uint256 price; // Price per token in wei
        uint256 filledAmount;
        address creator;
        OrderStatus status;
        uint256 createdAt;
        uint256 expiresAt;
        uint256 minFillAmount;
        bool allowPartialFill;
    }

    struct Trade {
        uint256 tradeId;
        uint256 buyOrderId;
        uint256 sellOrderId;
        address buyer;
        address seller;
        address tokenAddress;
        uint256 amount;
        uint256 price;
        uint256 timestamp;
        uint256 totalCost;
        uint256 buyerFee;
        uint256 sellerFee;
    }

    struct MarketData {
        address tokenAddress;
        uint256 lastPrice;
        uint256 volume24h;
        uint256 highPrice24h;
        uint256 lowPrice24h;
        uint256 totalVolume;
        uint256 totalTrades;
        uint256 lastTradeTime;
    }

    struct OrderBook {
        uint256[] buyOrders; // Sorted by price (highest first)
        uint256[] sellOrders; // Sorted by price (lowest first)
        mapping(uint256 => uint256) priceToOrderIndex; // price => index in orders array
    }

    mapping(uint256 => Order) public orders;
    mapping(uint256 => Trade) public trades;
    mapping(address => MarketData) public marketData;
    mapping(address => OrderBook) private orderBooks; // Changed to private due to mapping in struct
    mapping(address => uint256[]) public userOrders;
    mapping(address => uint256[]) public tokenOrders;
    mapping(address => mapping(address => uint256)) public escrowBalances; // user => token => amount

    uint256 public nextOrderId = 1;
    uint256 public nextTradeId = 1;
    uint256 public defaultOrderDuration = 30 days;
    uint256 public maxOrderDuration = 180 days;
    uint256 public minOrderValue = 0.001 ether;
    uint256 public maxOrdersPerUser = 100;

    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;
    IFeeManager public feeManager;

    function initialize(
        address admin,
        address _complianceManager,
        address _auditTrail,
        address _feeManager
    ) public initializer {
        __AccessControl_init();
        __Pausable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(MARKETPLACE_ADMIN_ROLE, admin);
        _grantRole(OPERATOR_ROLE, admin);

        complianceManager = IComplianceManager(_complianceManager);
        auditTrail = IAuditTrail(_auditTrail);
        feeManager = IFeeManager(_feeManager);
    }

    function createSellOrder(
        address tokenAddress,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) external whenNotPaused returns (uint256) {
        require(complianceManager.isKYCVerified(msg.sender), "KYC required");
        require(amount > 0, "Invalid amount");
        require(pricePerToken > 0, "Invalid price");
        require(amount * pricePerToken >= minOrderValue, "Order value too low");
        require(
            userOrders[msg.sender].length < maxOrdersPerUser,
            "Too many orders"
        );
        require(duration <= maxOrderDuration, "Duration too long");

        if (minFillAmount == 0) minFillAmount = amount;
        require(minFillAmount <= amount, "Min fill amount too high");

        IERC20 token = IERC20(tokenAddress);
        require(token.balanceOf(msg.sender) >= amount, "Insufficient balance");
        require(
            token.allowance(msg.sender, address(this)) >= amount,
            "Insufficient allowance"
        );

        token.transferFrom(msg.sender, address(this), amount);
        escrowBalances[msg.sender][tokenAddress] += amount;

        uint256 orderId = _createOrder(
            OrderType.SELL,
            tokenAddress,
            amount,
            pricePerToken,
            duration,
            minFillAmount,
            allowPartialFill
        );

        _addToOrderBook(tokenAddress, orderId, OrderType.SELL, pricePerToken);
        return orderId;
    }

    function createBuyOrder(
        address tokenAddress,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) external payable whenNotPaused returns (uint256) {
        require(complianceManager.isKYCVerified(msg.sender), "KYC required");
        require(amount > 0, "Invalid amount");
        require(pricePerToken > 0, "Invalid price");
        require(
            userOrders[msg.sender].length < maxOrdersPerUser,
            "Too many orders"
        );
        require(duration <= maxOrderDuration, "Duration too long");

        uint256 totalValue = amount * pricePerToken;
        require(totalValue >= minOrderValue, "Order value too low");
        require(msg.value >= totalValue, "Insufficient payment");

        if (minFillAmount == 0) minFillAmount = amount;
        require(minFillAmount <= amount, "Min fill amount too high");

        escrowBalances[msg.sender][address(0)] += totalValue;

        uint256 orderId = _createOrder(
            OrderType.BUY,
            tokenAddress,
            amount,
            pricePerToken,
            duration,
            minFillAmount,
            allowPartialFill
        );

        _addToOrderBook(tokenAddress, orderId, OrderType.BUY, pricePerToken);

        if (msg.value > totalValue) {
            payable(msg.sender).transfer(msg.value - totalValue);
        }
        return orderId;
    }

    function fillOrder(
        uint256 orderId,
        uint256 amount
    ) external payable nonReentrant whenNotPaused {
        Order storage order = orders[orderId];
        require(order.status == OrderStatus.ACTIVE, "Order not active");
        require(block.timestamp <= order.expiresAt, "Order expired");
        require(order.creator != msg.sender, "Cannot fill own order");
        require(
            complianceManager.canTransfer(order.creator, msg.sender, amount),
            "Transfer not compliant"
        );

        uint256 availableAmount = order.amount - order.filledAmount;
        require(amount > 0 && amount <= availableAmount, "Invalid fill amount");

        if (!order.allowPartialFill) {
            require(amount == availableAmount, "Must fill complete order");
        } else {
            require(amount >= order.minFillAmount, "Below minimum fill amount");
        }

        uint256 totalCost = amount * order.price;
        uint256 tradingFee = feeManager.calculateFees(
            totalCost,
            feeManager.TRADING_FEE()
        );
        uint256 buyerFee = tradingFee / 2;
        uint256 sellerFee = tradingFee - buyerFee;

        if (order.orderType == OrderType.SELL) {
            require(msg.value >= totalCost + buyerFee, "Insufficient payment");
            IERC20(order.tokenAddress).transfer(msg.sender, amount);
            escrowBalances[order.creator][order.tokenAddress] -= amount;
            payable(order.creator).transfer(totalCost - sellerFee);

            uint256 actualCost = totalCost + buyerFee;
            if (msg.value > actualCost) {
                payable(msg.sender).transfer(msg.value - actualCost);
            }
        } else {
            require(
                IERC20(order.tokenAddress).balanceOf(msg.sender) >= amount,
                "Insufficient tokens"
            );
            require(
                IERC20(order.tokenAddress).allowance(
                    msg.sender,
                    address(this)
                ) >= amount,
                "Insufficient allowance"
            );
            IERC20(order.tokenAddress).transferFrom(
                msg.sender,
                order.creator,
                amount
            );
            escrowBalances[order.creator][address(0)] -= totalCost;
            payable(msg.sender).transfer(totalCost - sellerFee);
        }

        if (tradingFee > 0) {
            feeManager.collectFees{value: tradingFee}(
                feeManager.TRADING_FEE(),
                msg.sender
            );
        }

        order.filledAmount += amount;
        bool isComplete = order.filledAmount >= order.amount;

        if (isComplete) {
            order.status = OrderStatus.COMPLETED;
            _removeFromOrderBook(order.tokenAddress, orderId, order.orderType);
        } else {
            order.status = OrderStatus.PARTIALLY_FILLED;
        }

        uint256 tradeId = _createTrade(
            order.orderType == OrderType.BUY ? orderId : 0,
            order.orderType == OrderType.SELL ? orderId : 0,
            order.orderType == OrderType.SELL ? msg.sender : order.creator,
            order.orderType == OrderType.SELL ? order.creator : msg.sender,
            order.tokenAddress,
            amount,
            order.price,
            totalCost,
            buyerFee,
            sellerFee
        );

        _updateMarketData(order.tokenAddress, order.price, totalCost);

        auditTrail.logTransaction(
            keccak256("ORDER_FILLED"),
            msg.sender,
            amount,
            abi.encodePacked(orderId, totalCost)
        );

        emit OrderFilled(orderId, msg.sender, amount, totalCost, isComplete);
        emit TradeExecuted(
            tradeId,
            order.orderType == OrderType.SELL ? msg.sender : order.creator,
            order.orderType == OrderType.SELL ? order.creator : msg.sender,
            order.tokenAddress,
            amount,
            order.price
        );
    }

    function cancelOrder(uint256 orderId) external nonReentrant {
        Order storage order = orders[orderId];
        require(order.creator == msg.sender, "Not order creator");
        require(
            order.status == OrderStatus.ACTIVE ||
                order.status == OrderStatus.PARTIALLY_FILLED,
            "Cannot cancel order"
        );

        order.status = OrderStatus.CANCELLED;

        // Calculate refund amount
        uint256 remainingAmount = order.amount - order.filledAmount; // Fixed: removed .sub()
        uint256 refundAmount = 0;

        if (order.orderType == OrderType.SELL) {
            // Refund remaining tokens
            if (remainingAmount > 0) {
                IERC20(order.tokenAddress).transfer(
                    order.creator,
                    remainingAmount
                );
                escrowBalances[order.creator][order.tokenAddress] =
                    escrowBalances[order.creator][order.tokenAddress] -
                    remainingAmount; // Fixed: removed .sub()
            }
        } else {
            // Refund remaining ETH
            refundAmount = remainingAmount * order.price; // Fixed: removed .mul()
            if (refundAmount > 0) {
                escrowBalances[order.creator][address(0)] =
                    escrowBalances[order.creator][address(0)] -
                    refundAmount; // Fixed: removed .sub()
                payable(order.creator).transfer(refundAmount);
            }
        }

        // Remove from order book
        _removeFromOrderBook(order.tokenAddress, orderId, order.orderType);

        auditTrail.logTransaction(
            keccak256("ORDER_CANCELLED"),
            msg.sender,
            orderId,
            abi.encodePacked(refundAmount)
        );

        emit OrderCancelled(orderId, msg.sender, refundAmount);
    }

    function _createOrder(
        OrderType orderType,
        address tokenAddress,
        uint256 amount,
        uint256 pricePerToken,
        uint256 duration,
        uint256 minFillAmount,
        bool allowPartialFill
    ) internal returns (uint256) {
        uint256 orderId = nextOrderId++;
        uint256 expiresAt = duration > 0
            ? block.timestamp + duration
            : block.timestamp + defaultOrderDuration; // Fixed: removed .add()

        orders[orderId] = Order({
            orderId: orderId,
            orderType: orderType,
            tokenAddress: tokenAddress,
            amount: amount,
            price: pricePerToken,
            filledAmount: 0,
            creator: msg.sender,
            status: OrderStatus.ACTIVE,
            createdAt: block.timestamp,
            expiresAt: expiresAt,
            minFillAmount: minFillAmount,
            allowPartialFill: allowPartialFill
        });

        userOrders[msg.sender].push(orderId);
        tokenOrders[tokenAddress].push(orderId);

        auditTrail.logTransaction(
            orderType == OrderType.BUY
                ? keccak256("BUY_ORDER_CREATED")
                : keccak256("SELL_ORDER_CREATED"),
            msg.sender,
            amount,
            abi.encodePacked(tokenAddress, pricePerToken)
        );

        emit OrderCreated(
            orderId,
            msg.sender,
            orderType,
            tokenAddress,
            amount,
            pricePerToken
        );
        return orderId;
    }

    function _createTrade(
        uint256 buyOrderId,
        uint256 sellOrderId,
        address buyer,
        address seller,
        address tokenAddress,
        uint256 amount,
        uint256 price,
        uint256 totalCost,
        uint256 buyerFee,
        uint256 sellerFee
    ) internal returns (uint256) {
        uint256 tradeId = nextTradeId++;

        trades[tradeId] = Trade({
            tradeId: tradeId,
            buyOrderId: buyOrderId,
            sellOrderId: sellOrderId,
            buyer: buyer,
            seller: seller,
            tokenAddress: tokenAddress,
            amount: amount,
            price: price,
            timestamp: block.timestamp,
            totalCost: totalCost,
            buyerFee: buyerFee,
            sellerFee: sellerFee
        });

        return tradeId;
    }

    function _updateMarketData(
        address tokenAddress,
        uint256 price,
        uint256 volume
    ) internal {
        MarketData storage market = marketData[tokenAddress];

        // Initialize if first trade
        if (market.lastTradeTime == 0) {
            market.tokenAddress = tokenAddress;
            market.highPrice24h = price;
            market.lowPrice24h = price;
        }

        market.lastPrice = price;
        market.lastTradeTime = block.timestamp;
        market.totalVolume = market.totalVolume + volume; // Fixed: removed .add()
        market.totalTrades = market.totalTrades + 1; // Fixed: removed .add()

        // Update 24h data
        if (block.timestamp <= market.lastTradeTime + 24 hours) {
            // Fixed: removed .add()
            market.volume24h = market.volume24h + volume; // Fixed: removed .add()
            if (price > market.highPrice24h) market.highPrice24h = price;
            if (price < market.lowPrice24h) market.lowPrice24h = price;
        } else {
            // Reset 24h data
            market.volume24h = volume;
            market.highPrice24h = price;
            market.lowPrice24h = price;
        }

        emit MarketDataUpdated(
            tokenAddress,
            price,
            market.volume24h,
            market.totalVolume
        );
    }

    function _addToOrderBook(
        address tokenAddress,
        uint256 orderId,
        OrderType orderType,
        uint256 price
    ) internal {
        OrderBook storage book = orderBooks[tokenAddress];

        if (orderType == OrderType.BUY) {
            // Insert in buy orders (sorted by price, highest first)
            _insertSorted(book.buyOrders, orderId, price, true);
        } else {
            // Insert in sell orders (sorted by price, lowest first)
            _insertSorted(book.sellOrders, orderId, price, false);
        }
    }

    function _removeFromOrderBook(
        address tokenAddress,
        uint256 orderId,
        OrderType orderType
    ) internal {
        OrderBook storage book = orderBooks[tokenAddress];

        if (orderType == OrderType.BUY) {
            _removeFromArray(book.buyOrders, orderId);
        } else {
            _removeFromArray(book.sellOrders, orderId);
        }
    }

    function _insertSorted(
        uint256[] storage orderArray,
        uint256 orderId,
        uint256 price,
        bool descending
    ) internal {
        orderArray.push(orderId);

        // Simple insertion sort for small arrays
        for (uint256 i = orderArray.length - 1; i > 0; i--) {
            uint256 currentPrice = orders[orderArray[i]].price;
            uint256 prevPrice = orders[orderArray[i - 1]].price;

            bool shouldSwap = descending
                ? currentPrice > prevPrice
                : currentPrice < prevPrice;

            if (shouldSwap) {
                uint256 temp = orderArray[i];
                orderArray[i] = orderArray[i - 1];
                orderArray[i - 1] = temp;
            } else {
                break;
            }
        }
    }

    function _removeFromArray(
        uint256[] storage array,
        uint256 orderId
    ) internal {
        for (uint256 i = 0; i < array.length; i++) {
            if (array[i] == orderId) {
                array[i] = array[array.length - 1];
                array.pop();
                break;
            }
        }
    }

    function getEscrowBalance(
        address user,
        address tokenAddress
    ) external view returns (uint256) {
        return escrowBalances[user][tokenAddress];
    }

    function getActiveOrders(
        address tokenAddress
    ) external view returns (uint256[] memory) {
        uint256[] memory tokenOrderList = tokenOrders[tokenAddress];
        uint256 activeCount = 0;

        // Count active orders
        for (uint256 i = 0; i < tokenOrderList.length; i++) {
            Order memory order = orders[tokenOrderList[i]];
            if (
                order.status == OrderStatus.ACTIVE ||
                order.status == OrderStatus.PARTIALLY_FILLED
            ) {
                activeCount++;
            }
        }

        // Create array of active orders
        uint256[] memory activeOrders = new uint256[](activeCount);
        uint256 index = 0;

        for (uint256 i = 0; i < tokenOrderList.length; i++) {
            Order memory order = orders[tokenOrderList[i]];
            if (
                order.status == OrderStatus.ACTIVE ||
                order.status == OrderStatus.PARTIALLY_FILLED
            ) {
                activeOrders[index] = tokenOrderList[i];
                index++;
            }
        }

        return activeOrders;
    }

    function getBestPrices(
        address tokenAddress
    ) external view returns (uint256 bestBuyPrice, uint256 bestSellPrice) {
        OrderBook storage book = orderBooks[tokenAddress];

        if (book.buyOrders.length > 0) {
            bestBuyPrice = orders[book.buyOrders[0]].price;
        }

        if (book.sellOrders.length > 0) {
            bestSellPrice = orders[book.sellOrders[0]].price;
        }
    }

    function getOrderStatistics(
        address tokenAddress
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
        uint256[] memory tokenOrderList = tokenOrders[tokenAddress];
        uint256 active = 0;

        for (uint256 i = 0; i < tokenOrderList.length; i++) {
            Order memory order = orders[tokenOrderList[i]];
            if (
                order.status == OrderStatus.ACTIVE ||
                order.status == OrderStatus.PARTIALLY_FILLED
            ) {
                active++;
            }
        }

        MarketData memory market = marketData[tokenAddress];

        return (
            tokenOrderList.length,
            active,
            market.totalTrades,
            market.totalVolume
        );
    }

    function getUserTrades(
        address user,
        uint256 limit
    ) external view returns (uint256[] memory) {
        uint256 count = 0;

        // Count user's trades
        for (uint256 i = 1; i < nextTradeId && count < limit; i++) {
            Trade memory trade = trades[i];
            if (trade.buyer == user || trade.seller == user) {
                count++;
            }
        }

        // Create array of user's trades
        uint256[] memory userTrades = new uint256[](count);
        uint256 index = 0;

        for (uint256 i = nextTradeId - 1; i >= 1 && index < limit; i--) {
            Trade memory trade = trades[i];
            if (trade.buyer == user || trade.seller == user) {
                userTrades[index] = i;
                index++;
            }
        }

        return userTrades;
    }

    function getRecentTrades(
        address tokenAddress,
        uint256 limit
    ) external view returns (uint256[] memory) {
        uint256 count = 0;

        // Count recent trades for token
        for (uint256 i = 1; i < nextTradeId && count < limit; i++) {
            Trade memory trade = trades[i];
            if (trade.tokenAddress == tokenAddress) {
                count++;
            }
        }

        // Create array of recent trades
        uint256[] memory recentTrades = new uint256[](
            count > limit ? limit : count
        );
        uint256 index = 0;

        // Get most recent trades first
        for (uint256 i = nextTradeId - 1; i >= 1 && index < limit; i--) {
            Trade memory trade = trades[i];
            if (trade.tokenAddress == tokenAddress) {
                recentTrades[index] = i;
                index++;
            }
        }

        return recentTrades;
    }

    function canFillOrder(
        uint256 orderId,
        address filler,
        uint256 amount
    ) external view returns (bool, string memory) {
        Order memory order = orders[orderId];

        if (order.status != OrderStatus.ACTIVE) {
            return (false, "Order not active");
        }

        if (block.timestamp > order.expiresAt) {
            return (false, "Order expired");
        }

        if (order.creator == filler) {
            return (false, "Cannot fill own order");
        }

        if (!complianceManager.canTransfer(order.creator, filler, amount)) {
            return (false, "Transfer not compliant");
        }

        uint256 availableAmount = order.amount - order.filledAmount; // Fixed: removed .sub()
        if (amount > availableAmount) {
            return (false, "Amount exceeds available");
        }

        if (!order.allowPartialFill && amount != availableAmount) {
            return (false, "Partial fill not allowed");
        }

        if (amount < order.minFillAmount) {
            return (false, "Below minimum fill amount");
        }

        return (true, "");
    }

    function estimateFillCost(
        uint256 orderId,
        uint256 amount
    )
        external
        view
        returns (uint256 totalCost, uint256 tradingFee, uint256 totalRequired)
    {
        Order memory order = orders[orderId];
        totalCost = amount * order.price; // Fixed: removed .mul()
        tradingFee = feeManager.calculateFees(
            totalCost,
            feeManager.TRADING_FEE()
        );

        if (order.orderType == OrderType.SELL) {
            // Buyer pays trading fee
            totalRequired = totalCost + (tradingFee / 2); // Fixed: removed .add() and .div()
        } else {
            // Seller receives less due to fee
            totalRequired = 0; // Only tokens needed, not ETH
        }

        return (totalCost, tradingFee, totalRequired);
    }

    // Admin functions
    function setOrderLimits(
        uint256 _minOrderValue,
        uint256 _maxOrdersPerUser,
        uint256 _defaultOrderDuration,
        uint256 _maxOrderDuration
    ) external onlyRole(MARKETPLACE_ADMIN_ROLE) {
        require(_minOrderValue > 0, "Invalid min order value");
        require(
            _maxOrdersPerUser > 0 && _maxOrdersPerUser <= 1000,
            "Invalid max orders"
        );
        require(
            _defaultOrderDuration >= 1 hours &&
                _defaultOrderDuration <= 365 days,
            "Invalid default duration"
        );
        require(
            _maxOrderDuration >= _defaultOrderDuration &&
                _maxOrderDuration <= 365 days,
            "Invalid max duration"
        );

        minOrderValue = _minOrderValue;
        maxOrdersPerUser = _maxOrdersPerUser;
        defaultOrderDuration = _defaultOrderDuration;
        maxOrderDuration = _maxOrderDuration;
    }

    function expireOrder(uint256 orderId) external onlyRole(OPERATOR_ROLE) {
        Order storage order = orders[orderId];
        require(block.timestamp > order.expiresAt, "Order not expired");
        require(
            order.status == OrderStatus.ACTIVE ||
                order.status == OrderStatus.PARTIALLY_FILLED,
            "Invalid status"
        );

        order.status = OrderStatus.EXPIRED;

        // Refund remaining amount
        uint256 remainingAmount = order.amount - order.filledAmount; // Fixed: removed .sub()

        if (order.orderType == OrderType.SELL && remainingAmount > 0) {
            IERC20(order.tokenAddress).transfer(order.creator, remainingAmount);
            escrowBalances[order.creator][order.tokenAddress] =
                escrowBalances[order.creator][order.tokenAddress] -
                remainingAmount; // Fixed: removed .sub()
        } else if (order.orderType == OrderType.BUY && remainingAmount > 0) {
            uint256 refundAmount = remainingAmount * order.price; // Fixed: removed .mul()
            escrowBalances[order.creator][address(0)] =
                escrowBalances[order.creator][address(0)] -
                refundAmount; // Fixed: removed .sub()
            payable(order.creator).transfer(refundAmount);
        }

        // Remove from order book
        _removeFromOrderBook(order.tokenAddress, orderId, order.orderType);
    }

    function batchExpireOrders(
        uint256[] calldata orderIds
    ) external onlyRole(OPERATOR_ROLE) {
        for (uint256 i = 0; i < orderIds.length; i++) {
            if (block.timestamp > orders[orderIds[i]].expiresAt) {
                this.expireOrder(orderIds[i]);
            }
        }
    }

    function pause() external onlyRole(MARKETPLACE_ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(MARKETPLACE_ADMIN_ROLE) {
        _unpause();
    }

    function emergencyWithdraw(
        address tokenAddress
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (tokenAddress == address(0)) {
            // Withdraw ETH
            uint256 balance = address(this).balance;
            require(balance > 0, "No ETH to withdraw");
            payable(msg.sender).transfer(balance);
        } else {
            // Withdraw ERC20 tokens
            IERC20 token = IERC20(tokenAddress);
            uint256 balance = token.balanceOf(address(this));
            require(balance > 0, "No tokens to withdraw");
            token.transfer(msg.sender, balance);
        }
    }

    receive() external payable {
        // Allow contract to receive ETH
    }

    function getOrder(uint256 orderId) external view returns (Order memory) {
        return orders[orderId];
    }

    function getTrade(uint256 tradeId) external view returns (Trade memory) {
        return trades[tradeId];
    }

    function getUserOrders(
        address user
    ) external view returns (uint256[] memory) {
        return userOrders[user];
    }

    function getTokenOrders(
        address tokenAddress
    ) external view returns (uint256[] memory) {
        return tokenOrders[tokenAddress];
    }

    function getOrderBook(
        address tokenAddress,
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
        OrderBook storage book = orderBooks[tokenAddress];

        uint256 buyDepth = depth > book.buyOrders.length
            ? book.buyOrders.length
            : depth;
        uint256 sellDepth = depth > book.sellOrders.length
            ? book.sellOrders.length
            : depth;

        buyOrderIds = new uint256[](buyDepth);
        buyPrices = new uint256[](buyDepth);
        buyAmounts = new uint256[](buyDepth);

        for (uint256 i = 0; i < buyDepth; i++) {
            uint256 orderId = book.buyOrders[i];
            Order memory order = orders[orderId];
            buyOrderIds[i] = orderId;
            buyPrices[i] = order.price;
            buyAmounts[i] = order.amount - order.filledAmount; // Fixed: removed .sub()
        }

        sellOrderIds = new uint256[](sellDepth);
        sellPrices = new uint256[](sellDepth);
        sellAmounts = new uint256[](sellDepth);

        for (uint256 i = 0; i < sellDepth; i++) {
            uint256 orderId = book.sellOrders[i];
            Order memory order = orders[orderId];
            sellOrderIds[i] = orderId;
            sellPrices[i] = order.price;
            sellAmounts[i] = order.amount - order.filledAmount; // Fixed: removed .sub()
        }
    }

    function getMarketData(
        address tokenAddress
    ) external view returns (MarketData memory) {
        return marketData[tokenAddress];
    }
}
