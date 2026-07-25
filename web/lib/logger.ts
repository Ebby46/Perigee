const STELLAR_PUBLIC_KEY_PREFIX = "G";
const STELLAR_SECRET_KEY_PREFIX = "S";
const STELLAR_CONTRACT_ID_PREFIX = "C";
const STELLAR_ADDRESS_LENGTH = 56;

const SENSITIVE_KEYS = [
  "secret",
  "secretKey",
  "privateKey",
  "private",
  "seed",
  "mnemonic",
  "password",
  "token",
  "auth",
  "authorization",
];

function isStellarPublicKey(str: string): boolean {
  return (
    typeof str === "string" &&
    str.startsWith(STELLAR_PUBLIC_KEY_PREFIX) &&
    str.length === STELLAR_ADDRESS_LENGTH &&
    /^[A-Z0-9]+$/.test(str)
  );
}

function isStellarSecretKey(str: string): boolean {
  return (
    typeof str === "string" &&
    str.startsWith(STELLAR_SECRET_KEY_PREFIX) &&
    str.length === STELLAR_ADDRESS_LENGTH &&
    /^[A-Z0-9]+$/.test(str)
  );
}

function isStellarContractId(str: string): boolean {
  return (
    typeof str === "string" &&
    str.startsWith(STELLAR_CONTRACT_ID_PREFIX) &&
    str.length === STELLAR_ADDRESS_LENGTH &&
    /^[A-Z0-9]+$/.test(str)
  );
}

function isSensitiveKey(key: string): boolean {
  const lowerKey = key.toLowerCase();
  return SENSITIVE_KEYS.some((sensitive) => lowerKey.includes(sensitive));
}

function sanitizeValue(value: unknown): unknown {
  if (value === null || value === undefined) {
    return value;
  }

  if (typeof value === "string") {
    if (isStellarPublicKey(value) || isStellarContractId(value)) {
      return `${value.slice(0, 4)}...${value.slice(-4)}`;
    }
    if (isStellarSecretKey(value)) {
      return "[REDACTED_SECRET_KEY]";
    }
    return value;
  }

  if (typeof value === "object") {
    if (Array.isArray(value)) {
      return value.map(sanitizeValue);
    }

    const sanitized: Record<string, unknown> = {};
    for (const [key, val] of Object.entries(value)) {
      if (isSensitiveKey(key)) {
        sanitized[key] = "[REDACTED]";
      } else {
        sanitized[key] = sanitizeValue(val);
      }
    }
    return sanitized;
  }

  return value;
}

function sanitizeArgs(args: unknown[]): unknown[] {
  return args.map(sanitizeValue);
}

function formatArgs(args: unknown[]): string {
  return args
    .map((arg) => {
      if (typeof arg === "object" && arg !== null) {
        try {
          return JSON.stringify(arg);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    })
    .join(" ");
}

export const logger = {
  log: (...args: unknown[]): void => {
    console.log(...sanitizeArgs(args));
  },

  info: (...args: unknown[]): void => {
    console.info(...sanitizeArgs(args));
  },

  warn: (...args: unknown[]): void => {
    console.warn(...sanitizeArgs(args));
  },

  error: (...args: unknown[]): void => {
    console.error(...sanitizeArgs(args));
  },

  debug: (...args: unknown[]): void => {
    if (process.env.NODE_ENV === "development") {
      console.debug(...sanitizeArgs(args));
    }
  },
};

export function sanitizeForLog(value: unknown): unknown {
  return sanitizeValue(value);
}