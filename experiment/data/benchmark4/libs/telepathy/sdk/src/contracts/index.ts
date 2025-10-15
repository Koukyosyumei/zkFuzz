import ethers from 'ethers';
import { BigNumber } from 'ethers';
import type { Provider } from '@ethersproject/providers';
import axios from 'axios';
import { safeWaitForTx } from './safe';

import {
    LightClient__factory,
    SourceAMB__factory,
    TargetAMB__factory,
    LightClientMock__factory
} from './typechain';
import { LightClient, SourceAMB, TargetAMB, LightClientMock } from './typechain';
import {
    LightClientStepStruct,
    LightClientRotateStruct,
    Groth16ProofStruct
} from './typechain/LightClient.sol/LightClient';

// Export this for convenience
export {
    LightClientMock,
    LightClientMock__factory,
    SourceAMB,
    TargetAMB,
    LightClient,
    LightClient__factory,
    LightClientStepStruct,
    LightClientRotateStruct,
    Groth16ProofStruct,
    safeWaitForTx
};

export enum ContractTypeEnum {
    LightClient = 'LightClient',
    SourceAMB = 'SourceAMB',
    TargetAMB = 'TargetAMB',
    LightClientMock = 'LightClientMock'
}

type ContractReturnType = {
    [ContractTypeEnum.LightClient]: LightClient;
    [ContractTypeEnum.SourceAMB]: SourceAMB;
    [ContractTypeEnum.TargetAMB]: TargetAMB;
    [ContractTypeEnum.LightClientMock]: LightClientMock;
};

export class Contracts {
    providers: Record<number, ethers.providers.JsonRpcProvider>; // chainId to provider
    signers: Record<number, ethers.Signer>; // chainId to signer
    contracts: Record<number, Record<ContractTypeEnum, ContractReturnType[ContractTypeEnum]>>;

    constructor() {
        this.providers = {};
        this.signers = {};
        this.contracts = {};
    }

    addProvider(chainId: number, rpcUrl: string) {
        const provider = new ethers.providers.JsonRpcProvider(rpcUrl);
        this.providers[chainId] = provider;
    }

    addSigner(chainId: number, privateKey: string, rpcUrl: string | undefined = undefined) {
        let provider;
        if (!rpcUrl) {
            provider = this.providers[chainId];
            if (!provider) {
                throw Error(
                    `No provider for chainId ${chainId} already existing, you must provide rpcUrl`
                );
            }
        } else {
            provider = new ethers.providers.JsonRpcProvider(rpcUrl);
            this.providers[chainId] = provider;
        }
        const signer = new ethers.Wallet(privateKey, provider);
        this.signers[chainId] = signer;
    }

    addContract(
        chainId: number,
        address: string,
        contractType: ContractTypeEnum,
        requireSigner = false
    ) {
        const provider = this.providers[chainId];
        if (!provider) {
            throw Error(`No provider for chainId ${chainId}, cannot add contract`);
        }
        let signerOrProvider: ethers.Signer | Provider = provider;
        if (requireSigner) {
            signerOrProvider = this.signers[chainId];
            if (!signerOrProvider) {
                throw Error(
                    `No signer for chainId ${chainId}, cannot add contract with requireSigner`
                );
            }
        }
        if (!this.contracts[chainId]) {
            this.contracts[chainId] = {} as Record<
                ContractTypeEnum,
                ContractReturnType[ContractTypeEnum]
            >;
        }
        switch (contractType) {
            case 'LightClient':
                this.contracts[chainId][contractType] = LightClient__factory.connect(
                    address,
                    signerOrProvider
                );
                break;
            case 'SourceAMB':
                this.contracts[chainId][contractType] = SourceAMB__factory.connect(
                    address,
                    signerOrProvider
                );
                break;
            case 'TargetAMB':
                this.contracts[chainId][contractType] = TargetAMB__factory.connect(
                    address,
                    signerOrProvider
                );
                break;
            case 'LightClientMock':
                this.contracts[chainId][contractType] = LightClientMock__factory.connect(
                    address,
                    signerOrProvider
                );
                break;
        }
        // Return what you created
        return this.contracts[chainId][contractType];
    }

    getContract(
        chainId: number,
        contractType: ContractTypeEnum,
        throwIfMissing = true
    ): ContractReturnType[ContractTypeEnum] {
        const res = this.contracts[chainId][contractType];
        if (!res && throwIfMissing) {
            throw Error(`No contract of type ${contractType} for chainId ${chainId}`);
        }
        return res;
    }

    static async getExtraOptions(chainId: number) {
        //  https://github.com/ethers-io/ethers.js/issues/2828#issuecomment-1073423774
        const extraOptions: {
            maxFeePerGas?: BigNumber;
            maxPriorityFeePerGas?: BigNumber;
            gasLimit?: BigNumber;
        } = {};
        if (chainId == 137) {
            // Polygon is annoying and ethers doesn't play nice with it
            let maxFeePerGas = ethers.BigNumber.from(40000000000); // fallback to 40 gwei
            let maxPriorityFeePerGas = ethers.BigNumber.from(40000000000); // fallback to 40 gwei
            try {
                const { data } = await axios.get('https://gasstation-mainnet.matic.network/v2');
                maxFeePerGas = ethers.utils.parseUnits(Math.ceil(data.fast.maxFee) + '', 'gwei');
                maxPriorityFeePerGas = ethers.utils.parseUnits(
                    Math.ceil(data.fast.maxPriorityFee) + '',
                    'gwei'
                );
            } catch {
                // ignore
            }
            extraOptions.maxFeePerGas = maxFeePerGas;
            extraOptions.maxPriorityFeePerGas = maxPriorityFeePerGas;
        } else if (chainId == 43114) {
            // On avalanche the max gas is slightly lower limit
            return {
                gasLimit: BigNumber.from(7500000)
            };
        } else {
            return {
                gasLimit: BigNumber.from(10000000)
            };
        }
        return extraOptions;
    }
}
