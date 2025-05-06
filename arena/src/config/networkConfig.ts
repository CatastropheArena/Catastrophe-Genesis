import {createNetworkConfig, NetworkConfig} from "@mysten/dapp-kit";
import { getRpcNodes } from "./rpcNodeList";

export const TESTNET_NEXUS_PACKAGE_ID =
    "0x499780b7b435096e8b39e870c81748d6bea1d71e115dd681952b6869dd3c3c4a";
export const TESTNET_CITADEL_OBJECT_ID =
    "0xe10eb87f0020b576a9394699e9cfbe35fe4bd4800411d8558b3f0e281142b27e";

// 定义具体的 Variables 类型
interface Variables {
    NexusPackage: string;
    CitadelPackage: string;
}

const {networkConfig, useNetworkVariable, useNetworkVariables} =
    createNetworkConfig({
        testnet: {
            url: getRpcNodes("testnet")[0].url,
            variables: {
                NexusPackage: TESTNET_NEXUS_PACKAGE_ID,
                CitadelPackage: TESTNET_CITADEL_OBJECT_ID,
            }
        },
        mainnet: {
            url: getRpcNodes("mainnet")[0].url,
            variables: {
                NexusPackage: TESTNET_NEXUS_PACKAGE_ID,
                CitadelPackage: TESTNET_CITADEL_OBJECT_ID,
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
