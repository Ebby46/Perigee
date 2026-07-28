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