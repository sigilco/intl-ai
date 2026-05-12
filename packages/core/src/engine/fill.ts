import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { loadConfig } from "../config/loader";
import { findMissingTranslations } from "../engine/diff";
import { translateBatch } from "../engine/translator";
import { LockfileManager } from "../lockfile/manager";
import { readJsonFile, writeJsonFile } from "../formats/json";

export interface RunFillOptions {
  debug?: boolean;
}

function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
  const parts = path.split(".");
  let current: unknown = obj;
  for (const part of parts) {
    if (current && typeof current === "object") {
      current = (current as Record<string, unknown>)[part];
    } else {
      return undefined;
    }
  }
  return current;
}

function setNestedValue(obj: Record<string, unknown>, path: string, value: unknown): void {
  const parts = path.split(".");
  const lastPart = parts.pop()!;
  let current: Record<string, unknown> = obj;
  for (const part of parts) {
    if (!(part in current)) current[part] = {};
    current = current[part] as Record<string, unknown>;
  }
  current[lastPart] = value;
}

export async function runFill(options: RunFillOptions = {}): Promise<void> {
  const { debug = false } = options;

  const config = await loadConfig();
  const lockfileManager = new LockfileManager(config.localeDir);
  const sourceLocalePath = join(config.localeDir, `${config.defaultLocale}.json`);

  if (!existsSync(sourceLocalePath)) return;

  const sourceLocale = readJsonFile(sourceLocalePath);

  const localesToProcess = config.locales.filter((l) => l !== config.defaultLocale);

  for (const locale of localesToProcess) {
    const targetLocalePath = join(config.localeDir, `${locale}.json`);
    let targetLocale: Record<string, unknown> = {};

    if (existsSync(targetLocalePath)) {
      targetLocale = readJsonFile(targetLocalePath);
    }

    const lockfileEntries = new Map<string, { sourceHash: string }>();
    for (const [key, entry] of Object.entries(lockfileManager.getAllEntries())) {
      const [entryLocale, entryKey] = key.split(":");
      if (entryLocale === locale) {
        lockfileEntries.set(entryKey, { sourceHash: entry.sourceHash });
      }
    }

    const diff = findMissingTranslations({
      sourceLocale,
      targetLocale,
      locale,
      lockfileEntries,
    });

    const entriesToTranslate: Array<{ key: string; source: string }> = [];

    for (const missing of diff.missing) {
      entriesToTranslate.push({ key: missing.key, source: missing.source });
    }

    for (const stale of diff.stale) {
      const sourceValue = getNestedValue(sourceLocale, stale.key);
      if (sourceValue !== undefined) {
        entriesToTranslate.push({ key: stale.key, source: String(sourceValue) });
      }
    }

    if (entriesToTranslate.length === 0) continue;

    if (debug) {
      console.log(
        `[intl-ai] Translating ${entriesToTranslate.length} entries for locale: ${locale}`,
      );
    }

    const results = await translateBatch({
      model: config.model,
      entries: entriesToTranslate,
      targetLocale: locale,
      sourceLocale: config.defaultLocale,
      glossary: config.glossary,
      maxRetries: config.maxRetries,
    });

    for (const result of results) {
      if (result.success) {
        setNestedValue(targetLocale, result.key, result.translated);
        const sourceValue = getNestedValue(sourceLocale, result.key);
        lockfileManager.setEntry(result.key, locale, {
          key: result.key,
          locale,
          sourceHash: lockfileManager.hashSource(String(sourceValue)),
          translated: result.translated,
          origin: "ai",
          model: config.model.toString(),
          timestamp: new Date().toISOString(),
        });
      }
    }

    mkdirSync(config.localeDir, { recursive: true });
    writeJsonFile(targetLocalePath, targetLocale);
    lockfileManager.save();
  }
}
