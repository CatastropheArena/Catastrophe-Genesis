"use client"

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Coins, Layers, Loader, RefreshCcw } from "lucide-react"
import { StakedCard, StakingPool } from "@/app/types"

// 定义组件props接口
interface CardStakingPoolsProps {
  myStakedCards: StakedCard[];
  stakingPools: StakingPool[];
  handleUnstakeCard: (card: StakedCard) => void;
  handleStakeCard: (pool: StakingPool) => void;
  handleClaimRewards: () => void;
}

export default function CardStakingPools({
  myStakedCards,
  stakingPools,
  handleUnstakeCard,
  handleStakeCard,
  handleClaimRewards
}: CardStakingPoolsProps) {
  // 添加状态以跟踪是否正在领取奖励
  const [isClaimingRewards, setIsClaimingRewards] = useState(false);
  
  // 添加奖励数据的状态管理
  const [rewardsData, setRewardsData] = useState({
    dailyEarnings: 15,
    totalEarned: 45,
    availableRewards: 45
  });
  
  // 添加已领取奖励的状态跟踪
  const [claimedRewards, setClaimedRewards] = useState(0);
  
  // 添加加载状态
  const [isLoadingRewards, setIsLoadingRewards] = useState(true);
  const [loadingError, setLoadingError] = useState<string | null>(null);
  
  // 在组件加载时获取最新奖励数据
  useEffect(() => {
    fetchRewardsData();
  }, []);
  
  // 获取奖励数据的函数
  const fetchRewardsData = async () => {
    setIsLoadingRewards(true);
    setLoadingError(null);
    
    try {
      // 这里应该是真实的API调用，暂时模拟异步获取数据
      const response = await new Promise<{
        dailyEarnings: number;
        totalEarned: number;
        availableRewards: number;
      }>((resolve) => {
        setTimeout(() => {
          // 计算总共可获得的奖励
          const totalAvailableRewards = myStakedCards.reduce((total, card) => total + card.earned, 0);
          // 模拟从API获取的数据，减去已领取的奖励
          resolve({
            dailyEarnings: myStakedCards.reduce((total, card) => total + card.earned, 0),
            totalEarned: myStakedCards.length * 15, // 示例计算
            availableRewards: Math.max(0, totalAvailableRewards - claimedRewards)
          });
        }, 700); // 模拟网络延迟
      });
      
      setRewardsData(response);
    } catch (error) {
      console.error("Failed to fetch rewards data:", error);
      setLoadingError("无法加载奖励数据，请稍后再试。");
    } finally {
      setIsLoadingRewards(false);
    }
  };
  
  // 处理领取奖励的函数，添加状态管理
  const onClaimRewards = async () => {
    if (isClaimingRewards || rewardsData.availableRewards <= 0) return;
    
    try {
      setIsClaimingRewards(true);
      await handleClaimRewards();
      
      // 更新已领取的奖励总额
      setClaimedRewards(prev => prev + rewardsData.availableRewards);
      
      // 重新获取最新的奖励数据
      await fetchRewardsData();
    } catch (error) {
      console.error("Failed to claim rewards:", error);
      setLoadingError("领取奖励失败，请稍后再试。");
    } finally {
      setIsClaimingRewards(false);
    }
  };

  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <Coins className="h-4 w-4 text-yellow-400" />
          Card Staking Pools
          {!isLoadingRewards && (
                  <Button 
                    variant="ghost" 
                    size="icon" 
                    className="h-2 w-2 p-2 pb-1 hover:bg-transparent"
                    onClick={fetchRewardsData}
                    disabled={isLoadingRewards || isClaimingRewards}
                  >
                    <RefreshCcw className="h-3 w-3 text-purple-300 hover:text-purple-100" />
                  </Button>
                )}
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Stake your cards to earn passive income from the rental pool
        </CardDescription>
      </CardHeader>
      <CardContent className="p-3">
        <div className="space-y-3">
          {/* My Staked Cards */}
          <div className="bg-black/30 rounded-lg p-3">
            <h3 className="text-sm font-medium text-white mb-2">My Staked Cards</h3>

            <div className="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-4 lg:grid-cols-5 gap-2">
              {myStakedCards.map((card) => (
                <div key={card.id} className="bg-purple-900/20 rounded-lg p-1.5 flex gap-1.5">
                  <img
                    src={card.image || "/placeholder.svg"}
                    alt={card.name}
                    className="w-8 h-10 object-cover rounded-md"
                  />
                  <div className="flex-1">
                    <div className="flex justify-between items-center">
                      <h4 className="font-medium text-white text-[10px]">{card.name}</h4>
                      <Badge
                        className={`
                          text-[8px] px-0.5 py-0 h-3
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
                    <div className="mt-0.5 space-y-0.5">
                      <div className="flex justify-between text-[8px]">
                        <span className="text-purple-300">Staked:</span>
                        <span className="text-white">{card.stakedCount} cards</span>
                      </div>
                      <div className="flex justify-between text-[8px]">
                        <span className="text-purple-300">Pool share:</span>
                        <span className="text-white">{card.poolShare}</span>
                      </div>
                      <div className="flex justify-between text-[8px]">
                        <span className="text-purple-300">Earned:</span>
                        <span className="text-yellow-400 flex items-center gap-0.5">
                          <Coins className="h-2 w-2" /> {card.earned}
                        </span>
                      </div>
                    </div>
                    <Button
                      size="sm"
                      className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 h-4 text-[8px] px-1 mt-1"
                      onClick={() => handleUnstakeCard(card)}
                    >
                      Unstake
                    </Button>
                  </div>
                </div>
              ))}

              <div
                className="flex items-center justify-center border-2 border-dashed border-purple-500/30 rounded-lg p-1.5 cursor-pointer hover:bg-purple-900/20 transition-colors"
                onClick={() => {
                  document.getElementById("staking-pools-section")?.scrollIntoView({ behavior: "smooth" })
                }}
              >
                <div className="text-center">
                  <div className="mx-auto rounded-full bg-purple-900/50 w-6 h-6 flex items-center justify-center mb-1">
                    <Layers className="h-3 w-3 text-purple-300" />
                  </div>
                  <p className="text-purple-300 font-medium text-[10px]">Stake More</p>
                </div>
              </div>
            </div>
          </div>

          {/* Staking Rewards */}
          <div className="grid grid-cols-3 gap-2">
            <div className="bg-purple-900/20 rounded-lg p-2">
              <div className="flex justify-between items-center mb-1">
                <h4 className="text-purple-200 text-xs">Daily Earnings</h4>
                <Coins className="h-3 w-3 text-yellow-400" />
              </div>
              {isLoadingRewards ? (
                <div className="text-base font-bold text-white flex items-center gap-2">
                  <Loader className="h-3 w-3 animate-spin" />
                  <span className="text-purple-200 opacity-70">Loading...</span>
                </div>
              ) : (
                <p className="text-base font-bold text-white">{rewardsData.dailyEarnings} COIN</p>
              )}
              <div className="mt-1 text-[10px] text-purple-300">Based on current pool share</div>
            </div>

            <div className="bg-purple-900/20 rounded-lg p-2">
              <div className="flex justify-between items-center mb-1">
                <h4 className="text-purple-200 text-xs">Total Earned</h4>
                <Coins className="h-3 w-3 text-yellow-400" />
              </div>
              {isLoadingRewards ? (
                <div className="text-base font-bold text-white flex items-center gap-2">
                  <Loader className="h-3 w-3 animate-spin" />
                  <span className="text-purple-200 opacity-70">Loading...</span>
                </div>
              ) : (
                <p className="text-base font-bold text-white">{rewardsData.totalEarned} COIN</p>
              )}
              <div className="mt-1 text-[10px] text-purple-300">Since you started staking</div>
            </div>

            <div className="bg-purple-900/20 rounded-lg p-2">
              <div className="flex justify-between items-center mb-1">
                <h4 className="text-purple-200 text-xs">Available</h4>
                <Coins className="h-3 w-3 text-yellow-400" />
              </div>
              {isLoadingRewards ? (
                <div className="text-base font-bold text-white flex items-center gap-2">
                  <Loader className="h-3 w-3 animate-spin" />
                  <span className="text-purple-200 opacity-70">Loading...</span>
                </div>
              ) : (
                <p className="text-base font-bold text-white">{rewardsData.availableRewards} COIN</p>
              )}
              <Button
                size="sm"
                className="mt-1 w-full bg-gradient-to-r from-yellow-600 to-amber-600 h-6 text-[10px]"
                onClick={onClaimRewards}
                disabled={isClaimingRewards || rewardsData.availableRewards <= 0 || isLoadingRewards}
              >
                {isClaimingRewards ? (
                  <span className="flex items-center gap-1">
                    <Loader className="h-3 w-3 animate-spin" /> 领取中...
                  </span>
                ) : (
                  "领取"
                )}
              </Button>
              {loadingError && (
                <div className="text-red-400 text-[10px] mt-1">{loadingError}</div>
              )}
            </div>
          </div>

          {/* Available Staking Pools */}
          <div id="staking-pools-section" className="bg-black/30 rounded-lg p-3">
            <h3 className="text-sm font-medium text-white mb-2">Available Staking Pools</h3>

            <div className="overflow-x-auto">
              <table className="w-full text-xs text-left">
                <thead className="text-[10px] text-purple-300 uppercase bg-black/30">
                  <tr>
                    <th className="px-2 py-1 rounded-tl-lg">Card Type</th>
                    <th className="px-2 py-1">Rarity</th>
                    <th className="px-2 py-1">Pool Size</th>
                    <th className="px-2 py-1">APR</th>
                    <th className="px-2 py-1">Rate</th>
                    <th className="px-2 py-1 rounded-tr-lg">Action</th>
                  </tr>
                </thead>
                <tbody>
                  {stakingPools.map((pool) => (
                    <tr key={pool.id} className="bg-black/20 border-b border-purple-500/10">
                      <td className="px-2 py-1 flex items-center gap-1">
                        <div className="w-6 h-6 rounded bg-purple-900/30 overflow-hidden">
                          <img
                            src={pool.image || "/placeholder.svg"}
                            alt={pool.name}
                            className="w-full h-full object-cover"
                          />
                        </div>
                        <span className="text-white">{pool.name}</span>
                      </td>
                      <td className="px-2 py-1">
                        <Badge
                          className={`
                            text-[10px] px-1 py-0
                            ${
                              pool.rarity === "Common"
                                ? "bg-blue-600"
                                : pool.rarity === "Uncommon"
                                  ? "bg-purple-600"
                                  : "bg-orange-600"
                            }
                          `}
                        >
                          {pool.rarity}
                        </Badge>
                      </td>
                      <td className="px-2 py-1 text-purple-200">{pool.poolSize}</td>
                      <td className="px-2 py-1 text-green-400">{pool.apr}</td>
                      <td className="px-2 py-1 text-yellow-400 flex items-center gap-0.5">
                        <Coins className="h-2 w-2" /> {pool.rate}/day
                      </td>
                      <td className="px-2 py-1">
                        <Button
                          size="sm"
                          className="bg-gradient-to-r from-purple-600 to-pink-600 h-6 text-[10px] px-2"
                          onClick={() => handleStakeCard(pool)}
                        >
                          Stake
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}