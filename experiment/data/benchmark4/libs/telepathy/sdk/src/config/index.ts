import toml from 'toml';
import fs from 'fs';
import config from 'dotenv';

const PREFIXES = {
    consensusRpc: 'CONSENSUS_RPC_{chainId}',
    rpc: 'RPC_{chainId}',
    etherscanApiKey: 'ETHERSCAN_API_KEY_{chainId}'
};
/**
 * Reads a configuation file from the given path and returns a Config object that is safely
 * read from environment variables.
 */
export class ConfigManager {
    _configPath: string;
    _config: any; // TODO make this not any
    _envPath: string;
    _envResult: config.DotenvConfigOutput;
    _chainIds: Record<string, number>;
    _consensusRPC: Record<string, string>;
    _rpc: Record<string, string>;
    _etherscanApiKey: Record<string, string>;
    _privateKey: string | undefined;
    _addressConfig: Record<string, Record<string, string>>;

    constructor(configPath: string, envPath: string, requirePrivateKey = true) {
        // This is the chain.toml
        this._configPath = configPath;
        const chainConfig = toml.parse(fs.readFileSync(configPath).toString());
        this._config = chainConfig;

        this._envPath = envPath;
        const envResult = config.config({ path: envPath });
        this._envResult = envResult;

        this._consensusRPC = {};
        this._rpc = {};
        this._etherscanApiKey = {};
        this._chainIds = {};
        for (const chainName of Object.keys(chainConfig)) {
            let chainId = chainConfig[chainName].chainId;
            if (!chainId) {
                throw Error(`Chain ${chainName} does not have a chainId`);
            }
            try {
                chainId = parseInt(chainId);
            } catch (e) {
                throw Error(`Chain ${chainName} has an invalid chainId ${chainId}`);
            }

            const castChainId = chainId as number;

            this._chainIds[chainName] = castChainId;

            const consensusRpc = PREFIXES.consensusRpc.replace('{chainId}', castChainId.toString());
            if (consensusRpc in process.env) {
                this._consensusRPC[chainName] = process.env[consensusRpc] as string;
            }

            const rpc = PREFIXES.rpc.replace('{chainId}', castChainId.toString());
            if (rpc in process.env) {
                this._rpc[chainName] = process.env[rpc] as string;
            }

            const etherscanApiKey = PREFIXES.etherscanApiKey.replace(
                '{chainId}',
                castChainId.toString()
            );
            if (etherscanApiKey in process.env) {
                this._etherscanApiKey[chainName] = process.env[etherscanApiKey] as string;
            }
        }

        if ('PRIVATE_KEY' in process.env) {
            this._privateKey = process.env['PRIVATE_KEY'];
        } else if (requirePrivateKey) {
            throw Error('No private key found in environment variables');
        }

        this._addressConfig = {};
    }

    consensusRpc(chainName: string): string {
        if (chainName in this._consensusRPC) {
            return this._consensusRPC[chainName];
        } else {
            throw Error(`No consensus RPC found for chain ${chainName}`);
        }
    }

    chainId(chainName: string): number {
        if (chainName in this._chainIds) {
            return this._chainIds[chainName];
        } else {
            throw Error(`No chainId found for chainName ${chainName}`);
        }
    }

    etherscanApiKey(chainName: string): string {
        if (chainName in this._etherscanApiKey) {
            return this._etherscanApiKey[chainName];
        } else {
            throw Error(`No etherscan API key found for chain ${chainName}`);
        }
    }

    rpc(chainName: string): string {
        if (chainName in this._rpc) {
            return this._rpc[chainName];
        } else {
            throw Error(`No RPC found for chain ${chainName}`);
        }
    }

    privateKey() {
        if (this._privateKey) {
            return this._privateKey;
        } else {
            throw Error('No private key found');
        }
    }

    getEnv(key: string): string {
        if (key in process.env) {
            return process.env[key] as string;
        } else {
            throw Error(`No environment variable found for key ${key}`);
        }
    }

    filterChains(key: string) {
        return Object.keys(this._config).filter((chainName) => {
            return key in this._config[chainName] && this._config[chainName][key];
        });
    }

    addAddressToml(tomlPath: string) {
        let tomlConfig;
        try {
            tomlConfig = toml.parse(fs.readFileSync(tomlPath).toString());
        } catch (e: any) {
            console.error(e);
        }
        for (const chainName of Object.keys(tomlConfig)) {
            this._addressConfig[chainName] = tomlConfig[chainName];
        }
    }

    address(chainName: string, key: string) {
        if (chainName in this._addressConfig && key in this._addressConfig[chainName]) {
            return this._addressConfig[chainName][key];
        } else {
            throw Error(`No address found for chain ${chainName} and key ${key}`);
        }
    }
}
