import {
  DEFAULT_FLAG_VALUE,
} from "./constants";

import {
  featureFlags,
} from "./feature-flags";

import {
  FeatureFlag,
  FeatureFlagMap,
} from "./types";

class FeatureFlagService {
  private flags: FeatureFlagMap;

  constructor() {
    this.flags = featureFlags.reduce(
      (acc, flag) => {
        acc[flag.key] = flag.enabled;

        return acc;
      },
      {} as FeatureFlagMap,
    );
  }

  getAll(): FeatureFlagMap {
    return this.flags;
  }

  isEnabled(
    flag: FeatureFlag,
  ): boolean {
    return (
      this.flags[flag] ??
      DEFAULT_FLAG_VALUE
    );
  }

  enable(
    flag: FeatureFlag,
  ): void {
    this.flags[flag] = true;
  }

  disable(
    flag: FeatureFlag,
  ): void {
    this.flags[flag] = false;
  }

  toggle(
    flag: FeatureFlag,
  ): void {
    this.flags[flag] =
      !this.isEnabled(flag);
  }

  set(
    flag: FeatureFlag,
    enabled: boolean,
  ) {
    this.flags[flag] = enabled;
  }

  reset() {
    this.flags = featureFlags.reduce(
      (acc, flag) => {
        acc[flag.key] = flag.enabled;

        return acc;
      },
      {} as FeatureFlagMap,
    );
  }
}

export const featureFlagService =
  new FeatureFlagService();