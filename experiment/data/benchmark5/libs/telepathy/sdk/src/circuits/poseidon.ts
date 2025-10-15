// @ts-ignore
import circom_tester from 'circom_tester';
import fs from 'fs';
import { CircomSerializer } from './serializer';
import path from 'path';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

/**
 * This function computes the poseidon root of the sync committee as according to what we do inside
 * our circuits for the lightclient. This function will actually compile the circuit to WASM
 * to compute the output. It only compiles for the first run of the function (takes a bit).
 */
export async function poseidonSyncCommittee(pubkeys: Uint8Array[]): Promise<bigint> {
    const circuitName = 'poseidon_sync_committee';
    const __dirname = dirname(fileURLToPath(import.meta.url));
    const wasm_tester = circom_tester.wasm;

    const currentDir = process.cwd();
    const targetPathCircom = path.join(__dirname, `../../src/circuits/${circuitName}.circom`);
    const relativePathCircom = path.relative(currentDir, targetPathCircom);

    let recompile = false;
    const targetPathWasm = path.join(
        __dirname,
        `../../src/circuits/${circuitName}_js/${circuitName}.wasm`
    );
    if (!fs.existsSync(targetPathWasm)) {
        recompile = true;
    }

    const targetPathFolder = path.join(__dirname, '../../src/circuits/');

    // If you see an error [Error: ENOENT: no such file or directory, open '.../sdk/src/circuits/poseidon_sync_committee_js/poseidon_sync_committee.wasm']
    // then likely your circom compilation failed
    const circuit = await wasm_tester(relativePathCircom, {
        recompile,
        output: targetPathFolder
    });

    if (recompile) {
        fs.rmSync(path.join(targetPathFolder, `${circuitName}.r1cs`));
        fs.rmSync(path.join(targetPathFolder, `${circuitName}.sym`));
    }

    const cs = new CircomSerializer();
    cs.writeG1PointsAsBigInt('pubkeys', pubkeys);
    const input = cs.flush();
    const witness = await circuit.calculateWitness(input);
    return witness[1];
}
