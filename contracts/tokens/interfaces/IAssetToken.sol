// ============================================================================

// contracts/tokens/interfaces/IAssetToken.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IAssetToken is IERC20 {
    function mint(address to, uint256 amount) external;
    function burn(uint256 amount) external;
    function distributeDividend() external payable;
    function claimDividend(uint256 dividendId) external;
    function freezeAccount(address account, bool frozen) external;
    
    function getMetadata() external view returns (
        string memory description,
        string memory imageURI,
        string memory documentURI,
        uint256 assetValue,
        string memory jurisdiction
    );
    
    function canTransfer(address from, address to, uint256 amount) external view returns (bool);
    function getUnclaimedDividends(address holder) external view returns (uint256);
}


