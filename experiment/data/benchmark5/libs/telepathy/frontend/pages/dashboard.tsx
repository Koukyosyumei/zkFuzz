import { useEffect, useState } from "react";
import type { NextPage } from "next";
import { useTheme } from "next-themes";
import Head from "next/head";
import Navbar from "../components/Navbar";
import Footer from "../components/Footer";
import Events from "../components/Events";
import { GNOSIS, GOERLI, OPTIMISM, POLYGON, AVALANCHE } from "../lib/Networks";
import { useEvents, useLCEventBlockNumbers, useDepositStatus } from "../hooks/events";
import { useAccount, useBalance, useContractRead } from "wagmi";
import { ethers } from "ethers";
import Instructions from "../components/Instructions";
import Modal from "../components/Alert";
import useMobileDetect from "../hooks/mobile";
import fancyToast from "../lib/clickableToast";
import toast, { Toaster } from "react-hot-toast";
import { useConfig } from "../context/config";
import axios from "axios";
import NetworkSelector from "../components/NetworkSelector";

const Home: NextPage = () => {
  const { config } = useConfig();

  const { theme, setTheme } = useTheme();
  useEffect(() => {
    setTheme("dark");
    setTheme("dark");
  }, []);

  const [mobileAlert, setMobileAlert] = useState(false);
  const { isMobile } = useMobileDetect();
  useEffect(() => {
    if (isMobile()) {
      setMobileAlert(true);
    }
  }, []);

  useEffect(() => {
    if (!isMobile()) {
      fancyToast({
        text: "Our relayer will automatically send a withdrawal for any pending deposits. Expect to wait between 12-24 mins for Ethereum finality and proof generation.",
        id: "dashboard",
      });
    }
  }, []);

  const [depositNetwork, setDepositNetwork] = useState(GNOSIS);

  const { address: wagmiAddress } = useAccount();
  const [address, setAddress] = useState<string | undefined>(undefined);
  useEffect(() => {
    setAddress(wagmiAddress);
  }, [wagmiAddress]);

  const { events: lightclientEvents } = useEvents({
    addressOrName: config?.address(depositNetwork.name.toLowerCase(), "LightClient"),
    abi: config?.abi("LightClient"),
    eventName: "HeadUpdate",
    contract: config?.contract(depositNetwork.name.toLowerCase(), "LightClient"),
    provider: config?.provider(depositNetwork.name.toLowerCase()),
    offsetBlock: Math.floor(86400 / depositNetwork.blockTime),
  });

  const consensusClient = axios.create({
    baseURL: config?.consensusRpc("goerli"),
    responseType: "json",
    headers: { "Content-Type": "application/json", "Access-Control-Allow-Origin": "*" },
  });
  const { metadata } = useLCEventBlockNumbers(lightclientEvents, consensusClient);

  const { events: depositEvents } = useEvents({
    addressOrName: config?.address("goerli", "Deposit"),
    abi: config?.abi("Deposit"),
    eventName: "DepositEvent",
    contract: config?.contract("goerli", "Deposit"),
    provider: config?.provider("goerli"),
    offsetBlock: Math.floor(86400 / GOERLI.blockTime),
    filterArgs: [wagmiAddress],
    maxEvents: 10000, // TODO: using this is a bit of a hack
  });

  const { events: sentMessages } = useEvents({
    addressOrName: config?.address("goerli", "SourceAMB"),
    abi: config?.abi("SourceAMB"),
    eventName: "SentMessage",
    contract: config?.contract("goerli", "SourceAMB"),
    provider: config?.provider("goerli"),
    offsetBlock: Math.floor(86400 / GOERLI.blockTime),
    maxEvents: 10000, // TODO: using this is a bit of a hack
  });

  const { events: executedMessages } = useEvents({
    addressOrName: config?.address(depositNetwork.name.toLowerCase(), "TargetAMB"),
    abi: config?.abi("TargetAMB"),
    eventName: "ExecutedMessage",
    contract: config?.contract(depositNetwork.name.toLowerCase(), "TargetAMB"),
    provider: config?.provider(depositNetwork.name.toLowerCase()),
    offsetBlock: Math.floor(86400 / depositNetwork.blockTime),
    filterArgs: [], // TODO eventually include address as a filter here, has to [null, address]
    maxEvents: 10000, // TODO: using this is a bit of a hack
  });

  // Get the deposit event status from the light client events, withdraw events, and deposit events
  const { metadata: depositStatus } = useDepositStatus(
    // TODO we should probably move depositEvents.filter(...) into a new state filteredDepositEvents
    // that we use everywhere. But since we'll likely transition to using The Graph, it doesn't
    // seem necesary for now.
    depositEvents.filter((event) => event.args?.chainId == depositNetwork.chainId),
    sentMessages,
    executedMessages,
    lightclientEvents,
    metadata,
    depositNetwork
  );

  return (
    <div>
      <Toaster position="bottom-left" />
      <Head>
        <title>Succinct Demo</title>
        <meta name="description" content="Succinct Demo" />
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <Modal open={mobileAlert} setOpen={setMobileAlert} />

      <div className="bg-pink-500 text-white" style={{ height: "35px" }}>
        <div className="flex justify-center items-center mb-2">
          <div className="grotesk-regular mt-1">
            Warning: this website is in active development and can break at any time.
          </div>
        </div>
      </div>

      <div className="background-glow"></div>
      <main
        className="dark:text-white"
        // style={{ backgroundImage: theme === "light" ? 'url("/pixel-bg.png")' : 'url("pixel-bg-dark.png")' }}
      >
        <Navbar active="dashboard"></Navbar>
        <div className="my-10"></div>
        <div className="flex justify-center item-center">
          <div className="flex-col" style={{ maxWidth: "650px" }}>
            <Instructions
              text="Confused? Read more about how to interpret this dashboard."
              linkName="Instructions"
              link="https://blog.succinct.xyz/post/2022/10/18/demo-instructions/"
              color="pink"
            />
            <div className="flex justify-center item-center align-middle mb-5">
              <div className="flex-none">
                <div className="text-3xl mr-5 pt-1">Dashboard</div>
              </div>
              <div className="grow"></div>
              <NetworkSelector
                selectedNetwork={depositNetwork}
                networks={[GNOSIS, OPTIMISM, POLYGON, AVALANCHE]}
                setSelectedNetwork={setDepositNetwork}
              />
            </div>
            {/* {address && (
              <div>
                <div className="pt-3"></div>
                <div>{`Succinct Token Balance (Gnosis Chain): ${
                  retrievedBalance?.value.toString() ?? "loading..."
                }`}</div>
              </div>
            )} */}
            {/* Deposits */}
            <div className="py-2"></div>
            <div>{`Only showing deposits sent to ${depositNetwork.name}.`}</div>
            <div className="py-2"></div>
            {address ? (
              <Events
                network={GOERLI}
                events={depositEvents.filter((event) => event.args?.chainId == depositNetwork.chainId)}
                title={`Your Deposits (past 24 hours)`}
                description={config?.address("goerli", "Deposit")}
                args={["chainId", "recipient"]}
                metadata={depositStatus}
                metadataCols={["amount", "status"]}
                colsToName={{ amount: "amount", status: "Deposit Status" }}
              />
            ) : (
              <div>Connect wallet to see activity</div>
            )}
            {/* Light Client Events */}
            <div className="py-5"></div>
            <Events
              network={depositNetwork}
              events={lightclientEvents}
              title={"Recent Light Client Updates"}
              description={config?.address(depositNetwork.name.toLowerCase(), "LightClient")}
              args={[]}
              metadata={metadata}
              metadataCols={["eth1BlockNumber"]}
              colsToName={{ eth1BlockNumber: "Light Client Update Block (Goerli)" }}
            />
            {/* <div>
              {events.map((e, i) => (
                <div key={i}>{JSON.stringify(e)}</div>
              ))}
            </div> */}

            <Footer></Footer>
          </div>
        </div>
      </main>
    </div>
  );
};

export default Home;
