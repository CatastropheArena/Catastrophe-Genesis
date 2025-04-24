"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { useCurrentAccount } from "@mysten/dapp-kit";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  ArrowLeft,
  Users,
  Clock,
  Coins,
  AlertCircle,
  Sparkles,
} from "lucide-react";
import {
  GameMatch,
  CardItem,
  RentalCard,
  StakingPool,
  StakedCard,
} from "@/app/types";

// Import components
import CardCollection from "./components/card-collection";
import CardRentalMarketplace from "./components/card-rental-marketplace";
import CardStakingPools from "./components/card-staking-pools";
import Exchange from "./components/exchange";
import GameMatches from "./components/game-matches";
import DialogModal from "./components/dialog-modal";

// Mock data
const mockCards: CardItem[] = [
  {
    id: 1,
    name: "Skip Card",
    rarity: "Common",
    image: "/placeholder.svg?height=200&width=150",
    count: 2,
    status: "owned",
  },
  {
    id: 2,
    name: "See the Future",
    rarity: "Uncommon",
    image: "/placeholder.svg?height=200&width=150",
    count: 1,
    status: "owned",
  },
  {
    id: 3,
    name: "Shuffle",
    rarity: "Rare",
    image: "/placeholder.svg?height=200&width=150",
    count: 1,
    status: "owned",
  },
];

const mockRentalCards: RentalCard[] = [
  {
    id: 1,
    name: "Skip Card",
    rarity: "Common",
    image: "/placeholder.svg?height=200&width=150",
    rate: 10,
    poolSize: 100,
  },
  {
    id: 2,
    name: "See the Future",
    rarity: "Uncommon",
    image: "/placeholder.svg?height=200&width=150",
    rate: 20,
    poolSize: 50,
  },
];

const mockStakingPools: StakingPool[] = [
  {
    id: 1,
    name: "Skip Card Pool",
    rarity: "Common",
    image: "/placeholder.svg?height=200&width=150",
    apr: "20%",
    rate: 20,
    poolSize: 1000,
  },
  {
    id: 2,
    name: "See the Future Pool",
    rarity: "Uncommon",
    image: "/placeholder.svg?height=200&width=150",
    apr: "30%",
    rate: 30,
    poolSize: 500,
  },
];

const mockGameMatches: GameMatch[] = [
  {
    id: 1,
    name: "Beginner Room",
    level: "Beginner",
    entryFee: 10,
    currentPlayers: 2,
    maxPlayers: 6,
    rewards: 50,
    bgClass: "from-green-500/20 to-green-700/20",
    badgeClass: "bg-green-500/20 text-green-300",
  },
  {
    id: 2,
    name: "Pro Room",
    level: "Advanced",
    entryFee: 50,
    currentPlayers: 4,
    maxPlayers: 6,
    rewards: 250,
    bgClass: "from-purple-500/20 to-purple-700/20",
    badgeClass: "bg-purple-500/20 text-purple-300",
  },
];

interface RentedCardData extends RentalCard {
  usesLeft: number;
  totalUses: number;
  expiresIn: number;
}

interface DialogStateType {
  open: boolean;
  title: string;
  description: string;
  type: "confirm" | "stakeInput" | "success" | "error";
  confirmText: string;
  cancelText?: string;
  data: any;
  isLoading: boolean;
}

export default function Dashboard() {
  const router = useRouter();
  const account = useCurrentAccount();

  // State management
  const [assets, setAssets] = useState({
    coins: 1000,
    fragments: 500,
    usdt: 100,
    cards: mockCards,
  });

  const [activeTab, setActiveTab] = useState("collection");
  const [gameState, setGameState] = useState("connecting");
  const [players, setPlayers] = useState([]);
  const [timeLeft, setTimeLeft] = useState(30);
  const [myRentedCards, setMyRentedCards] = useState<RentedCardData[]>([]);
  const [rentalHistory, setRentalHistory] = useState([]);
  const [myStakedCards, setMyStakedCards] = useState<StakedCard[]>([]);

  // Dialog state
  const [dialogState, setDialogState] = useState<DialogStateType>({
    open: false,
    title: "",
    description: "",
    type: "confirm",
    confirmText: "Confirm",
    cancelText: "Cancel",
    data: null,
    isLoading: false,
  });

  // Collection functions
  const handleGetMoreClick = () => {
    if (assets.coins < 100) {
      setDialogState({
        open: true,
        title: "Insufficient Funds",
        description: "You need 100 coins to purchase a card pack.",
        type: "error",
        confirmText: "OK",
        data: null,
        isLoading: false,
      });
      return;
    }

    setDialogState({
      open: true,
      title: "Get More Cards",
      description: "Would you like to purchase a card pack for 100 coins?",
      type: "confirm",
      confirmText: "Purchase",
      cancelText: "Cancel",
      data: null,
      isLoading: false,
    });
  };

  // Rental functions
  const handleRentCard = (card: RentalCard, uses: number, period: number) => {
    const cost = card.rate * period;
    if (assets.coins < cost) {
      setDialogState({
        open: true,
        title: "Insufficient Funds",
        description: "You don't have enough coins to rent this card.",
        type: "error",
        confirmText: "OK",
        data: null,
        isLoading: false,
      });
      return;
    }

    setDialogState({
      open: true,
      title: "Rent Card",
      description: `Rent ${card.name} for ${period} days with ${uses} uses for ${cost} coins?`,
      type: "confirm",
      confirmText: "Rent",
      cancelText: "Cancel",
      data: { card, uses, period, cost },
      isLoading: false,
    });
  };

  // Staking functions
  const handleStakeCard = (pool: StakingPool) => {
    const userCard = assets.cards.find(
      (c) => c.name === pool.name.replace(" Pool", "")
    );
    if (!userCard || userCard.count === 0) {
      setDialogState({
        open: true,
        title: "Cannot Stake",
        description:
          "You don't have any cards available to stake in this pool.",
        type: "error",
        confirmText: "OK",
        data: null,
        isLoading: false,
      });
      return;
    }

    setDialogState({
      open: true,
      title: "Stake Cards",
      description: `How many ${userCard.name} cards would you like to stake?`,
      type: "stakeInput",
      confirmText: "Stake",
      cancelText: "Cancel",
      data: { pool, maxAmount: userCard.count },
      isLoading: false,
    });
  };

  const handleUnstakeCard = (card: any) => {
    setDialogState({
      open: true,
      title: "Unstake Cards",
      description: `Are you sure you want to unstake your ${card.name}?`,
      type: "confirm",
      confirmText: "Unstake",
      cancelText: "Cancel",
      data: card,
      isLoading: false,
    });
  };

  const handleClaimRewards = async () => {
    try {
      setDialogState({
        open: true,
        title: "Claiming Rewards",
        description: "Processing your reward claim...",
        type: "confirm",
        confirmText: "OK",
        data: null,
        isLoading: true,
      });

      // 模拟API调用延迟
      await new Promise((resolve) => setTimeout(resolve, 2000));

      setAssets((prev) => ({
        ...prev,
        coins: prev.coins + 100,
      }));

      setDialogState({
        open: true,
        title: "Success",
        description: "Successfully claimed 100 coins in rewards!",
        type: "success",
        confirmText: "OK",
        data: null,
        isLoading: false,
      });
    } catch (error) {
      setDialogState({
        open: true,
        title: "Error",
        description: "Failed to claim rewards. Please try again.",
        type: "error",
        confirmText: "OK",
        data: null,
        isLoading: false,
      });
    }
  };

  // Game functions
  const handleJoinGame = (game: GameMatch) => {
    if (assets.coins < game.entryFee) {
      setDialogState({
        open: true,
        title: "Insufficient Funds",
        description: `You need ${game.entryFee} coins to join this game.`,
        type: "error",
        confirmText: "OK",
        data: null,
        isLoading: false,
      });
      return;
    }

    setDialogState({
      open: true,
      title: "Join Game",
      description: `Join ${game.name} for ${game.entryFee} coins?`,
      type: "confirm",
      confirmText: "Join",
      cancelText: "Cancel",
      data: game,
      isLoading: false,
    });
  };

  // Redirect to home if not connected
  useEffect(() => {
    if (!account) {
      router.push("/");
    }
  }, [account, router]);

  return (
    <div className="min-h-screen bg-gradient-to-b from-purple-900 via-violet-800 to-indigo-900 text-white">
      <div className="container mx-auto px-4 py-8">
        {/* Asset Overview */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          <Card className="bg-purple-900/50 border-purple-500/30">
            <CardContent className="p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Coins className="h-5 w-5 text-yellow-400" />
                  <span className="text-sm">Coins</span>
                </div>
                <span className="font-bold">{assets.coins}</span>
              </div>
            </CardContent>
          </Card>
          <Card className="bg-purple-900/50 border-purple-500/30">
            <CardContent className="p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Sparkles className="h-5 w-5 text-blue-400" />
                  <span className="text-sm">Fragments</span>
                </div>
                <span className="font-bold">{assets.fragments}</span>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Content Area */}
        <Tabs
          defaultValue={activeTab}
          onValueChange={setActiveTab}
          className="space-y-4"
        >
          <TabsList className="grid grid-cols-5 gap-4 bg-purple-900/50 p-2">
            <TabsTrigger
              value="collection"
              className="data-[state=active]:bg-purple-700"
            >
              Collection
            </TabsTrigger>
            <TabsTrigger
              value="rental"
              className="data-[state=active]:bg-purple-700"
            >
              Rental Market
            </TabsTrigger>
            <TabsTrigger
              value="staking"
              className="data-[state=active]:bg-purple-700"
            >
              Staking
            </TabsTrigger>
            <TabsTrigger
              value="exchange"
              className="data-[state=active]:bg-purple-700"
            >
              Exchange
            </TabsTrigger>
            <TabsTrigger
              value="game"
              className="data-[state=active]:bg-purple-700"
            >
              Game Lobby
            </TabsTrigger>
          </TabsList>

          <TabsContent value="collection" className="mt-4">
            <CardCollection
              cards={assets.cards}
              onGetMoreClick={handleGetMoreClick}
            />
          </TabsContent>

          <TabsContent value="rental" className="mt-4">
            <CardRentalMarketplace
              rentalCards={mockRentalCards}
              myRentedCards={myRentedCards}
              rentalHistory={rentalHistory}
              handleRentCard={handleRentCard}
            />
          </TabsContent>

          <TabsContent value="staking" className="mt-4">
            <CardStakingPools
              myStakedCards={myStakedCards}
              stakingPools={mockStakingPools}
              handleUnstakeCard={handleUnstakeCard}
              handleStakeCard={handleStakeCard}
              handleClaimRewards={handleClaimRewards}
            />
          </TabsContent>

          <TabsContent value="exchange" className="mt-4">
            <Exchange />
          </TabsContent>

          <TabsContent value="game" className="mt-4">
            <GameMatches
              gameMatches={mockGameMatches}
              handleJoinGame={handleJoinGame}
            />
          </TabsContent>
        </Tabs>

        {/* Dialog Modal */}
        <DialogModal
          open={dialogState.open}
          title={dialogState.title}
          description={dialogState.description}
          type={dialogState.type}
          confirmText={dialogState.confirmText}
          cancelText={dialogState.cancelText}
          data={dialogState.data}
          isLoading={dialogState.isLoading}
          onOpenChange={(open) => setDialogState((prev) => ({ ...prev, open }))}
          confirmAction={async () => {
            const { type, data } = dialogState;

            switch (type) {
              case "confirm":
                if (data?.card) {
                  // Rental confirmation
                  setAssets((prev) => ({
                    ...prev,
                    coins: prev.coins - data.cost,
                  }));
                  const rentedCard: RentedCardData = {
                    ...data.card,
                    usesLeft: data.uses,
                    totalUses: data.uses,
                    expiresIn: data.period,
                  };
                  setMyRentedCards((prev) => [...prev, rentedCard]);
                } else if (data?.id && data.level) {
                  // Game join confirmation
                  setAssets((prev) => ({
                    ...prev,
                    coins: prev.coins - data.entryFee,
                  }));
                  router.push(`/game/${data.id}`);
                }
                break;

              case "stakeInput":
                if (data?.pool) {
                  const amountInput = document.getElementById(
                    "stakeAmount"
                  ) as HTMLInputElement;
                  const amount = Number(amountInput?.value || "0");
                  if (amount > 0) {
                    const stakedCard: StakedCard = {
                      ...data.pool,
                      stakedCount: amount,
                      poolShare: "0%",
                      earned: 0,
                      stakedAmount: amount,
                    };
                    setMyStakedCards((prev) => [...prev, stakedCard]);
                    setAssets((prev) => ({
                      ...prev,
                      cards: prev.cards.map((c) =>
                        c.name === data.pool.name.replace(" Pool", "")
                          ? { ...c, count: c.count - amount }
                          : c
                      ),
                    }));
                  }
                }
                break;
            }

            setDialogState((prev) => ({ ...prev, open: false }));
          }}
        />
      </div>
    </div>
  );
}
