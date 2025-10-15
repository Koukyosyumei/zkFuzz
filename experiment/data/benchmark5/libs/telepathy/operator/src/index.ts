import {
    ConsensusClient,
    CircuitConfig,
    StepCircuit,
    RotateCircuit
} from '@succinctlabs/telepathy-sdk';
import { LightClient__factory } from '@succinctlabs/telepathy-sdk/contracts';
import { ethers } from 'ethers';
import toml from 'toml';
import fs from 'fs';
import { Operator, OperatorConfig, TargetConfig } from './operator';
import { logger } from './helper';
import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';

async function main() {
    const config = new ConfigManager('../toml/chain.toml', '../../.env', true);
    config.addAddressToml('../toml/address.prod.toml');
    const operatorParams = toml.parse(fs.readFileSync('../toml/operator.toml').toString());

    const sourceChains = config.filterChains('source');
    if (sourceChains.length !== 1) {
        throw new Error('config must have exactly one source chain');
    }
    const sourceChain = sourceChains[0];
    const consensusRpcUrl = config.consensusRpc(sourceChain);
    const client = new ConsensusClient(consensusRpcUrl);
    logger.info('Connected to consensus client at ' + consensusRpcUrl);

    const stepConfig: CircuitConfig = {
        witnessExecutablePath: operatorParams.step.witnessExecutablePath,
        proverExecutablePath: operatorParams.step.proverExecutablePath,
        proverKeyPath: operatorParams.step.proverKeyPath
    };
    const stepCircuit = new StepCircuit(stepConfig);
    logger.info('Initialized step circuit successfully');

    const rotateConfig: CircuitConfig = {
        witnessExecutablePath: operatorParams.rotate.witnessExecutablePath,
        proverExecutablePath: operatorParams.rotate.proverExecutablePath,
        proverKeyPath: operatorParams.rotate.proverKeyPath
    };
    const rotateCircuit = new RotateCircuit(rotateConfig);
    logger.info('Initialized rotate circuit successfully');

    const targets = [];
    const targetConfigs = [];
    const privateKey = config.privateKey();
    const destinationChains = config.filterChains('destination');
    for (let i = 0; i < destinationChains.length; i++) {
        const destChain = destinationChains[i];
        const targetConfig: TargetConfig = {
            name: destChain,
            address: config.address(destChain, 'LightClient'),
            chainId: config.chainId(destChain),
            executionRpcUrl: config.rpc(destChain)
        };
        const provider = new ethers.providers.JsonRpcProvider(targetConfig.executionRpcUrl);
        const wallet = new ethers.Wallet(privateKey, provider);
        const target = LightClient__factory.connect(targetConfig.address, wallet);
        targets.push(target);
        targetConfigs.push(targetConfig);
        logger.info('Connected to ' + targetConfig.name);
    }

    const operatorConfig: OperatorConfig = {
        client,
        stepCircuit,
        rotateCircuit,
        targets,
        targetConfigs,
        stepInterval: operatorParams.stepInterval,
        rotateInterval: operatorParams.rotateInterval
    };
    const operator = new Operator(operatorConfig);
    logger.info('Starting operator...');
    await operator.start();
}

main();
