"use client";

import { useEffect, useState } from "react";
import { getTelemetryConsent, setTelemetryConsent, initPrivacyTelemetry } from "../lib/telemetry";

export function Analytics() {
  const [showBanner, setShowBanner] = useState(false);

  useEffect(() => {
    const consent = getTelemetryConsent();
    if (consent === null) {
      setShowBanner(true);
    } else if (consent === true) {
      initPrivacyTelemetry();
    }
  }, []);

  function handleAccept() {
    setTelemetryConsent(true);
    setShowBanner(false);
  }

  function handleDecline() {
    setTelemetryConsent(false);
    setShowBanner(false);
  }

  if (!showBanner) return null;

  return (
    <aside aria-label="Privacy settings" className="fixed bottom-4 right-4 z-50 max-w-sm rounded-xl border border-slate-800 bg-slate-900/95 p-4 shadow-xl text-slate-200 text-xs backdrop-blur">
      <p className="font-semibold text-slate-100 mb-1">Privacy-First Analytics</p>
      <p className="text-slate-400 mb-3">
        We use anonymous telemetry without PII to improve Perigee. Do you consent?
      </p>
      <div className="flex gap-2 justify-end">
        <button
          onClick={handleDecline}
          className="rounded px-3 py-1.5 border border-slate-700 hover:bg-slate-800 text-slate-300 transition-colors"
        >
          Decline
        </button>
        <button
          onClick={handleAccept}
          className="rounded px-3 py-1.5 bg-cyan-600 hover:bg-cyan-500 font-medium text-white transition-colors"
        >
          Accept
        </button>
      </div>
    </aside>
  );
}
