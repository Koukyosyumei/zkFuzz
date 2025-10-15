import path from 'path';
import { exec } from 'child_process';
import fs, { rm } from 'fs';
import { CircomInput, stringifyCircomInput } from './serializer';
import util from 'util';
import crypto from 'crypto';

const execPromise = util.promisify(exec);

export type CircuitConfig = {
    witnessExecutablePath?: string;
    proverExecutablePath?: string;
    proverKeyPath?: string;
};

export type CircomWitness = string;
export type CircomProof = {
    pi_a: string[];
    pi_b: string[][];
    pi_c: string[];
};
export type CircomPublic = string[];

function getRandomId(): string {
    return crypto.randomBytes(16).toString('hex');
}

async function safeExec(command: string) {
    try {
        const { stdout, stderr } = await execPromise(command);
    } catch (error) {
        console.log(error);
        throw Error('Failed eo execute command');
    }
}

export class Circuit {
    witnessExecutablePath: string | undefined;
    proverExecutablePath: string | undefined;
    proverKeyPath: string | undefined;

    constructor(config: CircuitConfig = {}) {
        this.witnessExecutablePath = config.witnessExecutablePath;
        this.proverExecutablePath = config.proverExecutablePath;
        this.proverKeyPath = config.proverKeyPath;
    }

    async calculateWitness(input: CircomInput, witnessPath?: string): Promise<CircomWitness> {
        if (this.witnessExecutablePath == undefined) {
            throw Error('Witness executable path is not defined');
        }

        const uuid = getRandomId();
        const inputPath = `/tmp/input_${uuid}.json`;
        witnessPath = witnessPath == undefined ? `/tmp/witness_${uuid}.wtns` : witnessPath;

        fs.writeFileSync(inputPath, stringifyCircomInput(input));
        const command = `${this.witnessExecutablePath} ${inputPath} ${witnessPath}`;
        await safeExec(command);

        fs.rmSync(inputPath);
        return witnessPath;
    }

    async prove(witness: CircomWitness, proofPath?: string, publicPath?: string) {
        if (this.proverExecutablePath == undefined) {
            throw Error('Prover executable path is not defined');
        } else if (this.proverKeyPath == undefined) {
            throw Error('Prover zkey is not defined');
        }

        const uuid = getRandomId();
        const removeProof = proofPath == undefined;
        const removePublic = publicPath == undefined;
        proofPath = proofPath == undefined ? `/tmp/proof_${uuid}.json` : proofPath;
        publicPath = publicPath == undefined ? `/tmp/public_${uuid}.json` : publicPath;

        const command = `${this.proverExecutablePath} ${this.proverKeyPath} ${witness} ${proofPath} ${publicPath}`;
        await safeExec(command);
        const proof = JSON.parse(fs.readFileSync(proofPath).toString()) as CircomProof;
        const publicInputs = JSON.parse(fs.readFileSync(publicPath).toString()) as CircomPublic;

        if (witness.includes('/tmp/')) {
            fs.rmSync(witness);
        }
        if (removeProof) {
            fs.rmSync(proofPath);
        }
        if (removePublic) {
            fs.rmSync(publicPath);
        }
        try {
            fs.rmSync('./MyLogFile.log');
        } catch (err) {
            console.log(err);
        }

        return { proof, publicInputs };
    }
}
