import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { useCallback, useEffect, useState } from "react";
import { SuiObjectData } from "@mysten/sui/client";

interface Assets {
  coins: number;
  fragments: number;
}

export function useUserAssets() {
  const [assets, setAssets] = useState<Assets>({
    coins: 0,
    fragments: 0,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const client = useSuiClient();
  const account = useCurrentAccount();

  const fetchAssets = useCallback(async () => {
    if (!account?.address) {
      let tempAssets = Object.assign({}, assets, {
        coins: 0,
        fragments: 0,
      });
      setAssets(tempAssets);
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

      // Create a new assets object to trigger React re-render
      const newAssets = {
        coins: Number(coins.totalBalance),
        fragments: totalFragments,
      };

      console.log("newAssets", newAssets);

      // Update state with the new assets object
      setAssets(newAssets);
    } catch (err) {
      console.error("Failed to fetch assets:", err);
      setError("Failed to fetch assets");
    } finally {
      setIsLoading(false);
    }
  }, [account?.address, client]);

  // Fetch assets when wallet changes
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
