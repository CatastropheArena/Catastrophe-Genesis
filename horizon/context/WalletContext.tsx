import React, { useEffect } from "react";
import {
  SuiClientProvider,
  WalletProvider,
  lightTheme,
} from "@mysten/dapp-kit";
import { type StateStorage } from "zustand/middleware";
import { useUserStore } from "@/stores/useUserStore";
import { toast } from "react-toastify";
import { networkConfig, network } from "@/contracts";

type Props = {
  children: React.ReactNode;
};

const defaultNetwork = networkConfig[network].name;

const SuiWalletProvider = ({ children }: Props) => {
  if (typeof window === "undefined") return <></>;
  return (
    <>
      <SuiClientProvider
        networks={networkConfig}
        defaultNetwork={defaultNetwork}
      >
        <WalletProvider
          theme={lightTheme}
          autoConnect={true}
          storage={localStorage as StateStorage}
          storageKey="sui-wallet"
          preferredWallets={["Slash Wallet"]}
          stashedWallet={{
            name: "Catastrophe Genesis",
          }}
        >
          {children}
        </WalletProvider>
      </SuiClientProvider>
    </>
  );
};

export default SuiWalletProvider;
