import { useEffect, useState } from "react";
import type { NextPage } from "next";
import { useTheme } from "next-themes";
import Head from "next/head";
import Navbar from "../components/Navbar";
import Footer from "../components/Footer";
import TokenSelector from "../components/TokenSelector";
import NetworkSelector from "../components/NetworkSelector";
import DownArrow from "../components/DownArrow";
import { tokens, USDC } from "../lib/Tokens";
import { networks, GNOSIS, GOERLI, POLYGON, OPTIMISM, AVALANCHE } from "../lib/Networks";
import FloatInput from "../components/FloatInput";
import { useAccount, useBalance, useContractWrite, usePrepareContractWrite, useContractRead } from "wagmi";
import TransactionButton from "../components/TransactionButton";
import Instructions from "../components/Instructions";
import toast, { Toaster } from "react-hot-toast";
import useTransactionToast from "../hooks/transactionToast";
import Link from "next/link";
import useMobileDetect from "../hooks/mobile";
import Modal from "../components/Alert";
import fancyToast from "../lib/clickableToast";
import { utils, BigNumber } from "ethers";
import { useConfig } from "../context/config";

const Home: NextPage = () => {
  const { config } = useConfig();

  const { theme, setTheme } = useTheme();
  useEffect(() => {
    setTheme("dark");
  }, []);

  const [mobileAlert, setMobileAlert] = useState(false);
  const { isMobile } = useMobileDetect();
  useEffect(() => {
    if (isMobile()) {
      setMobileAlert(true);
    }
  }, []);

  // Have to do this workaround for nextjs hydration error
  const { address: wagmiAddress } = useAccount();
  const [address, setAddress] = useState<string | undefined>("");
  useEffect(() => {
    setAddress(wagmiAddress);
    if (!wagmiAddress) {
      fancyToast({ text: "Connect your wallet to get started", id: "connectWallet" });
    } else {
      toast.dismiss("connectWallet");
    }
    if (wagmiAddress !== address && address && wagmiAddress) {
      // They changed their connected address, dismiss all previous notifications
      toast.dismiss();
    }
  }, [wagmiAddress]);

  const [selectedToken, setSelectedToken] = useState(USDC);
  const [fromNetwork, setFromNetwork] = useState(GOERLI);
  const [toNetwork, setToNetwork] = useState(GNOSIS);
  const [fromValue, setFromValue] = useState("0.0");
  const [toValue, setToValue] = useState("0.0");
  const [balance, setBalance] = useState("0.0");

  const [lastClickApprove, setLastClickApprove] = useState(0);
  const [lastAmountLessThan, setLastAmountLessThan] = useState(0);

  useEffect(() => {
    setToValue(fromValue);
    const curTime = Math.floor(Date.now() / 1000);
    if (parseInt(fromValue) <= parseInt(balance)) {
      toast.dismiss("enterLessThan");
    }
    if (parseInt(fromValue) <= parseInt(balance) && fromValue != toValue) {
      if (curTime - lastClickApprove > 5) {
        // Need this complicated logic so that if we type "1" and then "0" in quick
        // succession that we don't have 2 "click approve" toasts show up
        // @ts-ignore
        // fancyToast({ text: "Click approve to let the contract withdraw your Succincts!" });
        setLastClickApprove(curTime);
      }
    } else if (parseInt(fromValue) > parseInt(balance) && fromValue != toValue) {
      if (curTime - lastAmountLessThan > 5) {
        // @ts-ignore
        fancyToast({ text: "Enter an amount less than your balance of " + balance, id: "enterLessThan" });
        setLastAmountLessThan(curTime);
      }
    }
  }, [fromValue]); // We only want this to trigger from fromValue is changed,
  // technically maybe we should move this into wherever setFromValue is used.

  const { data: goerliEthBalance } = useBalance({
    address: address as `0x${string}`,
    watch: true,
  });

  const {
    data: retrievedBalance,
    isError,
    isLoading,
  } = useBalance({
    address: address as `0x${string}`,
    token: config?.address(fromNetwork.name, "InfiniteMintSuccincts") as `0x${string}`,
    watch: true,
  });

  const { data: succinctTokenAddress } = useContractRead({
    address: config?.address(toNetwork.name.toLowerCase(), "Withdraw"),
    abi: config?.abi("Withdraw"),
    functionName: "token",
    chainId: config?.chainId(toNetwork.name.toLowerCase()),
  });

  const { data: retrievedBalanceGnosis } = useBalance({
    address: address as `0x${string}`,
    token: succinctTokenAddress as unknown as `0x${string}`,
    watch: true,
    chainId: config?.chainId(toNetwork.name.toLowerCase()),
  });

  useEffect(() => {
    if (retrievedBalance) {
      // setBalance(ethers.utils.formatEther(data.value));
      setBalance(retrievedBalance.formatted);
      if (retrievedBalance.formatted === "0" && retrievedBalance.formatted !== balance) {
        // @ts-ignore
        fancyToast({ text: "Use the faucet to get some Succincts!", id: "useFaucet" });
      } else if (retrievedBalance.formatted !== balance) {
        // This addresses the edge case where we don't want to show this toast
        // if your balance is decreased after deposit.
        // Since you will not be trying to send more over.
        // So we check for this condition.
        if (parseInt(retrievedBalance.formatted) < parseInt(balance)) {
          return;
        }
        toast.dismiss("useFaucet");
        // @ts-ignore
        fancyToast(
          {
            text: `You have ${retrievedBalance.formatted} Succincts. Enter an amount to send and click approve!`,
            id: "succinctBalanceAndApprove",
          }
          // @ts-ignore
        );
      }
    }
  }, [retrievedBalance, balance]);

  const { config: faucetConfig, error: faucetError } = usePrepareContractWrite({
    address: config?.address("goerli", "InfiniteMintSuccincts"),
    abi: config?.abi("InfiniteMintSuccincts"),
    functionName: "mint",
    args: [address, BigNumber.from(10).pow(18).mul(10)], // 10 ** 18 * 10
  });
  const {
    data: faucetData,
    isLoading: faucetLoading,
    isSuccess: faucetSuccess,
    write: callFaucet,
  } = useContractWrite(faucetConfig);

  useTransactionToast({ data: faucetData, network: fromNetwork, name: "Faucet" });

  function fromValueToAmt(fromValue: string) {
    if (parseInt(fromValue)) {
      return BigNumber.from(10).pow(18).mul(parseInt(fromValue));
    } else {
      return 0;
    }
  }

  const { config: approveConfig, error: approvePrepareError } = usePrepareContractWrite({
    address: config?.address("goerli", "InfiniteMintSuccincts"),
    abi: config?.abi("IERC20"),
    functionName: "approve",
    args: [config?.address(fromNetwork.name, "Deposit"), fromValueToAmt(fromValue)],
  });
  const {
    data: approvalData,
    isLoading: approvalLoading,
    isSuccess: approvalSuccess,
    write: callApprove,
  } = useContractWrite(approveConfig);

  const {
    data: depositData,
    isLoading: depositLoading,
    isSuccess: depositSuccess,
    write: callDeposit,
    error: depositError,
  } = useContractWrite({
    mode: "recklesslyUnprepared", // The reason we do this is because WAGMI seems
    // to have some sort of caching bug and won't update the deposit tranasction
    // to go through even after the approval transaction settles
    address: config?.address(fromNetwork.name, "Deposit"),
    abi: config?.abi("Deposit"),
    functionName: "deposit",
    args: [
      address,
      fromValueToAmt(fromValue),
      config?.address("goerli", "InfiniteMintSuccincts"),
      config?.chainId(toNetwork.name.toLowerCase()),
    ],
    // @ts-ignore
    overrides: {
      value: utils.parseEther("0.001"), // 1000000000000000 wei, required in Bridge contract
    },
    cacheTime: 2000,
    staleTime: 2000,
  });

  const [approvalSucceeded, setApprovalSucceeded] = useState(false);
  const [depositSucceeded, setDepositSucceeded] = useState(false);

  useEffect(() => {
    if (approvalSucceeded && !depositSucceeded) {
      fancyToast({ text: "Approval succeeded! Now click deposit.", id: "clickDeposit" });
    } else if (approvalSucceeded && depositSucceeded) {
      fancyToast({
        text: (
          <span>
            Deposit succeeded! Go to{" "}
            <span className="text-blue-500">
              <Link className="text-white" href="/dashboard" style={{ color: "white" }}>
                <button style={{ color: "white" }} onClick={() => toast.dismiss()}>
                  <u>dashboard</u>
                </button>
              </Link>
            </span>{" "}
            to view deposit status.
          </span>
        ),
        id: "depositSucceeded",
      });
    }
  }, [approvalSucceeded, depositSucceeded]);

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
            This bridge has no multisig and is 100% permissionless. Instead, Ethereum consensus is verified directly
            on-chain via{" "}
            <a href="https://blog.succinct.xyz/post/2022/09/20/proof-of-consensus/">
              <u>Proof of Consensus</u>
            </a>
            .
          </div>
        </div>
      </div>

      {/* {error && (
        <div className="bg-pink-500 text-white" style={{ height: "35px" }}>
          <div className="flex justify-center items-center mb-2">
            <div className="grotesk-regular mt-1">{error}</div>
          </div>
        </div>
      )} */}

      {/* <button
        onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
        style={{ position: "absolute", right: "0px", bottom: "0px", marginBottom: "30px", marginRight: "30px" }}
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="w-6 h-6">
          <path d="M12 2.25a.75.75 0 01.75.75v2.25a.75.75 0 01-1.5 0V3a.75.75 0 01.75-.75zM7.5 12a4.5 4.5 0 119 0 4.5 4.5 0 01-9 0zM18.894 6.166a.75.75 0 00-1.06-1.06l-1.591 1.59a.75.75 0 101.06 1.061l1.591-1.59zM21.75 12a.75.75 0 01-.75.75h-2.25a.75.75 0 010-1.5H21a.75.75 0 01.75.75zM17.834 18.894a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 10-1.061 1.06l1.59 1.591zM12 18a.75.75 0 01.75.75V21a.75.75 0 01-1.5 0v-2.25A.75.75 0 0112 18zM7.758 17.303a.75.75 0 00-1.061-1.06l-1.591 1.59a.75.75 0 001.06 1.061l1.591-1.59zM6 12a.75.75 0 01-.75.75H3a.75.75 0 010-1.5h2.25A.75.75 0 016 12zM6.697 7.757a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 00-1.061 1.06l1.59 1.591z" />
        </svg>
      </button>
      */}
      <div className="background-glow"></div>
      <main
        className="dark:text-white"
        // style={{ backgroundImage: theme === "light" ? 'url("/pixel-bg.png")' : 'url("pixel-bg-dark.png")' }}
      >
        <Navbar active="bridge"></Navbar>
        <div className="my-10"></div>
        <div className="flex justify-center item-center">
          <div className="flex-col">
            {/* Header */}{" "}
            {goerliEthBalance && goerliEthBalance.value.eq(0) && (
              <Instructions
                text="You'll need some Goerli Eth to send transactions!"
                linkName="Goerli Faucet"
                link="https://faucet.paradigm.xyz/"
                color="pink"
              />
            )}
            {!(goerliEthBalance && goerliEthBalance.value.eq(0)) && (
              <Instructions
                text="Confused about what to do? Read our instructions."
                linkName="Instructions"
                link="https://blog.succinct.xyz/post/2022/10/18/demo-instructions/"
                color="pink"
              />
            )}
            <div className="flex justify-center item-center align-middle">
              <div className="flex-none">
                <div className="text-3xl mr-5 pt-1">Bridge</div>
              </div>
              <div className="grow"></div>
              <div className="flex-none">
                {fromNetwork.name == "Goerli" && selectedToken.name == "Succincts" && (
                  // TODO include selectedToken = USDC & balance == 0
                  <button
                    className="bg-zinc-400 dark:bg-zinc-600 text-white px-5 py-2 text-xl mr-5"
                    style={{ borderRadius: "15px" }}
                    disabled={false && !callFaucet} // always enable for now
                    onClick={() => {
                      if (!callFaucet) {
                        alert("Connect wallet by clicking the button in the top right");
                      } else {
                        callFaucet?.();
                      }
                    }}
                  >
                    Faucet
                  </button>
                )}
              </div>
              <div className="flex-none">
                <TokenSelector tokens={tokens} selectedToken={selectedToken} setSelectedToken={setSelectedToken} />
              </div>
            </div>
            {/* From */}
            <div className="bg-zinc-100	dark:bg-zinc-900 mt-5" style={{ minWidth: "150px", borderRadius: "15px" }}>
              <div className="grotesk-regular pt-10 ml-10 pb-10 mr-10">
                <div className="flex">
                  <div className="flex-none">From</div>
                  <div className="grow"></div>
                  <div className="flex-none">Balance: {balance}</div>
                </div>
                <div className="my-8"></div>
                <div className="flex">
                  <div className="flex-none">
                    <NetworkSelector
                      selectedNetwork={fromNetwork}
                      setSelectedNetwork={setFromNetwork}
                      networks={[GOERLI]}
                    ></NetworkSelector>
                  </div>
                  <div className="grow"></div>
                  <div className="flex-none pt-1">
                    <FloatInput value={fromValue} setValue={setFromValue} />
                    <div className="text-3xl inline">{selectedToken.name}</div>
                  </div>
                </div>
              </div>
            </div>
            {/* Down Arrow */}
            <div className="flex justify-center items-center my-5">
              <DownArrow />
            </div>
            {/* To */}
            <div className="bg-zinc-100	dark:bg-zinc-900" style={{ minWidth: "150px", borderRadius: "15px" }}>
              <div className="grotesk-regular pt-10 ml-10 pb-10 mr-10">
                <div className="flex">
                  <div className="flex-none">To</div>
                  <div className="grow"></div>
                  <div className="flex-none">Balance: {retrievedBalanceGnosis?.formatted}</div>
                </div>
                <div className="my-8"></div>
                <div className="flex">
                  <div className="flex-none z-0">
                    <NetworkSelector
                      selectedNetwork={toNetwork}
                      setSelectedNetwork={setToNetwork}
                      networks={[GNOSIS, OPTIMISM, POLYGON, AVALANCHE]}
                    />
                  </div>
                  <div className="grow"></div>
                  <div className="flex-none pt-1">
                    <FloatInput value={toValue} setValue={setToValue} />
                    <div className="text-3xl inline">{selectedToken.name}</div>
                  </div>
                </div>
              </div>
            </div>
            {/* Approve & Send Buttons */}
            <div className="flex justify-center items-center mt-10">
              <TransactionButton
                name="Approve"
                loading={approvalLoading}
                success={approvalSuccess}
                data={approvalData}
                call={callApprove}
                network={GOERLI}
                callback={() => {
                  setApprovalSucceeded(true);
                  toast.dismiss("succinctBalanceAndApprove");
                }}
              />

              <TransactionButton
                name="Deposit"
                loading={depositLoading}
                success={depositSuccess}
                data={depositData}
                call={callDeposit}
                network={GOERLI}
                callback={() => {
                  setDepositSucceeded(true);
                  toast.dismiss("clickDeposit");
                }}
              />
            </div>
            <Footer></Footer>
          </div>
        </div>
      </main>
    </div>
  );
};

export default Home;
