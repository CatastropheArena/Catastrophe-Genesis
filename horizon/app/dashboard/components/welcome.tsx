"use client"

import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import {
  Coins,
  Sparkles,
  Gift,
} from "lucide-react"
import { Assets } from "@/app/types"

interface WelcomeProps {
  assets: Assets;
  isVisible: boolean;
  onClose: () => void;
}

export default function Welcome({ assets, isVisible, onClose }: WelcomeProps) {
    // 如果不可见，则不渲染组件
    if (!isVisible) return null;
    return (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
        <Card className="w-full max-w-md bg-gradient-to-br from-purple-800 to-indigo-900 border-purple-500/30 animate-float">
          <CardHeader className="p-4">
            <CardTitle className="text-xl text-center text-white">Welcome to Exploding Cats!</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 p-4">
            <div className="flex justify-center mb-2">
              <div className="relative">
                <Gift className="h-12 w-12 text-pink-400" />
                <div className="absolute -top-2 -right-2 bg-yellow-500 text-black text-xs font-bold rounded-full h-5 w-5 flex items-center justify-center animate-pulse">
                  !
                </div>
              </div>
            </div>

            <p className="text-center text-purple-100 text-sm">You've received your initial assets:</p>

            <div className="grid grid-cols-2 gap-2 my-2">
              <div className="bg-white/10 rounded-lg p-2 flex items-center gap-2">
                <Coins className="h-4 w-4 text-yellow-400" />
                <div>
                  <p className="text-xs text-purple-200">Coins</p>
                  <p className="font-bold text-white text-sm">{assets.coins}</p>
                </div>
              </div>
              <div className="bg-white/10 rounded-lg p-2 flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-blue-400" />
                <div>
                  <p className="text-xs text-purple-200">Fragments</p>
                  <p className="font-bold text-white text-sm">{assets.fragments}</p>
                </div>
              </div>
            </div>

            <p className="text-center text-purple-200 text-xs">
              These resources will help you get started. You can use coins to enter matches and fragments to craft
              cards.
            </p>
          </CardContent>
          <div className="p-3 flex justify-center">
            <Button
              onClick={onClose}
              className="bg-gradient-to-r from-pink-600 to-purple-600 hover:from-pink-700 hover:to-purple-700 text-sm"
              size="sm"
            >
              Let's Play!
            </Button>
          </div>
        </Card>
      </div>
    )
}