import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { exec } from 'node:child_process';

import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';

import { parseFoundryRunJson } from './parseFoundryDeploy';

async function deployTelepathy(onlyParse: boolean, dev: boolean) {
    const config = new ConfigManager('../toml/chain.toml', '../.env', true);
    const devOrProd = dev ? 'dev' : 'prod';
    console.log('Running deployment in ' + devOrProd + ' mode');
    config.addAddressToml(`../toml/address.${devOrProd}.toml`);

    const sourceChain = config.filterChains('source')[0];
    const targetChains = config.filterChains('destination');
    const sourceChainId = config.chainId(sourceChain);
    const targetChainIds = targetChains.map((chain: string) => config.chainId(chain));

    const vars = [
        `USE_CREATE_2=true`, // TODO read this from the CLI args
        `SALT=0x223368`, // TODO read this from the CLI args
        `USE_MOCK_LC=${dev}`,
        `SOURCE_CHAIN_ID=${sourceChainId}`,
        `DEST_CHAIN_IDS=${targetChainIds.join(',')}`
    ];

    for (let i = 0; i < targetChains.length; i++) {
        vars.push(
            `LightClient_ADDRESS_${targetChainIds[i]}=${config.address(
                targetChains[i],
                'LightClient'
            )}`
        );
    }

    let cmd = `cd ../contracts/script && \
        echo '${vars.join('\n')}' > .env && \
        forge script Counter.s.sol:Deploy \
        --private-key ${config.privateKey()} \
        --broadcast \
        --verify \
        --multi \
        -vvvv`;
    if (onlyParse) {
        cmd = 'echo "Only parsing the latest run.json file"';
    }

    const result = exec(cmd);
    if (result.stdout != undefined) {
        result.stdout.on('data', (data) => {
            console.log(data);
        });
    }
    if (result.stderr != undefined) {
        result.stderr.on('data', (data) => {
            console.error(data);
        });
    }
    result.on('close', (code) => {
        console.log(`child process exited with code ${code}`);
        if (code != 0) {
            console.log(
                "If using CREATE2=true, make sure you've set the salt to be a new variable"
            );
        }
        if (code == 0) {
            const runPath = '../contracts/broadcast/multi/Counter.s.sol-latest/run.json';
            parseFoundryRunJson(runPath, 'LightClientMock=LightClient');
        }
    });
}

async function verify(dev: boolean) {
    const config = new ConfigManager('../toml/chain.toml', '../.env', true);
    const devOrProd = dev ? 'dev' : 'prod';
    console.log('Running deployment in ' + devOrProd + ' mode');
    config.addAddressToml(`../toml/address.${devOrProd}.toml`);

    const sourceChain = config.filterChains('source')[0];
    const targetChains = config.filterChains('destination');
    const sourceChainId = config.chainId(sourceChain);
    const targetChainIds = targetChains.map((chain: string) => config.chainId(chain));
    const vars = [`SOURCE_CHAIN_ID=${sourceChainId}`, `CHAIN_IDS=(${targetChainIds.join(' ')})`];
    vars.push(`SourceAMB_ADDRESS_${sourceChainId}=${config.address(sourceChain, 'SourceAMB')}`);
    vars.push(`Counter_ADDRESS_${sourceChainId}=${config.address(sourceChain, 'Counter')}`);

    for (let i = 0; i < targetChains.length; i++) {
        vars.push(
            `LightClient_ADDRESS_${targetChainIds[i]}=${config.address(
                targetChains[i],
                'LightClient'
            )}`
        );
        vars.push(
            `Counter_ADDRESS_${targetChainIds[i]}=${config.address(targetChains[i], 'Counter')}`
        );
        vars.push(
            `TargetAMB_ADDRESS_${targetChainIds[i]}=${config.address(targetChains[i], 'TargetAMB')}`
        );
    }

    const cmd = `cd ../contracts/script && \
    echo '${vars.join('\n')}' > .env && \
    bash counter_verify.sh`;

    const result = exec(cmd);
    if (result.stdout != undefined) {
        result.stdout.on('data', (data) => {
            console.log(data);
        });
    }
    if (result.stderr != undefined) {
        result.stderr.on('data', (data) => {
            console.error(data);
        });
    }
    result.on('close', (code) => {
        console.log(`child process exited with code ${code}`);
    });
}

function main() {
    const y = yargs();

    y.command({
        command: 'deploy',
        describe: 'Deploy counter contracts.',
        builder: {
            onlyParse: {
                describe: 'Whether to only parse the run.json file',
                boolean: true
            },
            dev: {
                describe: 'Whether to run in dev or prod.',
                boolean: true
            }
        },
        handler(argv: any) {
            deployTelepathy(argv.onlyParse, argv.dev);
        }
    });

    y.command({
        command: 'verify',
        describe: 'Veify counter contracts for a source chain to a target chain.',
        builder: {
            dev: {
                describe: 'Whether to run in dev or prod.',
                boolean: true
            }
        },
        handler(argv: any) {
            verify(argv.dev);
        }
    });

    y.parse(hideBin(process.argv));
}

main();
