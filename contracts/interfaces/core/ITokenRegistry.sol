// contracts/interfaces/core/ITokenRegistry.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ITokenRegistry
 * @dev Interface for the TokenRegistry contract.
 */
interface ITokenRegistry {
    /**
     * @dev Enum for classifying token types.
     */
    enum TokenType { ASSET, UTILITY, SECURITY, GOVERNANCE }

    /**
     * @dev Initializes the contract.
     * @param admin The address of the admin.
     */
    function initialize(address admin) external;

    /**
     * @dev Registers a new token in the platform registry.
     * @param tokenAddress The address of the newly created token.
     * @param name The name of the token.
     * @param symbol The symbol of the token.
     * @param totalSupply The total supply of the token.
     * @param decimals The decimal precision of the token.
     * @param tokenType The type of the token from the TokenType enum.
     * @param creator The address of the token's creator.
     * @param metadataURI A URI pointing to the token's metadata.
     */
    function registerToken(
        address tokenAddress,
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        TokenType tokenType,
        address creator,
        string calldata metadataURI
    ) external;
}