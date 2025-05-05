"use client";

import { useState, useEffect } from "react";
import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { CardItem } from "@/app/types";

export function useUserCards() {
  const [cards, setCards] = useState<CardItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const currentAccount = useCurrentAccount();
  const client = useSuiClient();

  const fetchCards = async () => {
    if (!currentAccount?.address) return;

    setIsLoading(true);
    setError(null);

    try {
      // 获取用户拥有的所有卡片对象
      const { data: objects } = await client.getOwnedObjects({
        owner: currentAccount.address,
        filter: {
          MatchAll: [
            {
              StructType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::card::Card`,
            },
          ],
        },
        options: {
          showContent: true,
          showType: true,
        },
      });

      // 转换为前端需要的格式
      const cardItems: CardItem[] = objects.map((obj) => {
        const content = obj.data?.content as any;
        const cardType = content.fields.card_type;

        return {
          id: obj.data?.objectId || "",
          name: cardType.fields.name,
          rarity: getRarityString(cardType.fields.rarity),
          image: cardType.fields.image_url,
          count: 1,
          status: "owned",
        };
      });

      // 合并相同名称的卡片
      const mergedCards = cardItems.reduce((acc: CardItem[], curr) => {
        const existingCard = acc.find((card) => card.name === curr.name);
        if (existingCard) {
          existingCard.count += 1;
        } else {
          acc.push(curr);
        }
        return acc;
      }, []);

      setCards(mergedCards);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch cards");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchCards();
  }, [currentAccount?.address]);

  return {
    cards,
    isLoading,
    error,
    fetchCards,
  };
}

// 辅助函数：将稀有度数字转换为字符串
function getRarityString(rarity: number): string {
  switch (rarity) {
    case 70:
      return "Common";
    case 20:
      return "Uncommon";
    case 9:
      return "Rare";
    case 1:
      return "Legendary";
    default:
      return "Unknown";
  }
}
