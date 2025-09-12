// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "../contracts/core/ComplianceManager.sol";

contract ComplianceManagerTest is Test {
    ComplianceManager cm;

    function setUp() public {
        cm = new ComplianceManager();
    }

    function testDeployment() public {
        assertTrue(address(cm) != address(0));
    }
}
