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
        `SALT=0x1234`, // TODO read this from the CLI args or set to be random
        `SOURCE_CHAIN_ID=${sourceChainId}`,
        `DEST_CHAIN_IDS=${targetChainIds.join(',')}`
    ];
    vars.push(`SourceAMB_${sourceChainId}=${config.address(sourceChain, 'SourceAMB')}`);
    for (let i = 0; i < targetChains.length; i++) {
        vars.push(`TargetAMB_${targetChainIds[i]}=${config.address(targetChains[i], 'TargetAMB')}`);
    }

    let cmd = `cd ../contracts/script && \
        echo '${vars.join('\n')}' > .env && \
        forge script Bridge.s.sol:Deploy \
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
            const runPath = '../contracts/broadcast/multi/Bridge.s.sol-latest/run.json';
            parseFoundryRunJson(runPath);
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
    vars.push(
        `InfiniteMintSuccincts_ADDRESS_${sourceChainId}=${config.address(
            sourceChain,
            'InfiniteMintSuccincts'
        )}`
    );
    vars.push(`Deposit_ADDRESS_${sourceChainId}=${config.address(sourceChain, 'Deposit')}`);

    for (let i = 0; i < targetChains.length; i++) {
        vars.push(
            `Withdraw_ADDRESS_${targetChainIds[i]}=${config.address(targetChains[i], 'Withdraw')}`
        );
        vars.push(
            `TargetAMB_ADDRESS_${targetChainIds[i]}=${config.address(targetChains[i], 'TargetAMB')}`
        );
    }

    const cmd = `cd ../contracts/script && \
    echo '${vars.join('\n')}' > .env && \
    bash bridge_verify.sh`;

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
        describe: 'Deploy bridge contracts.',
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
