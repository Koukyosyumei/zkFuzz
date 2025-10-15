import merge from "lodash.merge";
import "../styles/globals.css";
import type { AppProps } from "next/app";
import "@rainbow-me/rainbowkit/styles.css";
import { Chain, getDefaultWallets, RainbowKitProvider, darkTheme, Theme } from "@rainbow-me/rainbowkit";
import { chain, configureChains, createClient, WagmiConfig } from "wagmi";
import { alchemyProvider } from "wagmi/providers/alchemy";
import { publicProvider } from "wagmi/providers/public";
import { AVALANCHE, GNOSIS } from "../lib/Networks";
import { ThemeProvider } from "next-themes";
import { ConfigContext, ConfigManager } from "../context/config";
import { useEffect, useState } from "react";
import { ethers } from "ethers";
import { Abi } from "abitype/zod";
import { Abi as AbiType } from "abitype";

const myTheme = merge(darkTheme(), {
  colors: {
    accentColor: "rgb(24 24 27)",
    accentColorForeground: "white",
  },
  fonts: {
    body: "GroteskRegular",
  },
} as Theme);

function MyApp({ Component, pageProps }: AppProps) {
  const gnosisChain: Chain = {
    id: 100,
    name: "Gnosis",
    network: "gnosis",
    iconUrl: GNOSIS.imgUrl,
    nativeCurrency: {
      decimals: 18,
      name: "xdai",
      symbol: "XDAI",
    },
    rpcUrls: {
      default: "https://rpc.ankr.com/gnosis",
    },
    blockExplorers: {
      default: {
        name: "Blockscout",
        url: "https://blockscout.com/xdai/mainnet/",
      },
    },
    testnet: false,
  };

  const avalanche: Chain = {
    id: 43114,
    name: "Avalanche",
    network: "avalanche",
    iconUrl: AVALANCHE.imgUrl,
    nativeCurrency: {
      decimals: 18,
      name: "avax",
      symbol: "AVAX",
    },
    rpcUrls: {
      default: "https://rpc.ankr.com/avalanche",
    },
    blockExplorers: {
      default: {
        name: "Snowtrace",
        url: "https://snowtrace.io/",
      },
    },
    testnet: false,
  };

  const { chains, provider } = configureChains(
    [chain.goerli, gnosisChain, chain.optimism, chain.polygon, avalanche],
    [alchemyProvider({ apiKey: "3gh9tkSyQi0IULwnCqzrEwqx0t4qk18q" }), publicProvider()]
  );

  const { connectors } = getDefaultWallets({
    appName: "Succinct Demo",
    chains: [chains[0]], // We only want to show Goerli as a connection option
  });

  const wagmiClient = createClient({
    autoConnect: true,
    connectors,
    provider,
  });

  const [config, setConfig] = useState<ConfigManager | null>(null);

  useEffect(() => {
    async function wrapper() {
      const devOrProd = process.env.USE_MOCK === "true" ? "dev" : "prod";
      const addressResp = await fetch(`/config/address.${devOrProd}.json`);
      const addressConfig = await addressResp.json();
      const chainIdResp = await fetch("/config/chainId.json");
      const chainIds = await chainIdResp.json();
      const rpcResp = await fetch("/config/rpc.json");
      const rpcConfig = await rpcResp.json();
      const consensusRpcResp = await fetch("/config/consensusRpc.json");
      const consensusRpcConfig = await consensusRpcResp.json();
      const abis: Record<string, AbiType> = {};
      for (const key of [
        "Deposit",
        "IERC20",
        "InfiniteMintSuccincts",
        "LightClient",
        "SourceAMB",
        "TargetAMB",
        "Withdraw",
      ]) {
        const abiResp = await fetch(`/abi/${key}.abi.json`);
        const abi = await abiResp.json();
        const abiConverted = Abi.parse(abi);
        abis[key] = abiConverted;
      }
      const config = new ConfigManager(chainIds, rpcConfig, consensusRpcConfig, addressConfig, abis);
      setConfig(config);
    }
    wrapper();
  }, []);

  return (
    <ConfigContext.Provider value={{ config: config }}>
      <WagmiConfig client={wagmiClient}>
        <RainbowKitProvider theme={myTheme} chains={chains}>
          <ThemeProvider attribute="class">
            <Component {...pageProps} />
          </ThemeProvider>
        </RainbowKitProvider>
      </WagmiConfig>
    </ConfigContext.Provider>
  );
}

export default MyApp;
