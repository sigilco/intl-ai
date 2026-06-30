import { command } from "cleye";
import { runCheck } from "@intl-ai/api";
import { loadConfig } from "../config/loader";
import { configureLogger, logger } from "../logger";

export async function runCheckCommand(opts: { config: string; locale?: string }): Promise<void> {
  await configureLogger(false);
  const config = await loadConfig(opts.config);
  const result = await runCheck(config, { locale: opts.locale });

  for (const localeResult of result.results) {
    if (localeResult.missing.length > 0) {
      logger.error`[${localeResult.locale}] Missing translations:`;
      for (const missing of localeResult.missing) {
        logger.info`  - ${missing.key}: "${missing.source}"`;
      }
    }

    if (localeResult.stale.length > 0) {
      logger.warn`[${localeResult.locale}] Stale translations (source changed):`;
      for (const stale of localeResult.stale) {
        logger.info`  - ${stale.key}: previously "${stale.previous}"`;
      }
    }

    if (localeResult.extra.length > 0) {
      logger.info`[${localeResult.locale}] Extra translations (not in source):`;
      for (const extra of localeResult.extra) {
        logger.info`  - ${extra}`;
      }
    }

    if (localeResult.missing.length === 0 && localeResult.stale.length === 0) {
      logger.info`[${localeResult.locale}] All translations complete`;
    }
  }

  process.exit(result.hasIssues ? 10 : 0);
}

export const checkCommand = command(
  {
    name: "check",
    flags: {
      config: {
        type: String,
        default: "intl-ai.config.json",
        description: "Path to JSON config file",
      },
      locale: {
        type: String,
        description: "Check a specific locale only",
      },
    },
  },
  (argv) => {
    const { config, locale } = argv.flags;
    runCheckCommand({ config: config ?? "intl-ai.config.json", locale }).catch((e: unknown) => {
      process.stderr.write(`[intl-ai] Fatal: ${e instanceof Error ? e.message : String(e)}\n`);
      process.exit(1);
    });
  },
);
