// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import "../contracts/HybridAssetTokenizer.sol";

contract InteractHybridAssetTokenizerScript is Script {
    HybridAssetTokenizer tokenizer;

    function setUp() public {
        // Load proxy address from .env
        address proxy = vm.envAddress("HYBRID_ASSET_TOKENIZER_PROXY");
        tokenizer = HybridAssetTokenizer(proxy);
    }

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployer);

        console.log("Using account:", deployer);
        console.log("Interacting with HybridAssetTokenizer at:", address(tokenizer));

        // ✅ Example: Create full hybrid asset (ERC721 + ERC20)
        // NOTE: AssetTokenizer must already have a VERIFIED asset with this assetId
        uint256 verifiedAssetId = 2; // change to an assetId that is VERIFIED in AssetTokenizer

        address tokenAddr = tokenizer.createHybridAsset(
            verifiedAssetId,
            "Fractional Real Estate",             // ERC20 name
            "FRE",                                // ERC20 symbol
            1_000_000 ether,                      // ERC20 supply
            18,                                   // decimals
            1,                                    // tokenType (adapt to your enum)
            "ipfs://fractional-token-metadata",   // ERC20 metadata
            "ipfs://deed-metadata"                // ERC721 deed metadata
        );

        console.log("Hybrid asset created for assetId:", verifiedAssetId);
        console.log("ERC20 token deployed at:", tokenAddr);

        vm.stopBroadcast();
    }
}
