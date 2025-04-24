"use client"

import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Progress } from "@/components/ui/progress"
import {
  Sparkles,
  Layers,
  X,
  Clock,
  ChevronRight,
  ChevronLeft,
  Flame,
  Coins,
} from "lucide-react"
import { Assets, CardItem, CardPack, DrawHistoryItem } from "@/app/types"

// 定义组件props接口
interface CardSynthesisGachaProps {
  assets: Assets;
  synthesisCards: (CardItem | null)[];
  upgradeCard: CardItem | null;
  cardPacks: CardPack[];
  selectedPack: CardPack;
  activePackIndex: number;
  drawHistory: DrawHistoryItem[];
  isLoading: boolean;
  isDrawing: boolean;
  drawProgress: number;
  showCardSelector: boolean;
  selectorType: string;
  
  // 回调函数
  handleSelectSynthesisCard: (index: number) => void;
  handleRemoveSynthesisCard: (index: number) => void;
  handleSelectUpgradeCard: () => void;
  handleSynthesizeCards: () => void;
  handleUpgradeCardAction: () => void;
  handleDrawCard: () => void;
  handleChangePack: (direction: "prev" | "next") => void;
  setActivePackIndex: (index: number) => void;
  setSelectedPack: (pack: CardPack) => void;
  setShowCardSelector: (show: boolean) => void;
  handleCardSelectorSelect: (card: CardItem) => void;
  setUpgradeCard: (card: CardItem | null) => void;
}

export default function CardSynthesisGacha({
  assets,
  synthesisCards,
  upgradeCard,
  cardPacks,
  selectedPack,
  activePackIndex,
  drawHistory,
  isLoading,
  isDrawing,
  drawProgress,
  showCardSelector,
  selectorType,
  handleSelectSynthesisCard,
  handleRemoveSynthesisCard,
  handleSelectUpgradeCard,
  handleSynthesizeCards,
  handleUpgradeCardAction,
  handleDrawCard,
  handleChangePack,
  setActivePackIndex,
  setSelectedPack,
  setShowCardSelector,
  handleCardSelectorSelect,
  setUpgradeCard
}: CardSynthesisGachaProps) {
  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <Sparkles className="h-4 w-4 text-blue-400" />
          Card Synthesis & Gacha
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Synthesize cards with fragments, upgrade existing cards, or draw new cards with coins
        </CardDescription>
      </CardHeader>

      <CardContent className="p-3">
        <div className="space-y-3">
          {/* 菜单选项 */}
          <div className="bg-black/30 rounded-lg p-1 mb-2">
            <Tabs defaultValue="synthesis" className="w-full">
              <TabsList className="w-full bg-transparent gap-2">
                <TabsTrigger value="synthesis" className="flex-1 data-[state=active]:bg-purple-700 h-7 text-xs rounded-md">
                  Synthesis
                </TabsTrigger>
                <TabsTrigger value="upgrade" className="flex-1 data-[state=active]:bg-purple-700 h-7 text-xs rounded-md">
                  Upgrade
                </TabsTrigger>
                <TabsTrigger value="gacha" className="flex-1 data-[state=active]:bg-purple-700 h-7 text-xs rounded-md">
                  Gacha
                </TabsTrigger>
              </TabsList>

              {/* 合成系统面板 */}
              <TabsContent value="synthesis" className="mt-2">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                  {/* 合成材料区 */}
                  <div className="bg-purple-900/20 rounded-lg p-3 col-span-2">
                    <h3 className="text-sm font-medium text-white mb-2">Select Synthesis Materials</h3>
                    <div className="grid grid-cols-3 gap-3">
                      {Array.from({ length: 3 }).map((_, index) => (
                        <div
                          key={index}
                          className={`aspect-square bg-purple-900/30 rounded-lg flex items-center justify-center border-2 border-dashed ${synthesisCards[index] ? "border-purple-500/70" : "border-purple-500/30"} cursor-pointer hover:bg-purple-900/50 transition-colors`}
                          onClick={() => handleSelectSynthesisCard(index)}
                        >
                          {synthesisCards[index] ? (
                            <div className="relative w-full h-full p-2">
                              <img
                                src={synthesisCards[index]?.image || "/placeholder.svg"}
                                alt={synthesisCards[index]?.name}
                                className="w-full h-full object-cover rounded-md"
                              />
                              <Badge
                                className={`absolute top-1 right-1 text-[8px] px-1 py-0
                                  ${
                                    synthesisCards[index]?.rarity === "Common"
                                      ? "bg-blue-600"
                                      : synthesisCards[index]?.rarity === "Uncommon"
                                        ? "bg-purple-600"
                                        : "bg-orange-600"
                                  }
                                `}
                              >
                                {synthesisCards[index]?.rarity}
                              </Badge>
                              <div className="absolute top-0 right-0 bg-black/70 rounded-bl-md p-0.5">
                                <X
                                  className="h-3 w-3 text-red-400 cursor-pointer"
                                  onClick={(e) => {
                                    e.stopPropagation()
                                    handleRemoveSynthesisCard(index)
                                  }}
                                />
                              </div>
                            </div>
                          ) : (
                            <div className="text-center">
                              <Sparkles className="h-5 w-5 text-purple-300 mx-auto mb-1" />
                              <span className="text-xs text-purple-300">Select Card</span>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                    
                    <div className="flex items-center gap-2 mt-3 mb-2">
                      <div className="flex-1 h-px bg-purple-500/30"></div>
                      <span className="text-purple-300 text-xs">Synthesis Requirements</span>
                      <div className="flex-1 h-px bg-purple-500/30"></div>
                    </div>
                    
                    <div className="flex justify-between items-center">
                      <div className="flex items-center gap-1.5 bg-black/30 px-2 py-1 rounded-md">
                        <Sparkles className="h-3.5 w-3.5 text-blue-400" />
                        <span className="font-medium text-xs text-blue-200">Requires 250 Fragments</span>
                      </div>
                      <div className="flex items-center gap-1.5 bg-black/30 px-2 py-1 rounded-md">
                        <Sparkles className="h-3.5 w-3.5 text-blue-400" />
                        <span className="font-medium text-xs text-blue-200">Current: {assets.fragments} Fragments</span>
                      </div>
                    </div>
                  </div>
                  
                  {/* 合成说明与按钮 */}
                  <div className="bg-purple-900/20 rounded-lg p-3">
                    <h3 className="text-sm font-medium text-white mb-2">Synthesis Guide</h3>
                    <div className="space-y-2 text-xs text-purple-200 mb-3">
                      <p>• Select 3 different cards as materials</p>
                      <p>• Consume 250 fragments for synthesis</p>
                      <p>• Synthesis card quality depends on materials</p>
                      <p>• Higher rarity materials increase chance of rare cards</p>
                    </div>
                    
                    <Button
                      className="w-full bg-gradient-to-r from-blue-600 to-purple-600 text-white font-medium h-9 rounded-md shadow-md hover:shadow-blue-500/20 transition-all duration-200"
                      onClick={handleSynthesizeCards}
                      disabled={synthesisCards.filter(Boolean).length !== 3 || assets.fragments < 250 || isLoading}
                    >
                      {isLoading ? (
                        <span className="flex items-center gap-2">
                          <div className="w-4 h-4 rounded-full border-2 border-white border-t-transparent animate-spin"></div>
                          Synthesizing...
                        </span>
                      ) : (
                        <span className="flex items-center gap-2">
                          <Sparkles className="h-4 w-4" />
                          Start Synthesis
                        </span>
                      )}
                    </Button>
                  </div>
                </div>
              </TabsContent>

              {/* 升级系统面板 */}
              <TabsContent value="upgrade" className="mt-2">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                  {/* 升级卡片选择区 */}
                  <div className="bg-purple-900/20 rounded-lg p-3 col-span-2">
                    <h3 className="text-sm font-medium text-white mb-2">Select Card to Upgrade</h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                      <div
                        className={`aspect-[3/4] bg-purple-900/30 rounded-lg flex items-center justify-center border-2 border-dashed ${upgradeCard ? "border-purple-500/70" : "border-purple-500/30"} cursor-pointer hover:bg-purple-900/40`}
                        onClick={handleSelectUpgradeCard}
                      >
                        {upgradeCard ? (
                          <div className="relative w-full h-full p-3">
                            <img
                              src={upgradeCard.image || "/placeholder.svg"}
                              alt={upgradeCard.name}
                              className="w-full h-full object-cover rounded-md"
                            />
                            <Badge
                              className={`absolute top-2 right-2 text-[10px] px-1.5 py-0.5
                                ${
                                  upgradeCard.rarity === "Common"
                                    ? "bg-blue-600"
                                    : upgradeCard.rarity === "Uncommon"
                                      ? "bg-purple-600"
                                      : "bg-orange-600"
                                }
                              `}
                            >
                              {upgradeCard.rarity}
                            </Badge>
                            <div className="absolute top-1 right-1 bg-black/70 rounded-bl-md p-0.5">
                              <X
                                className="h-3 w-3 text-red-400 cursor-pointer"
                                onClick={(e) => {
                                  e.stopPropagation()
                                  setUpgradeCard(null)
                                }}
                              />
                            </div>
                          </div>
                        ) : (
                          <div className="text-center">
                            <Layers className="h-6 w-6 text-purple-300 mx-auto mb-1" />
                            <span className="text-sm text-purple-300">Select Card</span>
                          </div>
                        )}
                      </div>
                      
                      <div className="flex flex-col justify-center space-y-3">
                        <div className="bg-black/30 rounded-lg p-3">
                          <h4 className="text-xs font-medium text-white mb-2">Upgrade Effects</h4>
                          <div className="space-y-2">
                            <div className="flex items-center gap-2">
                              <Clock className="h-4 w-4 text-blue-400" />
                              <span className="text-xs text-purple-200">Cooldown -10%</span>
                            </div>
                            <div className="flex items-center gap-2">
                              <Sparkles className="h-4 w-4 text-pink-400" />
                              <span className="text-xs text-purple-200">Card Effect +15%</span>
                            </div>
                          </div>
                        </div>
                        
                        <div className="flex items-center gap-1.5 bg-black/30 px-2 py-1.5 rounded-md">
                          <Sparkles className="h-3.5 w-3.5 text-blue-400" />
                          <span className="font-medium text-xs text-blue-200">Requires 150 Fragments</span>
                        </div>
                        
                        <div className="flex items-center gap-1.5 bg-black/30 px-2 py-1.5 rounded-md">
                          <Sparkles className="h-3.5 w-3.5 text-blue-400" />
                          <span className="font-medium text-xs text-blue-200">Current: {assets.fragments} Fragments</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  {/* 升级说明与按钮 */}
                  <div className="bg-purple-900/20 rounded-lg p-3">
                    <h3 className="text-sm font-medium text-white mb-2">Upgrade Guide</h3>
                    <div className="space-y-2 text-xs text-purple-200 mb-3">
                      <p>• Select a card to upgrade</p>
                      <p>• Consume 150 fragments for upgrade</p>
                      <p>• Reduces cooldown by 10%</p>
                      <p>• Increases card effect by 15%</p>
                      <p>• Each card can be upgraded up to 3 times</p>
                    </div>
                    
                    <Button
                      className="w-full bg-gradient-to-r from-blue-600 to-purple-600 text-white font-medium h-9 rounded-md shadow-md hover:shadow-blue-500/20 transition-all duration-200"
                      onClick={handleUpgradeCardAction}
                      disabled={!upgradeCard || assets.fragments < 150 || isLoading}
                    >
                      {isLoading ? (
                        <span className="flex items-center gap-2">
                          <div className="w-4 h-4 rounded-full border-2 border-white border-t-transparent animate-spin"></div>
                          Upgrading...
                        </span>
                      ) : (
                        <span className="flex items-center gap-2">
                          <Sparkles className="h-4 w-4" />
                          Start Upgrade
                        </span>
                      )}
                    </Button>
                  </div>
                </div>
              </TabsContent>

              {/* 抽卡系统面板 */}
              <TabsContent value="gacha" className="mt-2">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                  {/* 卡包选择区 */}
                  <div className="bg-purple-900/20 rounded-lg p-3 col-span-2">
                    <div className="flex justify-between items-center mb-2">
                      <h3 className="text-sm font-medium text-white">Available Card Packs</h3>
                      <div className="flex gap-1">
                        {cardPacks.map((_, index) => (
                          <div
                            key={index}
                            className={`w-2 h-2 rounded-full cursor-pointer ${index === activePackIndex ? "bg-purple-500" : "bg-purple-500/30"}`}
                            onClick={() => {
                              setActivePackIndex(index)
                              setSelectedPack(cardPacks[index])
                            }}
                          ></div>
                        ))}
                      </div>
                    </div>
                    
                    <div className="relative mb-3">
                      <div className="flex items-center">
                        <button
                          className="absolute left-0 z-10 bg-black/50 rounded-full p-1 text-white hover:bg-purple-800/50"
                          onClick={() => handleChangePack("prev")}
                        >
                          <ChevronLeft className="h-4 w-4" />
                        </button>

                        <div className="w-full bg-gradient-to-br from-purple-900/50 to-indigo-900/50 rounded-lg p-3 border border-purple-500/30">
                          <div className="grid grid-cols-3 gap-3">
                            <div className="aspect-[3/4] rounded-lg overflow-hidden relative">
                              <img
                                src={selectedPack.image || "/placeholder.svg"}
                                alt={selectedPack.name}
                                className="w-full h-full object-cover"
                              />
                              <div className="absolute inset-0 bg-gradient-to-t from-black/70 to-transparent flex items-end justify-center p-2">
                                <div className="flex items-center gap-1 bg-black/60 rounded-full px-2 py-1">
                                  <Coins className="h-3 w-3 text-yellow-400" />
                                  <span className="text-white text-xs font-bold">{selectedPack.price}</span>
                                </div>
                              </div>
                            </div>

                            <div className="col-span-2">
                              <h4 className="text-white font-bold text-sm mb-1">{selectedPack.name}</h4>
                              <p className="text-purple-200 text-xs mb-3 line-clamp-3">{selectedPack.description}</p>

                              <h5 className="text-white text-xs font-medium mb-1">Drop Probabilities:</h5>
                              <div className="space-y-1.5">
                                <div className="flex items-center gap-2">
                                  <div className="w-full bg-black/30 h-1.5 rounded-full">
                                    <div
                                      className="bg-blue-600 h-1.5 rounded-full"
                                      style={{ width: `${selectedPack.dropRates.Common}%` }}
                                    ></div>
                                  </div>
                                  <span className="text-blue-400 text-xs min-w-[80px] text-right">
                                    Common {selectedPack.dropRates.Common}%
                                  </span>
                                </div>
                                <div className="flex items-center gap-2">
                                  <div className="w-full bg-black/30 h-1.5 rounded-full">
                                    <div
                                      className="bg-purple-600 h-1.5 rounded-full"
                                      style={{ width: `${selectedPack.dropRates.Uncommon}%` }}
                                    ></div>
                                  </div>
                                  <span className="text-purple-400 text-xs min-w-[80px] text-right">
                                    Uncommon {selectedPack.dropRates.Uncommon}%
                                  </span>
                                </div>
                                <div className="flex items-center gap-2">
                                  <div className="w-full bg-black/30 h-1.5 rounded-full">
                                    <div
                                      className="bg-orange-600 h-1.5 rounded-full"
                                      style={{ width: `${selectedPack.dropRates.Rare}%` }}
                                    ></div>
                                  </div>
                                  <span className="text-orange-400 text-xs min-w-[80px] text-right">
                                    Rare {selectedPack.dropRates.Rare}%
                                  </span>
                                </div>
                                <div className="flex items-center gap-2">
                                  <div className="w-full bg-black/30 h-1.5 rounded-full">
                                    <div
                                      className="bg-red-600 h-1.5 rounded-full"
                                      style={{ width: `${selectedPack.dropRates.Legendary}%` }}
                                    ></div>
                                  </div>
                                  <span className="text-red-400 text-xs min-w-[80px] text-right">
                                    Legendary {selectedPack.dropRates.Legendary}%
                                  </span>
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>

                        <button
                          className="absolute right-0 z-10 bg-black/50 rounded-full p-1 text-white hover:bg-purple-800/50"
                          onClick={() => handleChangePack("next")}
                        >
                          <ChevronRight className="h-4 w-4" />
                        </button>
                      </div>
                    </div>
                    
                    {/* 抽卡按钮 */}
                    <div className="flex justify-center mb-3">
                      {isDrawing ? (
                        <div className="w-full">
                          <div className="text-center text-purple-200 text-xs mb-1">Drawing card...</div>
                          <Progress value={drawProgress} className="h-2 bg-purple-900/50" />
                        </div>
                      ) : (
                        <Button
                          className="bg-gradient-to-r from-yellow-600 to-amber-600 text-white font-medium h-9 px-6 rounded-full text-sm shadow-lg hover:shadow-amber-500/20 hover:scale-105 transition-all duration-200 flex items-center gap-2"
                          onClick={handleDrawCard}
                          disabled={assets.coins < selectedPack.price}
                        >
                          <Flame className="h-4 w-4 text-yellow-300" />
                          <span>Draw Card</span>
                          <Flame className="h-4 w-4 text-yellow-300" />
                        </Button>
                      )}
                    </div>
                    
                    <div className="flex items-center justify-between bg-black/30 rounded-lg px-3 py-2">
                      <span className="text-xs text-purple-200">Current Coins:</span>
                      <div className="flex items-center gap-1.5">
                        <Coins className="h-3.5 w-3.5 text-yellow-400" />
                        <span className="font-medium text-sm text-yellow-100">{assets.coins}</span>
                      </div>
                    </div>
                  </div>
                  
                  {/* 抽卡历史记录 */}
                  <div className="bg-purple-900/20 rounded-lg p-3">
                    <h3 className="text-sm font-medium text-white mb-2">Recent Draws</h3>
                    
                    {drawHistory.length > 0 ? (
                      <div className="space-y-2 max-h-80 overflow-y-auto pr-1">
                        {drawHistory.map((draw) => (
                          <div key={draw.id} className="bg-black/30 rounded-lg p-2 flex gap-2">
                            <div className="w-10 h-14 rounded overflow-hidden flex-shrink-0">
                              <img
                                src={draw.card.image || "/placeholder.svg"}
                                alt={draw.card.name}
                                className="w-full h-full object-cover"
                              />
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="flex justify-between items-start">
                                <h5 className="text-white text-xs font-medium truncate">{draw.card.name}</h5>
                                <Badge
                                  className={`
                                    text-[8px] px-1 py-0 h-3.5 flex-shrink-0
                                    ${
                                      draw.card.rarity === "Common"
                                        ? "bg-blue-600"
                                        : draw.card.rarity === "Uncommon"
                                          ? "bg-purple-600"
                                          : draw.card.rarity === "Rare"
                                            ? "bg-orange-600"
                                            : "bg-red-600"
                                    }
                                  `}
                                >
                                  {draw.card.rarity}
                                </Badge>
                              </div>
                              <div className="text-[10px] text-purple-300 mt-1">Source: {draw.packName}</div>
                              <div className="text-[10px] text-purple-300">Time: {draw.timestamp}</div>
                            </div>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div className="bg-black/30 rounded-lg p-3 text-center">
                        <p className="text-purple-300 text-xs">No draw history</p>
                      </div>
                    )}
                  </div>
                </div>
              </TabsContent>
            </Tabs>
          </div>
        </div>

        {/* 卡片选择模态框 */}
        {showCardSelector && (
          <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center p-4">
            <div className="bg-gradient-to-br from-purple-900 to-indigo-900 rounded-lg w-full max-w-lg p-4">
              <h3 className="text-white font-medium mb-3 flex items-center justify-between">
                <span>Select Card for {selectorType === "synthesis" ? "Synthesis" : "Upgrade"}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 w-6 p-0 rounded-full"
                  onClick={() => setShowCardSelector(false)}
                >
                  <X className="h-4 w-4 text-gray-300" />
                </Button>
              </h3>

              <div className="grid grid-cols-4 sm:grid-cols-5 gap-2 max-h-60 overflow-y-auto mb-4 p-1">
                {assets.cards
                  .filter((card) => card.count > 0)
                  .map((card) => (
                    <div
                      key={card.id}
                      className="bg-black/30 rounded-lg cursor-pointer hover:bg-purple-800/30 transition-colors p-1.5"
                      onClick={() => handleCardSelectorSelect(card)}
                    >
                      <img
                        src={card.image || "/placeholder.svg"}
                        alt={card.name}
                        className="w-full aspect-[3/4] object-cover rounded-md mb-1"
                      />
                      <div className="flex justify-between items-center">
                        <Badge
                          className={`
                          text-[8px] px-0.5 py-0 h-3
                          ${
                            card.rarity === "Common"
                              ? "bg-blue-600"
                              : card.rarity === "Uncommon"
                                ? "bg-purple-600"
                                : "bg-orange-600"
                          }
                        `}
                        >
                          {card.rarity}
                        </Badge>
                        <span className="text-white text-[8px]">x{card.count}</span>
                      </div>
                    </div>
                  ))}
              </div>

              <div className="flex justify-end gap-2">
                <Button
                  variant="outline"
                  className="border-purple-500 text-purple-200 bg-purple-950/50 hover:bg-purple-800/50 hover:text-white"
                  onClick={() => setShowCardSelector(false)}
                >
                  Cancel
                </Button>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}