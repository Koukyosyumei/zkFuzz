/**
 * This is a mock operator that is useful for testing the relayer.
 */

import {
    Contracts,
    LightClientMock,
    ContractTypeEnum
} from '@succinctlabs/telepathy-sdk/contracts';
import { commonSetup } from '@succinctlabs/telepathy-sdk/devops';
import { ConsensusClient } from '@succinctlabs/telepathy-sdk';
import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';
import winston from 'winston';
import { ssz } from '@lodestar/types';
import { toHexString } from '@chainsafe/ssz';
import { Relayer } from './relayer';

export class MockOperator {
    config: ConfigManager;
    sourceChain: string;
    targetChains: string[];
    contracts: Contracts;
    logger: winston.Logger;
    consensusClient: ConsensusClient;

    constructor(config: ConfigManager) {
        this.config = config;
        const { sourceChain, targetChains } = Relayer.parseConfig(config);
        this.sourceChain = sourceChain;
        this.targetChains = targetChains;
        this.contracts = MockOperator.initializeContracts(config, targetChains);
        this.consensusClient = new ConsensusClient(config.consensusRpc(sourceChain));
        this.logger = commonSetup().logger;
    }

    /** Initializes contracts from the relayer configuration */
    static initializeContracts(config: ConfigManager, targetChains: string[]) {
        const contracts = new Contracts();
        for (const destChain of targetChains) {
            const privateKey = config.privateKey();
            contracts.addSigner(config.chainId(destChain), privateKey, config.rpc(destChain));
            contracts.addContract(
                config.chainId(destChain),
                config.address(destChain, 'LightClient'),
                ContractTypeEnum.LightClientMock,
                true // requireSigner
            );
        }
        return contracts;
    }

    sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

    async getHeadBlock() {
        // Sometimes the head slot is a missed slot, so we need might need to try again
        // to get a valid head
        while (true) {
            try {
                const latestBlock = await this.consensusClient.getBlock('head');
                if (latestBlock) {
                    return latestBlock;
                }
            } catch (e) {
                // this.logger.error(e);
            }
        }
    }

    /** For all contracts, starts the relevant watcher */
    async start() {
        // Get the latest slot, then get latest block, then get state root for block
        while (true) {
            const latestBlock = await this.getHeadBlock();
            const latestSlot = latestBlock.slot;
            this.logger.info('Latest slot: ' + latestSlot);
            const executionStateRoot = latestBlock.body.executionPayload.stateRoot;
            const latestHeader = await this.consensusClient.getHeader(latestSlot);
            const latestHeaderRoot = ssz.phase0.BeaconBlockHeader.hashTreeRoot(latestHeader);
            for (const destChain of this.targetChains) {
                const destChainId = this.config.chainId(destChain);
                const lightClient = this.contracts.getContract(
                    destChainId,
                    ContractTypeEnum.LightClientMock
                ) as LightClientMock;

                const extraOptions = await this.contracts.getExtraOptions(destChainId);
                // console.log(latestSlot, toHexString(executionStateRoot));
                const executeTx = await lightClient.setExecutionRoot(
                    latestSlot,
                    executionStateRoot
                );
                const receipt = await executeTx.wait();
                this.logger.info(
                    `Sent mock operator update transaction for execution root on ${destChainId}, txHash ${JSON.stringify(
                        receipt.transactionHash
                    )}`
                );
                const stateRootTx = await lightClient.setHeader(
                    latestSlot,
                    latestHeaderRoot,
                    extraOptions
                );
                const receiptStateRoot = await stateRootTx.wait();
                this.logger.info(
                    `Sent mock operator update transaction for header root on ${destChainId}, txHash ${JSON.stringify(
                        receiptStateRoot.transactionHash
                    )}`
                );
            }
            await this.sleep(60 * 1000); // Every 60 seconds
        }
    }
}
