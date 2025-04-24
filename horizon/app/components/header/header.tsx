"use client";

import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Coins, Sparkles, LogOut, ChevronDown } from "lucide-react";
import {
  useCurrentAccount,
  useSuiClient,
  ConnectButton,
  useDisconnectWallet,
} from "@mysten/dapp-kit";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export default function Header() {
  const [coins, setCoins] = useState(1000); // 使用mock数据初始化
  const [fragments, setFragments] = useState(500); // 使用mock数据初始化

  const account = useCurrentAccount();
  const suiClient = useSuiClient();
  const disconnectWallet = useDisconnectWallet();

  useEffect(() => {
    const fetchCoinBalance = async () => {
      if (account) {
        try {
          const balance = await suiClient.getBalance({
            owner: account.address,
            coinType: "0x2::sui::SUI",
          });
          setCoins(Number(balance.totalBalance));
        } catch (error) {
          console.error("获取代币余额失败:", error);
        }
      }
    };

    fetchCoinBalance();
  }, [account, suiClient]);

  // 格式化钱包地址
  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const handleDisconnect = async () => {
    try {
      await disconnectWallet.mutateAsync();
    } catch (error) {
      console.error("断开钱包连接失败:", error);
    }
  };

  return (
    <header className="bg-purple-900/80 backdrop-blur-lg sticky top-0 z-40">
      <div className="container mx-auto px-2 py-2">
        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2">
            <img
              src="/placeholder.svg?height=40&width=40"
              alt="Logo"
              className="h-6 w-6"
            />
            <h1 className="text-base font-bold text-white hidden md:block">
              Exploding Cats
            </h1>
          </div>

          <div className="flex items-center gap-2">
            {account && (
              <>
                <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                  <Coins className="h-3 w-3 text-yellow-400" />
                  <span className="text-white font-medium text-xs">
                    {coins}
                  </span>
                </div>
                <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                  <Sparkles className="h-3 w-3 text-blue-400" />
                  <span className="text-white font-medium text-xs">
                    {fragments}
                  </span>
                </div>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-white flex items-center gap-1 px-2 py-1 h-7"
                    >
                      {formatAddress(account.address)}
                      <ChevronDown className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="w-48">
                    <DropdownMenuItem
                      onClick={handleDisconnect}
                      className="text-red-500 cursor-pointer"
                    >
                      <LogOut className="mr-2 h-4 w-4" />
                      <span>断开连接</span>
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </>
            )}
            {!account && (
              <ConnectButton className="bg-purple-600 hover:bg-purple-700 text-white px-4 py-1 rounded-full text-sm" />
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
