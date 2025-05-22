"use client";

import { createContext, useContext, ReactNode } from "react";
import { useUserAssets } from "@/hooks/useUserAssets";
import { CoinStruct, SuiObjectResponse } from "@mysten/sui/client";
interface Assets {
  sui: number;
  coins: number;
  fragments: number;
  suiCoins: CoinStruct[];
  fishCoins: CoinStruct[];
  passport: SuiObjectResponse | undefined;
}

interface AssetsContextType {
  assets: Assets;
  isLoading: boolean;
  error: string | null;
  fetchAssets: () => Promise<void>;
}

const AssetsContext = createContext<AssetsContextType | undefined>(undefined);

export function AssetsProvider({ children }: { children: ReactNode }) {
  const assetsData = useUserAssets();

  return (
    <AssetsContext.Provider value={assetsData}>
      {children}
    </AssetsContext.Provider>
  );
}

export function useAssets() {
  const context = useContext(AssetsContext);
  if (context === undefined) {
    throw new Error("useAssets must be used within an AssetsProvider");
  }
  return context;
}
