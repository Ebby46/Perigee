import {
  DEFAULT_FLAG_VALUE,
  DEFAULT_POLLING_INTERVAL_MS,
  ENV_PREFIX,
} from "./constants";

import { featureFlags } from "./feature-flags";

import type {
  FeatureFlag,
  FeatureFlagApiSource,
  FeatureFlagMap,
  FeatureFlagProviderConfig,
} from "./types";

function readEnvFlags(
  prefix: string,
): Partial<FeatureFlagMap> {
  if (typeof process === "undefined") return {};

  const result: Partial<FeatureFlagMap> = {};
  const allFlags = Object.values(
    featureFlags.reduce(
      (acc, f) => {
        acc[f.key] = f.key;
        return acc;
      },
      {} as Record<string, string>,
    ),
  );

  for (const key of allFlags) {
    const envKey = `${prefix}${key}`;
    const raw = process.env[envKey];
    if (raw !== undefined) {
      result[key as FeatureFlag] = raw === "true";
    }
  }

  return result;
}

class FeatureFlagService {
  private flags: FeatureFlagMap;
  private pollTimer: ReturnType<typeof setInterval> | null = null;

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
    return { ...this.flags };
  }

  isEnabled(flag: FeatureFlag): boolean {
    return this.flags[flag] ?? DEFAULT_FLAG_VALUE;
  }

  enable(flag: FeatureFlag): void {
    this.flags[flag] = true;
  }

  disable(flag: FeatureFlag): void {
    this.flags[flag] = false;
  }

  toggle(flag: FeatureFlag): void {
    this.flags[flag] = !this.isEnabled(flag);
  }

  set(flag: FeatureFlag, enabled: boolean): void {
    this.flags[flag] = enabled;
  }

  reset(): void {
    this.flags = featureFlags.reduce(
      (acc, flag) => {
        acc[flag.key] = flag.enabled;
        return acc;
      },
      {} as FeatureFlagMap,
    );
  }

  applyEnvOverrides(prefix: string = ENV_PREFIX): void {
    const envFlags = readEnvFlags(prefix);
    for (const [key, value] of Object.entries(envFlags)) {
      this.flags[key as FeatureFlag] = value as boolean;
    }
  }

  async fetchFromApi(
    source: FeatureFlagApiSource,
  ): Promise<void> {
    const controller = new AbortController();
    const timeout = setTimeout(
      () => controller.abort(),
      5_000,
    );

    try {
      const res = await fetch(source.url, {
        headers: source.headers ?? {},
        signal: controller.signal,
      });

      if (!res.ok) return;

      const data: unknown = await res.json();

      if (
        data &&
        typeof data === "object" &&
        !Array.isArray(data)
      ) {
        for (const [key, value] of Object.entries(
          data as Record<string, unknown>,
        )) {
          if (
            key in this.flags &&
            typeof value === "boolean"
          ) {
            this.flags[key as FeatureFlag] = value;
          }
        }
      }
    } catch {
      // Silently ignore — flags retain their previous state
    } finally {
      clearTimeout(timeout);
    }
  }

  startPolling(source: FeatureFlagApiSource): void {
    this.stopPolling();
    const interval =
      source.pollingInterval ??
      DEFAULT_POLLING_INTERVAL_MS;

    void this.fetchFromApi(source);
    this.pollTimer = setInterval(() => {
      void this.fetchFromApi(source);
    }, interval);
  }

  stopPolling(): void {
    if (this.pollTimer !== null) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }

  async initialize(
    config?: FeatureFlagProviderConfig,
  ): Promise<void> {
    this.reset();

    this.applyEnvOverrides(config?.envPrefix);

    if (config?.apiSource) {
      await this.fetchFromApi(config.apiSource);
      this.startPolling(config.apiSource);
    }
  }
}

export const featureFlagService =
  new FeatureFlagService();
