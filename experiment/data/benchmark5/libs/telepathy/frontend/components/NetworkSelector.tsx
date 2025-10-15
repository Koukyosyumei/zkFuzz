import { Fragment, useState, useRef, Dispatch, SetStateAction } from "react";
import { Menu, Transition } from "@headlessui/react";
import { Network } from "../lib/Types";

function classNames(...classes: any) {
  return classes.filter(Boolean).join(" ");
}

type NetworkSelectorProps = {
  selectedNetwork: Network;
  setSelectedNetwork: Dispatch<SetStateAction<Network>>;
  networks: Network[];
};

export default function NetworkDropdown({ selectedNetwork, setSelectedNetwork, networks }: NetworkSelectorProps) {
  const [customOpen, setCustomOpen] = useState(false);
  const ref = useRef(null);

  return (
    <Menu as="div" className="relative inline-block text-left grotesk-regular">
      <div onClick={(e) => setCustomOpen(true)}>
        <Menu.Button
          ref={ref}
          className="inline-flex justify-center w-full px-4 py-2 bg-zinc-400 dark:bg-zinc-600 text-white text-lg font-medium"
          style={{ borderRadius: "15px" }}
        >
          <div className="mr-3">
            <img src={selectedNetwork.imgUrl} alt="defaultToken.imgUrl" width="30px" height="30px" />
          </div>
          <div className="pr-2">{selectedNetwork.name}</div>
          <i className="arrow down ml-2" style={{ marginTop: "9px" }}></i>
        </Menu.Button>
      </div>

      {customOpen && (
        <Transition
          as={Fragment}
          enter="transition ease-out duration-400"
          enterFrom="transform opacity-0 scale-95"
          enterTo="transform opacity-100 scale-100"
          leave="transition ease-in duration-100"
          leaveFrom="transform opacity-100 scale-100"
          leaveTo="transform opacity-0 scale-95"
        >
          <Menu.Items className="origin-top-right absolute right-0 mt-2 w-52 rounded-md shadow-lg bg-zinc-400 dark:bg-zinc-600 text-white ring-1 ring-black ring-opacity-5 focus:outline-none z-10">
            <div className="py-1"></div>
            {networks.map((network: Network) => {
              return (
                <div
                  className="text-gray-100 block px-4 py-2 text-sm hover:bg-zinc-500"
                  key={network.name}
                  onClick={(e) => {
                    setSelectedNetwork(network);
                    setCustomOpen(false);
                    // @ts-ignore
                    ref.current?.click();
                  }}
                >
                  <div>
                    <img className="inline mr-3 mb-1" src={network.imgUrl} width="25px"></img>
                    <div className="inline">{network.name}</div>
                  </div>
                </div>
              );
            })}
            <div className="py-1"></div>
          </Menu.Items>
        </Transition>
      )}
    </Menu>
  );
}
