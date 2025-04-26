interface ContractAddresses {
  [key: string]: string;
}

type NetworkType = "testnet" | "mainnet";

const configs = {
  testnet: {
    Package: process.env.NEXT_PUBLIC_TESTNET_PACKAGE || "",
  },
  mainnet: {
    Package: process.env.NEXT_PUBLIC_MAINNET_PACKAGE || "",
  },
} as const satisfies Record<NetworkType, ContractAddresses>;

export function getContractConfig(network: NetworkType): ContractAddresses {
  return configs[network];
}
