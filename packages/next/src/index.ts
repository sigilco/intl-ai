import type { NextConfig } from "next";
import type { IntlAiConfig } from "@intl-ai/api";
import intlAiUnplugin from "@intl-ai/unplugin/webpack";
import { join, dirname } from "node:path";
import { fileURLToPath } from "url";

export interface IntlAiNextOptions extends Partial<IntlAiConfig> {
  debug?: boolean;
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
      // Use the @intl-ai/unplugin webpack adapter — single buildStart hook
      // does loadConfig() + runFill() once per build. Replaces the local
      // IntlAiWebpackPlugin and the eager runStartup() both of which were
      // calling runFill() a second time.
      config.plugins.push(intlAiUnplugin({ debug: options?.debug ?? false }));
      return config;
    },
  };
}

export default withIntlAi;
