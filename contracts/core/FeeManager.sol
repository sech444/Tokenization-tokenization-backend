// contracts/core/FeeManager.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title FeeManager
 * @dev Manages platform fees, distribution, and treasury operations
 * @notice Handles configurable fee structures with multi-recipient distribution
 */
contract FeeManager is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable
{
    // ⛔ SafeMath removed – Solidity 0.8+ has safe arithmetic

    bytes32 public constant FEE_ADMIN_ROLE = keccak256("FEE_ADMIN_ROLE");
    bytes32 public constant TREASURY_ROLE = keccak256("TREASURY_ROLE");

    struct FeeStructure {
        uint256 percentage; // In basis points (100 = 1%)
        uint256 flatFee;
        uint256 minFee;
        uint256 maxFee;
        bool isActive;
        bool isPercentageOnly;
    }

    struct FeeRecipient {
        address recipient;
        uint256 share; // In basis points
        bool isActive;
        string description;
    }

    struct FeeCollection {
        uint256 totalCollected;
        uint256 totalDistributed;
        uint256 pendingDistribution;
        uint256 lastDistribution;
    }

    mapping(bytes32 => FeeStructure) public feeStructures;
    mapping(bytes32 => FeeRecipient[]) public feeRecipients;
    mapping(bytes32 => FeeCollection) public feeCollections;
    mapping(address => uint256) public collectedFees;
    mapping(address => mapping(bytes32 => uint256)) public userFeeContributions;

    address public treasury;
    uint256 public constant BASIS_POINTS = 10000;
    uint256 public constant MAX_RECIPIENTS = 10;
    uint256 public totalFeesCollected;
    uint256 public totalFeesDistributed;

    // Fee type constants
    bytes32 public constant TOKEN_CREATION_FEE = keccak256("TOKEN_CREATION");
    bytes32 public constant TOKENIZATION_FEE = keccak256("TOKENIZATION");
    bytes32 public constant TRADING_FEE = keccak256("TRADING");
    bytes32 public constant WITHDRAWAL_FEE = keccak256("WITHDRAWAL");
    bytes32 public constant LISTING_FEE = keccak256("LISTING");

    event FeeStructureUpdated(
        bytes32 indexed feeType,
        uint256 percentage,
        uint256 flatFee
    );
    event FeesCollected(
        bytes32 indexed feeType,
        address indexed payer,
        uint256 amount
    );
    event FeesDistributed(bytes32 indexed feeType, uint256 totalAmount);
    event FeeRecipientAdded(
        bytes32 indexed feeType,
        address recipient,
        uint256 share
    );
    event FeeRecipientRemoved(bytes32 indexed feeType, address recipient);
    event FeeRecipientUpdated(
        bytes32 indexed feeType,
        address recipient,
        uint256 newShare
    );
    event TreasuryUpdated(address oldTreasury, address newTreasury);
    event EmergencyWithdrawal(address indexed recipient, uint256 amount);

    function initialize(address admin, address _treasury) public initializer {
        __AccessControl_init();
        __Pausable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(FEE_ADMIN_ROLE, admin);
        _grantRole(TREASURY_ROLE, admin);

        treasury = _treasury;

        // Initialize default fee structures
        _initializeDefaultFees();
    }

    function setFeeStructure(
        bytes32 feeType,
        uint256 percentage,
        uint256 flatFee,
        uint256 minFee,
        uint256 maxFee,
        bool isPercentageOnly
    ) external onlyRole(FEE_ADMIN_ROLE) {
        require(percentage <= BASIS_POINTS, "Percentage exceeds 100%");
        require(minFee <= maxFee || maxFee == 0, "Invalid fee range");

        feeStructures[feeType] = FeeStructure({
            percentage: percentage,
            flatFee: flatFee,
            minFee: minFee,
            maxFee: maxFee,
            isActive: true,
            isPercentageOnly: isPercentageOnly
        });

        emit FeeStructureUpdated(feeType, percentage, flatFee);
    }

    function addFeeRecipient(
        bytes32 feeType,
        address recipient,
        uint256 share,
        string calldata description
    ) external onlyRole(FEE_ADMIN_ROLE) {
        require(recipient != address(0), "Invalid recipient");
        require(share > 0 && share <= BASIS_POINTS, "Invalid share");
        require(
            feeRecipients[feeType].length < MAX_RECIPIENTS,
            "Too many recipients"
        );

        // Check total shares don't exceed 100%
        uint256 totalShares = share;
        for (uint256 i = 0; i < feeRecipients[feeType].length; i++) {
            if (feeRecipients[feeType][i].isActive) {
                totalShares += feeRecipients[feeType][i].share;
            }
        }
        require(totalShares <= BASIS_POINTS, "Total shares exceed 100%");

        feeRecipients[feeType].push(
            FeeRecipient({
                recipient: recipient,
                share: share,
                isActive: true,
                description: description
            })
        );

        emit FeeRecipientAdded(feeType, recipient, share);
    }

    function updateFeeRecipient(
        bytes32 feeType,
        uint256 index,
        uint256 newShare
    ) external onlyRole(FEE_ADMIN_ROLE) {
        require(index < feeRecipients[feeType].length, "Invalid index");
        require(newShare > 0 && newShare <= BASIS_POINTS, "Invalid share");

        FeeRecipient storage recipient = feeRecipients[feeType][index];
        require(recipient.isActive, "Recipient not active");

        // Check total shares don't exceed 100%
        uint256 totalShares = newShare;
        for (uint256 i = 0; i < feeRecipients[feeType].length; i++) {
            if (i != index && feeRecipients[feeType][i].isActive) {
                totalShares += feeRecipients[feeType][i].share;
            }
        }
        require(totalShares <= BASIS_POINTS, "Total shares exceed 100%");

        recipient.share = newShare;
        emit FeeRecipientUpdated(feeType, recipient.recipient, newShare);
    }

    function removeFeeRecipient(
        bytes32 feeType,
        uint256 index
    ) external onlyRole(FEE_ADMIN_ROLE) {
        require(index < feeRecipients[feeType].length, "Invalid index");

        FeeRecipient storage recipient = feeRecipients[feeType][index];
        require(recipient.isActive, "Already inactive");

        recipient.isActive = false;
        emit FeeRecipientRemoved(feeType, recipient.recipient);
    }

    function calculateFees(
        uint256 amount,
        bytes32 feeType
    ) external view returns (uint256) {
        FeeStructure memory fee = feeStructures[feeType];
        if (!fee.isActive || amount == 0) return 0;

        uint256 totalFee;

        if (fee.isPercentageOnly) {
            totalFee = (amount * fee.percentage) / BASIS_POINTS;
        } else {
            uint256 percentageFee = (amount * fee.percentage) / BASIS_POINTS;
            totalFee = percentageFee + fee.flatFee;
        }

        // Apply min/max constraints
        if (totalFee < fee.minFee) totalFee = fee.minFee;
        if (fee.maxFee > 0 && totalFee > fee.maxFee) totalFee = fee.maxFee;

        return totalFee;
    }

    function collectFees(
        bytes32 feeType,
        address payer
    ) external payable whenNotPaused returns (uint256) {
        require(msg.value > 0, "No fee amount provided");

        FeeCollection storage collection = feeCollections[feeType];
        collection.totalCollected += msg.value;
        collection.pendingDistribution += msg.value;

        collectedFees[address(this)] += msg.value;
        userFeeContributions[payer][feeType] += msg.value;
        totalFeesCollected += msg.value;

        emit FeesCollected(feeType, payer, msg.value);
        return msg.value;
    }

    function distributeFees(
        bytes32 feeType
    ) external nonReentrant whenNotPaused {
        FeeCollection storage collection = feeCollections[feeType];
        uint256 amount = collection.pendingDistribution;
        require(amount > 0, "No fees to distribute");

        collection.pendingDistribution = 0;
        collection.totalDistributed += amount;
        collection.lastDistribution = block.timestamp;

        FeeRecipient[] memory recipients = feeRecipients[feeType];
        uint256 remaining = amount;
        uint256 totalActiveShares = 0;

        // Calculate total active shares
        for (uint256 i = 0; i < recipients.length; i++) {
            if (recipients[i].isActive) {
                totalActiveShares += recipients[i].share;
            }
        }

        // Distribute to recipients
        for (uint256 i = 0; i < recipients.length; i++) {
            if (recipients[i].isActive) {
                uint256 share = (amount * recipients[i].share) /
                    totalActiveShares;
                if (share > 0) {
                    collectedFees[recipients[i].recipient] += share;
                    remaining -= share;
                }
            }
        }

        // Send remaining to treasury
        if (remaining > 0) {
            collectedFees[treasury] += remaining;
        }

        totalFeesDistributed += amount;
        emit FeesDistributed(feeType, amount);
    }

    function withdrawFees() external nonReentrant {
        uint256 amount = collectedFees[msg.sender];
        require(amount > 0, "No fees to withdraw");

        collectedFees[msg.sender] = 0;
        payable(msg.sender).transfer(amount);
    }

    function withdrawFeesTo(address recipient) external nonReentrant {
        require(
            msg.sender == recipient || hasRole(TREASURY_ROLE, msg.sender),
            "Unauthorized"
        );

        uint256 amount = collectedFees[recipient];
        require(amount > 0, "No fees to withdraw");

        collectedFees[recipient] = 0;
        payable(recipient).transfer(amount);
    }

    function emergencyWithdraw() external onlyRole(TREASURY_ROLE) {
        uint256 balance = address(this).balance;
        require(balance > 0, "No funds to withdraw");

        payable(treasury).transfer(balance);
        emit EmergencyWithdrawal(treasury, balance);
    }

    function updateTreasury(
        address newTreasury
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newTreasury != address(0), "Invalid treasury address");

        address oldTreasury = treasury;
        treasury = newTreasury;

        _revokeRole(TREASURY_ROLE, oldTreasury);
        _grantRole(TREASURY_ROLE, newTreasury);

        emit TreasuryUpdated(oldTreasury, newTreasury);
    }

    function _initializeDefaultFees() internal {
        // Token creation fee: 0.1% + 0.01 ETH flat fee
        feeStructures[TOKEN_CREATION_FEE] = FeeStructure({
            percentage: 10,
            flatFee: 0.01 ether,
            minFee: 0.01 ether,
            maxFee: 1 ether,
            isActive: true,
            isPercentageOnly: false
        });

        // Tokenization fee: 0.5%
        feeStructures[TOKENIZATION_FEE] = FeeStructure({
            percentage: 50,
            flatFee: 0,
            minFee: 0.1 ether,
            maxFee: 10 ether,
            isActive: true,
            isPercentageOnly: true
        });

        // Trading fee: 0.25%
        feeStructures[TRADING_FEE] = FeeStructure({
            percentage: 25,
            flatFee: 0,
            minFee: 0.001 ether,
            maxFee: 0,
            isActive: true,
            isPercentageOnly: true
        });

        // Withdrawal fee: 0.1%
        feeStructures[WITHDRAWAL_FEE] = FeeStructure({
            percentage: 10,
            flatFee: 0,
            minFee: 0.001 ether,
            maxFee: 0.1 ether,
            isActive: true,
            isPercentageOnly: true
        });

        // Listing fee: flat 0.05 ETH
        feeStructures[LISTING_FEE] = FeeStructure({
            percentage: 0,
            flatFee: 0.05 ether,
            minFee: 0.05 ether,
            maxFee: 0.05 ether,
            isActive: true,
            isPercentageOnly: false
        });
    }

    function getFeeStructure(
        bytes32 feeType
    ) external view returns (FeeStructure memory) {
        return feeStructures[feeType];
    }

    function getFeeRecipients(
        bytes32 feeType
    ) external view returns (FeeRecipient[] memory) {
        return feeRecipients[feeType];
    }

    function getFeeCollection(
        bytes32 feeType
    ) external view returns (FeeCollection memory) {
        return feeCollections[feeType];
    }

    function getUserFeeContributions(
        address user,
        bytes32 feeType
    ) external view returns (uint256) {
        return userFeeContributions[user][feeType];
    }

    function getPendingFees(address recipient) external view returns (uint256) {
        return collectedFees[recipient];
    }

    function getTotalFeeSummary()
        external
        view
        returns (uint256 collected, uint256 distributed, uint256 pending)
    {
        return (
            totalFeesCollected,
            totalFeesDistributed,
            address(this).balance
        );
    }

    receive() external payable {
        collectedFees[address(this)] += msg.value;
        totalFeesCollected += msg.value;
    }
}
