"use client";

import React, { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Layers,
  Sparkles,
  Loader2,
  AlertCircle,
  InboxIcon,
} from "lucide-react";
import { CardItem } from "@/app/types";

interface CardCollectionProps {
  cards: CardItem[];
  isLoading?: boolean;
  error?: string | null;
  onRefresh: () => void;
  onGetMoreClick: () => void;
  defaultFilter?: string;
}

export default function CardCollection({
  cards,
  isLoading = false,
  error = null,
  onRefresh,
  onGetMoreClick,
  defaultFilter = "All",
}: CardCollectionProps) {
  const [cardFilter, setCardFilter] = useState(defaultFilter);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-purple-300" />
          <p className="text-sm text-purple-100">加载卡牌中...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-3">
          <AlertCircle className="h-8 w-8 text-red-400" />
          <p className="text-sm text-red-400">{error}</p>
          <button
            onClick={onRefresh}
            className="px-4 py-2 bg-purple-500 text-white rounded-lg hover:bg-purple-600 transition-colors"
          >
            重试
          </button>
        </div>
      </div>
    );
  }

  // if (!cards.length) {
  //   return (
  //     <div className="flex items-center justify-center h-64">
  //       <div className="flex flex-col items-center gap-3">
  //         <InboxIcon className="h-8 w-8 text-purple-300" />
  //         <p className="text-sm text-purple-100">还没有卡牌哦，快去抽卡吧！</p>
  //       </div>
  //     </div>
  //   );
  // }

  // 过滤卡片
  const filteredCards =
    cardFilter === "All"
      ? cards
      : cards.filter(
          (card) => card?.status?.toLowerCase() === cardFilter.toLowerCase()
        );

  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <Layers className="h-4 w-4 text-purple-400" />
          My Card Collection
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Manage your cards, view details, and prepare for battles
        </CardDescription>
      </CardHeader>

      <CardContent className="p-3">
        <div className="flex justify-between items-center mb-3">
          <h3 className="text-sm font-medium text-white">All Cards</h3>
          <div className="flex items-center gap-1">
            <span className="text-xs text-purple-200">Filter:</span>
            <select
              className="bg-black/30 text-white text-xs rounded border border-purple-500/30 px-1 py-0.5"
              value={cardFilter}
              onChange={(e) => setCardFilter(e.target.value)}
            >
              <option value="All">All</option>
              <option value="Owned">Owned</option>
              <option value="Rented">Rented</option>
              <option value="Staked">Staked</option>
            </select>
          </div>
        </div>

        <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2">
          {filteredCards.map((card) => (
            <div key={`${card.id}-${card.status}`} className="group relative">
              <div className="relative overflow-hidden rounded-lg transition-all duration-300 transform group-hover:scale-105">
                <img
                  src={card.image || "/placeholder.svg"}
                  alt={card.name}
                  className="w-full aspect-[3/4] object-cover rounded-lg"
                />
                {/* {card.status !== "owned" && (
                  <div className="absolute top-0 left-0 right-0 bg-black/70 px-1 py-0.5 text-[8px] text-center">
                    {card.status === "rented" ? (
                      <span className="text-green-400">
                        RENTED · {card.usesLeft} uses left
                      </span>
                    ) : (
                      <span className="text-blue-400">
                        STAKED · {card.poolShare}
                      </span>
                    )}
                  </div>
                )} */}
                <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent flex flex-col justify-end p-1">
                  <h3 className="text-white font-bold text-xs">{card.name}</h3>
                  <div className="flex justify-between items-center mt-0.5">
                    <Badge
                      className={`
                        text-[10px] px-1 py-0
                        ${
                          card.rarity === "Common"
                            ? "bg-blue-600"
                            : card.rarity === "Uncommon"
                            ? "bg-purple-600"
                            : card.rarity === "Rare"
                            ? "bg-orange-600"
                            : "bg-red-600"
                        }
                      `}
                    >
                      {card.rarity}
                    </Badge>
                    <span className="text-white text-[10px]">
                      x{card.count}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          ))}

          {/* Add card button */}
          <div
            className="flex items-center justify-center border-2 border-dashed border-purple-500/30 rounded-lg aspect-[3/4] cursor-pointer hover:bg-purple-900/20 transition-colors"
            onClick={onGetMoreClick}
          >
            <div className="text-center p-2">
              <div className="mx-auto rounded-full bg-purple-900/50 w-8 h-8 flex items-center justify-center mb-1">
                <Sparkles className="h-4 w-4 text-purple-300" />
              </div>
              <p className="text-purple-300 font-medium text-xs">Get More</p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
