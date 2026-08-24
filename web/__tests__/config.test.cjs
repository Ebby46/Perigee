// config.test.cjs — unit tests for web/lib/config.ts (WEB-53 / #186)
// Contact/support link centralization: env overrides, defaults,
// mailto building, and a source guard against future hardcoding.
//
// Runs with: node --test ./__tests__/config.test.cjs

'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

// ── Pure helpers (mirror web/lib/config.ts logic) ────────────────────────────

function resolveLink(override, fallback) {
  return override ?? fallback;
}

function trimOverride(value) {
  const trimmed = typeof value === 'string' ? value.trim() : '';
  return trimmed || undefined;
}

function supportMailto(email, subject, body) {
  const params = [];
  if (subject) params.push(`subject=${encodeURIComponent(subject)}`);
  if (body) params.push(`body=${encodeURIComponent(body)}`);
  const query = params.length > 0 ? `?${params.join('&')}` : '';
  return `mailto:${email}${query}`;
}

const DEFAULTS = {
  supportUrl: 'https://perigee.app/support',
  contactEmail: 'support@perigee.app',
  docsUrl: 'https://perigee.app/docs',
  statusUrl: 'https://status.perigee.app',
};

// ── resolveLink ───────────────────────────────────────────────────────────────

test('resolveLink: falls back to default when override is undefined', () => {
  assert.equal(resolveLink(undefined, DEFAULTS.supportUrl), DEFAULTS.supportUrl);
});

test('resolveLink: prefers manager-configured override', () => {
  const custom = 'https://help.acme.example';
  assert.equal(resolveLink(custom, DEFAULTS.supportUrl), custom);
});

// ── blank env vars must not produce dead links ────────────────────────────────

test('trimOverride: whitespace-only env value resolves to undefined', () => {
  assert.equal(trimOverride('   '), undefined);
});

test('trimOverride: trims surrounding whitespace from valid values', () => {
  assert.equal(trimOverride('  https://help.acme.example '), 'https://help.acme.example');
});

test('blank override combined with fallback resolution yields the default', () => {
  const raw = process.env.NEXT_PUBLIC_SUPPORT_URL;
  delete process.env.NEXT_PUBLIC_SUPPORT_URL;
  try {
    process.env.NEXT_PUBLIC_SUPPORT_URL = '   ';
    const override = trimOverride(process.env.NEXT_PUBLIC_SUPPORT_URL);
    assert.equal(resolveLink(override, DEFAULTS.docsUrl), DEFAULTS.docsUrl);
  } finally {
    if (raw === undefined) delete process.env.NEXT_PUBLIC_SUPPORT_URL;
    else process.env.NEXT_PUBLIC_SUPPORT_URL = raw;
  }
});

// ── supportMailto ─────────────────────────────────────────────────────────────

test('supportMailto: plain address without params', () => {
  assert.equal(supportMailto(DEFAULTS.contactEmail), 'mailto:support@perigee.app');
});

test('supportMailto: encodes subject', () => {
  assert.equal(
    supportMailto(DEFAULTS.contactEmail, 'Simulation failed'),
    'mailto:support@perigee.app?subject=Simulation%20failed',
  );
});

test('supportMailto: encodes subject and body', () => {
  assert.equal(
    supportMailto(DEFAULTS.contactEmail, 'Error (NETWORK_ERROR)', 'line1\nline2'),
    'mailto:support@perigee.app?subject=Error%20(NETWORK_ERROR)&body=line1%0Aline2',
  );
});

// ── source guard: UI must not hardcode contact/support links ─────────────────

const WEB_ROOT = path.join(__dirname, '..');
// Contact/support links must come from lib/config. The bare perigee.app
// domain is allowed (e.g. SEO canonical origins); support endpoints,
// e-mail links, and the status host are not.
const HARDCODED_LINK_PATTERN = /(mailto:|support@|perigee\.app\/(support|docs)|status\.perigee\.app)/i;

function listFiles(dir, exts, acc = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules') continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) listFiles(full, exts, acc);
    else if (exts.some((e) => entry.name.endsWith(e))) acc.push(full);
    }
  return acc;
}

test('UI components import links from lib/config instead of hardcoding them', () => {
  const offenders = [];
  for (const file of listFiles(path.join(WEB_ROOT, 'components'), ['.tsx'])) {
    const src = fs.readFileSync(file, 'utf8');
    if (HARDCODED_LINK_PATTERN.test(src)) offenders.push(path.relative(WEB_ROOT, file));
  }
  assert.deepEqual(offenders, []);
});

test('lib/config.ts is the single place declaring the default links', () => {
  const src = fs.readFileSync(path.join(WEB_ROOT, 'lib', 'config.ts'), 'utf8');
  for (const url of Object.values(DEFAULTS)) {
    assert.ok(src.includes(url), `config.ts should declare ${url}`);
  }
});

// ── i18n keys used by the wired components exist ──────────────────────────────

test('en.json defines the support-link message keys', () => {
  const messages = JSON.parse(fs.readFileSync(path.join(WEB_ROOT, 'messages', 'en.json'), 'utf8'));
  for (const key of [
    'result.contactSupport',
    'result.viewDocs',
    'network.getHelp',
    'rpc.getHelp',
    'rpc.statusPage',
  ]) {
    assert.ok(messages[key], `missing message key: ${key}`);
  }
});
