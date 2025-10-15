import {
    ConsensusClient,
    poseidonSyncCommittee,
    toLittleEndianFromBigInt
} from '@succinctlabs/telepathy-sdk';
import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';
import { toHexString } from '@chainsafe/ssz';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { exec } from 'node:child_process';

async function deployLightClient(source: string, target: string) {
    const config = new ConfigManager('../toml/chain.toml', '../.env', true);

    const client = new ConsensusClient(config.consensusRpc(source));
    const update = await client.getTelepathyUpdate('finalized');
    const genesis = await client.getGenesis();
    const poseidon = await poseidonSyncCommittee(update.currentSyncCommittee.pubkeys);
    const genesisValidatorsRoot = toHexString(update.genesisValidatorsRoot);
    const genesisTime = genesis.genesisTime;
    const secondsPerSlot = 12;
    const syncCommitteePeriod = Math.floor(Math.floor(update.attestedHeader.slot / 32) / 256);
    const syncCommitteePoseidon = toHexString(toLittleEndianFromBigInt(poseidon));

    const vars = [
        `SALT=0x1233`, // TODO read this from the CLI args or set it to be random
        `GENESIS_VALIDATORS_ROOT=${genesisValidatorsRoot}`,
        `GENESIS_TIME=${genesisTime}`,
        `SECONDS_PER_SLOT=${secondsPerSlot}`,
        `SYNC_COMMITTEE_PERIOD=${syncCommitteePeriod}`,
        `SYNC_COMMITTEE_POSEIDON=${syncCommitteePoseidon}`
    ];

    const cmd = `cd ../contracts/script && \
        echo '${vars.join('\n')}' > .env && \
        forge script LightClient.s.sol:Deploy \
        --rpc-url ${config.rpc(target)} \
        --chain-id ${config.chainId(target)} \
        --private-key ${config.privateKey()} \
        --verify \
        --broadcast \
        --etherscan-api-key ${config.etherscanApiKey(target)} \
        -vvvv`;

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
        describe: 'Deploy telepathy light client for a source chain to a target chain.',
        builder: {
            source: {
                describe: 'the chain providing consensus info',
                demandOption: true,
                type: 'string'
            },
            target: {
                describe: 'the chain telepathy will be deployed on',
                demandOption: true,
                type: 'string'
            }
        },
        handler(argv: any) {
            deployLightClient(argv.source, argv.target);
        }
    });

    y.parse(hideBin(process.argv));
}

main();
