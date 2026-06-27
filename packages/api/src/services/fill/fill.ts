import {
  join,
  readText,
  writeText,
  pathExists,
  ensureDir,
  setNestedValue,
  getNestedValue,
} from "../../infrastructure/fs";
import { LockfileManager } from "../../lockfile/manager";
import { findMissingTranslations, lockfileEntryToMap } from "../../core/diff";
import { translateBatch } from "./translator";
import type { AIProvider } from "../../ports/provider";
import type { IntlAiConfig } from "../../types";

export interface RunFillOptions {
  locale?: string;
  force?: boolean;
  dryRun?: boolean;
}

export interface RunFillResult {
  locales: string[];
  translated: number;
  skipped: number;
  errors: number;
}

export async function runFill(
  config: IntlAiConfig,
  options?: RunFillOptions,
): Promise<RunFillResult> {
  const { force = false, dryRun = false } = options ?? {};
  const {
    defaultLocale,
    locales,
    localeDir,
    model: provider,
    baseURL,
    apiKey,
    glossary,
    maxRetries,
    processor,
    hook,
    modelParams,
  } = config;

  const targetLocales = options?.locale
    ? locales.filter((l) => l === options.locale)
    : locales.filter((l) => l !== defaultLocale);

  if (targetLocales.length === 0) {
    return { locales: [], translated: 0, skipped: 0, errors: 0 };
  }

  const lockfileManager = new LockfileManager(localeDir);
  await lockfileManager.load();

  const sourceLocalePath = join(localeDir, `${defaultLocale}.json`);
  if (!(await pathExists(sourceLocalePath))) {
    return { locales: [], translated: 0, skipped: 0, errors: 0 };
  }
  const sourceLocaleData = JSON.parse(await readText(sourceLocalePath)) as Record<string, unknown>;

  let translated = 0;
  let skipped = 0;
  let errors = 0;

  for (const targetLocale of targetLocales) {
    try {
      const targetLocalePath = join(localeDir, `${targetLocale}.json`);
      let targetLocaleData: Record<string, unknown> = {};
      if (await pathExists(targetLocalePath)) {
        targetLocaleData = JSON.parse(await readText(targetLocalePath));
      }

      const lockfileEntries = lockfileEntryToMap(lockfileManager.getAllEntries(), targetLocale);

      const diff = await findMissingTranslations(
        {
          sourceLocale: sourceLocaleData,
          targetLocale: targetLocaleData,
          locale: targetLocale,
          lockfileEntries,
        },
        force,
      );

      if (diff.missing.length === 0 && diff.stale.length === 0) {
        skipped += Object.keys(flattenKeys(targetLocaleData)).length;
        continue;
      }

      const entriesToTranslate: Array<{ key: string; source: string }> = [
        ...diff.missing,
        ...diff.stale.map((s) => ({ key: s.key, source: s.source })),
      ];

      const results = await translateBatch({
        provider,
        entries: entriesToTranslate.map((e) => ({ key: e.key, source: e.source })),
        targetLocale,
        sourceLocale: defaultLocale,
        glossary,
        maxRetries,
        processor,
        baseURL,
        apiKey,
        hook,
        modelParams,
      });

      for (const result of results) {
        if (result.success && result.translated !== undefined) {
          setNestedValue(targetLocaleData, result.key, result.translated);

          const source = getNestedValue(sourceLocaleData, result.key);
          const sourceHash = await lockfileManager.hashSource(source);
          lockfileManager.setEntry(result.key, targetLocale, {
            key: result.key,
            locale: targetLocale,
            sourceHash,
            translated: result.translated,
            origin: "ai",
            model: modelToString(provider),
            timestamp: new Date().toISOString(),
          });
          translated++;
        } else {
          errors++;
        }
      }

      if (!dryRun) {
        await ensureDir(localeDir);
        await writeText(targetLocalePath, JSON.stringify(targetLocaleData, null, 2));
        await lockfileManager.save();
      }
    } catch {
      errors++;
    }
  }

  return { locales: targetLocales, translated, skipped, errors };
}

function modelToString(model: AIProvider | string): string {
  if (typeof model === "string") return model;
  return model.id;
}

function flattenKeys(obj: Record<string, unknown>): Record<string, string> {
  const out: Record<string, string> = {};
  function walk(o: unknown, prefix: string) {
    if (o === null || o === undefined) return;
    if (typeof o !== "object") {
      if (prefix) out[prefix] = String(o);
      return;
    }
    if (Array.isArray(o)) {
      if (prefix) out[prefix] = JSON.stringify(o);
      return;
    }
    for (const [k, v] of Object.entries(o as Record<string, unknown>)) {
      const key = prefix ? `${prefix}.${k}` : k;
      if (v !== null && typeof v === "object" && !Array.isArray(v)) walk(v, key);
      else out[key] = Array.isArray(v) ? JSON.stringify(v) : String(v);
    }
  }
  walk(obj, "");
  return out;
}
