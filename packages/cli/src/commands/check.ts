import { defineCommand } from "citty";
import { consola } from "consola";
import { runCheck } from "@intl-ai/api";
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
    const result = await runCheck(config, { locale: args.locale });

    for (const localeResult of result.results) {
      if (localeResult.missing.length > 0) {
        consola.error(`[${localeResult.locale}] Missing translations:`);
        for (const missing of localeResult.missing) {
          consola.log(`  - ${missing.key}: "${missing.source}"`);
        }
      }

      if (localeResult.stale.length > 0) {
        consola.warn(`[${localeResult.locale}] Stale translations (source changed):`);
        for (const stale of localeResult.stale) {
          consola.log(`  - ${stale.key}: previously "${stale.previous}"`);
        }
      }

      if (localeResult.extra.length > 0) {
        consola.info(`[${localeResult.locale}] Extra translations (not in source):`);
        for (const extra of localeResult.extra) {
          consola.log(`  - ${extra}`);
        }
      }

      if (localeResult.missing.length === 0 && localeResult.stale.length === 0) {
        consola.success(`[${localeResult.locale}] All translations complete`);
      }
    }

    process.exit(result.hasIssues ? 10 : 0);
  },
});
