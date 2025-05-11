import { useCurrentAccount, useCurrentWallet } from "@mysten/dapp-kit";
import { useEffect, useState } from "react";

export function useNetwork() {
  const wallet = useCurrentWallet();
  const account = useCurrentAccount();
  const [isCorrectNetwork, setIsCorrectNetwork] = useState<boolean | null>(
    null
  );
  const expectedNetwork = process.env.NEXT_PUBLIC_NETWORK || "testnet";

  const checkNetwork = () => {
    if (!wallet.currentWallet || !account) {
      setIsCorrectNetwork(null);
      return;
    }

    const currentChain = account.chains[0];
    const isCorrect = currentChain
      ?.toLowerCase()
      .includes(expectedNetwork.toLowerCase());
    setIsCorrectNetwork(isCorrect);
  };

  useEffect(() => {
    checkNetwork();

    // Add event listener for network changes
    const handleNetworkChange = () => {
      checkNetwork();
    };

    window.addEventListener("sui_networkChange", handleNetworkChange);

    return () => {
      window.removeEventListener("sui_networkChange", handleNetworkChange);
    };
  }, [wallet.currentWallet, account]);

  return {
    isCorrectNetwork,
    expectedNetwork,
    checkNetwork,
  };
}
