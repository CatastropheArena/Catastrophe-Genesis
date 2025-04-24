"use client"

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { AlertCircle, Clock, Coins } from "lucide-react"
import { RentalCard, MyRentalCard, RentalHistoryItem } from "@/app/types"

// 定义组件props接口
interface CardRentalMarketplaceProps {
  rentalCards: RentalCard[];
  myRentedCards: MyRentalCard[];
  rentalHistory: RentalHistoryItem[];
  
  // 回调函数
  handleRentCard: (card: RentalCard, uses: number, period: number) => void;
}

export default function CardRentalMarketplace({
  rentalCards,
  myRentedCards,
  rentalHistory,
  handleRentCard,
}: CardRentalMarketplaceProps) {
  // 本地状态
  const [rentalFilter, setRentalFilter] = useState<string>("All");
  const [selectedUses, setSelectedUses] = useState<Record<string | number, number>>({});
  const [selectedPeriod, setSelectedPeriod] = useState<Record<string | number, number>>({});

  // 过滤卡片逻辑
  const filteredRentalCards = rentalCards.filter(card => {
    if (rentalFilter === "All") return true;
    return card.rarity === rentalFilter;
  });

  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <Clock className="h-4 w-4 text-green-400" />
          Card Rental Marketplace
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Rent cards from the staking pools to enhance your gameplay
        </CardDescription>
      </CardHeader>
      <CardContent className="p-3">
        <Tabs defaultValue="marketplace" className="w-full">
          <TabsList className="w-full max-w-md mx-auto mb-3 bg-black/20 p-1 rounded-lg">
            <TabsTrigger value="marketplace" className="flex-1 data-[state=active]:bg-purple-700 h-7 text-xs">
              Marketplace
            </TabsTrigger>
            <TabsTrigger value="rented" className="flex-1 data-[state=active]:bg-purple-700 h-7 text-xs">
              My Rentals
            </TabsTrigger>
          </TabsList>

          <TabsContent value="marketplace" className="mt-0">
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <h3 className="text-sm font-medium text-white">Available Card Pools</h3>
                <div className="flex items-center gap-1">
                  <span className="text-xs text-purple-200">Filter:</span>
                  <select
                    className="bg-black/30 text-white text-xs rounded border border-purple-500/30 px-1 py-0.5"
                    value={rentalFilter}
                    onChange={(e) => setRentalFilter(e.target.value)}
                  >
                    <option>All</option>
                    <option>Common</option>
                    <option>Uncommon</option>
                    <option>Rare</option>
                  </select>
                </div>
              </div>

              <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 gap-1.5">
                {filteredRentalCards.map((card) => (
                  <div
                    key={card.id}
                    className="bg-black/30 rounded-lg overflow-hidden hover:bg-purple-900/20 transition-colors"
                  >
                    <div className="relative">
                      <img
                        src={card.image || "/placeholder.svg"}
                        alt={card.name}
                        className="w-full h-16 object-cover"
                      />
                      <Badge
                        className={`
                          absolute top-1 right-1 text-[8px] px-0.5 py-0 h-3
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
                      <div className="absolute bottom-1 right-1 bg-black/60 text-white text-[7px] px-1 py-0.5 rounded-full">
                        Pool: {card.poolSize}
                      </div>
                    </div>
                    <div className="p-1.5">
                      <div className="flex justify-between items-start mb-0.5">
                        <h4 className="font-medium text-white text-[10px]">{card.name}</h4>
                        <div className="flex items-center text-yellow-400 text-[8px]">
                          <Coins className="h-2 w-2 mr-0.5" />
                          <span>{card.rate}/d</span>
                        </div>
                      </div>
                      <div className="space-y-0.5 mb-1.5">
                        <div className="flex justify-between text-[8px]">
                          <span className="text-purple-300">Uses:</span>
                          <select
                            className="bg-black/95 text-gray-200 text-[7px] rounded border border-purple-400/50 px-0.5 h-3 hover:border-purple-300"
                            value={selectedUses[card.id] || 5}
                            onChange={(e) =>
                              setSelectedUses({
                                ...selectedUses,
                                [card.id]: Number(e.target.value),
                              })
                            }
                          >
                            <option value={5}>5</option>
                            <option value={10}>10</option>
                          </select>
                        </div>
                        <div className="flex justify-between text-[8px]">
                          <span className="text-purple-300">Period:</span>
                          <select
                            className="bg-black/30 text-white text-[7px] rounded border border-purple-500/30 px-0.5 h-3"
                            value={selectedPeriod[card.id] || 1}
                            onChange={(e) =>
                              setSelectedPeriod({
                                ...selectedPeriod,
                                [card.id]: Number.parseInt(e.target.value),
                              })
                            }
                          >
                            <option value={1}>1d</option>
                            <option value={3}>3d</option>
                          </select>
                        </div>
                      </div>
                      <Button
                        className="w-full bg-gradient-to-r from-green-600 to-teal-600 h-5 text-[8px] px-1"
                        onClick={() => handleRentCard(
                          card, 
                          selectedUses[card.id] || 5, 
                          selectedPeriod[card.id] || 1
                        )}
                      >
                        Rent
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="rented" className="mt-0">
            <div className="space-y-3">
              <h3 className="text-sm font-medium text-white">My Rented Cards</h3>

              <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-1.5">
                {myRentedCards.length > 0 ? (
                  myRentedCards.map((card) => (
                    <div key={card.id} className="bg-black/30 rounded-lg overflow-hidden">
                      <div className="relative">
                        <img
                          src={card.image || "/placeholder.svg"}
                          alt={card.name}
                          className="w-full h-16 object-cover"
                        />
                        <Badge
                          className={`
                            absolute top-1 right-1 text-[8px] px-0.5 py-0 h-3
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
                      </div>
                      <div className="p-1.5">
                        <div className="flex justify-between items-start mb-0.5">
                          <h4 className="font-medium text-white text-[10px]">{card.name}</h4>
                          <div className="flex items-center text-yellow-400 text-[8px]">
                            <Coins className="h-2 w-2 mr-0.5" />
                            <span>{card.rate}/d</span>
                          </div>
                        </div>
                        <div className="space-y-0.5 mb-1">
                          <div className="flex justify-between text-[8px]">
                            <span className="text-purple-300">Uses:</span>
                            <span className="text-gray-100">{card.usesLeft} of {card.totalUses}</span>
                          </div>
                          <div className="flex justify-between text-[8px]">
                            <span className="text-purple-300">Expires:</span>
                            <span className="text-gray-100">{card.expiresIn}d</span>
                          </div>
                        </div>
                        <div className="w-full bg-black/30 rounded-full h-1 mb-0.5">
                          <div
                            className="bg-green-600 h-1 rounded-full"
                            style={{ width: `${(card.usesLeft / card.totalUses) * 100}%` }}
                          ></div>
                        </div>
                        <div className="text-center text-[7px] text-purple-200">{card.usesLeft} uses left</div>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="col-span-full flex flex-col items-center justify-center py-4 text-center">
                    <AlertCircle className="h-8 w-8 text-purple-400 mb-2" />
                    <p className="text-purple-200 text-sm">You haven't rented any cards yet.</p>
                    <p className="text-purple-300 text-xs mt-1">
                      Rent cards from the marketplace to enhance your gameplay.
                    </p>
                  </div>
                )}

                <div
                  className="flex items-center justify-center border-2 border-dashed border-purple-500/30 rounded-lg p-1.5 cursor-pointer hover:bg-purple-900/20 transition-colors h-full"
                  onClick={() => (document.querySelector('[value="marketplace"]') as HTMLElement)?.click()}
                >
                  <div className="text-center">
                    <div className="mx-auto rounded-full bg-purple-900/50 w-6 h-6 flex items-center justify-center mb-1">
                      <Clock className="h-3 w-3 text-purple-300" />
                    </div>
                    <p className="text-purple-300 font-medium text-[10px]">Rent More</p>
                  </div>
                </div>
              </div>

              {rentalHistory.length > 0 && (
                <div className="bg-black/30 rounded-lg p-2">
                  <h3 className="text-sm font-medium text-white mb-2">Rental History</h3>

                  <div className="overflow-x-auto">
                    <table className="w-full text-xs text-left">
                      <thead className="text-[10px] text-purple-300 uppercase bg-black/30">
                        <tr>
                          <th className="px-2 py-1 rounded-tl-lg">Card</th>
                          <th className="px-2 py-1">Source</th>
                          <th className="px-2 py-1">Period</th>
                          <th className="px-2 py-1">Uses</th>
                          <th className="px-2 py-1">Cost</th>
                          <th className="px-2 py-1 rounded-tr-lg">Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {rentalHistory.map((item) => (
                          <tr key={item.id} className="bg-black/20 border-b border-purple-500/10">
                            <td className="px-2 py-1 flex items-center gap-1">
                              <div className="w-6 h-6 rounded bg-purple-900/30 overflow-hidden">
                                <img
                                  src={item.cardImage || "/placeholder.svg?height=24&width=24"}
                                  alt={item.cardName}
                                  className="w-full h-full object-cover"
                                />
                              </div>
                              <span className="text-white">{item.cardName}</span>
                            </td>
                            <td className="px-2 py-1 text-purple-200">{item.source}</td>
                            <td className="px-2 py-1 text-purple-200">{item.period}d</td>
                            <td className="px-2 py-1 text-purple-200">{item.uses}</td>
                            <td className="px-2 py-1 text-yellow-400 flex items-center gap-0.5">
                              <Coins className="h-2 w-2" /> {item.cost}
                            </td>
                            <td className="px-2 py-1">
                              <Badge className={`${item.status === 'active' ? 'bg-green-600/80' : 'bg-red-600/80'} text-[10px] px-1 py-0`}>
                                {item.status === 'active' ? 'Active' : 'Expired'}
                              </Badge>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}