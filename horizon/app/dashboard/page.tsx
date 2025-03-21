"use client"

import React from 'react';
import { useState, useEffect, useRef } from "react"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Coins,
  Sparkles,
  Users,
  Layers,
  Clock,
  ArrowRightLeft,
} from "lucide-react"
import { useMobile } from "@/hooks/use-mobile"
import { useRouter } from "next/navigation"
import Welcome from './components/welcome';
import DrawnCardModal from './components/drawn-card';
import DialogModal from './components/dialog-modal';
import Header from './components/header';
import CardCollection from './components/card-collection';
import CardSynthesisGacha from './components/card-synthesis-gacha';
import CardStakingPools from './components/card-staking-pools';
import CardRentalMarketplace from './components/card-rental-marketplace';
import CardGameMatches from './components/game-matches';
import Exchange from './components/exchange';
import { 
  Assets, CardItem, RentalCard, MyRentalCard, RentalHistoryItem, 
  StakedCard, StakingPool, GameMatch, DrawHistoryItem,
  DrawnCard, DialogState
} from "@/app/types"

// Update the initialAssets.cards array to include a rented card indicator
const initialAssets: Assets = {
  coins: 1000,
  fragments: 500,
  usdt: 100,
  cards: [
    {
      id: 1,
      name: "Skip Card",
      rarity: "Common",
      image: "/placeholder.svg?height=200&width=150",
      count: 3,
      status: "owned",
    },
    {
      id: 2,
      name: "See the Future",
      rarity: "Uncommon",
      image: "/placeholder.svg?height=200&width=150",
      count: 2,
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
  ],
}

// Add getAllCards function before the component
const getAllCards = (
  ownedCards: CardItem[], 
  rentedCards: MyRentalCard[], 
  stakedCards: StakedCard[]
): CardItem[] => {
  const allCards: CardItem[] = [...ownedCards]

  rentedCards.forEach((card: MyRentalCard) => {
    allCards.push({
      id: card.id,
      name: card.name,
      rarity: card.rarity,
      image: card.image,
      count: 1,
      status: "rented",
      usesLeft: card.usesLeft,
      expiresIn: card.expiresIn,
    })
  })

  stakedCards.forEach((card: StakedCard) => {
    allCards.push({
      id: card.id,
      name: card.name,
      rarity: card.rarity,
      image: card.image,
      count: card.stakedCount,
      status: "staked",
      poolShare: card.poolShare,
    })
  })

  return allCards
}

// Mock data for rental cards
const rentalCards = [
  {
    id: 4,
    name: "Defuse",
    rarity: "Common",
    image: "/placeholder.svg?height=200&width=150",
    poolSize: 500,
    rate: 10,
  },
  {
    id: 5,
    name: "Attack",
    rarity: "Uncommon",
    image: "/placeholder.svg?height=200&width=150",
    poolSize: 300,
    rate: 25,
  },
  {
    id: 6,
    name: "Favor",
    rarity: "Rare",
    image: "/placeholder.svg?height=200&width=150",
    poolSize: 100,
    rate: 50,
  },
]

// Mock data for staking pools
const stakingPools = [
  {
    id: 1,
    name: "Skip Card",
    rarity: "Common",
    image: "/placeholder.svg?height=80&width=60",
    poolSize: 1000,
    apr: "15%",
    rate: 5,
  },
  {
    id: 2,
    name: "See the Future",
    rarity: "Uncommon",
    image: "/placeholder.svg?height=80&width=60",
    poolSize: 500,
    apr: "25%",
    rate: 15,
  },
  {
    id: 3,
    name: "Shuffle",
    rarity: "Rare",
    image: "/placeholder.svg?height=80&width=60",
    poolSize: 250,
    apr: "40%",
    rate: 30,
  },
]

// Mock data for game matches
const gameMatches = [
  {
    id: 1,
    name: "Beginner Match",
    level: "Beginner",
    entryFee: 10,
    currentPlayers: 5,
    maxPlayers: 10,
    rewards: 50,
    bgClass: "from-green-700 to-green-900",
    badgeClass: "bg-green-600",
  },
  {
    id: 2,
    name: "Intermediate Match",
    level: "Intermediate",
    entryFee: 50,
    currentPlayers: 3,
    maxPlayers: 6,
    rewards: 250,
    bgClass: "from-purple-700 to-purple-900",
    badgeClass: "bg-purple-600",
  },
  {
    id: 3,
    name: "Advanced Match",
    level: "Advanced",
    entryFee: 100,
    currentPlayers: 1,
    maxPlayers: 4,
    rewards: 500,
    bgClass: "from-red-700 to-red-900",
    badgeClass: "bg-red-600",
  },
]

// Card pack data for gacha system
const cardPacks = [
  {
    id: 1,
    name: "Basic Pack",
    price: 100,
    image: "/placeholder.svg?height=200&width=150",
    description: "Contains 1 card with higher chance for Common cards",
    dropRates: {
      Common: 70,
      Uncommon: 25,
      Rare: 5,
      Legendary: 0,
    },
  },
  {
    id: 2,
    name: "Premium Pack",
    price: 300,
    image: "/placeholder.svg?height=200&width=150",
    description: "Contains 1 card with higher chance for Uncommon cards",
    dropRates: {
      Common: 40,
      Uncommon: 45,
      Rare: 14,
      Legendary: 1,
    },
  },
  {
    id: 3,
    name: "Ultimate Pack",
    price: 500,
    image: "/placeholder.svg?height=200&width=150",
    description: "Contains 1 card with higher chance for Rare and Legendary cards",
    dropRates: {
      Common: 20,
      Uncommon: 40,
      Rare: 35,
      Legendary: 5,
    },
  },
]

// Extended card pool for gacha system
const extendedCardPool = [
  ...rentalCards,
  {
    id: 7,
    name: "Nope",
    rarity: "Common",
    image: "/placeholder.svg?height=200&width=150",
  },
  {
    id: 8,
    name: "Targeted Attack",
    rarity: "Uncommon",
    image: "/placeholder.svg?height=200&width=150",
  },
  {
    id: 9,
    name: "Alter Future",
    rarity: "Rare",
    image: "/placeholder.svg?height=200&width=150",
  },
  {
    id: 10,
    name: "Exploding Cat",
    rarity: "Legendary",
    image: "/placeholder.svg?height=200&width=150",
  },
]

export default function DashboardPage() {
  const router = useRouter()
  const [assets, setAssets] = useState(initialAssets)
  const [showWelcome, setShowWelcome] = useState(true)
  const [myRentedCards, setMyRentedCards] = useState<MyRentalCard[]>([])
  const [myStakedCards, setMyStakedCards] = useState([
    {
      id: 1,
      name: "Skip Card",
      rarity: "Common",
      image: "/placeholder.svg?height=80&width=60",
      stakedCount: 3,
      poolShare: "0.3%",
      earned: 45,
    },
  ])
  const rentalHistory = [
    {
      id: 100,
      cardId: 1,
      cardName: "See Future",
      cardImage: "/cards/see-future.png",
      source: "Pool",
      period: 3,
      uses: 10,
      cost: 75,
      status: "expired",
      expiryDate: "2025-03-10"
    },
    {
      id: 101,
      cardId: 2,
      cardName: "Skip Card",
      cardImage: "/cards/skip-card.png",
      source: "Pool",
      period: 2,
      uses: 5,
      cost: 30,
      status: "expired",
      expiryDate: "2025-03-15"
    }
  ] as RentalHistoryItem[]; 
  const [selectedUses, setSelectedUses] = useState<Record<string | number, number>>({})
  const [selectedPeriod, setSelectedPeriod] = useState<Record<string | number, number>>({})
  const [dialogState, setDialogState] = useState<DialogState>({
    open: false,
    title: "",
    description: "",
    type: "",
    confirmAction: () => {},
    confirmText: "Confirm",
    cancelText: "Cancel",
    data: null,
  })
  const [isLoading, setIsLoading] = useState(false)
  const [activeTab, setActiveTab] = useState("collection")
  const [rentalFilter, setRentalFilter] = useState("All")
  const isMobile = useMobile()

  // Add state for cardFilter
  const [cardFilter, setCardFilter] = useState("All")

  // Add these state variables at the beginning of the component
  const [synthesisCards, setSynthesisCards] = useState(Array(3).fill(null))
  const [upgradeCard, setUpgradeCard] = useState<CardItem | null>(null)
  const [showCardSelector, setShowCardSelector] = useState(false)
  const [selectorType, setSelectorType] = useState("") // "synthesis" or "upgrade"
  const [selectorIndex, setSelectorIndex] = useState<number | null>(null) // Used for synthesis card index

  // Gacha system states
  const [selectedPack, setSelectedPack] = useState(cardPacks[0])
  const [isDrawing, setIsDrawing] = useState(false)
  const [drawnCard, setDrawnCard] = useState<DrawnCard | null>(null)
  const [showDrawnCard, setShowDrawnCard] = useState(false)
  const [drawHistory, setDrawHistory] = useState<DrawHistoryItem[]>([])
  const [activePackIndex, setActivePackIndex] = useState(0)
  const [drawProgress, setDrawProgress] = useState(0)
  const cardRef = useRef<HTMLElement | null>(null)

  // Add filteredCards computation
  const allCards = getAllCards(assets.cards, myRentedCards, myStakedCards)
  const filteredCards =
    cardFilter === "All" ? allCards : allCards.filter((card) => card.status.toLowerCase() === cardFilter.toLowerCase())

  useEffect(() => {
    // Initialize selected uses and period for each rental card
    const usesObj: Record<string | number, number> = {}
    const periodObj: Record<string | number, number> = {}
    rentalCards.forEach((card) => {
      usesObj[card.id] = 5
      periodObj[card.id] = 1
    })
    setSelectedUses(usesObj)
    setSelectedPeriod(periodObj)

    // Simulate checking if user is new
    const timer = setTimeout(() => {
      setShowWelcome(false)
    }, 5000)

    return () => clearTimeout(timer)
  }, [])

  // 修复 stakingPools.find 可能返回 undefined 的问题
  const getPoolSize = (name: string): number => {
    return stakingPools.find((p) => p.name === name)?.poolSize || 1000;
  }

  // 使用这个辅助函数来设置对话框状态
  const setDefaultDialog = (dialogProps: Partial<DialogState>): DialogState => ({
    open: true,
    title: "",
    description: "",
    type: "",
    confirmAction: () => {},
    confirmText: "Confirm",
    cancelText: "Cancel",
    data: null,
    ...dialogProps
  });

  // 修改 confirmRentCard 函数
  const confirmRentCard = (card: RentalCard, cost: number) => {
    setIsLoading(true)

    // Simulate API call
    setTimeout(() => {
      // Add to rented cards
      const newRentedCard: MyRentalCard = {
        id: Date.now(),
        name: card.name,
        rarity: card.rarity,
        image: card.image,
        usesLeft: selectedUses[card.id],
        totalUses: selectedUses[card.id],
        expiresIn: selectedPeriod[card.id],
        rate: card.rate,
        poolSize: card.poolSize,
      }

      setMyRentedCards((prev) => [...prev, newRentedCard])

      // Deduct coins
      setAssets((prev) => ({
        ...prev,
        coins: prev.coins - cost,
      }))

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Rental Successful",
        description: `You have successfully rented ${card.name} for ${selectedPeriod[card.id]} day(s).`,
        type: "success",
        confirmText: "OK",
        confirmAction: () => {
          setDialogState((prev) => ({ ...prev, open: false }))
          setActiveTab("rental")
          // Switch to "My Rentals" tab
          const rentedTabElement = document.querySelector('[value="rented"]') as HTMLElement | null
          rentedTabElement?.click()
        },
      })
    }, 1000)
  }

  // Handle staking a card
  const handleStakeCard = (card: StakingPool) => {
    // Check if user has this card
    const userCard = assets.cards.find((c) => c.name === card.name)

    if (!userCard || userCard.count === 0) {
      setDialogState({
        open: true,
        title: "No Cards Available",
        description: `You don't have any ${card.name} cards to stake.`,
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setDialogState({
      open: true,
      title: "Stake Card",
      description: `How many ${card.name} cards would you like to stake? (Max: ${userCard.count})`,
      type: "stakeInput",
      confirmAction: (amount: number | undefined) => {
        if (amount !== undefined) {
          confirmStakeCard(card, amount, userCard);
        }
      },
      data: { card, maxAmount: userCard.count },
      confirmText: "Confirm",
      cancelText: "Cancel"
    })
  }

  // Confirm staking a card
  const confirmStakeCard = (card: StakingPool, amount: number, userCard: CardItem) => {
    if (amount <= 0 || amount > userCard.count) {
      setDialogState({
        open: true,
        title: "Invalid Amount",
        description: `Please enter a valid amount between 1 and ${userCard.count}.`,
        type: "error",
        confirmText: "OK",
        confirmAction: () => handleStakeCard(card),
      })
      return
    }

    setIsLoading(true)

    // Simulate API call
    setTimeout(() => {
      // Check if already staked this card type
      const existingStakedCard = myStakedCards.find((c) => c.name === card.name)

      if (existingStakedCard) {
        // Update existing staked card
        setMyStakedCards((prev) =>
          prev.map((c) =>
            c.name === card.name
              ? {
                  ...c,
                  stakedCount: c.stakedCount + amount,
                  poolShare: (((c.stakedCount + amount) / card.poolSize) * 100).toFixed(1) + "%",
                }
              : c,
          ),
        )
      } else {
        // Add new staked card
        const newStakedCard = {
          id: Date.now(),
          name: card.name,
          rarity: card.rarity,
          image: `/placeholder.svg?height=80&width=60`,
          stakedCount: amount,
          poolShare: ((amount / card.poolSize) * 100).toFixed(1) + "%",
          earned: 0,
        }
        setMyStakedCards((prev) => [...prev, newStakedCard])
      }

      // Remove cards from collection
      setAssets((prev) => ({
        ...prev,
        cards: prev.cards.map((c) => (c.name === card.name ? { ...c, count: c.count - amount } : c)),
      }))

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Staking Successful",
        description: `You have successfully staked ${amount} ${card.name} card(s).`,
        type: "success",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
    }, 1000)
  }

  // Handle claiming rewards
  const handleClaimRewards = () => {
    setDialogState({
      open: true,
      title: "Claim Rewards",
      description: "Are you sure you want to claim 45 COIN rewards?",
      type: "confirm",
      confirmAction: confirmClaimRewards,
    })
  }

  // Confirm claiming rewards
  const confirmClaimRewards = () => {
    setIsLoading(true)

    // Simulate API call
    setTimeout(() => {
      // Add coins
      setAssets((prev) => ({
        ...prev,
        coins: prev.coins + 45,
      }))

      // Reset earned rewards
      setMyStakedCards((prev) =>
        prev.map((card) => ({
          ...card,
          earned: 0,
        })),
      )

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Rewards Claimed",
        description: "You have successfully claimed 45 COIN rewards.",
        type: "success",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
    }, 1000)
  }

  // Handle joining a game
  const handleJoinGame = (game: GameMatch) => {
    if (assets.coins < game.entryFee) {
      setDialogState({
        open: true,
        title: "Insufficient Funds",
        description: `You need ${game.entryFee} coins to join this ${game.name}.`,
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setDialogState({
      open: true,
      title: "Join Game",
      description: `Join ${game.name} for ${game.entryFee} coins?`,
      type: "confirm",
      confirmAction: () => confirmJoinGame(game),
      data: game,
    })
  }

  // Confirm joining a game
  const confirmJoinGame = (game: GameMatch) => {
    setIsLoading(true)

    // Simulate API call
    setTimeout(() => {
      // Deduct entry fee
      setAssets((prev) => ({
        ...prev,
        coins: prev.coins - game.entryFee,
      }))

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Joining Game",
        description: "Preparing game session...",
        type: "success",
        confirmText: "Start Game",
        confirmAction: () => {
          setDialogState((prev) => ({ ...prev, open: false }))
          // 构建带有房间信息的 URL
          const params = new URLSearchParams({
            id: game.id.toString(),
            name: game.name,
            level: game.level,
            entryFee: game.entryFee.toString(),
            currentPlayers: game.currentPlayers.toString(),
            maxPlayers: game.maxPlayers.toString(),
            rewards: game.rewards.toString(),
            bgClass: game.bgClass,
            badgeClass: game.badgeClass,
          })
          // Navigate to game page with room config
          router.push(`/game?${params.toString()}`)
        },
      })
    }, 1000)
  }


  // Filter rental cards based on selected filter
  const filteredRentalCards =
    rentalFilter === "All" ? rentalCards : rentalCards.filter((card) => card.rarity === rentalFilter)

  // Add handleUnstakeCard function
  const handleUnstakeCard = (card: StakedCard) => {
    setDialogState({
      open: true,
      title: "Unstake Card",
      description: `How many ${card.name} cards would you like to unstake? (Max: ${card.stakedCount})`,
      type: "stakeInput",
      confirmAction: (amount) => confirmUnstakeCard(card, amount as number),
      data: { card, maxAmount: card.stakedCount },
    })
  }

  // Add confirmUnstakeCard function
  const confirmUnstakeCard = (card: StakedCard, amount: number) => {
    if (amount <= 0 || amount > card.stakedCount) {
      setDialogState({
        open: true,
        title: "Invalid Amount",
        description: `Please enter a valid amount between 1 and ${card.stakedCount}.`,
        type: "error",
        confirmText: "OK",
        confirmAction: () => handleUnstakeCard(card),
      })
      return
    }

    setIsLoading(true)

    // Simulate API call
    setTimeout(() => {
      // Update staked cards
      if (amount === card.stakedCount) {
        // Remove card completely if unstaking all
        setMyStakedCards((prev) => prev.filter((c) => c.id !== card.id))
      } else {
        // Reduce count if unstaking some
        setMyStakedCards((prev) =>
          prev.map((c) =>
            c.id === card.id
              ? {
                  ...c,
                  stakedCount: c.stakedCount - amount,
                  poolShare: (((c.stakedCount - amount) / getPoolSize(c.name)) * 100).toFixed(1) + "%",
                }
              : c,
          ),
        )
      }

      // Add cards back to collection
      const cardInCollection = assets.cards.find((c) => c.name === card.name)

      if (cardInCollection) {
        // Update existing card count
        setAssets((prev) => ({
          ...prev,
          cards: prev.cards.map((c) => (c.name === card.name ? { ...c, count: c.count + amount } : c)),
        }))
      } else {
        // Add card to collection if it doesn't exist
        setAssets((prev) => ({
          ...prev,
          cards: [
            ...prev.cards,
            {
              id: Date.now(),
              name: card.name,
              rarity: card.rarity,
              image: card.image,
              count: amount,
              status: "owned"
            },
          ],
        }))
      }

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Unstaking Successful",
        description: `You have successfully unstaked ${amount} ${card.name} card(s).`,
        type: "success",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
    }, 1000)
  }

  // Add these handler functions for synthesis
  const handleSelectSynthesisCard = (index: number) => {
    if (assets.cards.filter((card) => card.count > 0).length === 0) {
      setDialogState({
        open: true,
        title: "No Cards Available",
        description: "You don't have any cards to synthesize with.",
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setSelectorType("synthesis")
    setSelectorIndex(index)
    setShowCardSelector(true)
  }

  const handleRemoveSynthesisCard = (index: number) => {
    const newCards = [...synthesisCards]
    newCards[index] = null
    setSynthesisCards(newCards)
  }

  const handleSelectUpgradeCard = () => {
    if (assets.cards.filter((card) => card.count > 0).length === 0) {
      setDialogState({
        open: true,
        title: "No Cards Available",
        description: "You don't have any cards to upgrade.",
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setSelectorType("upgrade")
    setShowCardSelector(true)
  }

  const handleCardSelectorSelect = (card: CardItem) => {
    if (selectorType === "synthesis" && selectorIndex !== null) {
      const newCards = [...synthesisCards]
      newCards[selectorIndex] = card
      setSynthesisCards(newCards)
    } else if (selectorType === "upgrade") {
      setUpgradeCard(card)
    }
    setShowCardSelector(false)
  }

  const handleSynthesizeCards = () => {
    if (synthesisCards.filter(Boolean).length !== 3) {
      setDialogState({
        open: true,
        title: "Incomplete Selection",
        description: "Please select three cards for synthesis.",
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    if (assets.fragments < 250) {
      setDialogState({
        open: true,
        title: "Insufficient Fragments",
        description: "You need 250 fragments to synthesize a new card.",
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setDialogState({
      open: true,
      title: "Confirm Synthesis",
      description: "Are you sure you want to synthesize these 3 cards with 250 fragments?",
      type: "confirm",
      confirmAction: confirmSynthesis,
    })
  }

  const confirmSynthesis = () => {
    setIsLoading(true)

    setTimeout(() => {
      // Remove the selected cards from collection
      synthesisCards.forEach((card) => {
        setAssets((prev) => ({
          ...prev,
          cards: prev.cards.map((c) => (c.id === card.id ? { ...c, count: c.count - 1 } : c)),
        }))
      })

      // Deduct fragments
      setAssets((prev) => ({
        ...prev,
        fragments: prev.fragments - 250,
      }))

      // Add a random card (weighted towards higher rarity based on input cards)
      const raritiesUsed = synthesisCards.map((card) => card.rarity)
      let newRarity = "Common"

      if (raritiesUsed.includes("Rare") || raritiesUsed.filter((r) => r === "Uncommon").length >= 2) {
        newRarity = Math.random() > 0.7 ? "Rare" : "Uncommon"
      } else if (raritiesUsed.includes("Uncommon")) {
        newRarity = Math.random() > 0.6 ? "Uncommon" : "Common"
      }

      // Pick a random card from the rentalCards list with matching rarity
      const possibleCards = rentalCards.filter((card) => card.rarity === newRarity)
      const newCard = possibleCards[Math.floor(Math.random() * possibleCards.length)]

      // Check if we already have this card type
      const existingCard = assets.cards.find((c) => c.name === newCard.name)

      if (existingCard) {
        setAssets((prev) => ({
          ...prev,
          cards: prev.cards.map((c) => (c.name === newCard.name ? { ...c, count: c.count + 1 } : c)),
        }))
      } else {
        setAssets((prev) => ({
          ...prev,
          cards: [
            ...prev.cards,
            {
              id: Date.now(),
              name: newCard.name,
              rarity: newCard.rarity,
              image: newCard.image,
              count: 1,
              status: "owned"
            },
          ],
        }))
      }

      // Reset synthesis cards
      setSynthesisCards(Array(3).fill(null))

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Synthesis Successful",
        description: `You have successfully synthesized a ${newCard.name} card!`,
        type: "success",
        confirmText: "OK",
        confirmAction: () => {
          setDialogState((prev) => ({ ...prev, open: false }))
          setActiveTab("collection")
        },
      })
    }, 1500)
  }

  const handleUpgradeCardAction = () => {
    if (!upgradeCard) {
      setDialogState({
        open: true,
        title: "No Card Selected",
        description: "Please select a card to upgrade.",
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    if (assets.fragments < 150) {
      setDialogState({
        open: true,
        title: "Insufficient Fragments",
        description: "You need 150 fragments to upgrade a card.",
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setDialogState({
      open: true,
      title: "Confirm Upgrade",
      description: `Are you sure you want to upgrade your ${upgradeCard.name} with 150 fragments?`,
      type: "confirm",
      confirmAction: confirmUpgrade,
    })
  }

  const confirmUpgrade = () => {
    if (!upgradeCard) return
    setIsLoading(true)

    setTimeout(() => {
      // Deduct fragments
      setAssets((prev) => ({
        ...prev,
        fragments: prev.fragments - 150,
      }))

      // Remove the card being upgraded
      setAssets((prev) => ({
        ...prev,
        cards: prev.cards.map((c) => (c.id === upgradeCard.id ? { ...c, count: c.count - 1, upgraded: true } : c)),
      }))

      // Add the upgraded card with "+" in the name
      const upgradedCardName = `${upgradeCard.name}+`
      const existingUpgradedCard = assets.cards.find((c) => c.name === upgradedCardName)

      if (existingUpgradedCard) {
        setAssets((prev) => ({
          ...prev,
          cards: prev.cards.map((c) => (c.name === upgradedCardName ? { ...c, count: c.count + 1 } : c)),
        }))
      } else {
        setAssets((prev) => ({
          ...prev,
          cards: [
            ...prev.cards,
            {
              id: Date.now(),
              name: upgradedCardName,
              rarity: upgradeCard.rarity,
              image: upgradeCard.image,
              count: 1,
              upgraded: true,
              status: "owned"
            },
          ],
        }))
      }

      setUpgradeCard(null)

      setIsLoading(false)
      setDialogState({
        open: true,
        title: "Upgrade Successful",
        description: `You have successfully upgraded your ${upgradeCard.name} card!`,
        type: "success",
        confirmText: "OK",
        confirmAction: () => {
          setDialogState((prev: DialogState) => ({ ...prev, open: false }))
          setActiveTab("collection")
        },
      })
    }, 1500)
  }

  // Gacha system functions
  const handleDrawCard = () => {
    if (assets.coins < selectedPack.price) {
      setDialogState({
        open: true,
        title: "Insufficient Funds",
        description: `You need ${selectedPack.price} coins to purchase this pack.`,
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false })),
      })
      return
    }

    setIsDrawing(true)
    setDrawProgress(0)

    // Simulate drawing animation
    const interval = setInterval(() => {
      setDrawProgress((prev) => {
        if (prev >= 100) {
          clearInterval(interval)
          return 100
        }
        return prev + 5
      })
    }, 100)

    // Simulate API call
    setTimeout(() => {
      // Determine card rarity based on drop rates
      const rand = Math.random() * 100
      let selectedRarity = "Common"
      let cumulativeChance = 0

      for (const [rarity, chance] of Object.entries(selectedPack.dropRates)) {
        cumulativeChance += chance
        if (rand <= cumulativeChance) {
          selectedRarity = rarity
          break
        }
      }

      // Filter cards by rarity
      const possibleCards = extendedCardPool.filter((card) => card.rarity === selectedRarity)
      const newCard = possibleCards[Math.floor(Math.random() * possibleCards.length)]

      // Deduct coins
      setAssets((prev) => ({
        ...prev,
        coins: prev.coins - selectedPack.price,
      }))

      // Add card to collection
      const existingCard = assets.cards.find((c) => c.name === newCard.name)
      if (existingCard) {
        setAssets((prev) => ({
          ...prev,
          cards: prev.cards.map((c) => (c.name === newCard.name ? { ...c, count: c.count + 1 } : c)),
        }))
      } else {
        setAssets((prev) => ({
          ...prev,
          cards: [
            ...prev.cards,
            {
              id: Date.now(),
              name: newCard.name,
              rarity: newCard.rarity,
              image: newCard.image,
              count: 1,
              status: "owned"
            },
          ],
        }))
      }

      // Add to draw history
      setDrawHistory((prev: DrawHistoryItem[]) => [
        {
          id: Date.now(),
          packName: selectedPack.name,
          card: {
            ...newCard,
            count: 1,
            status: "owned"
          } as CardItem,
          timestamp: new Date().toLocaleTimeString(),
        },
        ...prev.slice(0, 9),
      ])

      // Show drawn card
      setDrawnCard(newCard)
      setShowDrawnCard(true)
      setIsDrawing(false)
    }, 2500)
  }

  const handleCloseDrawnCard = () => {
    setShowDrawnCard(false)
    setDrawnCard(null)
  }

  const handleChangePack = (direction: string) => {
    const newIndex =
      direction === "next"
        ? (activePackIndex + 1) % cardPacks.length
        : (activePackIndex - 1 + cardPacks.length) % cardPacks.length

    setActivePackIndex(newIndex)
    setSelectedPack(cardPacks[newIndex])
  }

  // Card flip animation
  useEffect(() => {
    if (showDrawnCard && cardRef.current) {
      (cardRef.current as HTMLElement).classList.add("flipped")
    }
  }, [showDrawnCard])

    // 添加钱包处理函数（可选）
    const handleWalletClick = () => {
      setDialogState({
        open: true,
        title: "连接钱包",
        description: "请选择要连接的钱包",
        type: "confirm",
        confirmText: "连接",
        cancelText: "取消",
        confirmAction: () => setDialogState((prev) => ({ ...prev, open: false }))
      });
    };
    
  // 添加 handleRentCard 函数
  const handleRentCard = (card: RentalCard, uses: number, period: number) => {
    const cost = card.rate * period;

    if (assets.coins < cost) {
      setDialogState(setDefaultDialog({
        title: "Insufficient Funds",
        description: `You need ${cost} coins to rent this card for ${period} day(s).`,
        type: "error",
        confirmText: "OK",
        confirmAction: () => setDialogState((prev: DialogState) => ({ ...prev, open: false }))
      }));
      return;
    }

    setDialogState(setDefaultDialog({
      title: "Confirm Rental",
      description: `Rent ${card.name} for ${period} day(s) with ${uses} uses for ${cost} coins?`,
      type: "confirm",
      confirmAction: () => confirmRentCard(card, cost),
      data: card
    }));
  }

  // 修改卡片相关的状态更新
  const updateCardInAssets = (card: Partial<CardItem> & { id: number | string; name: string; rarity: string; image: string; count: number }) => {
    setAssets((prev) => ({
      ...prev,
      cards: prev.cards.map((c) => 
        c.id === card.id 
          ? { ...c, ...card, status: card.status || "owned" }
          : c
      ),
    }));
  }

  // 修改添加新卡片的函数
  const addNewCardToAssets = (card: Partial<CardItem> & { name: string; rarity: string; image: string; count: number }) => {
    setAssets((prev) => ({
      ...prev,
      cards: [
        ...prev.cards,
        { ...card, id: Date.now(), status: "owned" },
      ],
    }));
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-purple-900 via-violet-800 to-indigo-900">
      {/* Header with assets */}
      <Header 
        assets={assets} 
        onWalletClick={handleWalletClick} 
      />

      {/* Welcome modal for new users */}
      <Welcome 
      assets={assets}
      isVisible={showWelcome}
      onClose={() => setShowWelcome(false)}
    />

      {/* Dialog for interactions */}
      <DialogModal
        open={dialogState.open}
        title={dialogState.title}
        description={dialogState.description}
        type={dialogState.type as "success" | "error" | "confirm" | "stakeInput" | ""}
        confirmAction={dialogState.confirmAction || (() => {})}
        confirmText={dialogState.confirmText || "Confirm"}
        cancelText={dialogState.cancelText || "Cancel"}
        data={dialogState.data}
        isLoading={isLoading}
        onOpenChange={(open) => setDialogState((prev) => ({ ...prev, open }))}
      />

      {/* Card drawing result modal */}
      <DrawnCardModal 
        drawnCard={drawnCard}
        isVisible={showDrawnCard}
        onClose={handleCloseDrawnCard}
      />

      {/* Main content area */}
      <main className="container max-w-3xl mx-auto px-2 py-3">
        <Tabs defaultValue="collection" value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid grid-cols-6 mb-4 bg-black/20 p-1 rounded-lg">
            <TabsTrigger value="collection" className="data-[state=active]:bg-purple-700 h-8 text-xs">
              <Layers className="h-3 w-3 mr-1" />
              <span className="hidden sm:inline">Collection</span>
            </TabsTrigger>
            <TabsTrigger value="synthesis" className="data-[state=active]:bg-purple-700 h-8 text-xs">
              <Sparkles className="h-3 w-3 mr-1" />
              <span className="hidden sm:inline">Synthesis</span>
            </TabsTrigger>
            <TabsTrigger value="staking" className="data-[state=active]:bg-purple-700 h-8 text-xs">
              <Coins className="h-3 w-3 mr-1" />
              <span className="hidden sm:inline">Staking</span>
            </TabsTrigger>
            <TabsTrigger value="rental" className="data-[state=active]:bg-purple-700 h-8 text-xs">
              <Clock className="h-3 w-3 mr-1" />
              <span className="hidden sm:inline">Rental</span>
            </TabsTrigger>
            <TabsTrigger value="exchange" className="data-[state=active]:bg-purple-700 h-8 text-xs" >
              <ArrowRightLeft className="h-3 w-3 mr-1" />
              <span className="hidden sm:inline">Exchange</span>
            </TabsTrigger>
            <TabsTrigger value="matchmaking" className="data-[state=active]:bg-purple-700 h-8 text-xs">
              <Users className="h-3 w-3 mr-1" />
              <span className="hidden sm:inline">Play</span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="collection" className="mt-0 max-w-3xl mx-auto">
            <CardCollection 
              cards={getAllCards(assets.cards, myRentedCards, myStakedCards)}
              onGetMoreClick={() => setActiveTab("synthesis")}
            />
          </TabsContent>
          
          <TabsContent value="synthesis" className="mt-0 max-w-3xl mx-auto">
            <CardSynthesisGacha
              assets={assets}
              synthesisCards={synthesisCards}
              upgradeCard={upgradeCard}
              cardPacks={cardPacks}
              selectedPack={selectedPack}
              activePackIndex={activePackIndex}
              drawHistory={drawHistory}
              isLoading={isLoading}
              isDrawing={isDrawing}
              drawProgress={drawProgress}
              showCardSelector={showCardSelector}
              selectorType={selectorType}
              handleSelectSynthesisCard={handleSelectSynthesisCard}
              handleRemoveSynthesisCard={handleRemoveSynthesisCard}
              handleSelectUpgradeCard={handleSelectUpgradeCard}
              handleSynthesizeCards={handleSynthesizeCards}
              handleUpgradeCardAction={handleUpgradeCardAction}
              handleDrawCard={handleDrawCard}
              handleChangePack={handleChangePack}
              setActivePackIndex={setActivePackIndex}
              setSelectedPack={setSelectedPack}
              setShowCardSelector={setShowCardSelector}
              handleCardSelectorSelect={handleCardSelectorSelect}
              setUpgradeCard={setUpgradeCard}
            />
          </TabsContent>

          <TabsContent value="staking" className="mt-0 max-w-3xl mx-auto">
            <CardStakingPools
              myStakedCards={myStakedCards}
              stakingPools={stakingPools}
              handleUnstakeCard={handleUnstakeCard}
              handleStakeCard={handleStakeCard}
              handleClaimRewards={handleClaimRewards}
            />
          </TabsContent>

          <TabsContent value="rental" className="mt-0 max-w-3xl mx-auto">
            <CardRentalMarketplace 
              rentalCards={rentalCards}
              myRentedCards={myRentedCards}
              rentalHistory={rentalHistory}
              handleRentCard={handleRentCard}
            />
          </TabsContent>

          <TabsContent value="exchange" className="mt-0 max-w-3xl mx-auto">
            <Exchange 
              initialAssets={assets}
              onAssetsChange={(newAssets) => setAssets(newAssets)}
            />
          </TabsContent>

          <TabsContent value="matchmaking" className="mt-0 max-w-3xl mx-auto">
            <CardGameMatches 
              gameMatches={gameMatches} 
              handleJoinGame={handleJoinGame}
            />
          </TabsContent>
          
        </Tabs>
      </main>
    </div>
  )
}