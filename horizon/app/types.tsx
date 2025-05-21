// 基础资产接口
import React from "react";
export interface Assets {
  coins: number;
  fragments: number;
  usdt: number;
  cards: CardItem[];
}

// 基础卡片接口
export interface CardItem {
  id: string | number;
  name: string;
  description: string;
  strategy: string;
  rarity: string;
  image: string;
  cost: number;
  dynamicCost: number;
  level: number;
  count: number;
  status: "owned" | "rented" | "staked";
}

// 租赁卡片接口
export interface RentalCard {
  id: number | string;
  name: string;
  rarity: string;
  image: string;
  poolSize: number;
  rate: number; // 每日租金
}

// 已租赁卡片接口（扩展自RentalCard）
export interface RentedCard extends RentalCard {
  usesLeft: number;
  totalUses: number;
  expiresIn: number;
}

// 我的租赁卡片接口
export interface MyRentalCard {
  id: number | string;
  name: string;
  rarity: string;
  image: string;
  usesLeft: number;
  totalUses: number;
  expiresIn: number;
  rate: number;
  poolSize?: number;
}

// 租赁历史记录接口
export interface RentalHistoryItem {
  id: number | string;
  cardId: number | string;
  cardName: string;
  cardImage: string;
  source: string;
  period: number;
  uses: number;
  cost: number;
  status: "active" | "expired";
  expiryDate: string;
}

// 质押卡片接口
export interface StakedCard {
  id: number | string;
  name: string;
  rarity: string;
  image: string;
  stakedCount: number;
  poolShare: string;
  earned: number;
  stakedAmount?: number;
}

// 质押池接口
export interface StakingPool {
  id: number | string;
  name: string;
  rarity: string;
  image: string;
  poolSize: number;
  apr: string;
  rate: number;
}

// 卡片包接口
export interface CardPack {
  id: number;
  name: string;
  price: number;
  image: string;
  description: string;
  dropRates: {
    Common: number;
    Uncommon: number;
    Rare: number;
    Legendary: number;
  };
}

// 抽卡历史记录接口
export interface DrawHistoryItem {
  id: number | string;
  packName: string;
  card: CardItem;
  timestamp: string;
}

// 游戏比赛接口
export interface GameMatch {
  id: number;
  name: string;
  level: string;
  entryFee: number;
  currentPlayers: number;
  maxPlayers: number;
  rewards: number;
  bgClass: string;
  badgeClass: string;
}

// 对话框类型
export type DialogType = "confirm" | "stakeInput" | "success" | "error" | "";

// 抽到的卡片接口
export interface DrawnCard {
  id: number;
  name: string;
  rarity: string;
  image: string;
  poolSize?: number;
  rate?: number;
}

// 代币价格接口
export interface TokenPrices {
  coins_fragments: number;
  fragments_coins: number;
  usdt_coins: number;
  coins_usdt: number;
}

// 代币信息接口
export interface TokenInfo {
  symbol: string;
  icon: React.ElementType;
  color: string;
  name: string;
}

// 对话框状态接口
export interface DialogState {
  open: boolean;
  title: string;
  description: string;
  type: DialogType;
  confirmText: string;
  cancelText?: string;
  data: any;
  isLoading: boolean;
  confirmAction?: (amount?: number) => void;
}

// 选择器状态接口
export interface SelectorState {
  type: string;
  index: number | null;
}

export interface SponsorTxRequestBody {
  network: "mainnet" | "testnet";
  txBytes: string;
  sender: string;
  allowedAddresses?: string[];
}

export interface CreateSponsoredTransactionApiResponse {
  bytes: string;
  digest: string;
}
