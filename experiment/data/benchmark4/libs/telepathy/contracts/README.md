### Installation of Dependencies

To install the succinctlabs dependencies, you must run `forge install succinctlabs/curve-merkle-oracle@main` (note that the tag is important here). Because there is a bug in Foundry, we then need to go to the `.gitmodules` and delete the duplicate entry and make sure the final entry looks something like this:

```
[submodule "contracts/lib/curve-merkle-oracle"]
	path = contracts/lib/curve-merkle-oracle
	url = https://github.com/succinctlabs/curve-merkle-oracle
	branch = main
```

### ABI generation

To generate ABIs that are up to date with the contracts, run the following. We also use typechain
in the SDK to manage access to all the contracts, which needs to be udpated when the contract
ABIs are updated.

```
forge build --extra-output-files abi --out abi
cd ../sdk && yarn typechain
```

Or you can just run

```
cd ../sdk && yarn abi-gen
```

### Testing deployment

For testing purposes, we use the `DeployMockCounter.s.sol` script that will deploy a Mock Light Client on every chain, as well as AMBs and a mock counter contract that can be used to test the system end-to-end.

```
cd scripts/
source ../../.env && forge script DeployMockCounter.s.sol:DeployMockContract --private-key ${PRIVATE_KEY} -vvvv --broadcast
```

Then update the `.env` in the root directory with the addresses that were deployed by running the following CLI command to get the values to paste in the env. This is a step that should be automated in the future.

```
cd ../../sdk
yarn build && yarn parse-deploy --path ../contracts/broadcast/multi/DeployMockCounter.s.sol-latest/run.json --map LightClientMock=LightClient
```

Then run the following to verify all the contracts. In the future, once multi-chain verification in
Foundry is less buggy, we can try to run `forge script --verify`. It's okay to run this script with both the mock light client deployment and real light client deployment, as long as the real light clients are already verified. In that case, this script will skip over verification over the light clients.

```
cd ../contracts/scripts
bash verify_mock_counter.sh
```

It's also important to do the following commands to generate the abi and the typechain:

```
cd ..
forge build --extra-output-files abi --out abi && cd ../sdk && yarn typechain
```

### Deployments

`LightClient.s.sol` is used for deploying the light client.

`Broadcaster.s.sol` is used for deploying the broadcaster.
`Reciever.s.sol` is used to deploy the reciever--specify which chain you want it deployed on, as well as an address for the broadcaster.

`DemoBridge.s.sol` is used to deploy the demo bridge, pointing to the Broadcaster and a reciever (again, specified which chains).
