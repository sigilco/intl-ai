import type { NextConfig } from "next";
import type { IntlAiConfig } from "@intl-ai/core";
import type { Compiler } from "webpack";
import type { Compilation } from "webpack";
import {
  loadConfig,
  findMissingTranslations,
  translateBatch,
  LockfileManager,
  readJsonFile,
  writeJsonFile,
} from "@intl-ai/core";
import { join, dirname } from "path";
import { existsSync, mkdirSync } from "fs";
import { fileURLToPath } from "url";

export interface IntlAiNextOptions extends Partial<IntlAiConfig> {
  debug?: boolean;
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

async function runFill(options?: IntlAiNextOptions): Promise<void> {
  try {
    const config = await loadConfig();
    const { defaultLocale, locales, localeDir, model, glossary, maxRetries } = config;

    if (!localeDir) {
      if (options?.debug) console.log("[intl-ai] No localeDir configured, skipping");
      return;
    }

    const lockfileManager = new LockfileManager(localeDir);
    const sourceLocalePath = join(localeDir, `${defaultLocale}.json`);
    const sourceLocaleData = await readJsonFile(sourceLocalePath);

    for (const targetLocale of locales) {
      if (targetLocale === defaultLocale) continue;

      try {
        const targetLocalePath = join(localeDir, `${targetLocale}.json`);
        let targetLocaleData: Record<string, unknown> = {};

        // Use existsSync guard before readJsonFile
        if (existsSync(targetLocalePath)) {
          targetLocaleData = readJsonFile(targetLocalePath);
        }

        // Build lockfileEntries map filtered to current locale (from fill.ts pattern)
        const lockfileEntries = new Map<string, { sourceHash: string }>();
        const allEntries = lockfileManager.getAllEntries();
        for (const [key, entry] of Object.entries(allEntries)) {
          const [entryLocale, entryKey] = key.split(":");
          if (entryLocale === targetLocale) {
            lockfileEntries.set(entryKey, { sourceHash: entry.sourceHash });
          }
        }

        const diff = findMissingTranslations({
          sourceLocale: sourceLocaleData,
          targetLocale: targetLocaleData,
          locale: targetLocale,
          lockfileEntries,
        });

        if (diff.missing.length === 0 && diff.stale.length === 0) {
          if (options?.debug) console.log(`[intl-ai] ${targetLocale}: up to date`);
          continue;
        }

        if (options?.debug) {
          console.log(
            `[intl-ai] Translating ${diff.missing.length + diff.stale.length} entries for ${targetLocale}`,
          );
        }

        // Build entriesToTranslate from missing + stale
        const entriesToTranslate: Array<{ key: string; source: string }> = [
          ...diff.missing,
          ...diff.stale.map((s) => ({
            key: s.key,
            source: getNestedValue(sourceLocaleData, s.key),
          })),
        ].filter((e) => e.source !== undefined) as Array<{
          key: string;
          source: string;
        }>;

        const results = await translateBatch({
          model,
          entries: entriesToTranslate.map((e) => ({
            key: e.key,
            source: e.source,
          })),
          targetLocale,
          sourceLocale: defaultLocale,
          glossary,
          maxRetries,
        });

        // Apply translations
        for (const result of results) {
          if (result.success) {
            setNestedValue(targetLocaleData, result.key, result.translated);

            const source = entriesToTranslate.find((e) => e.key === result.key)?.source || "";
            lockfileManager.setEntry(result.key, targetLocale, {
              key: result.key,
              locale: targetLocale,
              sourceHash: lockfileManager.hashSource(source),
              translated: result.translated,
              origin: "ai",
              model: config.model.toString(),
              timestamp: new Date().toISOString(),
            });
          }
        }

        // Write locale file and lockfile
        mkdirSync(localeDir, { recursive: true });
        writeJsonFile(targetLocalePath, targetLocaleData);
        lockfileManager.save();

        if (options?.debug) {
          const successful = results.filter((r) => r.success).length;
          console.log(
            `[intl-ai] Translated ${successful}/${results.length} entries for ${targetLocale}`,
          );
        }
      } catch (localeError) {
        console.warn(`[intl-ai] Error processing locale ${targetLocale}:`, localeError);
        // Continue to next locale - don't throw
      }
    }
  } catch (error) {
    console.warn("[intl-ai] Translation startup error:", error);
    // Don't throw - let build continue
  }
}

function getNestedValue(obj: Record<string, unknown>, path: string): string {
  const parts = path.split(".");
  let current: unknown = obj;
  for (const part of parts) {
    if (current && typeof current === "object") {
      current = (current as Record<string, unknown>)[part];
    } else {
      return "";
    }
  }
  return String(current || "");
}

class IntlAiWebpackPlugin {
  name = "intl-ai-webpack-plugin";
  options: IntlAiNextOptions;

  constructor(options?: IntlAiNextOptions) {
    this.options = options || {};
  }

  apply(compiler: Compiler) {
    compiler.hooks.environment.tap("intl-ai", async () => {
      if (this.options.debug) {
        console.log("[intl-ai] Webpack environment hook triggered");
      }
    });

    compiler.hooks.emit.tapPromise("intl-ai", async (compilation: Compilation) => {
      try {
        if (this.options.debug) {
          console.log("[intl-ai] Running translations during build");
        }

        const config = await loadConfig();
        const { defaultLocale, locales, localeDir, model, glossary, maxRetries } = config;

        if (!localeDir) {
          console.warn("[intl-ai] No localeDir configured, skipping translations");
          return;
        }

        if (this.options.debug) {
          console.log(
            `[intl-ai] Config: defaultLocale=${defaultLocale}, locales=${locales.join(", ")}`,
          );
        }

        const lockfileManager = new LockfileManager(localeDir);

        const sourceLocalePath = join(localeDir, `${defaultLocale}.json`);
        const sourceLocaleData = await readJsonFile(sourceLocalePath);

        for (const targetLocale of locales) {
          if (targetLocale === defaultLocale) continue;

          if (this.options.debug) {
            console.log(`[intl-ai] Processing locale: ${targetLocale}`);
          }

          // Build lockfileEntries map filtered to current locale (from fill.ts pattern)
          const lockfileEntries = new Map<string, { sourceHash: string }>();
          const allEntries = lockfileManager.getAllEntries();
          for (const [key, entry] of Object.entries(allEntries)) {
            const [entryLocale, entryKey] = key.split(":");
            if (entryLocale === targetLocale) {
              lockfileEntries.set(entryKey, { sourceHash: entry.sourceHash });
            }
          }

          const targetLocalePath = join(localeDir, `${targetLocale}.json`);
          const targetLocaleData = await readJsonFile(targetLocalePath);

          const diff = findMissingTranslations({
            sourceLocale: sourceLocaleData,
            targetLocale: targetLocaleData,
            locale: targetLocale,
            lockfileEntries,
          });

          if (this.options.debug) {
            console.log(
              `[intl-ai] Found ${diff.missing.length} missing, ${diff.stale.length} stale translations for ${targetLocale}`,
            );
          }

          if (diff.missing.length === 0 && diff.stale.length === 0) {
            continue;
          }

          const entriesToTranslate = [
            ...diff.missing,
            ...diff.stale.map((s) => ({
              key: s.key,
              source: this.getNestedValue(sourceLocaleData, s.key),
            })),
          ];

          const results = await translateBatch({
            model,
            entries: entriesToTranslate.map((e) => ({
              key: e.key,
              source: e.source,
            })),
            targetLocale,
            sourceLocale: defaultLocale,
            glossary,
            maxRetries,
          });

          for (const result of results) {
            if (result.success) {
              const source = entriesToTranslate.find((e) => e.key === result.key)?.source || "";
              lockfileManager.setEntry(result.key, targetLocale, {
                key: result.key,
                locale: targetLocale,
                sourceHash: lockfileManager.hashSource(source),
                translated: result.translated,
                origin: "ai",
                model: config.model.toString(),
                timestamp: new Date().toISOString(),
              });
            }
          }

          if (this.options.debug) {
            const successful = results.filter((r) => r.success).length;
            console.log(
              `[intl-ai] Translated ${successful}/${results.length} entries for ${targetLocale}`,
            );
          }
        }

        lockfileManager.save();

        if (this.options.debug) {
          console.log("[intl-ai] Translation process complete");
        }
      } catch (error) {
        console.warn("[intl-ai] Translation error:", error);
      }
    });
  }

  private getNestedValue(obj: Record<string, unknown>, path: string): string {
    const parts = path.split(".");
    let current: unknown = obj;

    for (const part of parts) {
      if (current && typeof current === "object") {
        current = (current as Record<string, unknown>)[part];
      } else {
        return "";
      }
    }

    return String(current || "");
  }
}

export function withIntlAi(options?: IntlAiNextOptions) {
  return async (
    nextConfig?: NextConfig,
  ): Promise<NextConfig> => {
    // Run fill before webpack starts
    try {
      await runFill(options);
    } catch (error) {
      console.warn("[intl-ai] Translation startup error (build continues):", error);
    }

    return addIntlAiToConfig(nextConfig, options);
  };
}

function addIntlAiToConfig(
  nextConfig?: NextConfig | undefined,
  options?: IntlAiNextOptions,
): NextConfig {
  // Resolve loader path dynamically to work in both TS and compiled contexts
  const loaderPath = (() => {
    try {
      // Try require.resolve first (works in compiled JS)
      return require.resolve("./next-loader");
    } catch {
      // Fallback for TS/ESM context
      const __filename = fileURLToPath(import.meta.url);
      const __dirname = dirname(__filename);
      return join(__dirname, "next-loader.ts");
    }
  })();

  return {
    ...nextConfig,
    turbopack: {
      ...(nextConfig as any)?.turbopack,
      rules: {
        "*.locale.json": {
          loaders: [
            {
              loader: loaderPath,
              options: { debug: options?.debug ?? false },
            },
          ],
          as: "*.js",
        },
        ...((nextConfig as any)?.turbopack?.rules ?? {}),
      },
    },
    webpack: (config: any, context: any) => {
      if (typeof nextConfig?.webpack === "function") {
        config = nextConfig.webpack(config, context);
      }

      config.plugins = config.plugins || [];
      config.plugins.push(new IntlAiWebpackPlugin(options));

      if (options?.debug) {
        console.log("[intl-ai] Webpack config modified");
      }

      return config;
    },
  };
}

export default withIntlAi;
