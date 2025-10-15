import { ethers } from "ethers";
import { truncateEthAddress } from "../lib/utils";
import { Network } from "../lib/Types";
import Transaction from "./Transaction";

type Props = {
  network: Network;
  title: string;
  events: ethers.Event[];
  args: string[];
  metadata?: Record<string, string | number | JSX.Element>[]; // If you want to include extra information
  metadataCols?: string[];
  description?: string;
  colsToName?: Record<string, string>;
};

export default function Example({
  network,
  events,
  args,
  title,
  description,
  metadata,
  metadataCols,
  colsToName,
}: Props) {
  return (
    <div className="grotesk-regular" style={{ minWidth: "650px" }}>
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold">{title}</h1>
          {/* Here description is the contract address */}
          {description && (
            <a
              className="text-blue-400"
              href={`${network.blockExplorer}address/${description}`}
              target="_blank"
              rel="noreferrer"
            >
              {description}
            </a>
          )}
          {/* <p className="mt-2 text-sm text-gray-700">{description}</p>} */}
        </div>
      </div>
      <div className="mt-3 flex flex-col">
        <div className="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div className="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
              <table className="min-w-full divide-y divide-gray-600">
                <thead className="bg-zinc-900 text-white">
                  <tr>
                    <th scope="col" className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold sm:pl-6">
                      Tx Hash
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold">
                      {network.name} Block #
                    </th>
                    {args.map((arg, i) => (
                      <th key={i} scope="col" className="px-3 py-3.5 text-left text-sm font-semibold">
                        {arg}
                      </th>
                    ))}
                    {metadataCols &&
                      metadataCols.map((col, i) => (
                        <th key={i} scope="col" className="px-3 py-3.5 text-left text-sm font-semibold">
                          {colsToName?.[col] ? colsToName[col] : `(metadata) ${col}`}
                        </th>
                      ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-600 bg-zinc-900 text-gray-400">
                  {/* TODO make the row span the whole table to show there are no events found */}
                  {/* {events.length === 0 && (
                    <tr>
                      <td className="whitespace-nowrap px-3 py-4 text-sm col-span-auto" style={{ columnSpan: "all" }}>
                        No events found
                      </td>
                    </tr>
                  )} */}
                  {events.map((event, i) => (
                    <tr key={i}>
                      <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
                        <Transaction transactionHash={event.transactionHash} network={network} />
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm">{event.blockNumber}</td>
                      {args.map((arg, j) => (
                        <td key={j} className="whitespace-nowrap px-3 py-4 text-sm">
                          {/* If event argument is a number, call toNumber(), otherwise just return the arg */}
                          {event.args?.[arg].toNumber
                            ? event.args?.[arg].toNumber()
                            : truncateEthAddress(event.args?.[arg])}
                        </td>
                      ))}
                      {metadata &&
                        metadataCols &&
                        metadataCols.map((col, k) => (
                          <td key={k} className="whitespace-nowrap px-3 py-4 text-sm">
                            {metadata[i]?.[col] ?? "Loading"}
                          </td>
                        ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
