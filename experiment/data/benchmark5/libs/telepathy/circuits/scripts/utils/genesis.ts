import {
    ConsensusClient,
    poseidonSyncCommittee,
    toLittleEndianFromBigInt
} from '@succinctlabs/telepathy-sdk';
import { toHexString } from '@chainsafe/ssz';

async function main() {
    const consensusRpcUrl = process.argv[2];
    const client = new ConsensusClient(consensusRpcUrl);
    const update = await client.getTelepathyUpdate('finalized');
    const genesis = await client.getGenesis();
    console.log('Genesis Validators Root:', toHexString(update.genesisValidatorsRoot));
    console.log('Genesis Time:', genesis.genesisTime);
    const syncCommitteePoseidon = await poseidonSyncCommittee(update.currentSyncCommittee.pubkeys);
    console.log(
        'Current Sync Committee Period',
        Math.floor(Math.floor(update.attestedHeader.slot / 32) / 256)
    );
    console.log(
        'Current Sync Committee Poseidon:',
        toHexString(toLittleEndianFromBigInt(syncCommitteePoseidon))
    );
}

main();
