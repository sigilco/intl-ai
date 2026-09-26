import type { NextConfig } from "next";
import type { IntlAiConfig } from "@intl-ai/api";
import intlAiUnplugin from "@intl-ai/unplugin/webpack";
import { join, dirname } from "node:path";
import { fileURLToPath } from "url";

export interface IntlAiNextOptions extends Omit<Partial<IntlAiConfig>, "quality"> {
  debug?: boolean;
  /**
   * Forwarded to the underlying unplugin. When `true`, the quality-aware
   * fill loop runs during `next build` and a build that contains keys
   * still below the quality threshold fails the build. Threshold and
   * `maxRetries` come from `intl-ai.config.json`; this option only
   * enables or disables the loop.
   */
  quality?: boolean;
}

export function withIntlAi(options?: IntlAiNextOptions) {
  return async (
    nextConfig?: NextConfig | (() => NextConfig | Promise<NextConfig>),
  ): Promise<NextConfig> => {
    const resolved = typeof nextConfig === "function" ? await nextConfig() : nextConfig;
    return addIntlAiToConfig(resolved, options);
  };
}

function addIntlAiToConfig(
  nextConfig?: NextConfig | undefined,
  options?: IntlAiNextOptions,
): NextConfig {
  // Resolve loader path dynamically to work in both compiled-JS and TS contexts
  const loaderPath = (() => {
    try {
      return require.resolve("./next-loader");
    } catch {
      const __filename = fileURLToPath(import.meta.url);
      const __dirname = dirname(__filename);
      return join(__dirname, "next-loader.ts");
    }
  })();

  const debug = options?.debug ?? false;
  const quality = options?.quality === true;

  return {
    ...nextConfig,
    turbopack: {
      ...(nextConfig as any)?.turbopack,
      rules: {
        "*.locale.json": {
          loaders: [
            {
              loader: loaderPath,
              options: { debug },
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
      // Use the @intl-ai/unplugin webpack adapter — single buildStart hook
      // does loadConfig() + runFill() once per build. Replaces the local
      // IntlAiWebpackPlugin and the eager runStartup() both of which were
      // calling runFill() a second time.
      config.plugins.push(intlAiUnplugin({ debug, quality }));
      return config;
    },
  };
}

export default withIntlAi;
