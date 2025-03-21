"use client"

import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Coins, Users } from "lucide-react"
import { GameMatch } from "@/app/types"

// 组件props接口
interface CardGameMatchesProps {
  gameMatches: GameMatch[];
  handleJoinGame: (game: GameMatch) => void;
}

export default function CardGameMatches({
  gameMatches,
  handleJoinGame
}: CardGameMatchesProps) {
  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <Users className="h-4 w-4 text-green-400" />
          Play Game
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Join a match, compete with other players, and earn rewards
        </CardDescription>
      </CardHeader>
      <CardContent className="p-3">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
          {gameMatches.map((game) => (
            <div
              key={game.id}
              className={`bg-gradient-to-br ${game.bgClass} rounded-lg p-3 hover:shadow-lg hover:shadow-purple-500/20 transition-all cursor-pointer`}
            >
              <div className="flex justify-between items-start mb-2">
                <h3 className="text-sm font-bold text-white">{game.name}</h3>
                <Badge className={`${game.badgeClass} text-[10px] px-1 py-0`}>{game.level}</Badge>
              </div>
              <div className="space-y-1 mb-3">
                <div className="flex justify-between text-[10px]">
                  <span className="text-purple-200">Entry Fee:</span>
                  <span className="text-white flex items-center gap-0.5">
                    <Coins className="h-2 w-2 text-yellow-400" /> {game.entryFee}
                  </span>
                </div>
                <div className="flex justify-between text-[10px]">
                  <span className="text-purple-200">Players:</span>
                  <span className="text-white">{game.currentPlayers}/{game.maxPlayers}</span>
                </div>
                <div className="flex justify-between text-[10px]">
                  <span className="text-purple-200">Rewards:</span>
                  <span className="text-white flex items-center gap-0.5">
                    <Coins className="h-2 w-2 text-yellow-400" /> {game.rewards}
                  </span>
                </div>
              </div>
              <Button
                className={`w-full bg-gradient-to-r from-${game.badgeClass.split("-")[1]}-600 to-${game.level === "Beginner" ? "purple" : game.level === "Intermediate" ? "pink" : "red"}-600 h-6 text-[10px]`}
                onClick={() => handleJoinGame(game)}
              >
                Join {game.name}
              </Button>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}