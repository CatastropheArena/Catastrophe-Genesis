"use client";

import { useState, useEffect } from "react";
import { ConnectButton } from "@mysten/dapp-kit";
import { Button } from "@/components/ui/button";

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

      {/* Main content */}
      <div className="relative z-10 flex flex-col items-center justify-center min-h-screen px-4">
        <h1 className="text-4xl md:text-6xl font-bold text-white mb-6 text-center">
          Welcome to Catastrophe Genesis
        </h1>
        <p className="text-xl text-purple-200 mb-8 text-center max-w-2xl">
          Experience the next generation of blockchain gaming
        </p>
        <ConnectButton className="bg-purple-600 hover:bg-purple-700 text-white px-8 py-3 rounded-full text-lg" />
      </div>
    </div>
  );
}
