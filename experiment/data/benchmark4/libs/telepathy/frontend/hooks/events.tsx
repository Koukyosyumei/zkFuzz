import { useState, useEffect } from "react";

import { AxiosInstance } from "axios";
import { ContractInterface, ethers, Contract, providers } from "ethers";
import { useContractEvent } from "wagmi";
import { Abi } from "abitype";

import Transaction from "../components/Transaction";
import { Network } from "../lib/Types";

function sortAndSlice(events: ethers.Event[], limit: number) {
  events.sort((a, b) => (a.blockNumber >= b.blockNumber ? -1 : 1));
  return events.slice(0, limit);
}

export function useEvents({
  addressOrName,
  abi,
  eventName,
  contract,
  provider,
  offsetBlock = undefined,
  maxEvents = 10,
  filterArgs = [],
}: {
  addressOrName: string | undefined; // unused for now
  abi: Abi | undefined; // unused for now
  eventName: string;
  contract: Contract | undefined;
  provider: providers.Provider | undefined; // unsued for now
  offsetBlock?: number;
  maxEvents?: number;
  filterArgs?: any[];
}) {
  const [events, setEvents] = useState<ethers.Event[]>([]);

  // To start, we load the historical events
  useEffect(() => {
    async function wrapper() {
      if (!contract) {
        setEvents([]);
        return;
      }

      let startBlock = 0;
      if (offsetBlock) {
        const currentBlockNum = await provider?.getBlockNumber();
        if (currentBlockNum) {
          startBlock = Math.max(currentBlockNum - offsetBlock, 0);
        }
      }
      let events: ethers.Event[] = [];
      try {
        events = await contract.queryFilter(contract.filters[eventName](...filterArgs), startBlock);
      } catch (e) {
        console.error(e);
      }
      setEvents(sortAndSlice(events, maxEvents));
    }

    wrapper();

    // Because filterArgs is an array, we need to stringify it to compare it to the previous value
  }, [contract, eventName, offsetBlock, maxEvents, JSON.stringify(filterArgs)]);

  // Then everytime we get a new event, we add it to the list
  // Get rid of this for now, since it does not have filterArgs in it and adds complications
  // As a result, to see new events, users will need to refresh the page
  // useContractEvent({
  //   addressOrName,
  //   contractInterface: abi,
  //   eventName,
  //   signerOrProvider: provider,
  //   listener: (event) => {
  //     setEvents(sortAndSlice([event, ...events], maxEvents));
  //   },
  // });

  return { events };
}

async function getEth1BlockNumber(slot: number, consensusClient: AxiosInstance) {
  if (!slot) {
    return undefined;
  }
  try {
    const { data } = await consensusClient.get(`/eth/v2/beacon/blocks/${slot}`);
    // @ts-ignore
    return data.data.message.body.execution_payload.block_number;
  } catch (e) {
    return "Error retreiving block number";
  }
}

export function useLCEventBlockNumbers(events: ethers.Event[], consensusClient: AxiosInstance) {
  // Given ethers light client events, grab the eth1 block number for each slot
  const [metadata, setMetadata] = useState<{ eth1BlockNumber: number }[]>([]);
  useEffect(() => {
    async function wrapper() {
      const metadata = await Promise.all(
        events.map(async (event) => {
          const slot = event.args?.slot.toNumber();
          const blockNumber = await getEth1BlockNumber(slot, consensusClient);
          return { eth1BlockNumber: blockNumber };
        })
      );
      setMetadata(metadata);
    }
    wrapper();
  }, [events]);

  return { metadata };
}

type DepositStatus = {
  amount: string;
  status: JSX.Element;
};

export function useDepositStatus(
  depositEvents: ethers.Event[],
  sentMessages: ethers.Event[],
  executedMessages: ethers.Event[],
  lightClientEvents: ethers.Event[],
  lightClientBlockNumbers: { eth1BlockNumber: number }[],
  network: Network // The Network of the *withdrawals*
) {
  const [metadata, setMetadata] = useState<DepositStatus[]>([]);
  useEffect(() => {
    const sentMessageByTransaction = new Map();
    sentMessages.map((sm) => {
      sentMessageByTransaction.set(sm.transactionHash, sm);
    });
    const executeMessageByNonce = new Map();
    executedMessages.map((em) => {
      executeMessageByNonce.set(em.topics[1], em);
    });

    const maxBlockNumber = Math.max(...lightClientBlockNumbers.map(({ eth1BlockNumber }) => eth1BlockNumber));

    const status = depositEvents.map((depositEvent) => {
      const transactionHash = depositEvent.transactionHash;
      const formattedAmount = ethers.utils.formatEther(depositEvent.args?.amount);
      const sentMessage = sentMessageByTransaction.get(transactionHash);
      if (sentMessage && sentMessage.topics) {
        const executeMessage = executeMessageByNonce.get(sentMessage.topics[1]);
        if (executeMessage) {
          return {
            status: (
              <Transaction
                prefix={"✅ Completed "}
                transactionHash={executeMessage.transactionHash}
                network={network}
              />
            ),
            amount: formattedAmount,
          };
        } else {
          if (depositEvent.blockNumber > maxBlockNumber) {
            return {
              status: <div>🚧 Waiting on light client update</div>,
              amount: formattedAmount,
            };
          } else {
            return {
              status: <div>⌛ Waiting on relayer to submit withdrawal</div>,
              amount: formattedAmount,
            };
          }
        }
      } else {
        // Weird stuff is happening
        return {
          status: <div>❓ Error: Deposit did not send message to AMB.</div>,
          amount: formattedAmount,
        };
      }
    });
    setMetadata(status);
  }, [depositEvents, sentMessages, executedMessages, lightClientEvents, lightClientBlockNumbers, network]);

  return { metadata };
}
