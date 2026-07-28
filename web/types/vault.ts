export type VaultStatus =
  | "ACTIVE"
  | "LOCKED"
  | "PENDING"
  | "ARCHIVED";

export interface Vault {
  id: string;

  name: string;

  owner: string;

  balance: number;

  asset: string;

  apy?: number;

  status: VaultStatus;

  createdAt: string;

  updatedAt: string;
}