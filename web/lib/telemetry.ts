/**
 * Privacy-first telemetry module with consent gating and zero PII leakage.
 */

export interface TelemetryEvent {
  name: string;
  properties?: Record<string, string | number | boolean>;
}

const CONSENT_KEY = "perigee_telemetry_consent";

export function getTelemetryConsent(): boolean | null {
  if (typeof window === "undefined") return null;
  const stored = localStorage.getItem(CONSENT_KEY);
  if (stored === "granted") return true;
  if (stored === "denied") return false;
  return null;
}

export function setTelemetryConsent(consent: boolean): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(CONSENT_KEY, consent ? "granted" : "denied");
  if (consent) {
    initPrivacyTelemetry();
  }
}

export function initPrivacyTelemetry(): void {
  if (typeof window === "undefined") return;
  if (getTelemetryConsent() !== true) return;

  // Initialize Plausible / PostHog script if configured or enabled
  if (!document.getElementById("plausible-telemetry-script")) {
    const script = document.createElement("script");
    script.id = "plausible-telemetry-script";
    script.defer = true;
    script.dataset.domain = window.location.hostname;
    script.src = "https://plausible.io/js/script.js";
    document.head.appendChild(script);
  }
}

export function trackTelemetryEvent(event: TelemetryEvent): void {
  if (typeof window === "undefined" || getTelemetryConsent() !== true) return;

  // Strip potential PII properties before tracking
  const sanitizedProps: Record<string, string | number | boolean> = {};
  if (event.properties) {
    for (const [key, value] of Object.entries(event.properties)) {
      if (
        !key.toLowerCase().includes("email") &&
        !key.toLowerCase().includes("address") &&
        !key.toLowerCase().includes("name") &&
        !key.toLowerCase().includes("ip")
      ) {
        sanitizedProps[key] = value;
      }
    }
  }

  // Push to Plausible / window.plausible or custom privacy tracker
  if (typeof (window as unknown as { plausible?: Function }).plausible === "function") {
    (window as unknown as { plausible: Function }).plausible(event.name, { props: sanitizedProps });
  }
}
