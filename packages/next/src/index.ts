import type { NextConfig } from "next";
import intlAiUnplugin from "@intl-ai/unplugin/webpack";
import { join, dirname } from "node:path";
import { fileURLToPath } from "url";

export interface IntlAiNextOptions {
  debug?: boolean;
  /**
   * Forwarded to the underlying unplugin. When `true`, the fill-time
   * validation gate runs during `next build` and unresolved keys fail
   * the build. Check selection and thresholds come from `intl-ai.toml`.
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
      // The unplugin webpack adapter spawns `intl-ai fill` once per
      // build; `quality` maps to the fill-time validation gate.
      config.plugins.push(intlAiUnplugin({ validate: quality || undefined }));
      return config;
    },
  };
}

export default withIntlAi;
