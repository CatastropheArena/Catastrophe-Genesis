import { create } from "zustand";
import { CardItem } from "@/app/types";
import { PaginatedObjectsResponse, SuiClient } from "@mysten/sui/client";

// 卡片稀有度映射
const RARITY_MAP = {
  70: "Common",
  20: "Uncommon",
  9: "Rare",
  1: "Legendary",
};

interface CardStore {
  cards: CardItem[];
  isLoading: boolean;
  error: string | null;
  fetchCards: (address: string, client: SuiClient) => Promise<void>;
  setCards: (cards: CardItem[]) => void;
}

export const useCardStore = create<CardStore>((set) => ({
  cards: [],
  isLoading: false,
  error: null,
  setCards: (cards) => set({ cards }),
  fetchCards: async (address: string, client: SuiClient) => {
    if (!address) return;

    set({ isLoading: true, error: null });

    try {
      let allObjects: PaginatedObjectsResponse["data"] = [];
      let hasNextPage = true;
      let nextCursor: string | null = null;

      // 使用分页获取所有卡片
      while (hasNextPage) {
        const response = await client.getOwnedObjects({
          owner: address,
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
          cursor: nextCursor,
        });

        allObjects = [...allObjects, ...response.data];
        hasNextPage = response.hasNextPage;
        nextCursor = response.nextCursor || null;
      }

      // 转换为前端需要的格式
      const cardItems: CardItem[] = allObjects.map((obj) => {
        const content = obj.data?.content as any;
        const cardType = content.fields.card_type;
        const fields = cardType.fields;

        return {
          id: obj.data?.objectId || "",
          name: fields.name,
          description: fields.description,
          strategy: fields.strategy,
          rarity:
            RARITY_MAP[fields.rarity as keyof typeof RARITY_MAP] || "Unknown",
          image: `/cards/${fields.image_url}.png`,
          cost: Number(fields.cost),
          dynamicCost: Number(fields.dynamic_cost),
          level: Number(content.fields.level),
          count: 1,
          status: "owned",
        };
      });

      // 合并相同名称的卡片并保留最高等级
      const mergedCards = cardItems.reduce((acc: CardItem[], curr) => {
        const existingCard = acc.find((card) => card.name === curr.name);
        if (existingCard) {
          existingCard.count += 1;
          if (curr.level > existingCard.level) {
            existingCard.level = curr.level;
          }
        } else {
          acc.push(curr);
        }
        return acc;
      }, []);

      set({ cards: mergedCards, isLoading: false });
    } catch (err) {
      console.error("Failed to fetch cards:", err);
      set({
        error: err instanceof Error ? err.message : "获取卡牌失败",
        isLoading: false,
      });
    }
  },
}));
