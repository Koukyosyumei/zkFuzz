import * as ssz from '../consensus/ssz';
import { CircomInput, CircomSerializer } from './serializer';
import { Circuit } from './circuit';
import { ConsensusClient, TelepathyUpdate } from '../consensus/client';
import { poseidonSyncCommittee } from './poseidon';

export class RotateCircuit extends Circuit {
    async calculateInputs(update: TelepathyUpdate): Promise<CircomInput> {
        const finalizedHeaderRoot = ssz.hashBeaconBlockHeader(update.finalizedHeader);
        const finalizedHeader = update.finalizedHeader;

        const nextSyncCommittee = update.nextSyncCommittee;
        const nextSyncCommitteeSSZ = ssz.hashSyncCommittee(nextSyncCommittee);
        const nextSyncCommitteeBranch = update.nextSyncCommitteeBranch;
        const nextSyncCommitteePoseidon = await poseidonSyncCommittee(
            update.nextSyncCommittee.pubkeys
        );

        const cs = new CircomSerializer();
        cs.writeBytes32('finalizedHeaderRoot', finalizedHeaderRoot);
        cs.writeBeaconBlockHeader('finalized', finalizedHeader);

        cs.writeG1PointsAsBytes('pubkeysBytes', nextSyncCommittee.pubkeys);
        cs.writeG1PointAsBytes('aggregatePubkeyBytes', nextSyncCommittee.aggregatePubkey);

        cs.writeG1PointsAsBigInt('pubkeysBigInt', nextSyncCommittee.pubkeys);
        cs.writeG1PointAsBigInt('aggregatePubkeyBigInt', nextSyncCommittee.aggregatePubkey);

        cs.writeBytes32('syncCommitteeSSZ', nextSyncCommitteeSSZ);
        cs.writeMerkleBranch('syncCommitteeBranch', nextSyncCommitteeBranch);
        cs.writeBigInt('syncCommitteePoseidon', nextSyncCommitteePoseidon);

        return cs.flush();
    }

    async calculateInputsForLatestPeriod(client: ConsensusClient): Promise<CircomInput> {
        const update = await client.getTelepathyUpdate('finalized');
        const inputs = await this.calculateInputs(update);
        return inputs;
    }
}
