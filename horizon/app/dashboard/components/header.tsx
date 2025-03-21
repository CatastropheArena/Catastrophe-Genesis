"use client"

import React from 'react';
import { Button } from "@/components/ui/button";
import { Coins, Sparkles, Wallet } from "lucide-react";
import { Assets } from "@/app/types";

// 定义组件接口
interface HeaderProps {
  assets: Assets;
  onWalletClick?: () => void;
}

export default function Header({ assets, onWalletClick }: HeaderProps) {
  return (
    <header className="bg-black/30 backdrop-blur-md border-b border-purple-500/20 sticky top-0 z-40">
      <div className="container mx-auto px-2 py-2">
        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2">
            <img src="/placeholder.svg?height=40&width=40" alt="Logo" className="h-6 w-6" />
            <h1 className="text-base font-bold text-white hidden md:block">Exploding Cats</h1>
          </div>

          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
              <Coins className="h-3 w-3 text-yellow-400" />
              <span className="text-white font-medium text-xs">{assets.coins}</span>
            </div>
            <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
              <Sparkles className="h-3 w-3 text-blue-400" />
              <span className="text-white font-medium text-xs">{assets.fragments}</span>
            </div>
            <Button 
              variant="ghost" 
              size="icon" 
              className="text-white h-7 w-7"
              onClick={onWalletClick}
            >
              <Wallet className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </header>
  );
}