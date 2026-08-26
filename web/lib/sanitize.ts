/**
 * XSS sanitization utilities.
 *
 * Uses DOMPurify to sanitize HTML, SVG, and markdown content before
 * rendering it into the DOM via innerHTML or dangerouslySetInnerHTML.
 *
 * Resolves WEB-16 (#102): No XSS sanitization on user-provided content.
 */
import DOMPurify from "dompurify";

/** Sanitize an HTML string, stripping all dangerous tags and attributes. */
export function sanitizeHtml(dirty: string): string {
  if (typeof window === "undefined") {
    // SSR fallback: strip all tags server-side
    return dirty.replace(/<[^>]*>/g, "");
  }
  return DOMPurify.sanitize(dirty);
}

/** Sanitize SVG content specifically (allows safe SVG tags like <svg>, <path>, etc.). */
export function sanitizeSvg(dirty: string): string {
  if (typeof window === "undefined") {
    return dirty.replace(/<[^>]*>/g, "");
  }
  return DOMPurify.sanitize(dirty, {
    USE_PROFILES: { svg: true, svgFilters: true },
  });
}

/** Sanitize a plain-text string for safe display (no HTML allowed at all). */
export function sanitizeText(dirty: string): string {
  if (typeof window === "undefined") {
    return dirty.replace(/<[^>]*>/g, "");
  }
  return DOMPurify.sanitize(dirty, { ALLOWED_TAGS: [] });
}

/** Default DOMPurify-based sanitizer for arbitrary user-controlled text. */
export function sanitize(dirty: string, context = "user input"): string {
  const sanitized = sanitizeText(dirty);

  if (sanitized !== dirty) {
    console.warn(`[security] Sanitized ${context}: removed unsafe HTML/JS content.`);
  }

  return sanitized;
}

/** Convenience helper for form/API arguments. */
export function sanitizeUserInput(value: string, fieldName = "input"): string {
  return sanitize(value, `${fieldName} value`);
}

/** Default export for convenience. */
export default sanitize;
