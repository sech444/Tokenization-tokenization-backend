// contracts/interfaces/core/ITokenFactory.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

// Import the ITokenRegistry to use its TokenType enum
import "./ITokenRegistry.sol";

/**
 * @title ITokenFactory
 * @dev The complete and correct interface for the TokenFactory contract.
 */
interface ITokenFactory {
    function initialize(
        address _tokenImplementation,
        address _complianceManager,
        address _auditTrail,
        address _feeManager,
        address _tokenRegistry,
        address admin
    ) external;

    function createToken(
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        ITokenRegistry.TokenType tokenType,
        string calldata metadataURI
    ) external payable returns (address);

    // ===== THIS IS THE FIX: The 'payable' keyword must be here =====
    function createTokenBatch(
        string[] calldata names,
        string[] calldata symbols,
        uint256[] calldata totalSupplies,
        uint8[] calldata decimalsArray,
        ITokenRegistry.TokenType[] calldata tokenTypes,
        string[] calldata metadataURIs
    ) external payable returns (address[] memory);
}