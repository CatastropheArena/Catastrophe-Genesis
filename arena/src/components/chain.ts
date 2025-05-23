import { create } from 'zustand';
import { SuiChain,suiTestnet } from '@ant-design/web3-sui';
import { Chain } from '@ant-design/web3';
// 定义 ChainStore 的类型
interface ChainStore {
  currentChain: SuiChain;
  setCurrentChain: (chain:  Chain|undefined) => void;
}

// 创建全局 chain store
export const useChainStore = create<ChainStore>((set) => ({
  currentChain: suiTestnet,
  setCurrentChain: (chain) => set({ currentChain: chain as SuiChain }),
}));