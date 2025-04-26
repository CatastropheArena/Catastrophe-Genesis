"use client";

import React, { useState, useContext } from "react";
import { Coins, Sparkles } from "lucide-react";
import { AppContext } from "@/context/AppContext";
import { ConnectModal } from "@mysten/dapp-kit";
import ConnectMenu from "@/app/components/ui/connectMenu";
import { Link as LinkIcon } from "lucide-react";

export default function Header() {
  const [coins, setCoins] = useState(1000); // 使用mock数据初始化
  const [fragments, setFragments] = useState(500); // 使用mock数据初始化

  const { walletAddress, suiName } = useContext(AppContext);

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
            {/* Connect Button */}
            {walletAddress ? (
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
                <ConnectMenu walletAddress={walletAddress} suiName={suiName} />
              </>
            ) : (
              <ConnectModal
                trigger={
                  <button
                    className="h-full rounded-[11px] outline-none ring-0 xl:button-animate-105 overflow-hidden p-[1px]"
                    disabled={!!walletAddress}
                  >
                    <div className="h-full px-5 py-4 flex items-center gap-2 rounded-xl bg-white/10">
                      <span className="text-sm">
                        {walletAddress ? "Connected" : "Connect Wallet"}
                      </span>
                      <LinkIcon size={17} className="text-white" />
                    </div>
                  </button>
                }
              />
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
