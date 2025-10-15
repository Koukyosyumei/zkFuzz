/**
 * A Telepathy relayer is responsible for the transmission of messages from a source chain
 * to a target (destination) chain.
 *
 * The relayer is configured with a source chain RPC and a SourceAMB contract address. The relayer
 * will read event logs from the SourceAMB contract that correspond to sent messages.
 * The relayer is also configured with a list of destination chain RPCs and for each destination
 * chain a BeaconLightClient contract address and a TargetAMB contract address.
 * The relayer will read event logs from each BeaconLightClient contract to know how up to date
 * each destination chain is. Based on the latest block in each BeaconLightClient, the relayer
 * is responsible for relaying any relevant messages to the corresponding target chain TargetAMB.
 *
 */
import { PrismaClient, SentMessage } from '@prisma/client';
import { AxiosInstance } from 'axios';
import axios from 'axios';
import ethers, { BigNumber } from 'ethers';

import {
    Contracts,
    LightClient,
    SourceAMB,
    TargetAMB,
    ContractTypeEnum
} from '@succinctlabs/telepathy-sdk/contracts';
import { HeadUpdateEvent } from '@succinctlabs/telepathy-sdk/contracts/typechain/LightClient.sol/LightClient';
import { SentMessageEvent } from '@succinctlabs/telepathy-sdk/contracts/typechain/SourceAMB.sol/SourceAMB';
import { ExecutedMessageEvent } from '@succinctlabs/telepathy-sdk/contracts/typechain/TargetAMB.sol/TargetAMB';
import IntegrationClient from '@succinctlabs/telepathy-sdk/integrationClient';
import { ConsensusClient } from '@succinctlabs/telepathy-sdk';
import { ConfigManager } from '@succinctlabs/telepathy-sdk/config';
import { getExecuteByStorageTx, getExecuteByReceiptTx } from './storageProof';
import type { Logger } from 'winston';

export class Relayer {
    config: ConfigManager;
    contracts: Contracts;
    sourceChain: string;
    sourceChainId: number;
    targetChains: string[];
    targetChainIds: number[];
    prisma: PrismaClient;
    consensusClient: ConsensusClient;
    executionClient: AxiosInstance;
    beaconChainClient: AxiosInstance;
    latestSlots: BigNumber[];
    integrationClient: IntegrationClient;
    logger: Logger;
    mode: 'receipts' | 'storage' = 'storage';

    constructor(config: ConfigManager) {
        this.config = config;
        const { sourceChain, targetChains } = Relayer.parseConfig(config);
        this.sourceChain = sourceChain;
        this.sourceChainId = config.chainId(sourceChain);
        this.targetChains = targetChains;
        this.targetChainIds = targetChains.map((chain) => config.chainId(chain));
        this.contracts = Relayer.initializeContracts(config, sourceChain, targetChains);

        this.consensusClient = new ConsensusClient(config.consensusRpc(sourceChain));
        this.executionClient = axios.create({
            baseURL: config.rpc(sourceChain),
            responseType: 'json',
            headers: { 'Content-Type': 'application/json' }
        });
        this.beaconChainClient = axios.create({
            baseURL: config.getEnv(`BEACON_CHAIN_RPC_${config.chainId(sourceChain)}`),
            responseType: 'json',
            headers: {
                'Content-Type': 'application/json',
                apikey: config.getEnv(`BEACON_CHAIN_API_KEY_${config.chainId(sourceChain)}`)
            }
        });
        this.latestSlots = [];

        this.prisma = new PrismaClient();
        this.integrationClient = new IntegrationClient();
        this.logger = this.integrationClient.logger;
    }

    static parseConfig(config: ConfigManager) {
        // The config has source=true
        const sourceChains = config.filterChains('source');
        if (sourceChains.length !== 1) {
            throw new Error('config must have exactly one source chain');
        }
        const sourceChain = sourceChains[0];
        const targetChains = config.filterChains('destination');
        if (targetChains.length === 0) {
            throw new Error('config must have at least one destination chain');
        }
        return { sourceChain, targetChains };
    }

    /** Initializes contracts from the relayer configuration */
    static initializeContracts(config: ConfigManager, sourceChain: string, targetChains: string[]) {
        const contracts = new Contracts();
        const sourceChainId = config.chainId(sourceChain);
        // We are not going to send transactions to source chain, so we don't need a signer
        contracts.addProvider(sourceChainId, config.rpc(sourceChain));
        contracts.addContract(
            sourceChainId,
            config.address(sourceChain, 'SourceAMB'),
            ContractTypeEnum.SourceAMB,
            false // requireSigner
        );
        for (const destChain of targetChains) {
            const chainId = config.chainId(destChain);
            contracts.addSigner(chainId, config.privateKey(), config.rpc(destChain));
            contracts.addContract(
                chainId,
                config.address(destChain, 'TargetAMB'),
                ContractTypeEnum.TargetAMB,
                true // requireSigner
            );
            contracts.addContract(
                chainId,
                config.address(destChain, 'LightClient'),
                ContractTypeEnum.LightClient,
                false // requireSigner
            );
        }
        return contracts;
    }

    /** For all contracts, starts the relevant watcher */
    async start() {
        const sourceAMB = this.contracts.getContract(
            this.sourceChainId,
            ContractTypeEnum.SourceAMB
        ) as SourceAMB;
        this.startSourceAmbWatcher(sourceAMB, this.sourceChainId);

        for (const destChain of this.targetChains) {
            const chainId = this.config.chainId(destChain);
            const lightClient = this.contracts.getContract(
                chainId,
                ContractTypeEnum.LightClient
            ) as LightClient;
            this.startLightClientWatcher(lightClient, chainId);

            const targetAmb = this.contracts.getContract(
                chainId,
                ContractTypeEnum.TargetAMB
            ) as TargetAMB;
            this.startTargetAmbWatcher(targetAmb, chainId);
        }
    }

    /** Gets the latest finalized update from the sync committee. */
    startLightClientWatcher(contract: LightClient, chainId: number) {
        const eventFilter = contract.filters.HeadUpdate();
        contract.on(eventFilter, async (slot, root, headUpdate) => {
            await this.processHeadUpdate(headUpdate, chainId);
        });
    }

    startSourceAmbWatcher(contract: SourceAMB, chainId: number) {
        const eventFilter = contract.filters.SentMessage();
        contract.on(eventFilter, async (nonce, messageRoot, message, sentMessage) => {
            await this.processSentMessage(sentMessage, chainId);
        });
    }

    startTargetAmbWatcher(contract: TargetAMB, chainId: number) {
        const eventFilter = contract.filters.ExecutedMessage();
        contract.on(eventFilter, async (nonce, messageRoot, message, status, executedMessage) => {
            await this.processExecutedMessage(executedMessage, chainId);
        });
    }

    async processHeadUpdate(headUpdate: HeadUpdateEvent, chainId: number) {
        this.logger.info('Got head update event', {
            chainId,
            slot: headUpdate.args.slot.toNumber(),
            txHash: headUpdate.transactionHash
        });
        const block = await headUpdate.getBlock();

        const existing = await this.prisma.headUpdate.findMany({
            where: {
                chainId: chainId,
                contractAddress: headUpdate.address,
                argSlot: headUpdate.args.slot.toBigInt(),
                argRoot: headUpdate.args.root,
                txHash: headUpdate.transactionHash
            }
        });
        if (existing.length > 0) {
            this.logger.info('Skipping duplicate head update', {
                chainId,
                argSlot: headUpdate.args.slot.toBigInt(),
                argRoot: headUpdate.args.root
            });
            return;
        }

        const blockForSlot = await this.consensusClient.getBlockNumberFromSlot(
            headUpdate.args.slot.toNumber()
        );
        await this.prisma.headUpdate.create({
            data: {
                argSlot: headUpdate.args.slot.toBigInt(),
                argRoot: headUpdate.args.root,
                augBlockForSlot: blockForSlot,
                // TODO reuse this code across all the different event types
                chainId: chainId,
                contractAddress: headUpdate.address,
                txHash: headUpdate.transactionHash,
                txIndex: headUpdate.transactionIndex,
                logIndex: headUpdate.logIndex,
                txBlockNumber: headUpdate.blockNumber,
                txBlockHash: headUpdate.blockHash,
                txTime: block.timestamp
            }
        });

        const currentSlot = headUpdate.args.slot;
        let prevSlot: BigNumber;
        if (this.latestSlots.length == 0) {
            this.latestSlots.push(currentSlot);
            const prevBlock = await this.consensusClient.getApproxSlotOffset(
                currentSlot.toNumber(),
                32
            );
            prevSlot = BigNumber.from(prevBlock.slot); // go to previous epoch, 32 blocks in epoch
        } else {
            const latestSlot = this.latestSlots[this.latestSlots.length - 1];
            if (currentSlot.gt(latestSlot)) {
                this.latestSlots.push(currentSlot);
                prevSlot = latestSlot;
            } else {
                const prevBlock = await this.consensusClient.getApproxSlotOffset(
                    currentSlot.toNumber(),
                    32
                );
                prevSlot = BigNumber.from(prevBlock.slot); // go to previous epoch, 32 blocks in epoch
            }
        }
        if (this.latestSlots.length > 2) {
            this.latestSlots = this.latestSlots.slice(this.latestSlots.length - 2);
        }

        await this.generateRelayTransactions(prevSlot, currentSlot);
    }

    // async processShortSentMessage(shortSentMessage: ShortSentMessageEvent, chainId: number) {
    //     const transaction = await shortSentMessage.getTransaction();
    //     const calldata = transaction.data;
    //     const iface = new ethers.utils.Interface(
    //         'sendViaLog(address recipient, uint16 recipientChainId, uint256 gasLimit, bytes calldata data)'
    //     );
    //     const res = iface.parseTransaction({ data: calldata });
    //     const argNonce = shortSentMessage.args.nonce.toBigInt();
    //     const argMessageRoot = shortSentMessage.args.msgHash;
    //     const augMsgSender = transaction.from;
    //     const augRecipient = res.args[0];
    //     const augRecipientChainId = res.args[1];
    //     const augGasLimit = res.args[2];
    //     const augData = res.args[3];
    //     const argMessage = ethers.utils.defaultAbiCoder.encode(
    //         ['uint256', 'address', 'address', 'uint16', 'uint256', 'bytes'],
    //         [argNonce, augMsgSender, augRecipient, augRecipientChainId, augGasLimit, augData]
    //     );
    // }

    async processSentMessage(sentMessage: SentMessageEvent, chainId: number) {
        this.logger.info('Got sent message event', {
            nonce: sentMessage.args.nonce.toNumber(),
            txHash: sentMessage.transactionHash
        });
        const block = await sentMessage.getBlock();
        const decoded = ethers.utils.defaultAbiCoder.decode(
            ['uint256', 'address', 'address', 'uint16', 'uint256', 'bytes'],
            sentMessage.args.message
        );
        if (!decoded[0].eq(sentMessage.args.nonce)) {
            throw new Error('Nonce does not match from decoded message');
        }
        const computedHash = ethers.utils.keccak256(sentMessage.args.message);
        if (computedHash !== sentMessage.args.msgHash) {
            throw new Error("Message root doesn't match computed hash");
        }
        // First duplicate sent message
        // The reason that we cannot simply use the messageRoot is that if we have
        // different sourceAMBs on different chains, then the message root could be the same
        // TODO: The reason we need the txHash is in the case of re-orgs? I haven't thought
        // through this scenario very carefully, and we need to invalidate the previous
        // sent message.
        const existing = await this.prisma.sentMessage.findMany({
            where: {
                chainId: chainId,
                contractAddress: sentMessage.address,
                argMessageRoot: sentMessage.args.msgHash,
                txHash: sentMessage.transactionHash
            }
        });
        if (existing.length > 0) {
            this.logger.info('Skipping duplicate sent message', {
                chainId,
                messageRoot: sentMessage.args.msgHash,
                txHash: sentMessage.transactionHash
            });
            return;
        }
        // TODO detect reorg in the case where there is a re-org and someone
        // submits duplicate message
        await this.prisma.sentMessage.create({
            data: {
                argNonce: sentMessage.args.nonce.toBigInt(),
                argMessageRoot: sentMessage.args.msgHash,
                argMessage: sentMessage.args.message,
                // Now get the augmented fields
                augMsgSender: decoded[1],
                augRecipient: decoded[2],
                augRecipientChainId: decoded[3],
                augGasLimit: decoded[4].toNumber(),
                augData: decoded[5],
                // TODO reuse this code with the above
                chainId: chainId,
                contractAddress: sentMessage.address,
                txHash: sentMessage.transactionHash,
                txIndex: sentMessage.transactionIndex,
                logIndex: sentMessage.logIndex,
                txBlockNumber: sentMessage.blockNumber,
                txBlockHash: sentMessage.blockHash,
                txTime: block.timestamp
            }
        });
    }

    async processExecutedMessage(executedMessage: ExecutedMessageEvent, chainId: number) {
        const block = await executedMessage.getBlock();
        // TODO put this in a shared library
        const existing = await this.prisma.executedMessage.findMany({
            where: {
                chainId: chainId,
                contractAddress: executedMessage.address,
                argMessageRoot: executedMessage.args.msgHash,
                txHash: executedMessage.transactionHash
            }
        });
        if (existing.length > 0) {
            this.logger.info('Skipping duplicate sent message', {
                chainId,
                messageRoot: executedMessage.args.msgHash,
                txHash: executedMessage.transactionHash
            });
            return;
        }
        await this.prisma.executedMessage.create({
            data: {
                argNonce: executedMessage.args.nonce.toBigInt(),
                argMessageRoot: executedMessage.args.msgHash,
                argMessage: executedMessage.args.message,
                argStatus: executedMessage.args.status,
                // TODO reuse this code with the above
                chainId: chainId,
                contractAddress: executedMessage.address,
                txHash: executedMessage.transactionHash,
                txIndex: executedMessage.transactionIndex,
                logIndex: executedMessage.logIndex,
                txBlockNumber: executedMessage.blockNumber,
                txBlockHash: executedMessage.blockHash,
                txTime: block.timestamp
            }
        });
    }

    async checkIfSentMessagedExecuted(sentMessage: SentMessage) {
        // TODO add a cache with our database here
        const targetAmb = this.contracts.getContract(
            sentMessage.augRecipientChainId,
            ContractTypeEnum.TargetAMB
        ) as TargetAMB;
        const status = await targetAmb.messageStatus(sentMessage.argMessageRoot);
        if (status === 0) {
            return false;
        }
        return true;
    }

    async getLatestSlot(chainId: number) {
        // TODO add a cache with our database here
        const lc = this.contracts.getContract(chainId, ContractTypeEnum.LightClient) as LightClient;
        const slot = await lc.head();
        return slot;
    }

    /** Given a slot range, query for sent messages in this range and send them out */
    async generateRelayTransactions(startSlot: BigNumber, endSlot: BigNumber) {
        // const startBlock = await this.consensusClient.getBlockNumberFromSlot(startSlot.toNumber());
        const startBlock = 0;
        const endBlock = await this.consensusClient.getBlockNumberFromSlot(endSlot.toNumber());
        const relevantSentMessages = await this.prisma.sentMessage.findMany({
            where: {
                txBlockNumber: {
                    gt: startBlock,
                    lte: endBlock
                },
                contractAddress: this.config.address(this.sourceChain, 'SourceAMB'),
                chainId: this.sourceChainId
            }
        });
        for (const sentMessage of relevantSentMessages) {
            // TODO before sending out the sent message for execution, check if it's already been
            // executed. First check our DB to see if the message has been executed.
            // Then check the chain to see if the message has been executed.
            // Finally, if both those pass, then execute the message. There is still a chance
            // that message execution will fail since there might be a pending transaction
            // in the mempool.
            await this.executeSentMessage(sentMessage);
        }
    }

    async executeSentMessage(sentMessage: SentMessage) {
        this.logger.info(
            `Trying to execute message... ${sentMessage.argNonce} ${sentMessage.augRecipientChainId}`
        );
        if (!this.targetChainIds.includes(sentMessage.augRecipientChainId)) {
            this.logger.info(
                `Skipping message because it's not for one of our target chains... ${sentMessage.argNonce} ${sentMessage.augRecipientChainId}`
            );
            return;
        }
        const messageExecuted = await this.checkIfSentMessagedExecuted(sentMessage);
        if (messageExecuted) {
            this.logger.info(
                `Message with nonce ${sentMessage.argNonce} already executed, skipping`
            );
            return;
        }
        this.logger.info(JSON.stringify(sentMessage));
        const destChainId = sentMessage.augRecipientChainId;
        const latestSlot = await this.getLatestSlot(destChainId);
        this.logger.info(`Latest slot: ${latestSlot} on chain ${destChainId}`);
        if (!latestSlot || latestSlot.toNumber() === 0) {
            this.logger.error('Skipping message...Could not get latest slot on destination chain');
            return;
        }
        const latestBlock = await this.consensusClient.getBlockNumberFromSlot(
            latestSlot.toNumber()
        );
        if (sentMessage.txBlockNumber > latestBlock) {
            this.logger.info(
                `Message too new (block number ${sentMessage.txBlockNumber})...skipping (latest update block ${latestBlock})`
            );
            return;
        }
        const targetAmb = this.contracts.getContract(
            destChainId,
            ContractTypeEnum.TargetAMB
        ) as TargetAMB;

        try {
            const extraOptions = await this.contracts.getExtraOptions(destChainId);
            let executeTx: ethers.ethers.ContractTransaction | null = null;
            if (this.mode == 'storage') {
                this.logger.info('Sending storage execution');
                const { message, accountProof, storageProof } = await getExecuteByStorageTx(
                    sentMessage,
                    latestBlock, // The target *block*, the storage proof is against this quantity
                    this.executionClient
                );
                // console.log(latestSlot.toNumber());
                // console.log(message);
                // console.log(JSON.stringify(accountProof));
                // console.log(JSON.stringify(storageProof));
                executeTx = await targetAmb.executeMessage(
                    latestSlot.toNumber(),
                    message,
                    accountProof,
                    storageProof,
                    extraOptions
                );
            } else {
                this.logger.info('Sending receipt execution');
                const {
                    srcSlot,
                    txSlotNumber,
                    message,
                    receiptsRootProof,
                    receiptsRoot,
                    receiptProof,
                    // txIndex, // TODO: this isn't used commenting out for now
                    rlpEncodedTxIndex,
                    logIndex
                } = await getExecuteByReceiptTx(
                    sentMessage,
                    latestSlot.toNumber(),
                    this.executionClient,
                    this.consensusClient,
                    this.beaconChainClient
                );
                const abiCoder = new ethers.utils.AbiCoder();
                executeTx = await targetAmb.executeMessageFromLog(
                    abiCoder.encode(['uint64', 'uint64'], [srcSlot, txSlotNumber]),
                    message,
                    receiptsRootProof,
                    receiptsRoot,
                    receiptProof,
                    rlpEncodedTxIndex,
                    logIndex,
                    extraOptions
                );
            }

            const receipt = await executeTx.wait();
            this.logger.info(
                `Sent relay transaction, receipt ${JSON.stringify(receipt.transactionHash)}`
            );
        } catch (e) {
            // TODO: Should be sent to Sentry
            this.logger.error(`Error sending relay transaction: ${e}`);
        }
    }

    async backfill(startBlock: number, endBlock: number | undefined) {
        // If endBlock is not specified, then we assume it to be the current block
        // If the startBlock is negative, then we start from endBlock + startBlock
        if (!endBlock) {
            endBlock = await this.consensusClient.getBlockNumberFromSlot('head');
        }
        if (startBlock < 0) {
            // TODO: This is an invalid TypeScript setting. Should either not use non-null assertion or set a default value for endBlock
            // eslint-disable-next-line
            startBlock = endBlock! + startBlock; // It's okay because we set it above
        }
        const sourceAMB = this.contracts.getContract(
            this.sourceChainId,
            ContractTypeEnum.SourceAMB
        ) as SourceAMB;
        const sentMessages = await sourceAMB.queryFilter(
            sourceAMB.filters.SentMessage(),
            startBlock,
            endBlock
        );
        this.logger.info(
            `Got ${sentMessages.length} sent messages from block ${startBlock} to ${endBlock}`
        );
        await Promise.all(
            sentMessages.map((sentMessage) =>
                this.processSentMessage(sentMessage, this.sourceChainId)
            )
        );

        // Now we try to execute them all
        const relevantSentMessages = await this.prisma.sentMessage.findMany({
            where: {
                txBlockNumber: {
                    gte: startBlock,
                    lte: endBlock
                },
                contractAddress: this.config.address(this.sourceChain, 'SourceAMB'),
                chainId: this.sourceChainId
            }
        });
        this.logger.info(
            'Messages queued for execution has length: ' + relevantSentMessages.length
        );
        for (const sentMessage of relevantSentMessages) {
            // TODO before sending out the sent message for execution, check if it's already been
            // executed. First check our DB to see if the message has been executed.
            // Then check the chain to see if the message has been executed.
            // Finally, if both those pass, then execute the message. There is still a chance
            // that message execution will fail since there might be a pending transaction
            // in the mempool.
            await this.executeSentMessage(sentMessage);
        }

        // TODO backfill the executed messages as well as the head updates
    }
}
