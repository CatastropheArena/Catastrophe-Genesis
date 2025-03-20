"use client"

import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ArrowLeft, Users, Clock, Coins, AlertCircle } from "lucide-react"
import { useRouter, useSearchParams } from "next/navigation"
import { GameMatch } from "@/app/types"

export default function GamePage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  
  // 从 URL 参数中获取房间信息
  const [roomConfig, setRoomConfig] = useState<GameMatch>({
    id: Number(searchParams.get("id")) || 0,
    name: searchParams.get("name") || "",
    level: searchParams.get("level") || "",
    entryFee: Number(searchParams.get("entryFee")) || 0,
    currentPlayers: Number(searchParams.get("currentPlayers")) || 0,
    maxPlayers: Number(searchParams.get("maxPlayers")) || 6,
    rewards: Number(searchParams.get("rewards")) || 0,
    bgClass: searchParams.get("bgClass") || "",
    badgeClass: searchParams.get("badgeClass") || "",
  })

  const [gameState, setGameState] = useState("connecting") // connecting, waiting, playing, finished
  const [players, setPlayers] = useState<{ id: string; name: string; avatar: string }[]>([])
  const [timeLeft, setTimeLeft] = useState(30)
  const [myCards, setMyCards] = useState([
    { id: 1, name: "Skip Card", rarity: "Common", image: "/placeholder.svg?height=200&width=150" },
    { id: 2, name: "See the Future", rarity: "Uncommon", image: "/placeholder.svg?height=200&width=150" },
    { id: 3, name: "Shuffle", rarity: "Rare", image: "/placeholder.svg?height=200&width=150" },
  ])

  useEffect(() => {
    // 验证房间配置
    if (!roomConfig.id || !roomConfig.name) {
      router.push("/dashboard")
      return
    }

    let connectTimeout: NodeJS.Timeout;
    let joinInterval: NodeJS.Timeout;

    const startGame = () => {
      // 只在初始连接时执行一次
      connectTimeout = setTimeout(() => {
        setGameState("waiting");
        
        // 创建加入玩家的 interval
        joinInterval = setInterval(() => {
          setPlayers(prev => {
            // 使用房间配置中的最大玩家数
            if (prev.length >= roomConfig.maxPlayers - 1) { // -1 因为还包括当前玩家
              clearInterval(joinInterval);
              setGameState("playing");
              return prev;
            }
            return [
              ...prev,
              {
                id: Math.random().toString(36).substr(2, 9),
                name: `Player${Math.floor(Math.random() * 1000)}`,
                avatar: `/placeholder.svg?height=40&width=40`,
              },
            ];
          });
        }, 2000);
      }, 2000);
    };

    startGame();

    // 清理函数
    return () => {
      clearTimeout(connectTimeout);
      clearInterval(joinInterval);
    };
  }, [roomConfig.maxPlayers, roomConfig.id, roomConfig.name, router]);

  // 更新倒计时逻辑
  useEffect(() => {
    if (gameState === "waiting") {
      const timer = setInterval(() => {
        setTimeLeft((prev) => {
          if (prev <= 1) {
            clearInterval(timer)
            return 0
          }
          return prev - 1
        })
      }, 1000)

      return () => clearInterval(timer)
    }
  }, [gameState])

  // 更新游戏开始条件
  useEffect(() => {
    // 当达到最小玩家数且倒计时结束，或者房间满员时开始游戏
    const minPlayers = Math.ceil(roomConfig.maxPlayers / 2); // 设置最小玩家数为最大玩家数的一半
    if ((timeLeft === 0 && players.length >= minPlayers - 1) || 
        players.length >= roomConfig.maxPlayers - 1) {
      setGameState("playing")
    }
  }, [timeLeft, players.length, roomConfig.maxPlayers])

  return (
    <div className="min-h-screen bg-gradient-to-b from-purple-900 via-violet-800 to-indigo-900 text-white">
      <header className="bg-black/30 backdrop-blur-md border-b border-purple-500/20 sticky top-0 z-40">
        <div className="container mx-auto px-2 py-2">
          <div className="flex justify-between items-center">
            <Button
              variant="ghost"
              size="sm"
              className="text-white flex items-center gap-1"
              onClick={() => router.push("/dashboard")}
            >
              <ArrowLeft className="h-4 w-4" />
              <span className="text-xs">Back to Dashboard</span>
            </Button>

            <div className="flex items-center gap-2">
              <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                <Users className="h-3 w-3 text-purple-300" />
                <span className="text-white font-medium text-xs">{players.length + 1}/6</span>
              </div>
              {gameState === "waiting" && (
                <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                  <Clock className="h-3 w-3 text-purple-300" />
                  <span className="text-white font-medium text-xs">{timeLeft}s</span>
                </div>
              )}
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-2 py-3">
        {gameState === "connecting" && (
          <div className="flex flex-col items-center justify-center h-[80vh]">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white mb-4"></div>
            <h2 className="text-xl font-bold text-white mb-2">Connecting to Game Server</h2>
            <p className="text-purple-200 text-sm">Please wait while we establish a connection...</p>
          </div>
        )}

        {gameState === "waiting" && (
          <div className="flex flex-col items-center justify-center h-[80vh]">
            <h2 className="text-xl font-bold text-white mb-6">Waiting for Players</h2>

            <div className="bg-black/30 rounded-lg p-4 w-full max-w-md mb-6">
              <h3 className="text-sm font-medium text-white mb-3">Players in Lobby</h3>

              <div className="space-y-2">
                {/* Current player */}
                <div className="flex items-center gap-2 bg-purple-900/30 rounded-lg p-2">
                  <div className="w-8 h-8 rounded-full bg-purple-600 flex items-center justify-center">
                    <span className="text-white font-bold">You</span>
                  </div>
                  <span className="text-white">You (Ready)</span>
                  <Badge className="ml-auto bg-green-600 text-[10px]">Ready</Badge>
                </div>

                {/* Other players */}
                {players.map((player) => (
                  <div key={player.id} className="flex items-center gap-2 bg-black/20 rounded-lg p-2">
                    <div className="w-8 h-8 rounded-full overflow-hidden">
                      <img
                        src={player.avatar || "/placeholder.svg"}
                        alt={player.name}
                        className="w-full h-full object-cover"
                      />
                    </div>
                    <span className="text-white">{player.name}</span>
                    <Badge className="ml-auto bg-green-600 text-[10px]">Ready</Badge>
                  </div>
                ))}

                {/* Empty slots */}
                {Array.from({ length: 5 - players.length }).map((_, i) => (
                  <div
                    key={i}
                    className="flex items-center gap-2 bg-black/10 rounded-lg p-2 border border-dashed border-purple-500/20"
                  >
                    <div className="w-8 h-8 rounded-full bg-black/20 flex items-center justify-center">
                      <Users className="h-4 w-4 text-purple-400/50" />
                    </div>
                    <span className="text-purple-400/50">Waiting for player...</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="text-center">
              <p className="text-purple-200 text-sm mb-2">
                Game will start in <span className="text-white font-bold">{timeLeft}</span> seconds
              </p>
              <p className="text-purple-300 text-xs">or when all 6 players have joined</p>
            </div>
          </div>
        )}

        {gameState === "playing" && (
          <div className="flex flex-col items-center justify-center h-[80vh]">
            <div className="bg-black/30 rounded-lg p-4 w-full max-w-xl mb-6 text-center">
              <AlertCircle className="h-12 w-12 text-yellow-400 mx-auto mb-4" />
              <h2 className="text-xl font-bold text-white mb-2">Game Implementation</h2>
              <p className="text-purple-200 text-sm mb-4">
                This is a placeholder for the actual game implementation. In a real application, this would be the
                interactive game interface.
              </p>
              <div className="grid grid-cols-3 gap-2 mb-4">
                {myCards.map((card) => (
                  <div key={card.id} className="bg-black/20 rounded-lg overflow-hidden">
                    <img src={card.image || "/placeholder.svg"} alt={card.name} className="w-full h-24 object-cover" />
                    <div className="p-2">
                      <h3 className="text-white text-xs font-medium">{card.name}</h3>
                      <Badge
                        className={`
                          text-[10px] px-1 py-0 mt-1
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
                    </div>
                  </div>
                ))}
              </div>
              <Button
                className="bg-gradient-to-r from-red-600 to-orange-600"
                onClick={() => {
                  setGameState("finished")
                }}
              >
                Simulate Game End
              </Button>
            </div>
          </div>
        )}

        {gameState === "finished" && (
          <div className="flex flex-col items-center justify-center h-[80vh]">
            <Card className="bg-gradient-to-br from-purple-900 to-indigo-900 border-purple-500/30 w-full max-w-md">
              <CardContent className="p-6">
                <div className="text-center mb-6">
                  <h2 className="text-2xl font-bold text-white mb-2">Game Finished!</h2>
                  <p className="text-purple-200">You placed 2nd out of 6 players</p>
                </div>

                <div className="bg-black/30 rounded-lg p-4 mb-6">
                  <h3 className="text-sm font-medium text-white mb-3">Rewards Earned</h3>

                  <div className="flex justify-between items-center mb-2">
                    <span className="text-purple-200">Base Reward:</span>
                    <div className="flex items-center gap-1 text-yellow-400">
                      <Coins className="h-3 w-3" />
                      <span>150</span>
                    </div>
                  </div>

                  <div className="flex justify-between items-center mb-2">
                    <span className="text-purple-200">Placement Bonus:</span>
                    <div className="flex items-center gap-1 text-yellow-400">
                      <Coins className="h-3 w-3" />
                      <span>50</span>
                    </div>
                  </div>

                  <div className="flex justify-between items-center mb-2">
                    <span className="text-purple-200">Card Usage Bonus:</span>
                    <div className="flex items-center gap-1 text-yellow-400">
                      <Coins className="h-3 w-3" />
                      <span>25</span>
                    </div>
                  </div>

                  <div className="border-t border-purple-500/20 mt-3 pt-3 flex justify-between items-center">
                    <span className="text-white font-medium">Total:</span>
                    <div className="flex items-center gap-1 text-yellow-400 font-bold">
                      <Coins className="h-4 w-4" />
                      <span>225</span>
                    </div>
                  </div>
                </div>

                <div className="flex flex-col gap-2">
                  <Button
                    className="bg-gradient-to-r from-green-600 to-teal-600"
                    onClick={() => router.push("/dashboard")}
                  >
                    Claim Rewards & Return
                  </Button>
                  <Button
                    variant="outline"
                    className="border-purple-500 text-purple-200 bg-purple-950/50 hover:bg-purple-800/50 hover:text-white"
                    onClick={() => {
                      setGameState("connecting")
                      setPlayers([])
                      setTimeLeft(30)

                      // Simulate connecting to game server again
                      setTimeout(() => {
                        setGameState("waiting")
                      }, 2000)
                    }}
                  >
                    Play Again
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </main>
    </div>
  )
}

