import { BigNumber } from 'ethers';
import { CircomProof } from '@succinctlabs/telepathy-sdk';
import { Groth16ProofStruct } from '@succinctlabs/telepathy-sdk/contracts';
import pino from 'pino';

export const logger = pino();

export function toGroth16ProofFromCircomProof(proof: CircomProof): Groth16ProofStruct {
    return {
        a: [BigNumber.from(proof.pi_a[0]), BigNumber.from(proof.pi_a[1])],
        b: [
            [BigNumber.from(proof.pi_b[0][1]), BigNumber.from(proof.pi_b[0][0])],
            [BigNumber.from(proof.pi_b[1][1]), BigNumber.from(proof.pi_b[1][0])]
        ],
        c: [BigNumber.from(proof.pi_c[0]), BigNumber.from(proof.pi_c[1])]
    };
}
