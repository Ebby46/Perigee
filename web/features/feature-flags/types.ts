import { FEATURE_FLAGS } from "./constants";

export type FeatureFlag =
  (typeof FEATURE_FLAGS)[keyof typeof FEATURE_FLAGS];

export interface FeatureFlagDefinition {
  key: FeatureFlag;

  enabled: boolean;

  description?: string;
}

export type FeatureFlagMap = Record<
  FeatureFlag,
  boolean
>;

export interface FeatureFlagApiSource {
  url: string;
  headers?: Record<string, string>;
  /** Interval in ms to re-fetch flags. Defaults to 60 000. */
  pollingInterval?: number;
}

export interface FeatureFlagProviderConfig {
  /** API endpoint to fetch remote flags from. */
  apiSource?: FeatureFlagApiSource;
  /** Additional env-var prefix to look for (default: "NEXT_PUBLIC_FEATURE_FLAG_"). */
  envPrefix?: string;
}