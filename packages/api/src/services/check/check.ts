import { LockfileManager } from "../../lockfile/manager";
import { findMissingTranslations, flattenObject, lockfileEntryToMap } from "../../core/diff";
import { findUnmatchedInstructionKeys } from "../../core/locale-instructions";
import { resolveDialect, variantForLocale } from "../../adapters/dialect/registry";
import { tokenSpans, type DialectHit } from "../../adapters/dialect/en";
import type { ResolvedIntlAiConfig } from "../../infrastructure/config/loader";
import type { MissingTranslationEntry } from "../../core/types";
import type { StaleEntry } from "../../lockfile/types";

export type { DialectHit };

export interface CheckDialectHit extends DialectHit {
  key: string;
}

export interface CheckLocaleResult {
  locale: string;
  missing: MissingTranslationEntry[];
  stale: StaleEntry[];
  extra: string[];
  dialect: CheckDialectHit[];
}

export interface RunCheckOptions {
  locale?: string;
  /** Locale to run the British/American dialect detector against, e.g. "en-US". */
  dialect?: string;
}

export interface RunCheckResult {
  results: CheckLocaleResult[];
  hasIssues: boolean;
  /** localeInstructions keys that match no configured locale (likely a typo). */
  unmatchedInstructionKeys: string[];
}

export async function runCheck(
  config: ResolvedIntlAiConfig,
  options?: RunCheckOptions,
): Promise<RunCheckResult> {
  const { defaultLocale, locales, localeDir, format, localeInstructions, processor } = config;
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

    const dialect =
      options?.dialect === locale ? await runDialectCheck(locale, targetLocale, processor) : [];

    results.push({
      locale,
      missing: diff.missing,
      stale: diff.stale,
      extra: diff.extra,
      dialect,
    });
  }

  if (options?.dialect && !results.some((r) => r.locale === options.dialect)) {
    const targetLocale = await format.readLocale(localeDir, options.dialect);
    const dialect = await runDialectCheck(options.dialect, targetLocale, processor);
    results.push({ locale: options.dialect, missing: [], stale: [], extra: [], dialect });
  }

  const hasIssues = results.some(
    (r) => r.missing.length > 0 || r.stale.length > 0 || r.dialect.length > 0,
  );

  const unmatchedInstructionKeys = findUnmatchedInstructionKeys(localeInstructions, locales);

  return { results, hasIssues, unmatchedInstructionKeys };
}

async function runDialectCheck(
  locale: string,
  catalog: Record<string, unknown>,
  processor: ResolvedIntlAiConfig["processor"],
): Promise<CheckDialectHit[]> {
  const variant = variantForLocale(locale);
  if (!variant) {
    throw new Error(
      `intl-ai check --dialect: cannot infer American/British variant from locale "${locale}". Use a region-qualified locale such as "en-US" or "en-GB".`,
    );
  }
  const subtag = locale.split("-")[0]!;
  const detect = resolveDialect(subtag);

  const flat = flattenObject(catalog);
  const hits: CheckDialectHit[] = [];
  for (const [key, value] of Object.entries(flat)) {
    const excludeSpans = processor ? tokenSpans(value, processor.extractTokens(value)) : [];
    const valueHits = await detect(value, variant, excludeSpans);
    for (const hit of valueHits) {
      hits.push({ ...hit, key });
    }
  }
  return hits;
}
