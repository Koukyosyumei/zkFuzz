import { useTheme } from "next-themes";
// import {  } from "wagmi";
import { SendTransactionResult } from "@wagmi/core";
import { useEffect, useState } from "react";
import { TransactionReceipt } from "@ethersproject/abstract-provider";
import { Network } from "../lib/Types";
import toast, { Toaster } from "react-hot-toast";
import useTransactionToast from "../hooks/transactionToast";
import { PinkSpinner } from "../hooks/transactionToast";

const StyledButton = ({
  text,
  txHash = undefined,
  network = undefined,
  spinner = false,
}: {
  text: string;
  txHash: string | undefined;
  network: Network | undefined;
  spinner: boolean;
}) => {
  if (txHash === undefined || network === undefined) {
    return (
      <div className="bg-zinc-400 dark:bg-zinc-600 text-white px-5 py-2 text-xl mr-5" style={{ borderRadius: "15px" }}>
        {text}
      </div>
    );
  } else {
    return (
      <div
        className="bg-zinc-400 dark:bg-zinc-600 text-white px-5 py-2 text-xl mr-5 inline-flex items-center"
        style={{ borderRadius: "15px" }}
      >
        {spinner && (
          <span>
            <svg
              role="status"
              className="inline mr-2 w-5 h-5 text-gray-200 animate-spin dark:text-gray-600"
              viewBox="0 0 100 101"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
                fill="currentColor"
              />
              <path
                d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
                fill="#1C64F2"
              />
            </svg>
          </span>
        )}
        <span>
          <a className="text-white" href={`${network.blockExplorer}/tx/${txHash}`} target="_blank" rel="noreferrer">
            {text}
          </a>
        </span>
      </div>
    );
  }
};

export default function TransactionButton({
  name,
  loading,
  success,
  data,
  call,
  network,
  callback,
}: {
  name: string;
  loading: boolean;
  success: boolean;
  data: SendTransactionResult | undefined;
  call: any;
  network: Network;
  callback: any;
}) {
  const [tx, setTx] = useState<string | undefined>(undefined);
  const [txReceipt, setTxReceipt] = useState<TransactionReceipt | undefined>(undefined);
  useEffect(() => {
    setTx(data?.hash);
    data?.wait(1).then((receipt) => {
      setTxReceipt(receipt);
      callback();
    });
  }, [data]);

  useTransactionToast({ data, network, name });

  return (
    <div>
      {loading && <StyledButton text={`Pending ${name}`} txHash={undefined} network={network} spinner={true} />}
      {tx && !txReceipt && (
        <div
          className="bg-zinc-400 dark:bg-zinc-600 text-white px-5 py-2 text-xl mr-5 inline-flex items-center"
          style={{ borderRadius: "15px" }}
        >
          <PinkSpinner />
          Waiting
        </div>
      )}
      {txReceipt && <StyledButton text={`${name} Success`} txHash={tx} network={network} spinner={false} />}
      {!loading && !success && (
        <button
          className="bg-zinc-400 dark:bg-zinc-600 text-white px-5 py-2 text-xl mr-5"
          style={{ borderRadius: "15px" }}
          disabled={!call}
          onClick={() => {
            call?.();
          }}
        >
          {name}
        </button>
      )}
      {/* This is the old button */}
      {/* {approvalLoading && <StyledButton text="Awaiting Approval" />}
              {approvalSuccess && <StyledButton text="Approval Success" />}
              {!approvalLoading && !approvalSuccess && (
                <button
                  className="bg-zinc-400 dark:bg-zinc-600 text-white px-5 py-2 text-xl mr-5"
                  style={{ borderRadius: "15px" }}
                  disabled={!callApprove}
                  onClick={() => {
                    callApprove?.();
                  }}
                >
                  Approve
                </button>
              )} */}
    </div>
  );
}
