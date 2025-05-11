//Type
type RpcNode = {
  name: string;
  url: string;
  latency: number;
};

type Network = "mainnet" | "testnet" | "devnet";

type TokenInfo = {
  token: COIN;
  symbol: string;
  iconPath: string;
  isLST?: boolean;
};

type BasicCoin = "SUI" | "USDC" | "USDT" | "BUCK";

type Passport = {
  id: string;
  owner: string;
  last_claim_time: number;
  daily_rewards_claimed: number;
  rental_cards: string[];
  created_at: number;
};
