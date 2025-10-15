import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';
import fs from 'fs';
import path from 'path';

function callback(err: any) {
    if (err) throw err;
}

function main(dev: boolean) {
    const config = new ConfigManager('../toml/chain.toml', '../.env', true);
    const devOrProd = dev ? 'dev' : 'prod';
    console.log('Running frontend build process in ' + devOrProd + ' mode');
    config.addAddressToml(`../toml/address.${devOrProd}.toml`);

    const abis = [
        'Bridge.sol/Deposit.abi.json',
        'Bridge.sol/Withdraw.abi.json',
        'LightClient.sol/LightClient.abi.json', // TODO change these to Interfaces
        'Tokens.sol/InfiniteMintSuccincts.abi.json',
        'IERC20.sol/IERC20.abi.json',
        'SourceAMB.sol/SourceAMB.abi.json',
        'TargetAMB.sol/TargetAMB.abi.json'
    ];
    for (const abi of abis) {
        const abiPath = path.join('../contracts/abi', abi);
        fs.copyFile(abiPath, '../frontend/public/abi/' + path.parse(abi).base, callback);
    }

    const addresses = config._addressConfig;
    fs.writeFileSync(
        `../frontend/public/config/address.${devOrProd}.json`,
        JSON.stringify(addresses, null, 2)
    );
    fs.writeFileSync('../frontend/public/config/rpc.json', JSON.stringify(config._rpc, null, 2));
    fs.writeFileSync(
        '../frontend/public/config/consensusRpc.json',
        JSON.stringify(config._consensusRPC, null, 2)
    );
    fs.writeFileSync(
        '../frontend/public/config/chainId.json',
        JSON.stringify(config._chainIds, null, 2)
    );
}

main(false);
