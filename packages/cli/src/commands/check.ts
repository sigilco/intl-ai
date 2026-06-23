import { defineCommand } from "citty";
import { consola } from "consola";
import {
  findMissingTranslations,
  LockfileManager,
  readJsonFile,
  lockfileEntryToMap,
} from "@intl-ai/api/internal";
import { join } from "pathe";
import { loadConfig } from "../config/loader";

export const checkCommand = defineCommand({
  meta: {
    name: "check",
    description: "Check for missing or stale translations",
  },
  args: {
    config: {
      type: "string",
      default: "intl-ai.config.json",
      description: "Path to JSON config file",
    },
    locale: {
      type: "string",
      description: "Check a specific locale only",
    },
  },
  async run({ args }) {
    const config = await loadConfig(args.config);

    const localesToCheck = args.locale
      ? [args.locale]
      : config.locales.filter((l) => l !== config.defaultLocale);

    const lockfileManager = new LockfileManager(config.localeDir);
    await lockfileManager.load();

    const sourceLocalePath = join(config.localeDir, `${config.defaultLocale}.json`);
    const sourceLocale = await readJsonFile(sourceLocalePath);

    let hasIssues = false;

    for (const locale of localesToCheck) {
      const targetLocalePath = join(config.localeDir, `${locale}.json`);
      let targetLocale: Record<string, unknown> = {};

      try {
        targetLocale = await readJsonFile(targetLocalePath);
      } catch {
        // Missing target file is treated as "all keys missing"
      }

      const lockfileEntries = lockfileEntryToMap(lockfileManager.getAllEntries(), locale);

      const diff = await findMissingTranslations({
        sourceLocale,
        targetLocale,
        locale,
        lockfileEntries,
      });

      if (diff.missing.length > 0) {
        hasIssues = true;
        consola.error(`[${locale}] Missing translations:`);
        for (const missing of diff.missing) {
          consola.log(`  - ${missing.key}: "${missing.source}"`);
        }
      }

      if (diff.stale.length > 0) {
        hasIssues = true;
        consola.warn(`[${locale}] Stale translations (source changed):`);
        for (const stale of diff.stale) {
          consola.log(`  - ${stale.key}: previously "${stale.previous}"`);
        }
      }

      if (diff.extra.length > 0) {
        consola.info(`[${locale}] Extra translations (not in source):`);
        for (const extra of diff.extra) {
          consola.log(`  - ${extra}`);
        }
      }

      if (diff.missing.length === 0 && diff.stale.length === 0) {
        consola.success(`[${locale}] All translations complete`);
      }
    }

    process.exit(hasIssues ? 10 : 0);
  },
});
