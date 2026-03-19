// contracts/tokens/AssetToken.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IAuditTrail.sol";

/**
 * @title AssetToken
 * @dev Compliant ERC20 token for tokenized assets
 * @notice Implements transfer restrictions and compliance integration
 */
contract AssetToken is
    Initializable,
    ERC20Upgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    struct TokenMetadata {
        string description;
        string imageURI;
        string documentURI;
        uint256 assetValue;
        string jurisdiction;
    }

    struct TransferRestriction {
        bool isActive;
        uint256 minAmount;
        uint256 maxAmount;
        uint256 cooldownPeriod;
        mapping(address => uint256) lastTransfer;
    }

    struct Dividend {
        uint256 totalAmount;
        uint256 timestamp;
        uint256 claimedAmount;
        mapping(address => bool) claimed;
        mapping(address => uint256) entitlement;
    }

    uint8 private _decimals;
    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;

    mapping(address => bool) public transferWhitelist;
    mapping(address => bool) public frozenAccounts;
    mapping(uint256 => Dividend) public dividends;

    TokenMetadata public metadata;
    TransferRestriction public transferRestriction;

    bool public transfersEnabled = true;
    bool public dividendsEnabled = false;
    uint256 public totalDividends = 0;
    uint256 public totalDividendsClaimed = 0;
    uint256 public nextDividendId = 1;
    uint256 public minHoldingPeriod = 0;

    mapping(address => uint256) public acquisitionTime;
    mapping(address => uint256) public totalDividendsCreditedTo;

    event TransferWhitelisted(address indexed account, bool whitelisted);
    event AccountFrozen(address indexed account, bool frozen);
    event TransfersToggled(bool enabled);
    event DividendDistributed(
        uint256 indexed dividendId,
        uint256 totalAmount,
        uint256 timestamp
    );
    event DividendClaimed(
        uint256 indexed dividendId,
        address indexed holder,
        uint256 amount
    );
    event MetadataUpdated(
        string description,
        string imageURI,
        string documentURI
    );
    event TransferRestrictionUpdated(
        bool isActive,
        uint256 minAmount,
        uint256 maxAmount,
        uint256 cooldownPeriod
    );
    event ComplianceManagerUpdated(
        address indexed oldManager,
        address indexed newManager
    );
    event MinHoldingPeriodUpdated(uint256 oldPeriod, uint256 newPeriod);

    function initialize(
        string memory name,
        string memory symbol,
        uint256 totalSupply,
        uint8 decimals_,
        address owner,
        address _complianceManager,
        address _auditTrail
    ) public initializer {
        __ERC20_init(name, symbol);
        __Ownable_init(owner); // Fixed: Added owner parameter
        __Pausable_init();

        _decimals = decimals_;
        _mint(owner, totalSupply * 10 ** decimals_);

        complianceManager = IComplianceManager(_complianceManager);
        auditTrail = IAuditTrail(_auditTrail);

        acquisitionTime[owner] = block.timestamp;

        // Initialize with basic transfer restrictions
        transferRestriction.isActive = false;
        transferRestriction.minAmount = 1;
        transferRestriction.maxAmount = type(uint256).max;
        transferRestriction.cooldownPeriod = 0;
    }

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }

    function setComplianceManager(
        address _complianceManager
    ) external onlyOwner {
        require(_complianceManager != address(0), "Invalid compliance manager");

        address oldManager = address(complianceManager);
        complianceManager = IComplianceManager(_complianceManager);

        emit ComplianceManagerUpdated(oldManager, _complianceManager);
    }

    function setAuditTrail(address _auditTrail) external onlyOwner {
        require(_auditTrail != address(0), "Invalid audit trail");
        auditTrail = IAuditTrail(_auditTrail);
    }

    function mint(address to, uint256 amount) external onlyOwner {
        require(!frozenAccounts[to], "Account is frozen");

        _mint(to, amount);
        acquisitionTime[to] = block.timestamp;

        if (address(auditTrail) != address(0)) {
            auditTrail.logTransaction(
                keccak256("TOKEN_MINTED"),
                to,
                amount,
                abi.encodePacked(name(), symbol())
            );
        }
    }

    function burn(uint256 amount) external {
        require(!frozenAccounts[msg.sender], "Account is frozen");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");

        _burn(msg.sender, amount);

        if (address(auditTrail) != address(0)) {
            auditTrail.logTransaction(
                keccak256("TOKEN_BURNED"),
                msg.sender,
                amount,
                abi.encodePacked(name(), symbol())
            );
        }
    }

    function distributeDividend() external payable onlyOwner {
        require(msg.value > 0, "No dividend amount");
        require(dividendsEnabled, "Dividends not enabled");
        require(totalSupply() > 0, "No tokens in circulation");

        uint256 dividendId = nextDividendId++;
        Dividend storage dividend = dividends[dividendId];

        dividend.totalAmount = msg.value;
        dividend.timestamp = block.timestamp;
        dividend.claimedAmount = 0;

        totalDividends += msg.value;

        emit DividendDistributed(dividendId, msg.value, block.timestamp);
    }

    function claimDividend(uint256 dividendId) external {
        require(!frozenAccounts[msg.sender], "Account is frozen");
        require(dividendId < nextDividendId, "Invalid dividend ID");

        Dividend storage dividend = dividends[dividendId];
        require(!dividend.claimed[msg.sender], "Already claimed");
        require(dividend.totalAmount > 0, "No dividend available");

        uint256 balance = balanceOf(msg.sender);
        require(balance > 0, "No tokens held");

        if (minHoldingPeriod > 0) {
            require(
                block.timestamp >=
                    acquisitionTime[msg.sender] + minHoldingPeriod,
                "Minimum holding period not met"
            );
        }

        uint256 entitlement = (dividend.totalAmount * balance) / totalSupply();

        dividend.claimed[msg.sender] = true;
        dividend.entitlement[msg.sender] = entitlement;
        dividend.claimedAmount += entitlement;

        totalDividendsClaimed += entitlement;
        totalDividendsCreditedTo[msg.sender] += entitlement;

        payable(msg.sender).transfer(entitlement);

        emit DividendClaimed(dividendId, msg.sender, entitlement);
    }

    function claimAllDividends() external {
        require(!frozenAccounts[msg.sender], "Account is frozen");

        uint256 totalEntitlement = 0;
        uint256 balance = balanceOf(msg.sender);
        require(balance > 0, "No tokens held");

        for (uint256 i = 1; i < nextDividendId; i++) {
            Dividend storage dividend = dividends[i];

            if (!dividend.claimed[msg.sender] && dividend.totalAmount > 0) {
                if (
                    minHoldingPeriod == 0 ||
                    block.timestamp >=
                    acquisitionTime[msg.sender] + minHoldingPeriod
                ) {
                    uint256 entitlement = (dividend.totalAmount * balance) /
                        totalSupply();

                    dividend.claimed[msg.sender] = true;
                    dividend.entitlement[msg.sender] = entitlement;
                    dividend.claimedAmount += entitlement;

                    totalEntitlement += entitlement;

                    emit DividendClaimed(i, msg.sender, entitlement);
                }
            }
        }

        require(totalEntitlement > 0, "No dividends to claim");

        totalDividendsClaimed += totalEntitlement;
        totalDividendsCreditedTo[msg.sender] += totalEntitlement;

        payable(msg.sender).transfer(totalEntitlement);
    }

    function setTransferRestriction(
        bool isActive,
        uint256 minAmount,
        uint256 maxAmount,
        uint256 cooldownPeriod
    ) external onlyOwner {
        require(maxAmount >= minAmount, "Invalid amount range");

        transferRestriction.isActive = isActive;
        transferRestriction.minAmount = minAmount;
        transferRestriction.maxAmount = maxAmount;
        transferRestriction.cooldownPeriod = cooldownPeriod;

        emit TransferRestrictionUpdated(
            isActive,
            minAmount,
            maxAmount,
            cooldownPeriod
        );
    }

    function updateMetadata(
        string calldata description,
        string calldata imageURI,
        string calldata documentURI,
        uint256 assetValue,
        string calldata jurisdiction
    ) external onlyOwner {
        metadata.description = description;
        metadata.imageURI = imageURI;
        metadata.documentURI = documentURI;
        metadata.assetValue = assetValue;
        metadata.jurisdiction = jurisdiction;

        emit MetadataUpdated(description, imageURI, documentURI);
    }

    function setMinHoldingPeriod(uint256 period) external onlyOwner {
        uint256 oldPeriod = minHoldingPeriod;
        minHoldingPeriod = period;

        emit MinHoldingPeriodUpdated(oldPeriod, period);
    }

    function enableDividends(bool enabled) external onlyOwner {
        dividendsEnabled = enabled;
    }

    function toggleTransfers() external onlyOwner {
        transfersEnabled = !transfersEnabled;
        emit TransfersToggled(transfersEnabled);
    }

    function whitelistTransfer(
        address account,
        bool whitelisted
    ) external onlyOwner {
        transferWhitelist[account] = whitelisted;
        emit TransferWhitelisted(account, whitelisted);
    }

    function freezeAccount(address account, bool frozen) external onlyOwner {
        frozenAccounts[account] = frozen;
        emit AccountFrozen(account, frozen);
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    // Fixed: Using _update instead of deprecated _beforeTokenTransfer and _afterTokenTransfer
    function _update(
        address from,
        address to,
        uint256 amount
    ) internal virtual override {
        // Pre-transfer checks (equivalent to old _beforeTokenTransfer)
        if (from != address(0) && to != address(0)) {
            require(!paused(), "Token transfers are paused");
            require(transfersEnabled, "Transfers disabled");
            require(
                !frozenAccounts[from] && !frozenAccounts[to],
                "Account is frozen"
            );

            if (!transferWhitelist[from] && !transferWhitelist[to]) {
                if (address(complianceManager) != address(0)) {
                    require(
                        complianceManager.canTransfer(from, to, amount),
                        "Transfer not compliant"
                    );
                }

                if (transferRestriction.isActive) {
                    require(
                        amount >= transferRestriction.minAmount,
                        "Amount below minimum"
                    );
                    require(
                        amount <= transferRestriction.maxAmount,
                        "Amount above maximum"
                    );

                    if (transferRestriction.cooldownPeriod > 0) {
                        require(
                            block.timestamp >=
                                transferRestriction.lastTransfer[from] +
                                    transferRestriction.cooldownPeriod,
                            "Cooldown period not elapsed"
                        );
                    }
                }
            }
        }

        // Execute the transfer
        super._update(from, to, amount);

        // Post-transfer updates (equivalent to old _afterTokenTransfer)
        if (from != address(0) && to != address(0)) {
            if (
                transferRestriction.isActive &&
                transferRestriction.cooldownPeriod > 0
            ) {
                transferRestriction.lastTransfer[from] = block.timestamp;
            }

            if (balanceOf(to) == amount) {
                acquisitionTime[to] = block.timestamp;
            }

            if (address(auditTrail) != address(0)) {
                auditTrail.logTransaction(
                    keccak256("TOKEN_TRANSFERRED"),
                    from,
                    amount,
                    abi.encodePacked(to)
                );
            }
        }
    }

    // View functions
    function getMetadata() external view returns (TokenMetadata memory) {
        return metadata;
    }

    function getTransferRestriction()
        external
        view
        returns (
            bool isActive,
            uint256 minAmount,
            uint256 maxAmount,
            uint256 cooldownPeriod
        )
    {
        return (
            transferRestriction.isActive,
            transferRestriction.minAmount,
            transferRestriction.maxAmount,
            transferRestriction.cooldownPeriod
        );
    }

    function getLastTransferTime(
        address account
    ) external view returns (uint256) {
        return transferRestriction.lastTransfer[account];
    }

    function getRemainingCooldown(
        address account
    ) external view returns (uint256) {
        if (
            !transferRestriction.isActive ||
            transferRestriction.cooldownPeriod == 0
        ) {
            return 0;
        }

        uint256 lastTransfer = transferRestriction.lastTransfer[account];
        uint256 cooldownEnd = lastTransfer + transferRestriction.cooldownPeriod;

        if (block.timestamp >= cooldownEnd) {
            return 0;
        }

        return cooldownEnd - block.timestamp;
    }

    function getDividendInfo(
        uint256 dividendId
    )
        external
        view
        returns (
            uint256 totalAmount,
            uint256 timestamp,
            uint256 claimedAmount,
            bool userClaimed,
            uint256 userEntitlement
        )
    {
        require(dividendId < nextDividendId, "Invalid dividend ID");

        Dividend storage dividend = dividends[dividendId];

        uint256 entitlement = 0;
        if (balanceOf(msg.sender) > 0 && dividend.totalAmount > 0) {
            entitlement =
                (dividend.totalAmount * balanceOf(msg.sender)) /
                totalSupply();
        }

        return (
            dividend.totalAmount,
            dividend.timestamp,
            dividend.claimedAmount,
            dividend.claimed[msg.sender],
            entitlement
        );
    }

    function getUnclaimedDividends(
        address holder
    ) external view returns (uint256 totalUnclaimed) {
        uint256 balance = balanceOf(holder);
        if (balance == 0) return 0;

        for (uint256 i = 1; i < nextDividendId; i++) {
            Dividend storage dividend = dividends[i];

            if (!dividend.claimed[holder] && dividend.totalAmount > 0) {
                if (
                    minHoldingPeriod == 0 ||
                    block.timestamp >=
                    acquisitionTime[holder] + minHoldingPeriod
                ) {
                    uint256 entitlement = (dividend.totalAmount * balance) /
                        totalSupply();
                    totalUnclaimed += entitlement;
                }
            }
        }

        return totalUnclaimed;
    }

    function canTransfer(
        address from,
        address to,
        uint256 amount
    ) external view returns (bool) {
        if (paused() || !transfersEnabled) return false;
        if (frozenAccounts[from] || frozenAccounts[to]) return false;
        if (balanceOf(from) < amount) return false;

        if (transferWhitelist[from] || transferWhitelist[to]) return true;

        if (address(complianceManager) != address(0)) {
            if (!complianceManager.canTransfer(from, to, amount)) return false;
        }

        if (transferRestriction.isActive) {
            if (
                amount < transferRestriction.minAmount ||
                amount > transferRestriction.maxAmount
            ) {
                return false;
            }

            if (transferRestriction.cooldownPeriod > 0) {
                uint256 lastTransfer = transferRestriction.lastTransfer[from];
                if (
                    block.timestamp <
                    lastTransfer + transferRestriction.cooldownPeriod
                ) {
                    return false;
                }
            }
        }

        return true;
    }

    function getTotalDividendsSummary()
        external
        view
        returns (
            uint256 totalDistributed,
            uint256 totalClaimed,
            uint256 totalUnclaimed,
            uint256 nextDividend
        )
    {
        return (
            totalDividends,
            totalDividendsClaimed,
            totalDividends - totalDividendsClaimed,
            nextDividendId
        );
    }

    function getUserDividendSummary(
        address user
    )
        external
        view
        returns (
            uint256 totalCredited,
            uint256 totalUnclaimed,
            uint256 eligibleDividends
        )
    {
        uint256 unclaimed = this.getUnclaimedDividends(user);
        uint256 eligibleCount = 0;

        for (uint256 i = 1; i < nextDividendId; i++) {
            if (dividends[i].totalAmount > 0) {
                eligibleCount++;
            }
        }

        return (totalDividendsCreditedTo[user], unclaimed, eligibleCount);
    }

    // Emergency functions
    function emergencyWithdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No funds to withdraw");

        payable(owner()).transfer(balance);
    }

    function recoverERC20(
        address tokenAddress,
        uint256 tokenAmount
    ) external onlyOwner {
        require(tokenAddress != address(this), "Cannot recover own token");
        ERC20Upgradeable(tokenAddress).transfer(owner(), tokenAmount);
    }

    receive() external payable {
        // Allow contract to receive ETH for dividends
    }
}
