import { createUnplugin } from "unplugin";
import type { UnpluginFactory } from "unplugin";

export interface UnpluginIntlAiOptions {
  debug?: boolean;
}

const unpluginFactory: UnpluginFactory<UnpluginIntlAiOptions | undefined> = (options) => {
  return {
    name: "unplugin-intl-ai",
    async buildStart() {
      try {
        const { runFill, loadConfig } = await import("@intl-ai/core");
        const config = await loadConfig();
        if (config) {
          await runFill({ debug: options?.debug ?? false });
        }
      } catch (error) {
        console.warn(`[intl-ai] Skipping translation fill due to error: ${error}`);
      }
    },
  };
};

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);
export default unplugin;
