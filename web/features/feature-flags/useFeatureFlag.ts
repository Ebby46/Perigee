import { useCallback, useMemo } from "react";
import { useFeatureFlagContext } from "./FeatureFlagContext";
import type { FeatureFlag } from "./types";

export function useFeatureFlag(flag: FeatureFlag): boolean {
  const { isEnabled } = useFeatureFlagContext();
  return isEnabled(flag);
}

export function useFeatureFlags(): {
  flags: Record<string, boolean>;
  isEnabled: (flag: FeatureFlag) => boolean;
  refresh: () => Promise<void>;
} {
  const { flags, isEnabled, refresh } =
    useFeatureFlagContext();

  const stableIsEnabled = useCallback(
    (f: FeatureFlag) => isEnabled(f),
    [isEnabled],
  );

  const memoisedFlags = useMemo(
    () => ({ ...flags }),
    [flags],
  );

  return {
    flags: memoisedFlags,
    isEnabled: stableIsEnabled,
    refresh,
  };
}
