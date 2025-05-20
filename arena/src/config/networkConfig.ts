import {createNetworkConfig, NetworkConfig} from "@mysten/dapp-kit";
import { getRpcNodes } from "./rpcNodeList";

export const TESTNET_CITADEL_OBJECT_ID =
    "0xe10eb87f0020b576a9394699e9cfbe35fe4bd4800411d8558b3f0e281142b27e";

// 定义具体的 Variables 类型
interface Variables {
    NexusPackage: string;
    NexusPassportState: string;
    NexusTreasury: string;
    NexusFragmentStore: string;
    CitadelPackage: string;
}

const {networkConfig, useNetworkVariable, useNetworkVariables} =
    createNetworkConfig({
        testnet: {
            url: getRpcNodes("testnet")[0].url,
            variables: {
                NexusPackage: import.meta.env.VITE_PUBLIC_TESTNET_PACKAGE || '',
                CitadelPackage: TESTNET_CITADEL_OBJECT_ID,
                NexusPassportState: import.meta.env.VITE_PUBLIC_TESTNET_PASSPORT_STATE || '',
                NexusTreasury: import.meta.env.VITE_PUBLIC_TESTNET_TREASURY || '',
                NexusFragmentStore: import.meta.env.VITE_PUBLIC_TESTNET_FRAGMENT_STORE || '',
            }
        },
        mainnet: {
            url: getRpcNodes("mainnet")[0].url,
            variables: {
                NexusPackage: import.meta.env.VITE_PUBLIC_TESTNET_PACKAGE || '',
                CitadelPackage: TESTNET_CITADEL_OBJECT_ID,
                NexusPassportState: import.meta.env.VITE_PUBLIC_TESTNET_PASSPORT_STATE || '',
                NexusTreasury: import.meta.env.VITE_PUBLIC_TESTNET_TREASURY || '',
                NexusFragmentStore: import.meta.env.VITE_PUBLIC_TESTNET_FRAGMENT_STORE || '',
            }
        },
    } as Record<string, NetworkConfig<Variables>>);

// 获取网络变量（合约地址等）
export function getNetworkVariables(network: Network) {
    return networkConfig[network].variables;
}

// 获取默认RPC URL
export function getDefaultRpcUrl(network: Network) {
    return getRpcNodes(network)[0].url;
}

export {useNetworkVariable, useNetworkVariables, networkConfig};
