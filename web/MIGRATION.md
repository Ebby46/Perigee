# App Router Migration Plan

## Status: Foundation Established — In Progress

## Why We're Migrating

Next.js 16 ships with App Router as the default. Continuing with Pages Router will eventually cause:
- Version lag (future major versions may drop Pages Router support)
- Missed performance improvements (React Server Components, streaming, parallel routes)
- Maintenance burden (two routers in one codebase)

### Current State

- **Router**: Pages Router (`pages/` directory)
- **Next.js version**: `16.1.6` (pinned — see `package.json`)
- **Migration started**: July 2026
- **Coexistence**: Pages Router and App Router will run simultaneously during migration

## Migration Strategy

**Incremental migration** — Pages Router and App Router can coexist during transition. This allows deploying changes incrementally without a risky "big bang" cutover.

### Key Principles

1. **One page at a time** — Migrate pages to `app/` without touching the rest
2. **No breaking changes** — Keep Pages Router routes working until App Router replacement is ready
3. **Test each phase** — Verify functionality after each page migration
4. **Document as you go** — Update this file as pages migrate

## Phase 1 — Foundation ✅ COMPLETE

- [x] Pin Next.js version (`16.1.6`) to prevent accidental upgrades
- [x] Create `app/` directory with root layout
- [x] Create `app/page.tsx` placeholder
- [x] Document migration plan in `MIGRATION.md`
- [x] Update `next.config.js` with migration status comments
- [x] Update `next.config.js` with experimental configuration section

## Phase 2 — Simple Pages (No Data Fetching)

Migrate pages with no `getServerSideProps` or `getStaticProps`:

### pages/index.tsx → app/page.tsx

**Current State**:
- Uses `useState`, `useEffect`, and client-side API calls
- Uses `next/head` for metadata — convert to `metadata` export in `app/layout.tsx`
- Wrapped in `WalletProvider` and `ErrorBoundary` in `_app.tsx`
- No server-side data fetching

**Migration Steps**:
1. Copy content from `pages/index.tsx` to `app/page.tsx`
2. Add `"use client"` directive at top of file (needs interactivity)
3. Remove `Head` import and `<Head>` component usage
4. Verify metadata is set in `app/layout.tsx` (already templated)
5. Test in dev environment (wallet connection, contract analysis)
6. Delete `pages/index.tsx` only after confirmed working

**Status**: ⏳ Pending

## Phase 3 — Providers & Layout

Convert `pages/_app.tsx` providers to `app/layout.tsx`:

### Move Global CSS

- [ ] Move `import '../styles/globals.css'` from `pages/_app.tsx` to `app/layout.tsx`

### Move ErrorBoundary

**Current Implementation**: Class component (`ErrorBoundary.tsx`)
- Catches render errors with custom error UI
- Shows error message, component stack (dev only), retry + reload buttons

**Migration Options**:
- **Option A**: Use App Router `error.tsx` file (native error boundary)
- **Option B**: Keep class component as wrapper if simpler

**Recommended**: Option A (native) for consistency with App Router patterns

### Move WalletProvider

**Current Implementation**: Already has `"use client"` directive (future-proof!)
- Handles Stellar wallet connection/disconnection
- Uses Zustand-like store pattern
- Persists to localStorage + sessionStorage
- Supports multiple wallet modules (Freighter, Albedo, xBull, Rabet, Lobstr)

**Migration Steps**:
- Wrap `{children}` with `<WalletProvider>` in `app/layout.tsx`
- Add `"use client"` directive to layout (if needed for providers)
- No code changes required to WalletProvider itself

### Cleanup

- [ ] Delete `pages/_app.tsx` after all providers migrated

**Status**: ⏳ Pending

## Phase 4 — Cleanup

- [ ] Delete `pages/` directory (only after all routes migrated)
- [ ] Remove `useFileSystemPublicRoutes: true` from `next.config.js`
- [ ] Update Next.js to latest version
- [ ] Remove or archive `MIGRATION.md` file

**Status**: ⏳ Pending

## Phase 5 — API Routes

**Status**: ⚠️ Partial — one catch-all proxy route exists

**File**: `pages/api/[[...path]].ts`

**Reason**: A catch-all API route was added to proxy `/api/*` requests to the Rust backend. This avoids CORS issues in production by ensuring the browser makes same-origin requests to the Next.js server, which then forwards them to the backend.

This phase would normally handle converting `pages/api/X.ts` to `app/api/X/route.ts`. When migrating to App Router, replace `pages/api/[[...path]].ts` with `app/api/[[...path]]/route.ts` using the App Router Route Handler pattern.

**Migration notes**:
- The proxy reads `API_URL` (server-side) to determine the backend target.
- In production, the client calls `/api/*` instead of the direct backend URL.
- The route forwards all HTTP methods, query parameters, and JSON bodies.

## Phase 6 — Final Verification

After Phase 4 cleanup, verify:

- [ ] Run full test suite: `npm test`
- [ ] Build production bundle: `npm run build`
- [ ] Check for console warnings or errors
- [ ] Test all user-facing features (wallet, contract analysis, WASM upload)
- [ ] Verify deployment works as expected
- [ ] Monitor error logs for any App Router-specific issues

**Status**: ⏳ Pending

## Key Differences to Handle

### Metadata: `next/head` → `metadata` export

```typescript
// BEFORE (Pages Router)
import Head from 'next/head';

export default function Page() {
  return (
    <>
      <Head>
        <title>Perigee - Soroban Smart Contract Resource Analyzer</title>
        <meta name="description" content="Explore, test, and analyze..." />
      </Head>
      <div>Content</div>
    </>
  );
}

// AFTER (App Router)
export const metadata = {
  title: 'Perigee - Soroban Smart Contract Resource Analyzer',
  description: 'Explore, test, and analyze...',
};

export default function Page() {
  return <div>Content</div>;
}
```

### Client Components: `"use client"` directive

```typescript
// BEFORE: Implicit client context in pages/
import { useState } from 'react';

export default function Page() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}

// AFTER: Explicit "use client" for client-side interactivity
'use client';

import { useState } from 'react';

export default function Page() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}
```

## Pages Inventory

All pages currently in the project:

| Page | File | Data Fetching | Complexity | Status |
|------|------|---------------|-----------|--------|
| Home | `pages/index.tsx` | Client-side API calls only | Low | ⏳ Pending |

**Total: 1 page** — Simple migration, no complex data fetching.

## API Routes Inventory

| Route | Method | Status |
|-------|--------|--------|
| None | — | ✅ N/A (all calls to external service) |

**Total: 0 API routes** — No backend routes to migrate.

## Providers & Context Used

The project uses these providers in `pages/_app.tsx`:

### 1. ErrorBoundary

- **Type**: Class component (React.Component)
- **Purpose**: Catches render errors and displays error UI
- **Current location**: `components/ErrorBoundary.tsx`
- **Features**:
  - Custom error display with red styling
  - Shows error message and component stack (dev only)
  - Retry button + reload button
- **Migration**: Convert to App Router error boundary (`error.tsx`)

### 2. WalletProvider

- **Type**: Custom context (already has `"use client"` directive!)
- **Location**: `context/WalletContext.tsx`
- **Purpose**: Stellar wallet connection management
- **Features**:
  - Zustand-like store pattern for state
  - Supports multiple wallet modules
  - Persists to localStorage + sessionStorage
  - Auto-reconnect on page load
- **Migration**: Wrap in `app/layout.tsx` (no code changes needed)

## Global Resources

- **CSS**: `styles/globals.css` (Tailwind imports + custom body styles)
- **Config**: `next.config.js`, `tsconfig.json`, `tailwind.config.js`
- **Environment**: `.env.example` (one var: `NEXT_PUBLIC_API_URL`)
- **Middleware**: `middleware.ts` (security headers — compatible with both routers)

## TypeScript & Build Configuration

**tsconfig.json**:
- ✅ `strict: true` — Strict type checking
- ✅ `moduleResolution: "bundler"` — App Router compatible
- ✅ `jsx: "react-jsx"` — React 19 compatible
- ❌ **No path aliases** — Uses relative imports (could add `@/*` pattern if desired)

## Migration Checklist

### Before You Start

- [ ] Read this file completely
- [ ] Understand the differences between Pages Router and App Router
- [ ] Have a test environment ready
- [ ] Ensure all current tests pass

### Phase 2 Checklist (Simple Pages)

- [ ] Backup current code or create a migration branch
- [ ] Copy `pages/index.tsx` to `app/page.tsx`
- [ ] Add `"use client"` directive to `app/page.tsx`
- [ ] Remove `next/head` usage (replace with metadata export)
- [ ] Test the app in dev environment
- [ ] Verify wallet connection still works
- [ ] Verify contract analysis features work
- [ ] Verify WASM upload feature works
- [ ] Commit changes with message: "chore: migrate pages/index.tsx to app/page.tsx"
- [ ] Delete `pages/index.tsx`

### Phase 3 Checklist (Providers)

- [ ] Move `ErrorBoundary` and `WalletProvider` setup to `app/layout.tsx`
- [ ] Import global CSS in layout
- [ ] Add `"use client"` directive to layout if needed
- [ ] Test all functionality in dev environment
- [ ] Test wallet connection
- [ ] Test error boundaries
- [ ] Delete `pages/_app.tsx`
- [ ] Commit changes with message: "chore: migrate _app.tsx providers to app/layout.tsx"

### Phase 4 Checklist (Cleanup)

- [ ] Delete `pages/` directory
- [ ] Remove `useFileSystemPublicRoutes: true` from `next.config.js`
- [ ] Remove experimental configuration section from `next.config.js`
- [ ] Run `npm run build` to verify production build
- [ ] Update Next.js to latest version (if desired)
- [ ] Commit changes with message: "chore: complete App Router migration"

## Testing Strategy

After each phase, verify:

```bash
# Development server
npm run dev

# Check for errors in console
# Test wallet connection (if applicable)
# Test contract analysis features
# Test WASM upload
# Test page metadata loads correctly

# Production build
npm run build
npm run start
```

## Rollback Plan

If migration goes wrong:

```bash
# Restore from git
git checkout -- pages/ app/

# Or reset to pre-migration commit
git reset --hard <commit-before-migration>
```

## Resources

- [Next.js App Router Migration Guide](https://nextjs.org/docs/app/building-your-application/upgrading/app-router-migration)
- [Incremental Adoption Strategy](https://nextjs.org/docs/app/building-your-application/upgrading/app-router-migration#migrating-from-pages-to-app)
- [React Server Components](https://nextjs.org/docs/app/building-your-application/rendering/server-components)
- [Metadata API](https://nextjs.org/docs/app/building-your-application/optimizing/metadata)
- [Error Handling](https://nextjs.org/docs/app/building-your-application/routing/error-handling)
- [File Conventions](https://nextjs.org/docs/app/api-reference/file-conventions)

## Questions & Notes

**Q: Can I use both pages/ and app/ at the same time?**
- A: Yes! Next.js prioritizes `app/` routes over `pages/` routes when both exist. This enables incremental migration.

**Q: Will my middleware.ts still work?**
- A: Yes! `middleware.ts` in the root works with both Pages Router and App Router.

**Q: Do I need to convert the Stellar wallet kit to Server Components?**
- A: No. The `WalletProvider` is already marked `"use client"` and works fine in App Router layouts.

**Q: What about TypeScript path aliases?**
- A: The project currently uses relative imports (no aliases). This doesn't change during migration, but aliases can be added to `tsconfig.json` if desired: `"@/*": ["./*"]`

**Q: How long should Phase 2 take?**
- A: With only 1 page to migrate, Phase 2 should take 30 minutes to 1 hour (copy, test, verify).

**Q: Should I update Next.js to a newer version after migration?**
- A: It's safe to update after Phase 4 cleanup. The migration itself uses 16.1.6, which is stable and compatible with the incremental approach.

---

**Last Updated**: July 2026  
**Next Review**: After Phase 2 completion
