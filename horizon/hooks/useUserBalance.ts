import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { useCallback, useEffect, useState } from "react";
import { SuiClient } from "@mysten/sui/client";
import { CoinStruct } from "@mysten/sui/client";

export interface CoinBalance {
  totalBalance: bigint;
  coins: CoinStruct[];
  suiBalance: bigint;
  suiCoins: CoinStruct[];
}

export function useUserBalance() {
  const client = useSuiClient();
  const account = useCurrentAccount();
  const [balance, setBalance] = useState<CoinBalance>({
    totalBalance: BigInt(0),
    coins: [],
    suiBalance: BigInt(0),
    suiCoins: [],
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchBalance = useCallback(async () => {
    if (!account?.address) return;

    setIsLoading(true);
    setError(null);

    try {
      // Get all coin objects owned by the user
      const { data: coins } = await client.getCoins({
        owner: account.address,
        coinType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fish::FISH`,
      });
      const { data: suiCoins } = await client.getCoins({
        owner: account.address,
        coinType: `0x2::sui::SUI`,
      });

      // Calculate total balance
      const totalBalance = coins.reduce(
        (sum, coin) => sum + BigInt(coin.balance),
        BigInt(0)
      );
      const suiBalance = suiCoins.reduce(
        (sum, coin) => sum + BigInt(coin.balance),
        BigInt(0)
      );

      setBalance({
        totalBalance,
        coins,
        suiBalance,
        suiCoins,
      });
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error("Failed to fetch balance")
      );
    } finally {
      setIsLoading(false);
    }
  }, [account?.address, client]);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  return {
    balance,
    isLoading,
    error,
    refetch: fetchBalance,
  };
}
