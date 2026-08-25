// DynamicForm.test.cjs — unit tests for DynamicForm validation helpers
// Closes: frontend/validation/stellar issue
//
// Tests cover validateStellarAddress, validateAssetCode, and validateField.
// Pure-function helpers are copied here rather than imported (CJS/ESM boundary).
//
// Run with:
//   node --test ./components/DynamicForm.test.cjs

'use strict';

const { test, describe } = require('node:test');
const assert = require('node:assert/strict');

// ---------------------------------------------------------------------------
// Validation helpers — copied from DynamicForm.tsx for CJS compatibility
// ---------------------------------------------------------------------------

const STRKEY_RE = /^[A-Z2-7]{56}$/;

/**
 * Validates a Stellar address (Ed25519 public key or contract address).
 * @param {string} value
 * @returns {string|null}
 */
function validateStellarAddress(value) {
  const trimmed = value.trim();
  if (!trimmed) return null;

  if (trimmed.startsWith('G') || trimmed.startsWith('C')) {
    if (!STRKEY_RE.test(trimmed)) {
      return `Must be 56 base32 characters starting with ${trimmed[0]}.`;
    }
    return null;
  }

  return "Stellar address must start with 'G' (Ed25519 public key) or 'C' (contract).";
}

/**
 * Validates a Stellar asset code (alphanumeric, 1–12 chars).
 * @param {string} value
 * @returns {string|null}
 */
function validateAssetCode(value) {
  const trimmed = value.trim();
  if (!trimmed) return null;

  if (!/^[A-Za-z0-9]{1,12}$/.test(trimmed)) {
    if (trimmed.length > 12) {
      return 'Asset code must be 1–12 characters.';
    }
    return 'Asset code must contain only letters and numbers (A-Z, 0-9).';
  }
  return null;
}

/**
 * Dispatches to the right validator based on SorobanType.
 * @param {string} type
 * @param {string} value
 * @returns {string|null}
 */
function validateField(type, value) {
  switch (type) {
    case 'address':
      return validateStellarAddress(value);
    case 'asset_code':
      return validateAssetCode(value);
    default:
      return null;
  }
}

/**
 * Sanitizes user-controlled strings before they are sent to the API.
 * Strips HTML tags and event handler attributes while preserving benign text.
 * @param {string} value
 * @param {string} fieldName
 * @returns {string}
 */
function sanitizeUserInput(value, fieldName = 'input') {
  const next = String(value ?? '').replace(/<[^>]*>/g, '');
  if (next !== String(value ?? '')) {
    console.warn(`[security] Sanitized ${fieldName}: removed unsafe HTML/JS content.`);
  }
  return next;
}

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

// A valid 56-char Ed25519 Stellar public key (G...) — uses only A-Z and 2-7
const VALID_G_ADDRESS = 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';

// A valid 56-char Soroban contract address (C...) — uses only A-Z and 2-7
const VALID_C_ADDRESS = 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';

// ---------------------------------------------------------------------------
// validateStellarAddress
// ---------------------------------------------------------------------------

describe('validateStellarAddress', () => {
  test('returns null for empty string (required check delegated to HTML)', () => {
    assert.equal(validateStellarAddress(''), null);
  });

  test('returns null for a valid G... address (56 chars)', () => {
    assert.equal(validateStellarAddress(VALID_G_ADDRESS), null);
  });

  test('returns null for a valid C... contract address (56 chars)', () => {
    assert.equal(validateStellarAddress(VALID_C_ADDRESS), null);
  });

  test('trims leading/trailing whitespace before validating', () => {
    assert.equal(validateStellarAddress('  ' + VALID_G_ADDRESS + '  '), null);
  });

  test('rejects G... address that is too short (55 chars)', () => {
    const short = VALID_G_ADDRESS.slice(0, 55);
    const err = validateStellarAddress(short);
    assert.ok(err !== null, 'expected an error for short address');
    assert.match(err, /56 base32 characters/);
  });

  test('rejects G... address that is too long (57 chars)', () => {
    const long = VALID_G_ADDRESS + 'A';
    const err = validateStellarAddress(long);
    assert.ok(err !== null, 'expected an error for long address');
    assert.match(err, /56 base32 characters/);
  });

  test('rejects G... address containing lowercase characters', () => {
    const lower = VALID_G_ADDRESS.toLowerCase();
    const err = validateStellarAddress(lower);
    assert.ok(err !== null, 'expected an error for lowercase address');
  });

  test('rejects address starting with a wrong prefix (X...)', () => {
    const wrong = 'X' + VALID_G_ADDRESS.slice(1);
    const err = validateStellarAddress(wrong);
    assert.ok(err !== null, 'expected an error for wrong prefix');
    assert.match(err, /must start with 'G'/i);
  });

  test("rejects address starting with 'S' (secret key format)", () => {
    const secret = 'S' + VALID_G_ADDRESS.slice(1);
    const err = validateStellarAddress(secret);
    assert.ok(err !== null, 'expected an error for S... address');
    assert.match(err, /must start with 'G'/i);
  });

  test('rejects completely random string', () => {
    const err = validateStellarAddress('not-an-address');
    assert.ok(err !== null);
  });

  test('rejects address with invalid base32 characters (1, 0, 6, 7, 9)', () => {
    // Replace first char after G with '1', which is not in Stellar's base32 alphabet
    const invalid = 'G1' + VALID_G_ADDRESS.slice(2);
    const err = validateStellarAddress(invalid);
    assert.ok(err !== null, 'expected an error for invalid base32 char');
  });
});

// ---------------------------------------------------------------------------
// validateAssetCode
// ---------------------------------------------------------------------------

describe('validateAssetCode', () => {
  test('returns null for empty string (required check delegated to HTML)', () => {
    assert.equal(validateAssetCode(''), null);
  });

  test('returns null for a valid 3-char code', () => {
    assert.equal(validateAssetCode('XLM'), null);
  });

  test('returns null for a valid 4-char code', () => {
    assert.equal(validateAssetCode('USDC'), null);
  });

  test('returns null for a single character code', () => {
    assert.equal(validateAssetCode('A'), null);
  });

  test('returns null for a 12-char code (max length)', () => {
    assert.equal(validateAssetCode('ABCDEFGHIJKL'), null);
  });

  test('returns null for numeric-only code', () => {
    assert.equal(validateAssetCode('1234'), null);
  });

  test('returns null for mixed alphanumeric code', () => {
    assert.equal(validateAssetCode('BTC2025'), null);
  });

  test('trims whitespace before validating', () => {
    assert.equal(validateAssetCode('  USDC  '), null);
  });

  test('rejects code longer than 12 characters', () => {
    const err = validateAssetCode('TOOLONGASSET1');
    assert.ok(err !== null, 'expected an error for >12 char code');
    assert.match(err, /1.12 characters/);
  });

  test('rejects code with special characters (hyphen)', () => {
    const err = validateAssetCode('USDC-2025');
    assert.ok(err !== null, 'expected an error for hyphenated code');
    assert.match(err, /letters and numbers/i);
  });

  test('rejects code with spaces', () => {
    const err = validateAssetCode('MY TOKEN');
    assert.ok(err !== null, 'expected an error for code with space');
  });

  test('rejects code with underscore', () => {
    const err = validateAssetCode('MY_TOKEN');
    assert.ok(err !== null);
    assert.match(err, /letters and numbers/i);
  });
});

// ---------------------------------------------------------------------------
// validateField (dispatcher)
// ---------------------------------------------------------------------------

describe('validateField', () => {
  test("delegates 'address' type to validateStellarAddress", () => {
    // Invalid address should return an error
    const err = validateField('address', 'bad');
    assert.ok(err !== null);
    assert.match(err, /must start with 'G'/i);
  });

  test("delegates 'asset_code' type to validateAssetCode", () => {
    const err = validateField('asset_code', 'TOO-LONG-AND-INVALID!');
    assert.ok(err !== null);
  });

  test("returns null for 'address' type with valid address", () => {
    assert.equal(validateField('address', VALID_G_ADDRESS), null);
  });

  test("returns null for 'asset_code' type with valid code", () => {
    assert.equal(validateField('asset_code', 'USDC'), null);
  });

  test("returns null for unknown types (e.g., 'u32')", () => {
    assert.equal(validateField('u32', '99999'), null);
  });

  test("returns null for 'string' type — no validation applied", () => {
    assert.equal(validateField('string', 'any value goes here!'), null);
  });

  test("returns null for 'bool' type", () => {
    assert.equal(validateField('bool', 'true'), null);
  });

  test("returns null for 'symbol' type", () => {
    assert.equal(validateField('symbol', 'sym'), null);
  });

  test('returns null for empty value on any validated type (required handled by HTML)', () => {
    assert.equal(validateField('address', ''), null);
    assert.equal(validateField('asset_code', ''), null);
  });
});

// ---------------------------------------------------------------------------
// sanitizeUserInput (security regression)
// ---------------------------------------------------------------------------

describe('sanitizeUserInput', () => {
  test('strips XSS payloads before API submission', () => {
    const dirty = 'hello<img src=x onerror=alert(1)>world';
    const sanitized = sanitizeUserInput(dirty, 'amount');

    assert.equal(sanitized, 'helloworld');
    assert.doesNotMatch(sanitized, /<img|onerror|alert\(/i);
  });
});
