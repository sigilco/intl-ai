import type { NextConfig } from "next";
import type { IntlAiConfig } from "@intl-ai/api";
import type { Compiler } from "webpack";
import { runFill } from "@intl-ai/api";
import { loadConfig } from "./config";
import { join, dirname } from "node:path";
import { fileURLToPath } from "url";

export interface IntlAiNextOptions extends Partial<IntlAiConfig> {
  debug?: boolean;
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

    compiler.hooks.emit.tapPromise("intl-ai", async () => {
      try {
        if (this.options.debug) {
          console.log("[intl-ai] Running translations during webpack build");
        }
        const config = await loadConfig();
        await runFill(config);
        if (this.options.debug) {
          console.log("[intl-ai] Translation process complete");
        }
      } catch (error) {
        console.warn("[intl-ai] Translation error:", error);
      }
    });
  }
}

export function withIntlAi(options?: IntlAiNextOptions) {
  async function runStartup(): Promise<void> {
    try {
      const config = await loadConfig();
      await runFill(config);
    } catch (error) {
      console.warn("[intl-ai] Translation startup error (build continues):", error);
    }
  }

  return async (
    nextConfig?: NextConfig | (() => NextConfig | Promise<NextConfig>),
  ): Promise<NextConfig> => {
    await runStartup();

    const resolved = typeof nextConfig === "function" ? await nextConfig() : nextConfig;
    return addIntlAiToConfig(resolved, options);
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
