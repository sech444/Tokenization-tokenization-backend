// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import "../contracts/HybridAssetTokenizer.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";

contract DeployHybridAssetTokenizer is Script {
    function run() external {
        // Load env variables
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        address admin = vm.envOr("ADMIN_ADDRESS", deployer);

        address assetTokenizer = vm.envAddress("ASSET_TOKENIZER");
        address tokenFactory   = vm.envAddress("TOKEN_FACTORY");
        address tokenRegistry  = vm.envAddress("TOKEN_REGISTRY");

        vm.startBroadcast(deployerPrivateKey);

        console.log(" Deploying HybridAssetTokenizer...");
        console.log("Deployer:", deployer);
        console.log("Admin:", admin);

        // 1. Deploy implementation
        HybridAssetTokenizer implementation = new HybridAssetTokenizer();
        console.log("Implementation deployed at:", address(implementation));

        // 2. Encode initializer call
        bytes memory initData = abi.encodeWithSelector(
            HybridAssetTokenizer.initialize.selector,
            admin,
            assetTokenizer,
            tokenFactory,
            tokenRegistry
        );

        // 3. Deploy proxy
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        HybridAssetTokenizer deployed = HybridAssetTokenizer(address(proxy));
        console.log("Proxy deployed at:", address(proxy));

        // 4. Verify ERC721 support
        bool erc721Supported = deployed.supportsInterface(type(IERC721).interfaceId);
        console.log("ERC721 Supported:", erc721Supported);

        vm.stopBroadcast();

        // 5. Persist addresses for backend/frontend use
        string memory path = string.concat(vm.projectRoot(), "/deployments/HybridAssetTokenizer.json");
        string memory json = vm.serializeAddress("addresses", "implementation", address(implementation));
        json = vm.serializeAddress("addresses", "proxy", address(proxy));

        vm.writeJson(json, path);
        console.log(" Deployment info written to:", path);
    }
}

