"use client";

import React, { useEffect, useState, useCallback } from "react";
import { useTranslations } from "next-intl";
import { supportLinks } from "../lib/config";

interface RpcHealth {
  healthy: boolean;
  checkedAt: number;
}

const RPC_CHECK_INTERVAL_MS = 30_000; // re-check every 30 s
const RPC_TIMEOUT_MS = 5_000;

/**
 * Checks whether the configured backend API is reachable.
 * Uses a lightweight HEAD request to avoid transferring a body.
 */
async function checkRpcHealth(apiUrl: string): Promise<boolean> {
  try {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), RPC_TIMEOUT_MS);
    const res = await fetch(`${apiUrl}/health`, {
      method: "HEAD",
      signal: controller.signal,
      cache: "no-store",
    });
    clearTimeout(timer);
    return res.ok;
  } catch {
    return false;
  }
}

/**
 * RpcFallbackBanner
 *
 * Displays a non-intrusive banner at the top of the page when the Perigee
 * backend RPC endpoint is unreachable.  The banner:
 *  - Auto-dismisses when connectivity is restored.
 *  - Provides a manual "Retry" button.
 *  - Does not block the UI — users can still browse cached data.
 *
 * Resolves WEB-29 (#115): no graceful RPC fallback.
 *
 * Usage: render once in _app.tsx above <Component />.
 */
export function RpcFallbackBanner({ apiUrl }: { apiUrl: string }) {
  const t = useTranslations();
  const [health, setHealth] = useState<RpcHealth>({ healthy: true, checkedAt: 0 });
  const [checking, setChecking] = useState(false);

  const runCheck = useCallback(async () => {
    setChecking(true);
    const ok = await checkRpcHealth(apiUrl);
    setHealth({ healthy: ok, checkedAt: Date.now() });
    setChecking(false);
  }, [apiUrl]);

  // Initial check + periodic polling
  useEffect(() => {
    runCheck();
    const interval = setInterval(runCheck, RPC_CHECK_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [runCheck]);

  if (health.healthy) return null;

  return (
    <div
      role="alert"
      aria-live="assertive"
      className="flex items-center justify-between gap-3 bg-amber-900/80 border-b border-amber-700 px-4 py-2.5 text-sm text-amber-100"
    >
      <div className="flex items-center gap-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
          className="h-4 w-4 shrink-0 text-amber-300"
          aria-hidden="true"
        >
          <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <span>
          <strong>{t("rpc.unreachableTitle")}</strong>{" "}
          {t("rpc.unreachableBody")}{" "}
          <a
            href={supportLinks.supportUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="underline underline-offset-2 hover:opacity-100 opacity-90"
          >
            {t("rpc.getHelp")}
          </a>
        </span>
      </div>

      <div className="flex shrink-0 items-center gap-2">
        <a
          href={supportLinks.statusUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="rounded border border-amber-600 px-2.5 py-1 text-xs font-medium
                     text-amber-200 hover:bg-amber-800 focus:outline-none focus:ring-2
                     focus:ring-amber-400 disabled:opacity-50 transition-colors"
        >
          {t("rpc.statusPage")}
        </a>
        <button
          type="button"
          onClick={runCheck}
          disabled={checking}
          aria-label="Retry connecting to backend"
          className="rounded border border-amber-600 px-2.5 py-1 text-xs font-medium
                     text-amber-200 hover:bg-amber-800 focus:outline-none focus:ring-2
                     focus:ring-amber-400 focus:ring-offset-2 focus:ring-offset-amber-900
                     disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {checking ? t("rpc.checking") : t("rpc.retry")}
        </button>
      </div>
    </div>
  );
}
