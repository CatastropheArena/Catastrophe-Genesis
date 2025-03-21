"use client"

import React, { useRef, useEffect } from 'react';
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Star } from "lucide-react"
import { DrawnCard } from "@/app/types"

// 定义组件接口
interface DrawnCardModalProps {
  drawnCard: DrawnCard | null;
  isVisible: boolean;
  onClose: () => void;
}

export default function DrawnCardModal({ drawnCard, isVisible, onClose }: DrawnCardModalProps) {
  const cardRef = useRef<HTMLDivElement>(null);

  // 卡片翻转动画
  useEffect(() => {
    if (isVisible && cardRef.current) {
      cardRef.current.classList.add("flipped");
    } else if (cardRef.current) {
      cardRef.current.classList.remove("flipped");
    }
  }, [isVisible]);

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black/90 backdrop-blur-md z-50 flex items-center justify-center p-4">
      <div className="w-full max-w-md text-center">
        <div className="card-container perspective-1000 w-48 h-72 sm:w-64 sm:h-96 mx-auto mb-6">
          <div
            ref={cardRef}
            className="card-inner relative w-full h-full transition-transform duration-1000 transform-style-3d"
          >
            <div className="card-back absolute w-full h-full backface-hidden bg-gradient-to-br from-purple-800 to-indigo-900 rounded-xl border-2 border-purple-500/50 flex items-center justify-center">
              <div className="text-4xl sm:text-6xl text-purple-300 animate-pulse">?</div>
            </div>
            <div className="card-front absolute w-full h-full backface-hidden rotateY-180 bg-gradient-to-br from-purple-900 to-indigo-900 rounded-xl border-2 border-purple-500/50 p-3 flex flex-col">
              <div
                className={`
                  w-full h-3/4 rounded-lg mb-2 overflow-hidden
                  ${
                    drawnCard?.rarity === "Common"
                      ? "bg-blue-600/20"
                      : drawnCard?.rarity === "Uncommon"
                        ? "bg-purple-600/20"
                        : drawnCard?.rarity === "Rare"
                          ? "bg-orange-600/20"
                          : "bg-red-600/20"
                  }
                `}
              >
                <img
                  src={drawnCard?.image || "/placeholder.svg"}
                  alt={drawnCard?.name}
                  className="w-full h-full object-cover"
                />
              </div>
              <div className="flex-1 flex flex-col justify-between">
                <h3 className="text-white font-bold text-sm sm:text-lg">{drawnCard?.name}</h3>
                <div className="flex justify-between items-center">
                  <Badge
                    className={`
                      px-1 py-0.5 text-[8px] sm:text-[10px]
                      ${
                        drawnCard?.rarity === "Common"
                          ? "bg-blue-600"
                          : drawnCard?.rarity === "Uncommon"
                            ? "bg-purple-600"
                            : drawnCard?.rarity === "Rare"
                              ? "bg-orange-600"
                              : "bg-red-600"
                      }
                    `}
                  >
                    {drawnCard?.rarity}
                  </Badge>
                  <div className="text-yellow-400 flex items-center gap-1">
                    <Star className="h-3 w-3 sm:h-4 sm:w-4 fill-yellow-400" />
                    <Star className="h-3 w-3 sm:h-4 sm:w-4 fill-yellow-400" />
                    <Star className="h-3 w-3 sm:h-4 sm:w-4 fill-yellow-400" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <h2 className="text-xl sm:text-2xl font-bold text-white mb-2">
          {drawnCard?.rarity === "Legendary"
            ? "LEGENDARY CARD!"
            : drawnCard?.rarity === "Rare"
              ? "Rare Card!"
              : drawnCard?.rarity === "Uncommon"
                ? "Uncommon Card!"
                : "New Card!"}
        </h2>
        <p className="text-purple-200 mb-6">
          You've drawn a {drawnCard?.rarity.toLowerCase()} {drawnCard?.name} card!
        </p>
        <Button onClick={onClose} className="bg-gradient-to-r from-purple-600 to-pink-600 px-6 sm:px-8">
          Awesome!
        </Button>
      </div>

      <style jsx global>{`
        .perspective-1000 {
          perspective: 1000px;
        }
        .transform-style-3d {
          transform-style: preserve-3d;
        }
        .backface-hidden {
          backface-visibility: hidden;
        }
        .rotateY-180 {
          transform: rotateY(180deg);
        }
        .flipped {
          transform: rotateY(180deg);
        }
      `}</style>
    </div>
  );
}