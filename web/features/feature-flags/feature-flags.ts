import {
  FEATURE_FLAGS,
} from "./constants";

import {
  FeatureFlagDefinition,
} from "./types";

export const featureFlags: FeatureFlagDefinition[] = [
  {
    key: FEATURE_FLAGS.NEW_VAULT_UI,
    enabled: false,
    description:
      "Enable the redesigned vault experience.",
  },

  {
    key: FEATURE_FLAGS.DASHBOARD_V2,
    enabled: false,
    description:
      "Enable the next-generation dashboard.",
  },

  {
    key: FEATURE_FLAGS.EXPERIMENTAL_CHARTS,
    enabled: true,
    description:
      "Enable experimental analytics charts.",
  },

  {
    key: FEATURE_FLAGS.NOTIFICATIONS_V2,
    enabled: false,
    description:
      "Enable the new notifications experience.",
  },
];