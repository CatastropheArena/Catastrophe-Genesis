import { useEffect, useState } from "react";
import { useCurrentAccount } from "@mysten/dapp-kit";

export const useNetwork = () => {
  const [isCorrectNetwork, setIsCorrectNetwork] = useState<boolean | null>(
    null
  );
  const account = useCurrentAccount();

  const checkNetwork = () => {
    if (!account) {
      setIsCorrectNetwork(null);
      return;
    }

    // 检查当前链是否匹配期望的网络
    const currentChain = account.chains[0];
    const expectedNetwork =
      process.env.NEXT_PUBLIC_NETWORK?.toLowerCase() || "testnet";
    const isCorrect = currentChain === `sui:${expectedNetwork}`;
    setIsCorrectNetwork(isCorrect);
  };

  useEffect(() => {
    checkNetwork();
  }, [account?.chains]);

  return {
    isCorrectNetwork,
    expectedNetwork: process.env.NEXT_PUBLIC_NETWORK || "testnet",
  };
};
