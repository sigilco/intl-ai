import { createUnplugin } from "unplugin";
import type { UnpluginFactory } from "unplugin";
import { loadConfig } from "./config";
import type { QualityOptions } from "@intl-ai/api";

export interface UnpluginIntlAiOptions {
  debug?: boolean;
  /**
   * Enable the quality-aware fill loop. When `true`, `runFill` is called
   * with a `quality` block that merges `intl-ai.config.json` settings
   * (threshold, maxRetries) and forces `failOnLowQuality: true` so any
   * unresolved low-quality key fails the build.
   */
  quality?: boolean;
}

const unpluginFactory: UnpluginFactory<UnpluginIntlAiOptions | undefined> = (options) => {
  const enableQuality = options?.quality === true;
  return {
    name: "@intl-ai/unplugin",
    async buildStart() {
      try {
        const { runFill } = await import("@intl-ai/api");
        const config = await loadConfig();
        const quality: QualityOptions | undefined = enableQuality
          ? { ...config.quality, failOnLowQuality: true }
          : undefined;
        await runFill(config, quality ? { quality } : undefined);
      } catch (error) {
        console.warn(`[intl-ai] Skipping translation fill due to error: ${error}`);
      }
    },
  };
};

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);
export default unplugin;
