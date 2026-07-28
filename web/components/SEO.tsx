import Head from "next/head";

/**
 * SEO — Centralized per-page metadata.
 *
 * Wraps `next/head` so every Pages-Router page renders, at minimum:
 *   • `title`
 *   • `description`
 *   • canonical URL
 *   • Open Graph + Twitter Card tags for sharing and indexing
 *
 * Pages must use `<SEO title="…" description="…" path="/…" />` instead of
 * raw `<Head>` blocks. Closes #104 / WEB-18.
 *
 * Defaults:
 *   - `path` is appended to the canonical site origin (https://perigee.app).
 *   - `ogImage` falls back to `/og-default.png`.
 */
export interface SeoProps {
  /** Page-specific title; " | Perigee" is appended automatically. */
  title: string;
  /** One-sentence summary that explains what the page is about. */
  description: string;
  /** Path component of the canonical URL, e.g. "/admin/managers". */
  path?: string;
  /** Absolute or root-relative image URL for shared previews. */
  ogImage?: string;
  /** When `true`, adds `<meta name="robots" content="noindex,nofollow">`. */
  noIndex?: boolean;
}

// Allow staging / preview deployments to advertise their own origin so
// canonical URLs and OG tags don't lie about the host a page was rendered at.
const SITE_ORIGIN =
  process.env.NEXT_PUBLIC_SITE_URL?.replace(/\/+$/, "") || "https://perigee.app";
const DEFAULT_OG_IMAGE = "/og-default.png";

function buildTitle(title: string): string {
  if (/(Perigee)/i.test(title)) {
    return title;
  }
  return `${title} | Perigee`;
}

function buildCanonical(path?: string): string | undefined {
  if (!path) return undefined;
  return `${SITE_ORIGIN}${path.startsWith("/") ? path : `/${path}`}`;
}

export function SEO({
  title,
  description,
  path,
  ogImage,
  noIndex = false,
}: SeoProps) {
  const fullTitle = buildTitle(title);
  const canonical = buildCanonical(path);
  const image = ogImage ?? DEFAULT_OG_IMAGE;

  return (
    <Head>
      <title>{fullTitle}</title>
      <meta name="description" content={description} />
      {noIndex && <meta name="robots" content="noindex,nofollow" />}
      {canonical && <link rel="canonical" href={canonical} />}

      {/* Open Graph */}
      <meta property="og:title" content={fullTitle} />
      <meta property="og:description" content={description} />
      <meta property="og:type" content="website" />
      <meta property="og:image" content={image} />
      {canonical && <meta property="og:url" content={canonical} />}
      <meta property="og:site_name" content="Perigee" />

      {/* Twitter / X */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content={fullTitle} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={image} />
    </Head>
  );
}
