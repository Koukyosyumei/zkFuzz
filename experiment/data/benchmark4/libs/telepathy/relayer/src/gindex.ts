import { altair, bellatrix, phase0, ssz } from '@lodestar/types';
import {
    ProofType,
    createProof,
    SingleProof,
    concatGindices
} from '@chainsafe/persistent-merkle-tree';

function compute(targetSlot: number) {
    // BigInt(gindices.reduce((acc, gindex) => acc + gindex.toString(2).slice(1), "0b1"));
    let index = 11;
    index = 2 ** 5 * index + 6;
    index = index * 8192 + (targetSlot % 8192);
    index = index * 32 + 24;
    index = index * 16 + 3;
    return index;
}

function main() {
    const targetSlot = 5;
    const headerGindex = ssz.phase0.BeaconBlockHeader.getPathInfo(['stateRoot']).gindex;
    const stateRootGindex = ssz.bellatrix.BeaconState.getPathInfo([
        'stateRoots',
        targetSlot % 8192
    ]).gindex;
    const receiptGindex = ssz.bellatrix.BeaconState.getPathInfo([
        'latestExecutionPayloadHeader',
        'receiptsRoot'
    ]).gindex;

    console.log('COMPUTED', compute(targetSlot));
    console.log('CONCAT', concatGindices([headerGindex, stateRootGindex, receiptGindex]));
}

main();

function sameSlot() {
    const headerGindex = ssz.phase0.BeaconBlockHeader.getPathInfo(['stateRoot']).gindex;
    const receiptGindex = ssz.bellatrix.BeaconState.getPathInfo([
        'latestExecutionPayloadHeader',
        'receiptsRoot'
    ]).gindex;
    console.log(headerGindex);
    console.log(receiptGindex);
    console.log('CONCAT', concatGindices([headerGindex, receiptGindex]));
    // index = 32 + 24;
    // index = index * 16 + 3;
}

const HISTORICAL_ROOTS_LIMIT = 16777216;
const SLOTS_PER_HISTORICAL_ROOT = 8192;

function farAwaySlotCompute(targetSlot: number) {
    let index = 0;
    index = 32 + 7;
    index = index * 2 + 0;
    index = index * HISTORICAL_ROOTS_LIMIT + Math.floor(targetSlot / SLOTS_PER_HISTORICAL_ROOT);
    index = index * 2 + 1;
    index = index * SLOTS_PER_HISTORICAL_ROOT + (targetSlot % SLOTS_PER_HISTORICAL_ROOT);
    index = index * 32 + 24;
    index = index * 16 + 3;
    return index;
}

function farAwaySlot() {
    const targetSlot = 5;
    const headerGindex = ssz.phase0.BeaconBlockHeader.getPathInfo(['stateRoot']).gindex;
    const historicalRootsGIndex = ssz.bellatrix.BeaconState.getPathInfo([
        'historicalRoots',
        Math.floor(targetSlot / SLOTS_PER_HISTORICAL_ROOT)
    ]).gindex;
    const historicalBatchGIndex = ssz.bellatrix.HistoricalBatch.getPathInfo([
        'state_roots',
        targetSlot % SLOTS_PER_HISTORICAL_ROOT
    ]).gindex;
    const receiptGindex = ssz.bellatrix.BeaconState.getPathInfo([
        'latestExecutionPayloadHeader',
        'receiptsRoot'
    ]).gindex;
    console.log(concatGindices([historicalRootsGIndex, historicalBatchGIndex, receiptGindex]));
    console.log(farAwaySlotCompute(targetSlot));
}

farAwaySlot();
