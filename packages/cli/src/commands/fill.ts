import { command } from "cleye";
import { batchedFill } from "@intl-ai/api/internal";
import { loadConfig } from "../config/loader";
import { configureLogger, logger } from "../logger";
import { join as pathJoin } from "pathe";

export async function runFillCommand(opts: {
  config: string;
  locale?: string;
  force: boolean;
  silent: boolean;
  dryRun: boolean;
  concurrency?: number;
}): Promise<void> {
  const { config: configPath, locale, force, silent, dryRun, concurrency = 4 } = opts;
  await configureLogger(silent);
  const config = await loadConfig(configPath);

  logger.info`Translating missing keys...`;

  const failureCounts = new Map<string, number>();

  try {
    const result = await batchedFill(config, {
      locale,
      force,
      dryRun,
      concurrency,
      onProgress: ({ locale: loc, completed, total }) => {
        if (!silent) {
          process.stdout.write(`\r[${loc}] ${completed}/${total} keys`);
        }
      },
      hook: {
        onRequest: ({ locale: loc, entryCount }) => {
          if (!silent) {
            process.stderr.write(`[${loc}] Requesting ${entryCount} keys...\n`);
          }
        },
        onSuccess: ({ locale: loc, results, durationMs }) => {
          const ok = results.filter((r) => r.success).length;
          const fail = results.filter((r) => !r.success).length;
          if (fail > 0) {
            for (const r of results.filter((r) => !r.success)) {
              const et = (r as { errorType?: string }).errorType ?? "unknown";
              failureCounts.set(et, (failureCounts.get(et) ?? 0) + 1);
            }
            if (!silent) {
              process.stderr.write(
                ` [${loc}] ok: ${ok} | fail: ${fail} (${[...failureCounts.entries()]
                  .map(([k, v]) => `${v} ${k}`)
                  .join(", ")})\n`,
              );
            }
          } else if (!silent) {
            process.stderr.write(` [${loc}] ok: ${ok}/${results.length} (${durationMs}ms)\n`);
          }
        },
        onAttemptFailure: ({ locale: loc, errorType, error, attempt, maxRetries, durationMs }) => {
          failureCounts.set(errorType, (failureCounts.get(errorType) ?? 0) + 1);
          if (!silent) {
            process.stderr.write(
              ` [${loc}] Attempt ${attempt}/${maxRetries} failed (${errorType}, ${durationMs}ms): ${error}\n`,
            );
          }
        },
        onError: ({ locale: loc, errorType, error }) => {
          failureCounts.set(errorType, (failureCounts.get(errorType) ?? 0) + 1);
          if (!silent) {
            process.stderr.write(` [${loc}] All retries exhausted (${errorType}): ${error}\n`);
          }
        },
      },
    });

    if (!silent && result.translated > 0) {
      process.stdout.write("\n");
    }

    logger.info`Translated ${result.translated} keys across ${result.locales.join(", ")}`;

    if (result.skipped > 0) {
      logger.info`${result.skipped} keys already up to date`;
    }
    if (result.errors > 0) {
      const breakdown = [...failureCounts.entries()].map(([k, v]) => `  ${v}x ${k}`).join("\n");
      process.stderr.write(`\n[intl-ai] ${result.errors} failure(s):\n${breakdown}\n`);
      process.stderr.write(`See ${pathJoin(config.localeDir!, ".intl-ai")} for full report.\n`);
    }
    if (dryRun) {
      logger.info`Dry-run: no files were modified`;
    }

    if (result.errors > 0) {
      process.exit(1);
    }
  } catch (error) {
    logger.error`Translation failed: ${error instanceof Error ? error.message : String(error)}`;
    process.exit(1);
  }
}

export const fillCommand = command(
  {
    name: "fill",
    flags: {
      config: {
        type: String,
        default: "intl-ai.config.json",
        description: "Path to JSON config file",
      },
      locale: {
        type: String,
        description: "Target a specific locale only",
      },
      force: {
        type: Boolean,
        default: false,
        description: "Overwrite human-edited entries",
      },
      silent: {
        type: Boolean,
        default: false,
        description: "Suppress all output except errors",
      },
      dryRun: {
        type: Boolean,
        default: false,
        description: "Preview changes without modifying files",
      },
      concurrency: {
        type: Number,
        default: 4,
        description: "Max parallel locale processing",
      },
    },
  },
  (argv) => {
    const { config, locale, force, silent, dryRun, concurrency } = argv.flags;
    runFillCommand({
      config: config ?? "intl-ai.config.json",
      locale,
      force: force ?? false,
      silent: silent ?? false,
      dryRun: dryRun ?? false,
      concurrency: concurrency ?? 4,
    }).catch((e: unknown) => {
      process.stderr.write(`[intl-ai] Fatal: ${e instanceof Error ? e.message : String(e)}\n`);
      process.exit(1);
    });
  },
);
