import Link from "next/link";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import toast, { Toaster } from "react-hot-toast";

type NavbarProps = {
  active: string;
};

export default function Navbar({ active }: NavbarProps) {
  return (
    <div className="px-10 py-7 grotesk-regular">
      <div className="flex">
        <div className="flex-none text-2xl text-black dark:text-white" style={{ paddingTop: "3px", width: "400px" }}>
          <Link href="/">
            <button
              onClick={() => {
                toast.dismiss();
              }}
            >
              Succinct Demo
            </button>
          </Link>
        </div>
        <div className="grow">
          <div className="flex justify-center item-center pt-2">
            <div className={active == "bridge" ? "text-white" : "text-gray-500"}>
              <Link href="/">
                <button
                  onClick={() => {
                    toast.dismiss();
                  }}
                >
                  Bridge
                </button>
              </Link>
            </div>
            <div className="mx-5"></div>
            <div className={active == "dashboard" ? "text-white" : "text-gray-500"}>
              <Link href="/dashboard">
                <button
                  onClick={() => {
                    toast.dismiss();
                  }}
                >
                  Dashboard
                </button>
              </Link>
            </div>
            <div className="mx-5"></div>
            <div className={active == "instructions" ? "text-white" : "text-gray-500"}>
              <a href="https://blog.succinct.xyz/post/2022/09/20/proof-of-consensus/" target="_blank" rel="noreferrer">
                FAQ
              </a>
            </div>
          </div>
        </div>
        <div className="flex-none" style={{ width: "400px" }}>
          <div style={{ marginLeft: "auto", marginRight: "0px", float: "right" }}>
            <ConnectButton showBalance={false} accountStatus="address" />
          </div>
        </div>
      </div>
    </div>
  );
}
