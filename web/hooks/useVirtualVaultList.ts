import { RefObject } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import {
  DEFAULT_VIRTUAL_HEIGHT,
  VAULT_OVERSCAN,
  VAULT_ROW_HEIGHT,
} from "@/constants/vault.constants";

export function useVirtualVaultList(
  count: number,
  parentRef: RefObject<HTMLDivElement>,
) {
  const rowVirtualizer = useVirtualizer({
    count,

    getScrollElement: () => parentRef.current,

    estimateSize: () => VAULT_ROW_HEIGHT,

    overscan: VAULT_OVERSCAN,
  });

  return {
    rowVirtualizer,

    containerHeight: DEFAULT_VIRTUAL_HEIGHT,
  };
}