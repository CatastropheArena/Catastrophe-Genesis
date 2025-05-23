"use client";

import { useState, useEffect } from "react";
import { ConnectButton, ConnectModal } from "@mysten/dapp-kit";
import { Gamepad2, Coins, WalletCardsIcon as Cards, Trophy, ArrowRight } from "lucide-react";
import FeatureCard from "@/app/components/home/feature-card";
import GameCard from "@/app/components/home/game-card";
import TokenDistribution from "@/app/components/home/token-distribution";
import UserJourney from "@/app/components/home/user-journey";

export default function HomePage() {
  const [particles, setParticles] = useState<
    Array<{
      width: number;
      height: number;
      top: number;
      left: number;
      duration: number;
      delay: number;
    }>
  >([]);

  useEffect(() => {
    // 在客户端生成随机粒子
    const newParticles = Array.from({ length: 20 }).map(() => ({
      width: Math.random() * 10 + 5,
      height: Math.random() * 10 + 5,
      top: Math.random() * 100,
      left: Math.random() * 100,
      duration: Math.random() * 10 + 10,
      delay: Math.random() * 5,
    }));
    setParticles(newParticles);
  }, []);

  return (
    <div className="relative min-h-screen bg-gradient-to-b from-purple-900 via-violet-800 to-indigo-900 overflow-hidden">
      {/* Background particles */}
      <div className="absolute inset-0 overflow-hidden">
        {particles.map((particle, i) => (
          <div
            key={i}
            className="absolute rounded-full bg-white/10 animate-float"
            style={{
              width: `${particle.width}px`,
              height: `${particle.height}px`,
              top: `${particle.top}%`,
              left: `${particle.left}%`,
              animationDuration: `${particle.duration}s`,
              animationDelay: `${particle.delay}s`,
            }}
          />
        ))}
      </div>
      {/* Hero Section */}
      <section className="container mx-auto py-20 flex flex-col lg:flex-row items-center gap-12 relative z-10">
        <div className="flex-1 space-y-6">
          <h1 className="text-5xl md:text-6xl font-bold leading-tight text-white">
            Strategy Meets Profit with
            <br />
            <span className="bg-clip-text text-transparent bg-gradient-to-r from-purple-400 to-violet-300">
              Catastrophe
            </span>{" "}
            GameFi
          </h1>
          <p className="text-xl text-purple-200 max-w-xl">
            A strategic card battle game combining classic gameplay with GameFi economic models. Have
            fun and earn rewards.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 pt-4">
          <ConnectModal
              trigger={
                <div className="flex items-center gap-4">
                  <button className="bg-purple-600 hover:bg-purple-700 text-white px-6 py-3 rounded-lg font-medium flex items-center justify-center gap-2 transition-colors">
                    Start Game
                    <ArrowRight className="h-4 w-4" />
                  </button>
              </div>
              }
            />
            <button className="bg-purple-800/50 hover:bg-purple-800 text-white px-6 py-3 rounded-lg font-medium">
              Learn More
            </button>
          </div>
        </div>
        <div className="flex-1 relative">
          <div className="relative w-full aspect-square max-w-md mx-auto">
            <GameCard
              className="absolute top-0 left-0 rotate-[-15deg] z-10 scale-90 hover:z-40 transition-all duration-300 hover:scale-100"
              title="Exploding Kitten"
              description="Draw this card and you're dead! Unless you have a Defuse card"
              image="/cards/SINGLE_ATTACK_CARD.png"
              color="bg-red-500"
            />
            <GameCard
              className="absolute top-10 left-20 rotate-[5deg] z-20 hover:z-40 transition-all duration-300 hover:scale-100"
              title="Defuse Card"
              description="Use this card to defuse an Exploding Kitten and place it back in the deck"
              image="/cards/SHIELD_CARD.png"
              color="bg-purple-600"
            />
            <GameCard
              className="absolute top-20 left-0 rotate-[25deg] z-30 scale-90 hover:z-40 transition-all duration-300 hover:scale-100"
              title="Attack Card"
              description="End your turn without drawing a card and force the next player to take two turns"
              image="/cards/GROUP_ATTACK_CARD.png"
              color="bg-violet-500"
            />
            <GameCard
              className="absolute top-30 left-10 rotate-[15deg] z-15 scale-85 hover:z-40 transition-all duration-300 hover:scale-100"
              title="Skip Card"
              description="End your turn without drawing a card"
              image="/cards/ANTI_EXPLOSION_CARD.png"
              color="bg-green-500"
            />
            <GameCard
              className="absolute top-40 left-30 rotate-[-10deg] z-25 scale-95 hover:z-40 transition-all duration-300 hover:scale-100"
              title="Favor Card"
              description="Force another player to give you a card from their hand"
              image="/cards/DISCOUNT_CARD.png"
              color="bg-yellow-500"
            />
            <GameCard
              className="absolute top-50 left-5 rotate-[20deg] z-35 scale-88 hover:z-40 transition-all duration-300 hover:scale-100"
              title="See The Future"
              description="Peek at the top 3 cards from the deck and put them back in the same order"
              image="/cards/DOUBLE_REWARD_CARD.png"
              color="bg-orange-500"
            />
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="container mx-auto py-20 relative z-10">
        <h2 className="text-3xl font-bold text-center mb-12 text-white">Game Features</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          <FeatureCard
            icon={<Cards className="h-10 w-10 text-primary" />}
            title="Strategic Card Battles"
            description="Classic gameplay combined with new strategic elements, making each game unpredictable"
          />
          <FeatureCard
            icon={<Coins className="h-10 w-10 text-primary" />}
            title="Dual Economy System"
            description="Burn cards in-game for fragments, craft powerful cards outside the game, forming a complete economic loop"
          />
          <FeatureCard
            icon={<Trophy className="h-10 w-10 text-primary" />}
            title="Play-to-Earn"
            description="Win resources from game matches, stake cards for mining, multiple ways to profit"
          />
        </div>
      </section>

      {/* Economic System */}
      <section className="container mx-auto py-20 bg-purple-900/50 rounded-xl p-8 relative z-10">
        <h2 className="text-3xl font-bold text-center mb-4 text-white">Economic System</h2>
        <p className="text-center text-purple-200 max-w-2xl mx-auto mb-12">
          A comprehensive dual-token economic model, with game tokens and card fragments working together to create a
          sustainable game ecosystem
        </p>

        <div className="mb-12">
          <h3 className="text-2xl font-semibold mb-6 text-center text-white">Token Distribution</h3>
          <TokenDistribution />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="bg-purple-800/30 p-6 rounded-lg">
            <h3 className="text-xl font-semibold mb-4 flex items-center gap-2 text-white">
              <Coins className="h-5 w-5 text-purple-400" />
              Game Token (FISH)
            </h3>
            <ul className="space-y-2 text-purple-200">
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Total Supply: 1 billion, tradable on DEX</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Uses: Buy fragments, draw cards, rent cards, enter game matches</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Generation: Game match rewards, staking card NFTs for mining</span>
              </li>
            </ul>
          </div>

          <div className="bg-purple-800/30 p-6 rounded-lg">
            <h3 className="text-xl font-semibold mb-4 flex items-center gap-2 text-white">
              <Cards className="h-5 w-5 text-purple-400" />
              Card Fragments
            </h3>
            <ul className="space-y-2 text-purple-200">
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>Dynamic supply, generated when players burn cards</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>In-game use: Activate external cards to convert to hand cards</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-purple-400">•</span>
                <span>External use: Synthesize specific functional card NFTs</span>
              </li>
            </ul>
          </div>
        </div>
      </section>

      {/* User Journey */}
      <section className="container mx-auto py-20 relative z-10">
        <h2 className="text-3xl font-bold text-center mb-4 text-white">User Journey</h2>
        <p className="text-center text-purple-200 max-w-2xl mx-auto mb-12">
          From wallet login to becoming a game master, start your Catastrophe GameFi journey in four simple steps
        </p>

        <UserJourney />
      </section>

      {/* FISN Section */}
      <section className="container mx-auto py-20 relative z-10">
        <div className="bg-gradient-to-r from-purple-600/20 to-purple-800/5 rounded-2xl p-12 text-center">
          <h2 className="text-3xl font-bold mb-4 text-white">Ready to Start Your Gaming Journey?</h2>
          <p className="text-purple-200 max-w-2xl mx-auto mb-8">
            Connect your wallet, claim initial assets, and immediately experience the fun and profits of Catastrophe GameFi
          </p>
          <ConnectButton className="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-full flex items-center gap-2 transition-colors hover:scale-105" />
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-purple-800 relative z-10">
        <div className="container mx-auto py-8">
          <div className="flex flex-col md:flex-row justify-between items-center gap-4">
            <div className="flex items-center gap-2">
              <Gamepad2 className="h-6 w-6 text-purple-400" />
              <span className="text-xl font-bold text-white">Catastrophe</span>
            </div>
            <div className="text-sm text-purple-200">© 2024 Catastrophe. All rights reserved.</div>
          </div>
        </div>
      </footer>
    </div>
  );
}
