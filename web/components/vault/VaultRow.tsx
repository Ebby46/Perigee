import { memo } from "react";

import { Vault } from "@/types/vault";

interface VaultRowProps {
  vault: Vault;

  onClick?(vault: Vault): void;
}

function VaultRowComponent({
  vault,
  onClick,
}: VaultRowProps) {
  return (
    <div
      onClick={() => onClick?.(vault)}
      className="
      flex
      items-center
      justify-between
      border-b
      border-gray-200
      px-4
      py-4
      hover:bg-gray-50
      cursor-pointer
      transition-colors
      "
    >
      <div>
        <h3 className="font-medium">
          {vault.name}
        </h3>

        <p className="text-sm text-gray-500">
          {vault.owner}
        </p>
      </div>

      <div className="text-right">
        <p>{vault.balance}</p>

        <p className="text-xs text-gray-500">
          {vault.asset}
        </p>
      </div>

      <span
        className="
        rounded-full
        bg-green-100
        px-3
        py-1
        text-xs
        "
      >
        {vault.status}
      </span>
    </div>
  );
}

export const VaultRow = memo(VaultRowComponent);