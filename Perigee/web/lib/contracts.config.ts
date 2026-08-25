export type Network = "testnet" | "mainnet";

export const contractsConfig = {
  testnet: { contractId: "CAEZJVJ4N7P7GRUVD5NG5LYYH23AQHJUKQEUHW54LR5PGQX3V7FXD_Q" },
  mainnet: { contractId: "" },
} as const;

export const getContractsConfig = (network: Network) => contractsConfig[network];