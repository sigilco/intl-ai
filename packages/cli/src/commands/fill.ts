import { Command } from "commander";
import {
  loadConfig,
  findMissingTranslations,
  translateBatch,
  LockfileManager,
  readJsonFile,
  writeJsonFile,
} from "@intl-ai/core";
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";
import { createProgressReporter } from "../utils/progress";

export interface FillOptions {
  dryRun?: boolean;
  locale?: string;
  force?: boolean;
  silent?: boolean;
}

export async function fillAction(options: FillOptions): Promise<void> {
  const { dryRun = false, locale: targetLocale, force = false, silent = false } = options;

  const progress = createProgressReporter({ silent });

  const config = await loadConfig();

  const lockfileManager = new LockfileManager(config.localeDir);

  const sourceLocalePath = join(config.localeDir, `${config.defaultLocale}.json`);

  if (!existsSync(sourceLocalePath)) {
    throw new Error(`Source locale file not found: ${sourceLocalePath}`);
  }

  const sourceLocale = readJsonFile(sourceLocalePath);

  const localesToProcess = targetLocale
    ? [targetLocale]
    : config.locales.filter((l) => l !== config.defaultLocale);

  for (const locale of localesToProcess) {
    const targetLocalePath = join(config.localeDir, `${locale}.json`);
    let targetLocale: Record<string, unknown> = {};

    if (existsSync(targetLocalePath)) {
      targetLocale = readJsonFile(targetLocalePath);
    }

    const lockfileEntries = new Map<string, { sourceHash: string }>();
    const allEntries = lockfileManager.getAllEntries();

    for (const [key, entry] of Object.entries(allEntries)) {
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
      const entry = lockfileManager.getEntry(stale.key, locale);
      const shouldTranslate = force || !lockfileManager.isHumanEdited(stale.key, locale);

      if (shouldTranslate) {
        const sourceValue = getNestedValue(sourceLocale, stale.key);
        if (sourceValue !== undefined) {
          entriesToTranslate.push({
            key: stale.key,
            source: String(sourceValue),
          });
        }
      }
    }

    if (entriesToTranslate.length === 0) {
      progress.log(`No missing or stale translations for locale: ${locale}`);
      continue;
    }

    progress.log(`Translating ${entriesToTranslate.length} entries for locale: ${locale}`);

    const spinner = progress.startSpinner(`Processing translations for ${locale}...`);

    const results = await translateBatch({
      model: config.model,
      entries: entriesToTranslate,
      targetLocale: locale,
      sourceLocale: config.defaultLocale,
      glossary: config.glossary,
      maxRetries: config.maxRetries,
    });

    progress.stopSpinner(spinner);

    if (dryRun) {
      progress.log("\nDry-run preview:");
      for (const result of results) {
        if (result.success) {
          progress.log(`  ${result.key}: "${result.translated}"`);
        } else {
          progress.error(`  ${result.key}: FAILED - ${result.error}`);
        }
      }
      continue;
    }

    let successCount = 0;
    let failureCount = 0;

    for (const result of results) {
      if (result.success) {
        successCount++;
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
      } else {
        failureCount++;
        progress.error(`Failed to translate ${result.key}: ${result.error}`);
      }
    }

    mkdirSync(config.localeDir, { recursive: true });
    writeJsonFile(targetLocalePath, targetLocale);
    lockfileManager.save();

    progress.log(`Successfully updated locale file: ${targetLocalePath}`);

    progress.logSummary({
      locale,
      total: results.length,
      successful: successCount,
      failed: failureCount,
    });
  }
}

export const fillCommand = new Command("fill")
  .description("Fill missing translations using AI")
  .option("--dry-run", "Preview changes without modifying files")
  .option("--locale <locale>", "Target a specific locale only")
  .option("--force", "Overwrite human-edited entries")
  .option("--silent", "Suppress all output except errors")
  .action(async (options) => {
    try {
      await fillAction(options);
      process.exit(0);
    } catch (error) {
      console.error("Error during fill:", error);
      process.exit(1);
    }
  });

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
    if (!(part in current)) {
      current[part] = {};
    }
    current = current[part] as Record<string, unknown>;
  }

  current[lastPart] = value;
}
