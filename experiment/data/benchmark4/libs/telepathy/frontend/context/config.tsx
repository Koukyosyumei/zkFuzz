import { createContext, useContext, useState, ReactNode } from "react";
import { ethers } from "ethers";
import { Abi } from "abitype";
// TODO maybe unite this with the telepathy-sdk once we figure out how to use it in this
// workspace
export class ConfigManager {
  _chainIds: Record<string, number>;
  _rpc: Record<string, string>;
  _consensusRpc: Record<string, string>;
  _addressConfig: Record<string, Record<string, string>>;
  _abis: Record<string, Abi>;
  _providers: Record<string, ethers.providers.JsonRpcProvider>;
  _contracts: Record<string, Record<string, ethers.Contract>>;

  constructor(
    chainIds: Record<string, number>,
    rpc: Record<string, string>,
    consensusRpc: Record<string, string>,
    addressConfig: Record<string, Record<string, string>>,
    abis: Record<string, Abi>
  ) {
    this._chainIds = chainIds;
    this._rpc = rpc;
    this._consensusRpc = consensusRpc;
    this._addressConfig = addressConfig;
    this._abis = abis;
    this._providers = {};
    this._contracts = {};

    if (this._consensusRpc["goerli"]) {
      // TODO this is a hack because our consensus RPCs have CORs issues
      this._consensusRpc["goerli"] = "https://lodestar-goerli.chainsafe.io/";
    }
  }

  chainId(chainName: string): number {
    chainName = chainName.toLowerCase();
    if (chainName in this._chainIds) {
      return this._chainIds[chainName];
    } else {
      throw Error(`No chainId found for chainName ${chainName}`);
    }
  }

  rpc(chainName: string): string {
    chainName = chainName.toLowerCase();
    if (chainName in this._rpc) {
      return this._rpc[chainName];
    } else {
      throw Error(`No RPC found for chain ${chainName}`);
    }
  }

  consensusRpc(chainName: string): string {
    chainName = chainName.toLowerCase();
    if (chainName in this._rpc) {
      return this._consensusRpc[chainName];
    } else {
      throw Error(`No consensus RPC found for chain ${chainName}`);
    }
  }

  address(chainName: string, key: string) {
    chainName = chainName.toLowerCase();
    if (chainName in this._addressConfig && key in this._addressConfig[chainName]) {
      return this._addressConfig[chainName][key];
    } else {
      throw Error(`No address found for chain ${chainName} and key ${key}`);
    }
  }

  provider(chainName: string): ethers.providers.JsonRpcProvider {
    chainName = chainName.toLowerCase();
    if (chainName in this._providers) {
      return this._providers[chainName];
    } else {
      const provider = new ethers.providers.JsonRpcProvider(this.rpc(chainName));
      this._providers[chainName] = provider;
      return provider;
    }
  }

  contract(chainName: string, contractName: string) {
    chainName = chainName.toLowerCase();
    if (chainName in this._contracts && contractName in this._contracts[chainName]) {
      return this._contracts[chainName][contractName];
    }
    const address = this.address(chainName, contractName);
    const abi = this._abis[contractName] as ethers.ContractInterface;
    const contract = new ethers.Contract(address, abi, this.provider(chainName));
    if (!(chainName in this._contracts)) {
      this._contracts[chainName] = {};
    }
    this._contracts[chainName][contractName] = contract;
    return contract;
  }

  abi(contractName: string) {
    return this._abis[contractName];
  }
}

type configContextType = {
  config: ConfigManager | null;
};

const defaultValues: configContextType = {
  config: null,
};

export const ConfigContext = createContext<configContextType>(defaultValues);

export function useConfig() {
  return useContext(ConfigContext);
}
