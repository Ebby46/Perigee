/**
 * contracts.config.ts
 *
 * Single source of truth for Soroban contract IDs on each deployment stage.
 *
 * Values are read from Next.js NEXT_PUBLIC_* environment variables so they are
 * baked into the client bundle at build time (Next.js requirement for browser
 * access). Set them in:
 *
 *   - .env.local           — local dev overrides (git-ignored)
 *   - .env.testnet         — values written by deploy_testnet.sh (git-ignored)
 *   - Vercel / CI env vars — production / preview overrides
 *
 * Usage:
 *   import { contractIds } from "@/lib/contracts.config";
 *   const id = contractIds.policyVault;      // string | undefined
 *   const id = contractIds.policyVaultStrict // throws if not set
 */

export type Stage = "local" | "testnet" | "mainnet";

export interface ContractIds {
  /** Policy Vault — user fund custody + scoped permissions */
  policyVault: string | undefined;
  /** Strategy Trigger — cycle-phase / drawdown / volatility rule evaluation */
  strategyTrigger: string | undefined;
  /** Fee Accrual — performance fee & high-water-mark tracking */
  feeAccrual: string | undefined;
  /** Emergency Guard — circuit breaker for vault operations */
  emergencyGuard: string | undefined;
  /** Liquidity Pool — AMM used for stable LP rotation (bear phase) */
  liquidityPool: string | undefined;
  /** Token (USDC-equivalent anchor) */
  token: string | undefined;
  /** Oracle Aggregator — price feed aggregation */
  oracleAggregator: string | undefined;
  /** Cross-Chain Verifier — bridge payload / signature verification */
  crossChainVerifier: string | undefined;
  /** Hello Soroban — dev/smoke-test contract */
  helloSoroban: string | undefined;
}

/**
 * Strict version of ContractIds where every field is a non-null string.
 * Use contractIds.strict() to get this; it will throw for any unset ID.
 */
export type StrictContractIds = {
  [K in keyof ContractIds]: string;
};

// ---------------------------------------------------------------------------
// Read from environment variables
// ---------------------------------------------------------------------------

function env(key: string): string | undefined {
  return process.env[key] ?? undefined;
}

/**
 * Contract IDs resolved from NEXT_PUBLIC_* env vars.
 * Undefined means the env var was not set at build time.
 */
export const contractIds: ContractIds & {
  /**
   * Returns a strict copy of contractIds, throwing a descriptive error for any
   * field that is undefined. Call this inside request handlers, not at module
   * load time, so missing IDs surface at runtime with a clear message.
   */
  strict(): StrictContractIds;
} = {
  policyVault:        env("NEXT_PUBLIC_CONTRACT_POLICY_VAULT"),
  strategyTrigger:    env("NEXT_PUBLIC_CONTRACT_STRATEGY_TRIGGER"),
  feeAccrual:         env("NEXT_PUBLIC_CONTRACT_FEE_ACCRUAL"),
  emergencyGuard:     env("NEXT_PUBLIC_CONTRACT_EMERGENCY_GUARD"),
  liquidityPool:      env("NEXT_PUBLIC_CONTRACT_LIQUIDITY_POOL"),
  token:              env("NEXT_PUBLIC_CONTRACT_TOKEN"),
  oracleAggregator:   env("NEXT_PUBLIC_CONTRACT_ORACLE_AGGREGATOR"),
  crossChainVerifier: env("NEXT_PUBLIC_CONTRACT_CROSS_CHAIN_VERIFIER"),
  helloSoroban:       env("NEXT_PUBLIC_CONTRACT_HELLO_SOROBAN"),

  strict(): StrictContractIds {
    const missing: string[] = [];
    const result = {} as StrictContractIds;

    const keys = [
      "policyVault",
      "strategyTrigger",
      "feeAccrual",
      "emergencyGuard",
      "liquidityPool",
      "token",
      "oracleAggregator",
      "crossChainVerifier",
      "helloSoroban",
    ] as const;

    for (const k of keys) {
      const v = contractIds[k];
      if (!v) {
        missing.push(k);
      } else {
        result[k] = v;
      }
    }

    if (missing.length > 0) {
      throw new Error(
        `Missing contract ID environment variable(s): ${missing
          .map((k) => `NEXT_PUBLIC_CONTRACT_${toEnvSuffix(k)}`)
          .join(", ")}. ` +
          "Copy web/.env.example to web/.env.local and fill in the values.",
      );
    }

    return result;
  },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function toEnvSuffix(camelKey: string): string {
  return camelKey
    .replace(/([A-Z])/g, "_$1")
    .toUpperCase();
}

/**
 * Derive the active stage from NEXT_PUBLIC_STELLAR_NETWORK.
 * Falls back to "local" if not set.
 */
export function getStage(): Stage {
  const network = env("NEXT_PUBLIC_STELLAR_NETWORK")?.toLowerCase();
  if (network === "mainnet" || network === "public") return "mainnet";
  if (network === "testnet") return "testnet";
  return "local";
}
