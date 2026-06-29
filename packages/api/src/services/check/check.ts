import { join } from "../../infrastructure/fs";
import { LockfileManager } from "../../lockfile/manager";
import { findMissingTranslations, lockfileEntryToMap } from "../../core/diff";
import { readJsonFile } from "../../adapters/formats/json";
import type { IntlAiConfig } from "../../types";
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
  config: IntlAiConfig,
  options?: RunCheckOptions,
): Promise<RunCheckResult> {
  const { defaultLocale, locales, localeDir } = config;

  const localesToCheck = options?.locale
    ? [options.locale]
    : locales.filter((l) => l !== defaultLocale);

  const lockfileManager = new LockfileManager(localeDir);
  await lockfileManager.load();

  const sourceLocalePath = join(localeDir, `${defaultLocale}.json`);
  const sourceLocale = await readJsonFile(sourceLocalePath);

  const results: CheckLocaleResult[] = [];

  for (const locale of localesToCheck) {
    const targetLocalePath = join(localeDir, `${locale}.json`);
    let targetLocale: Record<string, unknown> = {};

    try {
      targetLocale = await readJsonFile(targetLocalePath);
    } catch {
      // Missing target file = all keys missing
    }

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
