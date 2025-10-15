import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';

import { Relayer } from './relayer';
import { MockOperator } from './mockOperator';

async function runRelayer(mock: boolean, dev: boolean, backfill: number | undefined) {
    const config = new ConfigManager('../toml/chain.toml', '../.env', true);
    const devOrProd = dev ? 'dev' : 'prod';
    console.log('Running relayer in ' + devOrProd + ' mode');
    config.addAddressToml(`../toml/address.${devOrProd}.toml`);

    const relayer = new Relayer(config);
    relayer.start();

    if (mock) {
        console.log('Starting mock operator');
        const mockOperator = new MockOperator(config);
        mockOperator.start();
    }

    if (backfill) {
        console.log('Starting backfill');
        await relayer.backfill(backfill, undefined);
    }
}

function main() {
    const y = yargs();

    y.command({
        command: 'relay',
        describe: 'Run a relayer.',
        builder: {
            mock: {
                describe: 'Whether to run mock operator or not',
                demandOption: false,
                type: 'boolean'
            },
            dev: {
                describe: 'Whether to run in dev or prod mode',
                demandOption: false,
                type: 'boolean'
            },
            backfill: {
                describe: 'Where to backfill from',
                demandOption: false,
                type: 'number'
            }
        },
        handler(argv: any) {
            runRelayer(argv.mock, argv.dev, argv.backfill);
        }
    });

    y.parse(hideBin(process.argv));
}

main();
