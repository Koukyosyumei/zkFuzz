#! node
// Above shebang is used for making this an executable CLI
// https://stackoverflow.com/questions/23298295/how-to-make-a-shell-executable-node-file-using-typescript/62672365#62672365

import fs from 'fs';

type TransactionInfo = {
    transactionType: string;
    contractName: string;
    contractAddress: string;
};

function parseMaps(maps: string[]): Map<string, string> {
    const res = new Map<string, string>();
    if (!maps) {
        return res;
    }
    if (typeof maps == 'string') {
        maps = [maps];
    }
    for (const map of maps) {
        const [k, v] = map.split('=');
        if (!k || !v) {
            throw Error("Invalid map format. Expected 'k=v', got " + map);
        }
        res.set(k, v);
    }
    return res;
}

export async function parseFoundryRunJson(path: string, maps: any = undefined) {
    const mapping = parseMaps(maps);
    const runInfo = JSON.parse(fs.readFileSync(path).toString());
    // TODO add a typesafe deserialize here
    const deployments = runInfo.deployments;
    if (!deployments) {
        throw Error('No deployments in runInfo');
    }

    for (const deployment of deployments) {
        const transactions: TransactionInfo[] = deployment.transactions;
        let chainId = deployment.chain;
        if (!transactions || !chainId) {
            throw Error('Either or both of transaction and chainId are missing');
        }
        chainId = parseInt(chainId);
        console.log(`[${chainId}]`);
        for (const tx of transactions) {
            const type = tx.transactionType;
            const contractName = tx.contractName;
            const contractAddress = tx.contractAddress;
            if (type == 'CREATE' || type == 'CREATE2') {
                const mappedContractName = mapping.get(contractName) ?? contractName;
                const envName = `${mappedContractName}`;
                console.log(`${envName} = "${contractAddress}"`);
            }
        }
        console.log('');
    }
}
