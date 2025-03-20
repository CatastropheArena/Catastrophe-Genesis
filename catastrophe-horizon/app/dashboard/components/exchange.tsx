"use client"

import React, { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import {
  ArrowRightLeft,
  ArrowUpDown,
  Coins,
  DollarSign,
  Sparkles,
} from "lucide-react"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Assets, TokenPrices, TokenInfo } from "@/app/types"

// 组件props接口
interface ExchangeProps {
  initialAssets?: Assets
  onAssetsChange?: (newAssets: Assets) => void
  onSwapComplete?: (from: string, to: string, fromAmount: number, toAmount: number) => void
}

export default function Exchange({
  initialAssets,
  onAssetsChange,
  onSwapComplete
}: ExchangeProps) {
  // 如果提供了初始资产则使用它，否则使用默认值
  const [assets, setAssets] = useState<Assets>(initialAssets || {
    coins: 1000,
    fragments: 500,
    usdt: 100,
    cards: []
  })

  const [fromToken, setFromToken] = useState<string>("coins")
  const [toToken, setToToken] = useState<string>("fragments") 
  const [amount, setAmount] = useState<string>("")

  const tokens: Record<string, TokenInfo> = {
    coins: { symbol: "COINS", icon: Coins, color: "text-yellow-400", name: "Coins" },
    fragments: { symbol: "FRAG", icon: Sparkles, color: "text-blue-400", name: "Fragments" },
    usdt: { symbol: "USDT", icon: DollarSign, color: "text-green-400", name: "USDT" }
  }

  const prices: TokenPrices = {
    coins_fragments: 0.1,
    fragments_coins: 10,
    usdt_coins: 100,
    coins_usdt: 0.01
  }

  const handleSwap = () => {
    const inputAmount = Number(amount)
    if (isNaN(inputAmount) || inputAmount <= 0) {
      alert("请输入有效金额")
      return
    }

    const rate = prices[`${fromToken}_${toToken}` as keyof typeof prices]
    const outputAmount = inputAmount * rate

    // 修复类型安全问题
    const fromValue = assets[fromToken as keyof Assets]
    if (typeof fromValue !== 'number') {
      alert("资产类型错误")
      return
    }

    if (inputAmount > fromValue) {
      alert(`${tokens[fromToken as keyof typeof tokens].name}余额不足`)
      return
    }

    // 修复类型安全问题
    const toValue = assets[toToken as keyof Assets]
    if (typeof toValue !== 'number') {
      alert("资产类型错误")
      return
    }

    const newAssets = {
      ...assets,
      [fromToken]: fromValue - inputAmount,
      [toToken]: toValue + outputAmount
    }

    setAssets(newAssets)
    
    // 通知父组件资产已更改
    if (onAssetsChange) {
      onAssetsChange(newAssets)
    }
    
    // 通知父组件交换已完成
    if (onSwapComplete) {
      onSwapComplete(fromToken, toToken, inputAmount, outputAmount)
    }

    setAmount("")
  }

  const switchTokens = () => {
    setFromToken(toToken)
    setToToken(fromToken)
    setAmount("")
  }

  // 安全地获取资产值并确保它是数字
  const getAssetValue = (key: string): number => {
    const value = assets[key as keyof Assets]
    return typeof value === 'number' ? value : 0
  }

  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <ArrowRightLeft className="h-4 w-4 text-purple-400" />
          Exchange
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Swap between different tokens
        </CardDescription>
      </CardHeader>
      <CardContent className="p-3">
        <div className="space-y-4">
          {/* Input Amount */}
          <div className="bg-black/30 rounded-lg p-3">
            <h3 className="text-sm font-medium text-white mb-2">Token Exchange</h3>
            <div className="space-y-4">
              <div className="relative group">
                <Input
                  type="number"
                  placeholder="0.0"
                  className="bg-black/50 border border-purple-400/50 h-16 px-4 pt-6 text-2xl text-white font-light"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                />
                <div className="absolute top-2 left-4 text-[10px] text-purple-200 font-medium tracking-wide">
                  Amount
                </div>
                <div className="absolute right-2 top-1/2 -translate-y-1/2">
                  <Select value={fromToken} onValueChange={setFromToken}>
                    <SelectTrigger className="bg-black/50 border border-purple-400/50 hover:bg-purple-900/30 hover:border-purple-400/70 transition-colors min-w-[100px] h-8">
                      <SelectValue>
                        <div className="flex items-center gap-1.5">
                          {React.createElement(tokens[fromToken as keyof typeof tokens].icon, {
                            className: `h-3 w-3 ${tokens[fromToken as keyof typeof tokens].color}`
                          })}
                          <span className="font-medium text-gray-100 text-xs">{tokens[fromToken as keyof typeof tokens].name}</span>
                        </div>
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent className="bg-black/95 border border-purple-400/50">
                      {Object.entries(tokens).map(([key, token]) => (
                        <SelectItem 
                          key={key} 
                          value={key} 
                          className="relative flex items-center hover:bg-purple-500/20 focus:bg-purple-500/20 focus:text-white data-[highlighted]:bg-purple-500/20 text-gray-200 cursor-pointer"
                        >
                          <div className="flex items-center gap-1.5 py-1 px-2">
                            {React.createElement(token.icon, {
                              className: `h-3 w-3 ${token.color}`
                            })}
                            <span className="text-xs">{token.name}</span>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="text-[10px] text-purple-200/80 px-1 font-light">
                Balance: {getAssetValue(fromToken)} {tokens[fromToken as keyof typeof tokens].symbol}
              </div>

              {/* Switch Button */}
              <div className="flex justify-center -my-2 relative z-10">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={switchTokens}
                  className="rounded-full bg-purple-500/5 hover:bg-purple-500/20 h-8 w-8 shadow-lg shadow-purple-500/20
                    transition-all duration-200 hover:scale-110 active:scale-95"
                >
                  <ArrowUpDown className="h-3 w-3 text-purple-400" />
                </Button>
              </div>

              {/* Output Amount */}
              <div className="relative group">
                <div className="bg-black/50 border border-purple-400/50 rounded-md h-16 px-4 pt-6 flex items-center text-2xl text-gray-400 font-light">
                  {amount ? (Number(amount) * prices[`${fromToken}_${toToken}` as keyof typeof prices]).toFixed(6) : "0.0"}
                </div>
                <div className="absolute top-2 left-4 text-[10px] text-purple-200 font-medium tracking-wide">
                  Estimated Output
                </div>
                <div className="absolute right-2 top-1/2 -translate-y-1/2">
                  <Select value={toToken} onValueChange={setToToken}>
                    <SelectTrigger className="bg-black/50 border border-purple-400/50 hover:bg-purple-900/30 hover:border-purple-400/70 transition-colors min-w-[100px] h-8">
                      <SelectValue>
                        <div className="flex items-center gap-1.5">
                          {React.createElement(tokens[toToken as keyof typeof tokens].icon, {
                            className: `h-3 w-3 ${tokens[toToken as keyof typeof tokens].color}`
                          })}
                          <span className="font-medium text-gray-100 text-xs">{tokens[toToken as keyof typeof tokens].name}</span>
                        </div>
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent className="bg-black/95 border border-purple-400/50">
                      {Object.entries(tokens).map(([key, token]) => (
                        <SelectItem 
                          key={key} 
                          value={key} 
                          className="relative flex items-center hover:bg-purple-500/20 focus:bg-purple-500/20 focus:text-white data-[highlighted]:bg-purple-500/20 text-gray-200 cursor-pointer"
                        >
                          <div className="flex items-center gap-1.5 py-1 px-2">
                            {React.createElement(token.icon, {
                              className: `h-3 w-3 ${token.color}`
                            })}
                            <span className="text-xs">{token.name}</span>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="text-[10px] text-purple-200/80 px-1 font-light">
                Balance: {getAssetValue(toToken)} {tokens[toToken as keyof typeof tokens].symbol}
              </div>

              <Button 
                onClick={handleSwap}
                className="w-full bg-gradient-to-r from-purple-600 to-pink-600 h-8 text-xs font-medium tracking-wide
                  transition-all duration-200 hover:scale-[1.02] active:scale-[0.98] shadow-xl shadow-purple-500/20
                  disabled:opacity-50 disabled:hover:scale-100 disabled:cursor-not-allowed"
                disabled={!amount || Number(amount) <= 0}
              >
                {Number(amount) > getAssetValue(fromToken) ? "Insufficient Balance" : "Confirm Swap"}
              </Button>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}