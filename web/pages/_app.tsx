import type { AppProps } from "next/app";
import "../styles/globals.css";
import { WalletProvider } from "../context/WalletContext";
import { ErrorBoundary } from "../components/ErrorBoundary";
import { Analytics } from "../components/Analytics";
import { RpcFallbackBanner } from "../components/RpcFallbackBanner";
import { API_URL } from "../lib/api";

export default function App({ Component, pageProps }: AppProps) {
  return (
    <WalletProvider>
      {/* Graceful RPC fallback — shown when the backend is unreachable (#115) */}
      <RpcFallbackBanner apiUrl={API_URL} />
      <ErrorBoundary>
        <Component {...pageProps} />
        <Analytics />
      </ErrorBoundary>
    </WalletProvider>
  );
}
