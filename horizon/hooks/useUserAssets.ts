import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { useCallback, useEffect, useState } from "react";
import {
  CoinStruct,
  SuiObjectData,
  SuiObjectResponse,
} from "@mysten/sui/client";

interface Assets {
  sui: number;
  coins: number;
  suiCoins: CoinStruct[];
  fishCoins: CoinStruct[];
  passport: SuiObjectResponse | undefined;
  fragments: number;
}

export function useUserAssets() {
  const [assets, setAssets] = useState<Assets>({
    sui: 0,
    coins: 0,
    suiCoins: [],
    fishCoins: [],
    passport: undefined,
    fragments: 0,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = useSuiClient();
  const account = useCurrentAccount();

  const fetchAssets = useCallback(async () => {
    if (!account?.address) {
      setAssets({
        sui: 0,
        coins: 0,
        suiCoins: [],
        fishCoins: [],
        passport: undefined,
        fragments: 0,
      });
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Fetch FISH tokens
      const coins = await client.getBalance({
        owner: account.address,
        coinType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fish::FISH`,
      });

      const sui = await client.getBalance({
        owner: account.address,
        coinType: `0x2::sui::SUI`,
      });

      // Get all coin objects owned by the user
      const { data: fishCoins } = await client.getCoins({
        owner: account.address,
        coinType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fish::FISH`,
      });
      const { data: suiCoins } = await client.getCoins({
        owner: account.address,
        coinType: `0x2::sui::SUI`,
      });

      // Get passport
      const passportObjects = await client.getOwnedObjects({
        owner: account.address,
        options: {
          showContent: true,
        },
        filter: {
          MatchAny: [
            {
              StructType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::passport::Passport`,
            },
          ],
        },
      });

      // Initialize fragment total
      let totalFragments = 0;

      // Fetch all fragment tokens using pagination
      let hasNextPage = true;
      let nextCursor: string | null = null;

      while (hasNextPage) {
        const objects = await client.getOwnedObjects({
          owner: account.address,
          options: {
            showContent: true,
          },
          filter: {
            MatchAny: [
              {
                StructType: `0x2::token::Token<${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fragment::FRAGMENT>`,
              },
            ],
          },
          cursor: nextCursor,
        });

        // Update pagination state
        nextCursor = objects.nextCursor || null;
        hasNextPage = objects.hasNextPage;

        // Process fragment tokens in current page
        objects.data.forEach((object) => {
          const data = object.data as unknown as SuiObjectData;
          if (data.content?.dataType !== "moveObject") {
            return;
          }
          const contentType = data.content?.type;
          if (
            contentType ===
            `0x2::token::Token<${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::fragment::FRAGMENT>`
          ) {
            const balance = Number(
              (data.content?.fields as unknown as { balance: number })
                ?.balance || 0
            );
            totalFragments += balance;
          }
        });
      }

      // 创建一个全新的对象来触发重新渲染
      const newAssets = {
        sui: Number(sui.totalBalance),
        coins: Number(coins.totalBalance),
        suiCoins: suiCoins,
        fishCoins: fishCoins,
        passport: passportObjects.data.find((i) => {
          const data = i.data as unknown as SuiObjectData;
          return data.content?.dataType === "moveObject" &&
            data.content?.type ===
              `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::passport::Passport`
            ? (data.content?.fields as unknown as Passport)
            : null;
        }),
        fragments: totalFragments,
      };

      console.log("Fetched new assets:", newAssets);

      // 使用函数式更新确保使用最新状态
      setAssets(newAssets);
    } catch (err) {
      console.error("Failed to fetch assets:", err);
      setError("Failed to fetch assets");
    } finally {
      setIsLoading(false);
    }
  }, [account?.address, client]);

  // 监听钱包地址变化时重新获取资产
  useEffect(() => {
    fetchAssets();
  }, [account?.address, fetchAssets]);

  return {
    assets,
    isLoading,
    error,
    fetchAssets,
  };
}
