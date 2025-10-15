import { AxiosInstance } from 'axios';

import { Trie } from '@ethereumjs/trie';
import { Log } from '@ethereumjs/evm';
import { TxReceipt, PostByzantiumTxReceipt, PreByzantiumTxReceipt } from '@ethereumjs/vm';
// import { encodeReceipt } from '@ethereumjs/vm/dist/runBlock';
import { RLP } from '@ethereumjs/rlp';
import { toHexString, fromHexString } from '@chainsafe/ssz';
import { bigIntToBuffer, bufArrToArr, intToBuffer } from '@ethereumjs/util';

// We copied this from @ethereumjs/vm/dist/runBlock, which we couldn't import due to
// weird typescript issues with LRU cache being missing
/**
 * Returns the encoded tx receipt.
 */
export function encodeReceipt(receipt: TxReceipt, txType: number) {
    const encoded = Buffer.from(
        RLP.encode(
            bufArrToArr([
                (receipt as PreByzantiumTxReceipt).stateRoot ??
                    ((receipt as PostByzantiumTxReceipt).status === 0
                        ? Buffer.from([])
                        : Buffer.from('01', 'hex')),
                bigIntToBuffer(receipt.cumulativeBlockGasUsed),
                receipt.bitvector,
                receipt.logs
            ])
        )
    );

    if (txType === 0) {
        return encoded;
    }

    // Serialize receipt according to EIP-2718:
    // `typed-receipt = tx-type || receipt-data`
    return Buffer.concat([intToBuffer(txType), encoded]);
}

/* Given a transaction hash and RPC execution client, generates a receipt proof against the
 * receipt root of the block.
 */
export async function getReceiptProof(txHash: string, client: AxiosInstance) {
    // https://goerli.etherscan.io/block/7943444 this is a block with 1 transaction
    // The transaction is: 0xae19ab98415b8515309cb794165457c9ebe1d874ad169eddf8d41c059f4ed618

    // Get the transaction
    const { data } = await client.post('', {
        id: 1,
        jsonrpc: '2.0',
        method: 'eth_getTransactionByHash',
        params: [txHash]
    });
    const blockHash = data.result.blockHash;
    const block = await client.post('', {
        id: 1,
        jsonrpc: '2.0',
        method: 'eth_getBlockByHash',
        params: [blockHash, true]
    });
    const transactions = block.data.result.transactions;

    let txIndex = -1;
    let txReceipt: TxReceipt | null = null;
    const receiptTrie = new Trie(); // use an in memory Trie
    for (let i = 0; i < transactions.length; i++) {
        const receipt = await client.post('', {
            id: 1,
            jsonrpc: '2.0',
            method: 'eth_getTransactionReceipt',
            params: [transactions[i].hash]
        });

        // TxReceipt
        const txResultTyped: TxReceipt = {
            cumulativeBlockGasUsed: BigInt(receipt.data.result.cumulativeGasUsed),
            bitvector: Buffer.from(fromHexString(receipt.data.result.logsBloom)),
            logs: receipt.data.result.logs.map((log: any) => {
                return [
                    Buffer.from(fromHexString(log.address)),
                    log.topics.map((x: string) => Buffer.from(fromHexString(x))),
                    Buffer.from(fromHexString(log.data))
                ];
            }) as Log[],
            status: receipt.data.result.status === '0x1' ? 1 : 0
        };

        if (transactions[i].hash == txHash) {
            txIndex = i;
            txReceipt = txResultTyped;
        }

        let type = 0;
        if (receipt.data.result.type == '0x1') {
            type = 1;
        } else if (receipt.data.result.type == '0x2') {
            type = 2;
        } else if (receipt.data.result.type != '0x0') {
            throw Error(`Unknown receipt type ${receipt.data.result.type}`);
        }

        const encodedReceipt = encodeReceipt(txResultTyped, type);
        await receiptTrie.put(Buffer.from(RLP.encode(i)), encodedReceipt);
    }
    if (txIndex == -1) {
        throw Error('transaction with hash ' + txHash + ' not found in block ' + blockHash);
    }
    if (txReceipt == null) {
        throw Error('transaction receipt with hash ' + txHash + ' not found in block ' + blockHash);
    }

    const computedReceiptRoot = receiptTrie.root();
    const receiptProof = await receiptTrie.createProof(Buffer.from(RLP.encode(txIndex)));
    const receiptsRoot = block.data.result.receiptsRoot;
    if (receiptsRoot !== toHexString(computedReceiptRoot)) {
        throw Error('The computed and true receipts root do not match');
    }
    return {
        receiptsRoot, //receipts root
        receiptProof: receiptProof.map((x) => toHexString(x)), // receipt proof
        txIndex, // index of the transaction in the block
        rlpEncodedTxIndex: toHexString(RLP.encode(txIndex)),
        txReceipt // we need this for determining the log index within the transaction
    };
}
