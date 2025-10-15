import { StepCircuit, ConsensusClient, CircuitConfig } from '@succinctlabs/telepathy-sdk';

const CONSENSUS_RPC_URL = 'https://prysm-goerli.succinct.xyz/';

async function main() {
    const client = new ConsensusClient(CONSENSUS_RPC_URL);
    const config: CircuitConfig = {
        witnessExecutablePath: '~/telepathy/circuits/build/step_cpp/step',
        proverExecutablePath: '~/rapidsnark/build/prover',
        proverKeyPath: '~/telepathy/circuits/build/step_cpp/p2.zkey'
    };
    const circuit = new StepCircuit(config);
    console.log('Calculating inputs for latest finalized slot...');
    const input = await circuit.calculateInputsForLatestSlot(client);
    console.log('Finished.');
    console.log('Calculating witness...');
    const witness = await circuit.calculateWitness(input, 'witness.wtns');
    console.log('Finished.');
    console.log('Generating proof...');
    const proof = await circuit.prove(witness, 'proof.json', 'public.json');
    console.log('Finished');
}

main();
