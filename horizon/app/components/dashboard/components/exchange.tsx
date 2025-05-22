"use client";

import React, { useState, useEffect } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ArrowRightLeft, ArrowUpDown } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/components/ui/use-toast";
import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { Transaction } from "@mysten/sui/transactions";
import { useBetterSignAndExecuteTransaction } from "@/hooks/useBetterTx";
import { useAssets } from "@/context/AssetsContext";
import { useLoading } from "@/context/LoadingContext";

// Token configuration
const TOKENS = {
  sui: {
    symbol: "SUI",
    name: "SUI",
    decimals: 9,
  },
  fish: {
    symbol: "FISH",
    name: "FISH",
    decimals: 0,
  },
} as const;

// Exchange rate: 1 SUI = 100 FISH
const EXCHANGE_RATE = {
  sui_fish: 100,
};

export default function Exchange() {
  const { toast } = useToast();
  const { showLoading, hideLoading } = useLoading();
  const { assets, fetchAssets } = useAssets();
  const account = useCurrentAccount();

  // State for input amount and token selection
  const [fromToken, setFromToken] = useState<"sui">("sui");
  const [toToken, setToToken] = useState<"fish">("fish");
  const [amount, setAmount] = useState<string>("");

  // Transaction handler
  const { handleSignAndExecuteTransaction } =
    useBetterSignAndExecuteTransaction({
      tx: () => {
        const tx = new Transaction();

        // Convert input amount to MIST (SUI's smallest unit)
        const suiAmount = BigInt(parseFloat(amount) * 1e9);
        const fishAmount = BigInt(parseFloat(amount) * 100);

        // If multiple coin objects exist, merge them first
        if (assets.suiCoins.length > 1) {
          let baseCoin = assets.suiCoins[0].coinObjectId;
          let otherCoins = assets.suiCoins
            .slice(1)
            .map((coin) => coin.coinObjectId);
          tx.mergeCoins(baseCoin, otherCoins);
        }

        // Split exact amount needed for the swap
        const splitCoin = tx.splitCoins(tx.gas, [tx.pure.u64(suiAmount)]);

        // Call buy_fish function
        tx.moveCall({
          target: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::user::buy_fish`,
          arguments: [
            tx.object(`${process.env.NEXT_PUBLIC_TESTNET_TREASURY}`),
            splitCoin,
            tx.pure.u64(fishAmount),
            tx.object("0x6"),
          ],
        });

        return tx;
      },
      options: {
        showEffects: true,
        showObjectChanges: true,
      },
    });

  // Calculate output amount based on input
  const calculateOutputAmount = (inputAmount: string): string => {
    if (!inputAmount || isNaN(parseFloat(inputAmount))) return "0";
    return (parseFloat(inputAmount) * EXCHANGE_RATE.sui_fish).toString();
  };

  // Handle swap execution
  const handleSwap = async () => {
    console.log("test");

    if (!amount || parseFloat(amount) <= 0) {
      toast({
        title: "Invalid amount",
        description: "Please enter a valid amount",
        variant: "destructive",
      });
      return;
    }

    const suiAmount = parseFloat(amount);
    const availableSui = Number(assets.sui) / 1e9;

    if (suiAmount > availableSui) {
      toast({
        title: "Insufficient balance",
        description: "You don't have enough SUI",
        variant: "destructive",
      });
      return;
    }

    try {
      showLoading("Processing swap...");
      await handleSignAndExecuteTransaction()
        .beforeExecute(async () => {
          return true;
        })
        .onSuccess(async () => {
          toast({
            title: "Success",
            description: "Swap completed successfully",
            variant: "default",
          });
          await fetchAssets();
          setAmount("");
        })
        .onError((error: any) => {
          toast({
            title: "Error",
            description: error.message || "Failed to complete swap",
            variant: "destructive",
          });
        })
        .execute();
    } catch (error) {
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Swap failed",
        variant: "destructive",
      });
    } finally {
      hideLoading();
    }
  };

  return (
    <Card className="bg-black/40 backdrop-blur-md border-purple-500/30">
      <CardHeader className="p-3">
        <CardTitle className="text-white flex items-center gap-2 text-base">
          <ArrowRightLeft className="h-4 w-4 text-purple-400" />
          Exchange
        </CardTitle>
        <CardDescription className="text-purple-200 text-xs">
          Swap between SUI and FISH tokens
        </CardDescription>
      </CardHeader>
      <CardContent className="p-3">
        <div className="space-y-4">
          <div className="bg-black/30 rounded-lg p-3">
            <h3 className="text-sm font-medium text-white mb-2">
              Token Exchange
            </h3>
            <div className="space-y-4">
              {/* Input Amount */}
              <div className="relative group">
                <Input
                  type="number"
                  placeholder="0.0"
                  className="bg-black/50 border border-purple-400/50 h-16 px-4 pt-6 text-2xl text-white font-light"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  min="0"
                  step="0.000000001"
                />
                <div className="absolute top-2 left-4 text-[10px] text-purple-200 font-medium tracking-wide">
                  Amount
                </div>
                <div className="absolute right-2 top-1/2 -translate-y-1/2">
                  <div className="bg-black/50 border border-purple-400/50 px-4 py-1 rounded">
                    <span className="text-white">
                      {TOKENS[fromToken].symbol}
                    </span>
                  </div>
                </div>
              </div>
              <div className="text-[10px] text-purple-200/80 px-1 font-light">
                Balance: {(Number(assets.sui) / 1e9).toFixed(9)}{" "}
                {TOKENS[fromToken].symbol}
              </div>

              {/* Switch Button */}
              <div className="flex justify-center -my-2 relative z-10">
                <Button
                  variant="ghost"
                  size="icon"
                  className="rounded-full bg-purple-500/5 hover:bg-purple-500/20 h-8 w-8 shadow-lg shadow-purple-500/20
                    transition-all duration-200 hover:scale-110 active:scale-95"
                  disabled
                >
                  <ArrowUpDown className="h-3 w-3 text-purple-400" />
                </Button>
              </div>

              {/* Output Amount */}
              <div className="relative group">
                <div className="bg-black/50 border border-purple-400/50 rounded-md h-16 px-4 pt-6 flex items-center text-2xl text-gray-400 font-light">
                  {calculateOutputAmount(amount)}
                </div>
                <div className="absolute top-2 left-4 text-[10px] text-purple-200 font-medium tracking-wide">
                  You will receive
                </div>
                <div className="absolute right-2 top-1/2 -translate-y-1/2">
                  <div className="bg-black/50 border border-purple-400/50 px-4 py-1 rounded">
                    <span className="text-white">{TOKENS[toToken].symbol}</span>
                  </div>
                </div>
              </div>

              <Button
                onClick={handleSwap}
                className="w-full bg-gradient-to-r from-purple-600 to-pink-600 h-8 text-xs font-medium tracking-wide
                  transition-all duration-200 hover:scale-[1.02] active:scale-[0.98] shadow-xl shadow-purple-500/20
                  disabled:opacity-50 disabled:hover:scale-100 disabled:cursor-not-allowed"
                disabled={
                  !amount ||
                  Number(amount) <= 0 ||
                  Number(amount) > Number(assets.sui) / 1e9
                }
              >
                {!amount || Number(amount) <= 0
                  ? "Enter amount"
                  : Number(amount) > Number(assets.sui) / 1e9
                  ? "Insufficient Balance"
                  : "Confirm Swap"}
              </Button>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
