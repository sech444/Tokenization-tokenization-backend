// ============================================================================

// contracts/interfaces/core/ITokenFactory.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface ITokenFactory {
    enum TokenType { ASSET, UTILITY, SECURITY, GOVERNANCE }
    
    function createToken(
        string calldata name,
        string calldata symbol,
        uint256 totalSupply,
        uint8 decimals,
        TokenType tokenType,
        string calldata metadataURI
    ) external payable returns (address);
    
    function getTokenInfo(address tokenAddress) external view returns (
        address tokenAddr,
        string memory name,
        string memory symbol,
        uint256 totalSupply,
        uint8 decimals,
        TokenType tokenType,
        address creator,
        uint256 createdAt,
        bool isActive,
        bool isCompliant,
        string memory metadataURI
    );
    
    function isTokenValid(address tokenAddress) external view returns (bool);
}