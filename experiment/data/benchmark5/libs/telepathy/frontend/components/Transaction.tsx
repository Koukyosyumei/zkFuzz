import { truncateEthAddress } from "../lib/utils";
import { Network } from "../lib/Types";

type Props = {
  transactionHash: string;
  network: Network;
  prefix?: string;
};

export default function Transaction({ transactionHash, network, prefix = "" }: Props) {
  return (
    <div>
      <span>{prefix}</span>
      <a
        className="text-blue-500"
        href={`${network.blockExplorer}/tx/${transactionHash}`}
        target="_blank"
        rel="noreferrer"
      >
        {truncateEthAddress(transactionHash)}
      </a>
    </div>
  );
}
