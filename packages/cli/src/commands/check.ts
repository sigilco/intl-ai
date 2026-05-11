import { Command } from "commander";
import { loadConfig, findMissingTranslations, readJsonFile, LockfileManager } from "@intl-ai/core";
import { existsSync } from "fs";
import { join } from "path";

export interface CheckOptions {
  locale?: string;
}

export async function checkAction(options: CheckOptions): Promise<void> {
  const { locale: targetLocale } = options;

  const config = await loadConfig();

  const lockfileManager = new LockfileManager(config.localeDir);

  const sourceLocalePath = join(config.localeDir, `${config.defaultLocale}.json`);

  if (!existsSync(sourceLocalePath)) {
    throw new Error(`Source locale file not found: ${sourceLocalePath}`);
  }

  const sourceLocale = readJsonFile(sourceLocalePath);

  const localesToCheck = targetLocale
    ? [targetLocale]
    : config.locales.filter((l) => l !== config.defaultLocale);

  let hasIssues = false;

  for (const locale of localesToCheck) {
    const targetLocalePath = join(config.localeDir, `${locale}.json`);
    let targetLocaleData: Record<string, unknown> = {};

    if (existsSync(targetLocalePath)) {
      targetLocaleData = readJsonFile(targetLocalePath);
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
      targetLocale: targetLocaleData,
      locale,
      lockfileEntries,
    });

    if (diff.missing.length > 0) {
      hasIssues = true;
      console.log(`\n[${locale}] Missing translations:`);
      for (const missing of diff.missing) {
        console.log(`  - ${missing.key}: "${missing.source}"`);
      }
    }

    if (diff.stale.length > 0) {
      hasIssues = true;
      console.log(`\n[${locale}] Stale translations (source changed):`);
      for (const stale of diff.stale) {
        console.log(`  - ${stale.key}: ${stale.reason}`);
      }
    }

    if (diff.extra.length > 0) {
      console.log(`\n[${locale}] Extra translations (not in source):`);
      for (const extra of diff.extra) {
        console.log(`  - ${extra}`);
      }
    }

    if (diff.missing.length === 0 && diff.stale.length === 0) {
      console.log(`[${locale}] ✓ All translations complete`);
    }
  }

  if (hasIssues) {
    process.exit(10);
  }
}

export const checkCommand = new Command("check")
  .description("Check for missing translations")
  .option("--locale <locale>", "Check a specific locale only")
  .action(async (options) => {
    try {
      await checkAction(options);
      process.exit(0);
    } catch (error) {
      console.error("Error during check:", error);
      process.exit(1);
    }
  });
