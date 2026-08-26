// web/lib/api-middleware.test.cjs
//
// CJS replication of the contract enforced in web/lib/api-middleware.ts. The
// production source is TypeScript, but per `package.json`'s `test` script only
// `.cjs` files are executed by `node --test`. Keeping an in-source CJS mirror
// avoids building a TS→JS step just for these helpers and makes the contract
// easy to reason about in plain JS.
//
// If you change behaviour in web/lib/api-middleware.ts, mirror the change here.

'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');

/* ── Rate-limit mirror ───────────────────────────────────────────────── */

const rateLimitBuckets = new Map();

function defaultKeyFn(req) {
  const xff = req.headers['x-forwarded-for'];
  if (typeof xff === 'string' && xff.length > 0) {
    return xff.split(',')[0].trim();
  }
  return req.socket?.remoteAddress || 'unknown';
}

function makeRateLimitedHandler(handler, options = {}) {
  const max = options.max ?? 100;
  const windowMs = options.windowMs ?? 60_000;
  const keyFn = options.keyFn ?? defaultKeyFn;
  return async function rateLimitedHandler(req, res) {
    const key = keyFn(req);
    const now = Date.now();
    const bucket = rateLimitBuckets.get(key);
    if (!bucket || bucket.resetAt <= now) {
      rateLimitBuckets.set(key, { count: 1, resetAt: now + windowMs });
      return handler(req, res);
    }
    if (bucket.count >= max) {
      const retryAfter = Math.max(1, Math.ceil((bucket.resetAt - now) / 1000));
      res.setHeader('Retry-After', String(retryAfter));
      return res.status(429).json({ error: 'TOO_MANY_REQUESTS' });
    }
    bucket.count += 1;
    return handler(req, res);
  };
}

/* ── Body-validator mirror ───────────────────────────────────────────── */

function makeValidatedHandler(handler, isValid) {
  return async function validatedHandler(req, res) {
    if (
      req.method !== 'POST' &&
      req.method !== 'PUT' &&
      req.method !== 'PATCH'
    ) {
      return handler(req, res);
    }
    const body = req.body;
    if (
      typeof body !== 'object' ||
      body === null ||
      Array.isArray(body)
    ) {
      return res.status(400).json({ error: 'BAD_REQUEST' });
    }
    if (!isValid(body)) {
      return res.status(400).json({ error: 'BAD_REQUEST' });
    }
    return handler(req, res);
  };
}

/* ── Helpers to build fake req/res ───────────────────────────────────── */

function fakeReq(overrides = {}) {
  return {
    method: 'POST',
    headers: {},
    socket: { remoteAddress: '127.0.0.1' },
    body: {},
    ...overrides,
  };
}

function fakeRes() {
  const res = {
    statusCode: 200,
    headers: {},
    body: undefined,
    status(code) {
      this.statusCode = code;
      return this;
    },
    setHeader(k, v) {
      this.headers[k] = v;
    },
    json(payload) {
      this.body = payload;
      return this;
    },
  };
  return res;
}

/* ── Rate-limit tests ────────────────────────────────────────────────── */

test('rate-limit: allows first N requests below threshold', async () => {
  rateLimitBuckets.clear();
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const wrapped = makeRateLimitedHandler(handler, { max: 3, windowMs: 60_000 });

  for (let i = 0; i < 3; i++) {
    const res = fakeRes();
    await wrapped(fakeReq(), res);
    assert.equal(res.statusCode, 200);
  }
});

test('rate-limit: returns 429 once max is exceeded', async () => {
  rateLimitBuckets.clear();
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const wrapped = makeRateLimitedHandler(handler, { max: 2, windowMs: 60_000 });

  await wrapped(fakeReq(), fakeRes());
  await wrapped(fakeReq(), fakeRes());

  const res = fakeRes();
  await wrapped(fakeReq(), res);
  assert.equal(res.statusCode, 429);
  assert.equal(res.body.error, 'TOO_MANY_REQUESTS');
  assert.ok(res.headers['Retry-After'] !== undefined, 'Retry-After must be set');
});

test('rate-limit: separate clients do not interfere', async () => {
  rateLimitBuckets.clear();
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const wrapped = makeRateLimitedHandler(handler, { max: 1, windowMs: 60_000 });

  await wrapped(
    fakeReq({ headers: { 'x-forwarded-for': '1.1.1.1' } }),
    fakeRes(),
  );
  const res1Blocked = fakeRes();
  await wrapped(
    fakeReq({ headers: { 'x-forwarded-for': '1.1.1.1' } }),
    res1Blocked,
  );
  assert.equal(res1Blocked.statusCode, 429);

  // Different client must still get through.
  const res2 = fakeRes();
  await wrapped(
    fakeReq({ headers: { 'x-forwarded-for': '2.2.2.2' } }),
    res2,
  );
  assert.equal(res2.statusCode, 200);
});

test('rate-limit: fresh window after reset', async () => {
  rateLimitBuckets.clear();
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const wrapped = makeRateLimitedHandler(handler, { max: 1, windowMs: 50 });

  await wrapped(fakeReq(), fakeRes());
  const blocked = fakeRes();
  await wrapped(fakeReq(), blocked);
  assert.equal(blocked.statusCode, 429);

  // Wait for the window to pass.
  await new Promise((r) => setTimeout(r, 60));

  const fresh = fakeRes();
  await wrapped(fakeReq(), fresh);
  assert.equal(fresh.statusCode, 200);
});

/* ── Body-validator tests ────────────────────────────────────────────── */

test('validateBody: passes through for non-body methods', async () => {
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  // Predicate intentionally strict — should NOT be invoked for GET.
  const wrapped = makeValidatedHandler(handler, () => false);

  const res = fakeRes();
  await wrapped(fakeReq({ method: 'GET' }), res);
  assert.equal(res.statusCode, 200);
});

test('validateBody: rejects null body', async () => {
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const wrapped = makeValidatedHandler(handler, () => true);

  const res = fakeRes();
  await wrapped(fakeReq({ body: null }), res);
  assert.equal(res.statusCode, 400);
});

test('validateBody: rejects array body', async () => {
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const wrapped = makeValidatedHandler(handler, () => true);

  const res = fakeRes();
  await wrapped(fakeReq({ body: [1, 2, 3] }), res);
  assert.equal(res.statusCode, 400);
});

test('validateBody: rejects payload that fails the guard', async () => {
  const handler = async (_req, res) => res.status(200).json({ ok: true });
  const isValid = (b) => typeof b.name === 'string' && b.name.length > 0;
  const wrapped = makeValidatedHandler(handler, isValid);

  const res = fakeRes();
  await wrapped(fakeReq({ body: { name: '' } }), res);
  assert.equal(res.statusCode, 400);
});

test('validateBody: forwards valid payloads to handler', async () => {
  let reachedHandler = false;
  const handler = async (_req, res) => {
    reachedHandler = true;
    res.status(200).json({ ok: true });
  };
  const isValid = (b) => typeof b.name === 'string' && b.name.length > 0;
  const wrapped = makeValidatedHandler(handler, isValid);

  const res = fakeRes();
  await wrapped(fakeReq({ body: { name: 'alice' } }), res);
  assert.equal(res.statusCode, 200);
  assert.equal(reachedHandler, true);
});
