// tokenization-backend/contracts/core/TokenFactory.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/proxy/Clones.sol";

import "../tokens/AssetToken.sol";
import "../interfaces/core/IComplianceManager.sol";
import "../interfaces/core/IAuditTrail.sol";
import "../interfaces/core/IFeeManager.sol";
import "../interfaces/core/ITokenRegistry.sol";
import "../interfaces/core/ITokenFactory.sol";

contract TokenFactory is Initializable, AccessControlUpgradeable, ITokenFactory {
    using Clones for address;

    bytes32 public constant TOKEN_CREATOR_ROLE = keccak256("TOKEN_CREATOR_ROLE");
    bytes32 public constant SYSTEM_ROLE = keccak256("SYSTEM_ROLE");

    address public tokenImplementation;
    IComplianceManager public complianceManager;
    IAuditTrail public auditTrail;
    IFeeManager public feeManager;
    ITokenRegistry public tokenRegistry;

    event TokenCreated(address indexed tokenAddress, address indexed creator, string name, string symbol, uint256 totalSupply);
    event ImplementationUpdated(address oldImpl, address newImpl);
    event TokenRegistryUpdated(address oldRegistry, address newRegistry);

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _tokenImplementation,
        address _complianceManager,
        address _auditTrail,
        address _feeManager,
        address _tokenRegistry,
        address admin
    ) public override initializer {
        require(_tokenImplementation != address(0), "impl required");
        require(_tokenRegistry != address(0), "registry required");

        __AccessControl_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(TOKEN_CREATOR_ROLE, admin);
        _grantRole(SYSTEM_ROLE, admin);

        tokenImplementation = _tokenImplementation;
        complianceManager = IComplianceManager(_complianceManager);
        auditTrail = IAuditTrail(_auditTrail);
        feeManager = IFeeManager(_feeManager);
        tokenRegistry = ITokenRegistry(_tokenRegistry);
    }

    function createToken(
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        ITokenRegistry.TokenType tokenType,
        string calldata metadataURI
    ) external override payable returns (address) {
        require(
            hasRole(TOKEN_CREATOR_ROLE, msg.sender) || complianceManager.isKYCVerified(msg.sender),
            "Creator role or KYC required"
        );
        require(totalSupply > 0, "totalSupply>0 required");
        require(decimals <= 18, "decimals too high");

        uint256 fee = 0;
        if (address(feeManager) != address(0)) {
            // Placeholder for fee logic
        }

        if (msg.value < fee) {
             revert("Insufficient fee");
        }
        if (fee > 0 && address(feeManager) != address(0)) {
            // Placeholder for fee collection
        }

        address tokenAddress = tokenImplementation.clone();

        AssetToken(payable(tokenAddress)).initialize(
            name,
            symbol,
            totalSupply,
            decimals,
            msg.sender,
            address(complianceManager),
            address(auditTrail)
        );

        tokenRegistry.registerToken(
            tokenAddress,
            name,
            symbol,
            totalSupply,
            decimals,
            tokenType,
            msg.sender,
            metadataURI
        );

        emit TokenCreated(tokenAddress, msg.sender, name, symbol, totalSupply);

        if (msg.value > fee) {
            payable(msg.sender).transfer(msg.value - fee);
        }

        return tokenAddress;
    }

    function createTokenBatch(
        string[] calldata names,
        string[] calldata symbols,
        uint256[] calldata totalSupplies,
        uint8[] calldata decimalsArray,
        ITokenRegistry.TokenType[] calldata tokenTypes,
        string[] calldata metadataURIs
    ) external override payable onlyRole(TOKEN_CREATOR_ROLE) returns (address[] memory) {
        require(
            names.length == symbols.length &&
            symbols.length == totalSupplies.length &&
            totalSupplies.length == decimalsArray.length &&
            decimalsArray.length == tokenTypes.length &&
            tokenTypes.length == metadataURIs.length,
            "Array length mismatch"
        );
        require(names.length <= 20, "Too many tokens");

        address[] memory created = new address[](names.length);
        uint256 totalFeeRequired = 0;

        require(msg.value >= totalFeeRequired, "Insufficient total fee");

        for (uint256 i = 0; i < names.length; i++) {
            created[i] = _createTokenInternal(
                names[i],
                symbols[i],
                totalSupplies[i],
                decimalsArray[i],
                tokenTypes[i],
                metadataURIs[i]
            );
        }

        if (totalFeeRequired > 0 && address(feeManager) != address(0)) {
            // Placeholder for fee collection
        }

        if (msg.value > totalFeeRequired) {
            payable(msg.sender).transfer(msg.value - totalFeeRequired);
        }

        return created;
    }

    function _createTokenInternal(
        string memory name,
        string memory symbol,
        uint256 totalSupply,
        uint8 decimals,
        ITokenRegistry.TokenType tokenType,
        string memory metadataURI
    ) internal returns (address) {
        address tokenAddress = tokenImplementation.clone();
        AssetToken(payable(tokenAddress)).initialize(
            name,
            symbol,
            totalSupply,
            decimals,
            msg.sender,
            address(complianceManager),
            address(auditTrail)
        );

        tokenRegistry.registerToken(
            tokenAddress,
            name,
            symbol,
            totalSupply,
            decimals,
            tokenType,
            msg.sender,
            metadataURI
        );

        emit TokenCreated(tokenAddress, msg.sender, name, symbol, totalSupply);
        return tokenAddress;
    }

    function updateImplementation(address newImpl) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newImpl != address(0), "invalid impl");
        address old = tokenImplementation;
        tokenImplementation = newImpl;
        emit ImplementationUpdated(old, newImpl);
    }

    function updateRegistry(address newRegistry) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newRegistry != address(0), "invalid registry");
        address old = address(tokenRegistry);
        tokenRegistry = ITokenRegistry(newRegistry);
        emit TokenRegistryUpdated(old, newRegistry);
    }
}