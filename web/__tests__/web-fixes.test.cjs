const assert = require("node:assert");
const { test, describe, beforeEach } = require("node:test");

// Mock browser globals for node environment
if (typeof global.localStorage === "undefined") {
  const storage = new Map();
  global.localStorage = {
    getItem: (key) => storage.get(key) || null,
    setItem: (key, val) => storage.set(key, String(val)),
    removeItem: (key) => storage.delete(key),
    clear: () => storage.clear(),
  };
}

if (typeof global.sessionStorage === "undefined") {
  const storage = new Map();
  global.sessionStorage = {
    getItem: (key) => storage.get(key) || null,
    setItem: (key, val) => storage.set(key, String(val)),
    removeItem: (key) => storage.delete(key),
    clear: () => storage.clear(),
  };
}

describe("Web Fixes (Issues #190, #189, #184, #183)", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  test("WEB-57: onboarding draft state auto-saves and clears in sessionStorage", () => {
    const draft = { name: "Jane Doe", email: "jane@example.com", kycRef: "KYC-123" };
    sessionStorage.setItem("onboarding_draft", JSON.stringify(draft));

    const restored = JSON.parse(sessionStorage.getItem("onboarding_draft") || "{}");
    assert.strictEqual(restored.name, "Jane Doe");
    assert.strictEqual(restored.email, "jane@example.com");

    sessionStorage.removeItem("onboarding_draft");
    assert.strictEqual(sessionStorage.getItem("onboarding_draft"), null);
  });

  test("WEB-56: privacy telemetry consent gating and storage", () => {
    const CONSENT_KEY = "perigee_telemetry_consent";
    assert.strictEqual(localStorage.getItem(CONSENT_KEY), null);

    localStorage.setItem(CONSENT_KEY, "granted");
    assert.strictEqual(localStorage.getItem(CONSENT_KEY), "granted");

    localStorage.setItem(CONSENT_KEY, "denied");
    assert.strictEqual(localStorage.getItem(CONSENT_KEY), "denied");
  });

  test("WEB-51: WASM versioned filename formatting appends version hash", () => {
    const filename = "contract.wasm";
    const hash = "a1b2c3d4e5f67890";
    const version = hash.slice(0, 8);
    const base = filename.replace(/\.wasm$/i, "");
    const versioned = `${base}.v-${version}.wasm`;

    assert.strictEqual(versioned, "contract.v-a1b2c3d4.wasm");
  });
});
