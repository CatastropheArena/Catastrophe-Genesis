import { useState, useEffect } from "react";
import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";

export const useUserAssets = () => {
  const [assets, setAssets] = useState({
    coins: 0,
    fragments: 0,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = useSuiClient();
  const account = useCurrentAccount();

  const fetchAssets = async () => {
    if (!account?.address) return;

    setIsLoading(true);
    setError(null);

    try {
      // 获取用户的 FISH 代币余额
      const coins = await client.getBalance({
        owner: account.address,
        coinType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fish::FISH`,
      });

      // 获取用户的碎片余额
      const fragments = await client.getBalance({
        owner: account.address,
        coinType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fragment::FRAGMENT`,
      });

      setAssets({
        coins: Number(coins.totalBalance),
        fragments: Number(fragments.totalBalance),
      });
    } catch (err) {
      console.error("Failed to fetch assets:", err);
      setError("获取资产失败");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (account?.address) {
      fetchAssets();
    }
  }, [account?.address]);

  return {
    assets,
    isLoading,
    error,
    fetchAssets,
  };
};
