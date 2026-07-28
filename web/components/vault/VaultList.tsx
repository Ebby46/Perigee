import { useRef } from "react";

import { Vault } from "@/types/vault";

import { useVirtualVaultList } from "@/hooks/useVirtualVaultList";

import { VaultRow } from "./VaultRow";

interface VaultListProps {
  vaults: Vault[];

  onVaultClick?(vault: Vault): void;
}

export function VaultList({
  vaults,
  onVaultClick,
}: VaultListProps) {
  const parentRef =
    useRef<HTMLDivElement>(null);

  const { rowVirtualizer, containerHeight } =
    useVirtualVaultList(
      vaults.length,
      parentRef,
    );

  return (
    <div
      ref={parentRef}
      style={{
        height: containerHeight,
        overflow: "auto",
      }}
    >
      <div
        style={{
          height:
            rowVirtualizer.getTotalSize(),

          position: "relative",
        }}
      >
        {rowVirtualizer
          .getVirtualItems()
          .map((virtualRow) => {
            const vault =
              vaults[virtualRow.index];

            return (
              <div
                key={vault.id}
                style={{
                  position: "absolute",

                  left: 0,

                  top: 0,

                  width: "100%",

                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <VaultRow
                  vault={vault}
                  onClick={onVaultClick}
                />
              </div>
            );
          })}
      </div>
    </div>
  );
}