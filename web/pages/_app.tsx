import type { AppProps } from "next/app";
import "../styles/globals.css";
import { WalletProvider } from "../context/WalletContext";
import { ErrorBoundary } from "../components/ErrorBoundary";
import { Analytics } from "../components/Analytics";
import { NetworkStatusBanner } from "../components/NetworkStatusBanner";
import { API_URL } from "../lib/api";
import { useEffect } from "react";

/**
 * Initialize @axe-core/react in development so accessibility violations
 * are reported to the browser console during local development.
 *
 * The import is dynamic so axe-core is never bundled into production builds.
 * Resolves WEB-24 (#110): missing accessibility audit tooling.
 */
async function initAxe() {
  if (
    typeof window !== "undefined" &&
    process.env.NODE_ENV === "development"
  ) {
    const axe = await import("@axe-core/react");
    const React = await import("react");
    const ReactDOM = await import("react-dom");
    // 1000 ms debounce — avoids flooding the console on rapid re-renders
    await axe.default(React.default, ReactDOM.default, 1000);
  }
}

export default function App({ Component, pageProps }: AppProps) {
  useEffect(() => {
    initAxe().catch(() => {
      // axe-core is a dev-only optional dependency; swallow import errors
      // in environments where it is not installed.
    });
  }, []);

  return (
    <WalletProvider>
      {/* Network status and API availability (#109) */}
      <NetworkStatusBanner apiUrl={API_URL} />
      <ErrorBoundary>
        <Component {...pageProps} />
        <Analytics />
      </ErrorBoundary>
    </WalletProvider>
  );
}
