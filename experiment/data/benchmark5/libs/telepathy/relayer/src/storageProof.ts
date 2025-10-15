import ethers from 'ethers';
import { AxiosInstance } from 'axios';
import { SentMessage } from '@prisma/client';
import { getReceiptProof } from './receiptProof';
import { ConsensusClient } from '@succinctlabs/telepathy-sdk';
import { toHexString } from '@chainsafe/ssz';

export async function getExecuteByStorageTx(
    sentMessage: SentMessage,
    targetBlock: number,
    client: AxiosInstance
) {
    const abiCoder = new ethers.utils.AbiCoder();
    const storageSlot = ethers.utils.keccak256(
        abiCoder.encode(['uint256', 'uint256'], [sentMessage.argNonce, 0])
    );

    // TODO maybe move this to an execution client class, similar to the consensus client
    const resp = await client.post('', {
        id: 1,
        jsonrpc: '2.0',
        method: 'eth_getProof',
        params: [sentMessage.contractAddress, [storageSlot], '0x' + targetBlock.toString(16)]
    });

    const accountProof = resp.data.result.accountProof;
    const storageProof = resp.data.result.storageProof[0].proof;
    return {
        message: sentMessage.argMessage,
        accountProof,
        storageProof
    };
}

export async function getExecuteByReceiptTx(
    sentMessage: SentMessage,
    srcSlot: number, // The slot against which we are generating the proof
    client: AxiosInstance,
    consensusClient: ConsensusClient,
    beaconChainAPI: AxiosInstance
) {
    // Get the slot number from the tx block number
    // Uses beaconcha.in API: https://beaconcha.in/api/v1/execution/block/15980920
    // TODO replace this with something more robust
    const { data } = await beaconChainAPI.get(`/v1/execution/block/${sentMessage.txBlockNumber}`);
    const txSlotNumber = parseInt(data.data[0].posConsensus.slot) as number;

    // TODO move this to an execution client class, similar to consensus client?
    const { receiptsRoot, receiptProof, txIndex, rlpEncodedTxIndex, txReceipt } =
        await getReceiptProof(sentMessage.txHash, client);

    let logIndex = -1;
    for (let i = 0; i < txReceipt.logs.length; i++) {
        const log = txReceipt.logs[i];
        if (toHexString(log[0]).toLowerCase() === sentMessage.contractAddress.toLowerCase()) {
            const topics = log[1];
            // topic[0] is the event signature
            const nonce = BigInt(toHexString(topics[1]));
            const messageRoot = toHexString(topics[2]);
            if (
                nonce === sentMessage.argNonce &&
                messageRoot.toLowerCase() === sentMessage.argMessageRoot.toLowerCase()
            ) {
                logIndex = i;
            }
        }
    }

    if (logIndex == -1) {
        throw Error('Log not found within transaction!');
    }

    const {
        receiptsRootProof,
        receiptsRoot: receiptsRootFromSlot,
        gindex
    } = await consensusClient.getReceiptsRootProof(srcSlot, txSlotNumber);
    if (receiptsRootFromSlot !== receiptsRoot) {
        throw new Error(
            'Receipts root mismatch.' +
                'Slot root: ' +
                receiptsRootFromSlot +
                'Tx root: ' +
                receiptsRoot
        );
    }

    return {
        srcSlot,
        txSlotNumber,
        message: sentMessage.argMessage,
        receiptsRootProof,
        receiptsRoot,
        receiptProof,
        txIndex,
        rlpEncodedTxIndex,
        logIndex
    };
}
