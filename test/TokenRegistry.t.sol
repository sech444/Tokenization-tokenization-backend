// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "../contracts/core/TokenRegistry.sol";
import "@openzeppelin/contracts/access/IAccessControl.sol";


contract TokenRegistryTest is Test {
    TokenRegistry registry;

    address admin = address(0xABCD);
    address factory = address(0xFACA);
    address user = address(0xBEEF);
    address fakeToken = address(0xCAFE);

    function setUp() public {
        vm.startPrank(admin);
        registry = new TokenRegistry();
        registry.initialize(admin);

        // Grant FACTORY_ROLE to factory
        registry.grantRole(registry.FACTORY_ROLE(), factory);
        vm.stopPrank();
    }

    // ✅ Successful deployment: admin has DEFAULT_ADMIN_ROLE
    function test_SuccessWhen_DeployedAndInitialized() public view {
        assertTrue(registry.hasRole(registry.DEFAULT_ADMIN_ROLE(), admin));
    }

    // ✅ Factory can register token
    function test_SuccessWhen_FactoryRegistersToken() public {
        vm.startPrank(factory);
        registry.registerToken(
            fakeToken,
            "Fake",
            "FAK",
            1000,
            18,
            TokenRegistry.TokenType.UTILITY,
            user,
            "ipfs://fake"
        );
        vm.stopPrank();

        TokenRegistry.TokenInfo memory info = registry.getTokenInfo(fakeToken);
        assertEq(info.name, "Fake");
        assertEq(info.symbol, "FAK");
        assertTrue(info.isActive);
    }

    // ✅ Unauthorized user trying to register token should revert
    function test_RevertWhen_UnauthorizedRegister() public {
        vm.startPrank(user);

        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                user,
                registry.FACTORY_ROLE()
            )
        );

        registry.registerToken(
            fakeToken,
            "Fake",
            "FAK",
            1000,
            18,
            TokenRegistry.TokenType.UTILITY,
            user,
            "ipfs://fake"
        );

        vm.stopPrank();
    }

    // ✅ Updating metadata by admin should succeed
    function test_SuccessWhen_UpdateTokenMetadata() public {
        // factory registers first
        vm.startPrank(factory);
        registry.registerToken(
            fakeToken,
            "Fake",
            "FAK",
            1000,
            18,
            TokenRegistry.TokenType.UTILITY,
            user,
            "ipfs://fake"
        );
        vm.stopPrank();

        // admin updates metadata
        vm.startPrank(admin);
        registry.updateTokenMetadata(fakeToken, "ipfs://updated");
        vm.stopPrank();

        TokenRegistry.TokenInfo memory info = registry.getTokenInfo(fakeToken);
        assertEq(info.metadataURI, "ipfs://updated");
    }
}
