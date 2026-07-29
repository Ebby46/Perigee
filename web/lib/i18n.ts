/**
 * i18n / white-label translation layer.
 *
 * Built on next-intl.  The default English messages live in
 * `messages/en.json`.  Wealth managers can override every visible string
 * by setting the `NEXT_PUBLIC_MANAGER_LABEL` environment variable to a
 * key that maps to a custom messages file (e.g. `messages/acme.json`).
 *
 * Resolves WEB-17 (#103): Navigation/metadata not localized for white-label.
 */
import { getRequestConfig } from "next-intl/server";

async function loadMessages(locale: string): Promise<Record<string, string>> {
  const defaultMessages: Record<string, string> = (
    await import(`../messages/${locale}.json`)
  ).default;

  const managerLabel = process.env.NEXT_PUBLIC_MANAGER_LABEL?.trim();
  if (!managerLabel) return defaultMessages;

  try {
    const managerOverrides: Partial<Record<string, string>> = (
      await import(`../messages/${managerLabel}.json`)
    ).default;
    return { ...defaultMessages, ...managerOverrides };
  } catch {
    console.warn(
      `[i18n] Manager override "${managerLabel}" messages file not found. Using defaults.`,
    );
    return defaultMessages;
  }
}

export default getRequestConfig(async ({ locale }) => ({
  messages: await loadMessages(locale),
  timeZone: "UTC",
}));
