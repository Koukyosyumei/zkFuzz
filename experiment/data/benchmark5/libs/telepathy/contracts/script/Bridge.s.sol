// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import "forge-std/Script.sol";

import "openzeppelin-contracts/utils/Strings.sol";

import "src/bridge/Bridge.sol";
import "src/bridge/Tokens.sol";

contract Deploy is Script {
	function run() external {
        bytes32 SALT = vm.envBytes32("SALT");

		string memory SOURCE_CHAIN_ID = vm.envString("SOURCE_CHAIN_ID");
        string memory source_rpc = vm.envString(string.concat("RPC_", SOURCE_CHAIN_ID));
        uint256 sourceForkId = vm.createFork(source_rpc);
        vm.selectFork(sourceForkId); 

		vm.startBroadcast();
		address sourceAMB = vm.envAddress(string.concat("SourceAMB_", SOURCE_CHAIN_ID));
		// TODO: be careful here if using CREATE2, since we have onlyOwner in the contract
		// to take out fees and set token mappings
		address myRandomAddress = 0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496;
		InfiniteMintSuccincts sourceToken = new InfiniteMintSuccincts(0, myRandomAddress);
		Deposit deposit = new Deposit(sourceAMB, address(sourceToken));
		vm.stopBroadcast();

		address withdrawAddress;
        string[] memory DEST_CHAIN_IDS = vm.envString("DEST_CHAIN_IDS", ",");
        for (uint i = 0; i < DEST_CHAIN_IDS.length; i++) {
            string memory rpc = vm.envString(string.concat("RPC_", DEST_CHAIN_IDS[i]));
            uint256 forkId = vm.createFork(rpc);
            vm.selectFork(forkId);
            vm.startBroadcast();
			address targetAMB = vm.envAddress(string.concat("TargetAMB_", DEST_CHAIN_IDS[i]));
			// NOTE: We want to use CREATE2 here to keep the withdrawal addresses the same
			// This is predicated on the TargetAMB contract being deployed with CREATE2
			// so those addresses are the same (which requires the LightClient to also be deployed with CREATE2)
			Withdraw withdraw = new Withdraw{salt: SALT}(targetAMB, address(deposit));
			if (i == 0) {
				withdrawAddress = address(withdraw);
			} else {
				require(withdrawAddress == address(withdraw), "Withdraw address mismatch");
			}
			vm.stopBroadcast();
		}

		vm.selectFork(sourceForkId); 
		vm.startBroadcast();
		deposit.setWithdraw(withdrawAddress);
		vm.stopBroadcast();

	}
}
