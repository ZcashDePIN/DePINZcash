/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL?: string;
  readonly VITE_ZCASH_NETWORK?: "mainnet" | "testnet";
  readonly VITE_SOLANA_CLUSTER?: "devnet" | "testnet" | "mainnet-beta";
  readonly VITE_VAULT_WALLET?: string;
  readonly VITE_VAULT_LABEL?: string;
  readonly VITE_TOKEN_MINT?: string;
  readonly VITE_SOLANA_RPC_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
