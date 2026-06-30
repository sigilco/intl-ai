import { LockfileManager } from "../../lockfile/manager";
import { findMissingTranslations, lockfileEntryToMap } from "../../core/diff";
import type { ResolvedIntlAiConfig } from "../../infrastructure/config/loader";
import type { MissingTranslationEntry } from "../../core/types";
import type { StaleEntry } from "../../lockfile/types";

export interface CheckLocaleResult {
  locale: string;
  missing: MissingTranslationEntry[];
  stale: StaleEntry[];
  extra: string[];
}

export interface RunCheckOptions {
  locale?: string;
}

export interface RunCheckResult {
  results: CheckLocaleResult[];
  hasIssues: boolean;
}

export async function runCheck(
  config: ResolvedIntlAiConfig,
  options?: RunCheckOptions,
): Promise<RunCheckResult> {
  const { defaultLocale, locales, localeDir, format } = config;
  // Loader resolves the format string to a LocaleFormat object before runCheck is called.

  const localesToCheck = options?.locale
    ? [options.locale]
    : locales.filter((l) => l !== defaultLocale);

  const lockfileManager = new LockfileManager(localeDir);
  await lockfileManager.load();

  const sourceLocale = await format.readLocale(localeDir, defaultLocale);

  const results: CheckLocaleResult[] = [];

  for (const locale of localesToCheck) {
    const targetLocale = await format.readLocale(localeDir, locale);

    const lockfileEntries = lockfileEntryToMap(lockfileManager.getAllEntries(), locale);

    const diff = await findMissingTranslations({
      sourceLocale,
      targetLocale,
      locale,
      lockfileEntries,
    });

    results.push({
      locale,
      missing: diff.missing,
      stale: diff.stale,
      extra: diff.extra,
    });
  }

  const hasIssues = results.some((r) => r.missing.length > 0 || r.stale.length > 0);

  return { results, hasIssues };
}
