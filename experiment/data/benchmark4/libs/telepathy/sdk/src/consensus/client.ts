import axios from 'axios';
import { AxiosInstance } from 'axios';
import { toHexString } from '@chainsafe/ssz';
import { altair, bellatrix, phase0, ssz } from '@lodestar/types';
import {
    ProofType,
    createProof,
    SingleProof,
    concatGindices
} from '@chainsafe/persistent-merkle-tree';
import { UnparsedResponse } from './utils';
import { computeBitSum } from './ssz';
import { start } from 'repl';

const ROUTES = {
    getBlock: '/eth/v2/beacon/blocks/{block_id}',
    getGenesis: '/eth/v1/beacon/genesis',
    getHeader: '/eth/v1/beacon/headers/{block_id}',
    getFinalityUpdate: '/eth/v1/beacon/light_client/finality_update',
    getUpdates: '/eth/v1/beacon/light_client/updates',
    getBeaconState: '/eth/v2/debug/beacon/states/{state_id}'
};

/** Can be a slot, a hexstring, or 'head' / 'finalized' / 'justified' / etc */
export type BeaconId = number | Uint8Array | string;
export type TelepathyUpdate = {
    attestedHeader: phase0.BeaconBlockHeader;
    currentSyncCommittee: altair.SyncCommittee;
    nextSyncCommittee: altair.SyncCommittee;
    nextSyncCommitteeBranch: Uint8Array[];
    finalizedHeader: phase0.BeaconBlockHeader;
    finalityBranch: Uint8Array[];
    syncAggregate: altair.SyncAggregate;
    genesisValidatorsRoot: Uint8Array;
    forkVersion: Uint8Array;
    executionStateRoot: Uint8Array;
    executionStateBranch: Uint8Array[];
};

/**
 * Connects to an Ethereum 2.0 Beacon Node and exposes methods to query and access data about
 * the chain.
 */
export class ConsensusClient {
    consensusRpc: string;
    client: AxiosInstance;
    SLOTS_PER_HISTORICAL_ROOT = 8192;

    constructor(consensusRpc: string) {
        this.consensusRpc = consensusRpc;
        this.client = axios.create({
            baseURL: this.consensusRpc,
            responseType: 'json',
            headers: { 'Content-Type': 'application/json' }
        });
    }

    toStringFromBeaconId(identifier: BeaconId) {
        if (identifier instanceof Uint8Array) {
            return toHexString(identifier);
        }
        return identifier.toString();
    }

    async getState(stateIdentifier: BeaconId): Promise<bellatrix.BeaconState> {
        const id = this.toStringFromBeaconId(stateIdentifier);
        const response = await this.client.get(ROUTES.getBeaconState.replace('{state_id}', id));
        const state = ssz.bellatrix.BeaconState.fromJson(
            response.data.data
        ) as bellatrix.BeaconState;
        return state;
    }

    async getBlock(blockIdentifier: BeaconId): Promise<bellatrix.BeaconBlock> {
        const id = this.toStringFromBeaconId(blockIdentifier);
        const response = await this.client.get(ROUTES.getBlock.replace('{block_id}', id));
        const block = ssz.bellatrix.BeaconBlock.fromJson(
            response.data.data.message
        ) as bellatrix.BeaconBlock;
        return block;
    }

    async getHeader(blockIdentifier: BeaconId): Promise<phase0.BeaconBlockHeader> {
        const id = this.toStringFromBeaconId(blockIdentifier);
        const response = await this.client.get(ROUTES.getHeader.replace('{block_id}', id));
        const header = ssz.phase0.BeaconBlockHeader.fromJson(response.data.data.header.message);
        return header;
    }

    async getFinalizedTelepathyUpdateInPeriod(period: number) {
        const startSlot = period * (32 * 256);
        const endSlot = startSlot + 32 * 256 - 1;

        let data: TelepathyUpdate;
        for (let i = endSlot; i >= startSlot; i--) {
            try {
                data = await this.getTelepathyUpdate(i);
            } catch (err) {
                console.error(`Failed to get update at slot, so moving onto next one`);
                continue;
            }
            if (3 * Number(computeBitSum(data.syncAggregate.syncCommitteeBits)) > 2 * 512) {
                return data;
            }
        }
    }

    async getTelepathyUpdate(blockIdentifier: BeaconId): Promise<TelepathyUpdate> {
        const currentHeader = await this.getHeader(blockIdentifier);
        const currentBlock = await this.getBlock(blockIdentifier);
        const currentBody = currentBlock.body;
        const currentState = await this.getState(currentHeader.slot);

        const attestedHeader = await this.getHeader(currentHeader.parentRoot);
        const attestedState = await this.getState(attestedHeader.slot);
        const attestedStateView = ssz.bellatrix.BeaconState.toView(attestedState);

        const finalizedHeader = await this.getHeader(attestedState.finalizedCheckpoint.root);
        const finalizedState = await this.getState(finalizedHeader.slot);
        const finalizedStateView = ssz.bellatrix.BeaconState.toView(finalizedState);
        const finalityBranchIndex = ssz.bellatrix.BeaconState.getPathInfo([
            'finalized_checkpoint',
            'root'
        ]).gindex;
        const finalityBranch = (
            createProof(attestedStateView.node, {
                type: ProofType.single,
                gindex: finalityBranchIndex
            }) as SingleProof
        ).witnesses;

        const currentSyncCommittee = currentState.currentSyncCommittee;
        const nextSyncCommitteeIndex = ssz.bellatrix.BeaconState.getPathInfo([
            'next_sync_committee'
        ]).gindex;
        const nextSyncCommittee = finalizedState.nextSyncCommittee;
        const nextSyncCommitteeBranch = (
            createProof(finalizedStateView.node, {
                type: ProofType.single,
                gindex: nextSyncCommitteeIndex
            }) as SingleProof
        ).witnesses;

        const syncAggregate = currentBody.syncAggregate;
        const genesisValidatorsRoot = currentState.genesisValidatorsRoot;
        const forkVersion =
            Math.floor(currentState.slot / 32) < currentState.fork.epoch
                ? currentState.fork.previousVersion
                : currentState.fork.currentVersion;

        const executionStateRootAndBranch = await this.getExecutionStateRootProof(
            finalizedHeader.slot
        );
        const executionStateRoot = executionStateRootAndBranch.root;
        const executionStateBranch = executionStateRootAndBranch.branch;

        return {
            attestedHeader,
            currentSyncCommittee,
            nextSyncCommittee,
            nextSyncCommitteeBranch,
            finalizedHeader,
            finalityBranch,
            syncAggregate,
            genesisValidatorsRoot,
            forkVersion,
            executionStateRoot,
            executionStateBranch
        };
    }

    async getExecutionStateRootProof(blockIdentifier: BeaconId) {
        const id = this.toStringFromBeaconId(blockIdentifier);
        const block = await this.getBlock(id);
        const view = ssz.bellatrix.BeaconBlockBody.toView(block.body as bellatrix.BeaconBlockBody);
        const proof = createProof(view.node, {
            type: ProofType.single,
            gindex: BigInt(402)
        }) as SingleProof;
        return { root: proof.leaf, branch: proof.witnesses };
    }

    async getReceiptsRootProof(srcBlockId: BeaconId, targetBlockId: BeaconId) {
        // console.log('Getting receipts root proof');
        // Given a source block and target block, generate a proof for the target block's
        // receipts root against the source slot
        const srcId = this.toStringFromBeaconId(srcBlockId);
        const targetId = this.toStringFromBeaconId(targetBlockId);
        // console.log('Getting source state');
        const srcState = await this.getState(srcId);
        // console.log('Getting target state');
        const targetState = await this.getState(targetId);
        const srcView = ssz.bellatrix.BeaconState.toView(srcState as bellatrix.BeaconState);
        const targetView = ssz.bellatrix.BeaconState.toView(targetState as bellatrix.BeaconState);
        const srcSlot = srcState.slot;
        const targetSlot = targetState.slot;

        const srcHeader = await this.getHeader(srcId);
        const srcHeaderView = ssz.phase0.BeaconBlockHeader.toView(
            srcHeader as phase0.BeaconBlockHeader
        );

        let receiptsRootProof;
        let receiptsRoot;
        let gindex;
        if (srcSlot == targetSlot) {
            const receiptGindex = ssz.bellatrix.BeaconState.getPathInfo([
                'latestExecutionPayloadHeader',
                'receiptsRoot'
            ]).gindex;
            const receiptProof = createProof(targetView.node, {
                type: ProofType.single,
                gindex: receiptGindex
            }) as SingleProof;
            receiptsRootProof = receiptProof.witnesses.map((l) => toHexString(l));
            receiptsRoot = toHexString(receiptProof.leaf);
            gindex = receiptGindex;
        } else if (srcSlot - targetSlot < 8192) {
            // Verify against the latest header root instead of just the state root
            // console.log('The target slot is within 8192 slots of src slot');
            // They are sufficiently close together, within 27 hours
            const headerGindex = ssz.phase0.BeaconBlockHeader.getPathInfo(['stateRoot']).gindex;
            const headerProof = createProof(srcHeaderView.node, {
                type: ProofType.single,
                gindex: headerGindex
            }) as SingleProof;
            const stateRootGindex = ssz.bellatrix.BeaconState.getPathInfo([
                'stateRoots',
                targetSlot % this.SLOTS_PER_HISTORICAL_ROOT
            ]).gindex;
            const proof = createProof(srcView.node, {
                type: ProofType.single,
                gindex: stateRootGindex
            }) as SingleProof;
            const receiptGindex = ssz.bellatrix.BeaconState.getPathInfo([
                'latestExecutionPayloadHeader',
                'receiptsRoot'
            ]).gindex;
            const receiptProof = createProof(targetView.node, {
                type: ProofType.single,
                gindex: receiptGindex
            }) as SingleProof;
            const concatGindex = concatGindices([stateRootGindex, receiptGindex]);
            receiptsRootProof = receiptProof.witnesses.concat(proof.witnesses);
            receiptsRootProof = receiptsRootProof.map((l) => toHexString(l));
            receiptsRoot = toHexString(receiptProof.leaf);
            gindex = concatGindex;
        } else {
            throw Error(
                'Have not implemented receipt proof generation for blocks that are too far apart'
            );
        }
        return { receiptsRootProof, receiptsRoot, gindex };
    }

    async getFinalityUpdate(): Promise<altair.LightClientFinalityUpdate> {
        const response = await this.client.get(ROUTES.getFinalityUpdate);
        const finalityUpdate = ssz.altair.LightClientFinalityUpdate.fromJson(
            response.data.data
        ) as altair.LightClientFinalityUpdate;
        return finalityUpdate;
    }

    async getUpdates(period: number, count: number): Promise<altair.LightClientUpdate[]> {
        const response = await this.client.get(ROUTES.getUpdates, {
            params: {
                start_period: period,
                count: count
            }
        });

        if (response.data.length !== count) {
            throw Error(`Expected ${count} updates, got ${response.data.length} `);
        }

        return response.data.map((update: UnparsedResponse) => {
            return ssz.altair.LightClientUpdate.fromJson(update.data) as altair.LightClientUpdate;
        });
    }

    async getGenesis(): Promise<phase0.Genesis> {
        const response = await this.client.get(ROUTES.getGenesis);
        return ssz.phase0.Genesis.fromJson(response.data.data);
    }

    async getBlockNumberFromSlot(blockIdentifier: BeaconId): Promise<number> {
        const block = await this.getBlock(blockIdentifier);
        return block.body.executionPayload.blockNumber;
    }

    async getApproxSlotOffset(slot: number, offset: number): Promise<bellatrix.BeaconBlock> {
        let target = slot - offset;
        while (true) {
            try {
                const block = await this.getBlock(target);
                return block;
            } catch (e) {
                target -= 1;
            }
        }
    }
}
