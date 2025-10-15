// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import "forge-std/Script.sol";
import "forge-std/console.sol";

import "openzeppelin-contracts/utils/Strings.sol";

import "../src/amb/SourceAMB.sol";
import "../src/amb/TargetAMB.sol";
import "../test/counter/Counter.sol";
import "../test/amb/LightClientMock.sol";

// The reason we can't simply deploy these contracts to Anvil, is to test the storage proofs
// against the light client, we need to deploy the contracts to a real chain where we can use
// the eth_getProof RPC (that is currently unsupported on Anvil).
contract Deploy is Script {

    function stringToUint(string memory s) public returns (uint result) {
        bytes memory b = bytes(s);
        uint i;
        result = 0;
        for (i = 0; i < b.length; i++) {
            uint c = uint(uint8(b[i]));
            if (c >= 48 && c <= 57) {
                result = result * 10 + (c - 48);
            }
        }
    }

	function run() external {
        bool USE_CREATE_2 = vm.envBool("USE_CREATE_2");
        bytes32 SALT;
        if (USE_CREATE_2) {
            SALT = vm.envBytes32("SALT");
        }
        bool USE_MOCK_LC = vm.envBool("USE_MOCK_LC");
        string memory SOURCE_CHAIN_ID = vm.envString("SOURCE_CHAIN_ID");
        string[] memory DEST_CHAIN_IDS = vm.envString("DEST_CHAIN_IDS", ",");

        string memory source_rpc = vm.envString(string.concat("RPC_", SOURCE_CHAIN_ID));
        uint256 sourceForkId = vm.createFork(source_rpc);
        vm.selectFork(sourceForkId); 

        vm.startBroadcast();
        SourceAMB sourceAMB = new SourceAMB();
        Counter sendingCounter = new Counter(sourceAMB, address(0), address(0));
        vm.stopBroadcast();

        address[] memory counterAddress = new address[](DEST_CHAIN_IDS.length);

        for (uint i = 0; i < DEST_CHAIN_IDS.length; i++) {
            string memory rpc = vm.envString(string.concat("RPC_", DEST_CHAIN_IDS[i]));
            uint256 forkId = vm.createFork(rpc);
            vm.selectFork(forkId);
            vm.startBroadcast();
            address lc;
            if (USE_MOCK_LC) {
                LightClientMock lightClient = new LightClientMock{salt: SALT}();
                lc = address(lightClient);
            } else {
                lc = vm.envAddress(string.concat("LightClient_ADDRESS_", DEST_CHAIN_IDS[i]));
            }

            TargetAMB targetAMB;
            Counter counter;

            if (USE_CREATE_2) {
                targetAMB = new TargetAMB{salt: SALT}(lc, address(sourceAMB));
                counter = new Counter{salt: SALT}(SourceAMB(address(0)), address(sendingCounter), address(targetAMB));
            } else {
                targetAMB = new TargetAMB(lc, address(sourceAMB));
                counter = new Counter(SourceAMB(address(0)), address(sendingCounter), address(targetAMB)); 
            }

            vm.stopBroadcast();
            counterAddress[i] = address(counter);
        } 

        vm.selectFork(sourceForkId);
        vm.startBroadcast();
        for (uint i = 0; i < DEST_CHAIN_IDS.length; i++) {
            uint16 destChainId = uint16(stringToUint(DEST_CHAIN_IDS[i]));
            sendingCounter.setOtherSideCounterMap(destChainId, counterAddress[i]);
            sendingCounter.increment(destChainId);
        }
        vm.stopBroadcast();
	}
}