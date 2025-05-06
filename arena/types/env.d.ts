interface ImportMetaEnv {
  readonly VITE_BACKEND_URL: string;
  readonly VITE_ENV: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

type RpcNode = {
  name: string;
  url: string;
  latency: number;
};

type Network = "mainnet" | "testnet" | "devnet";
