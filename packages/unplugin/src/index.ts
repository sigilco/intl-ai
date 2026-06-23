import { createUnplugin } from "unplugin";
import type { UnpluginFactory } from "unplugin";
import { loadConfig } from "./config";

export interface UnpluginIntlAiOptions {
  debug?: boolean;
}

const unpluginFactory: UnpluginFactory<UnpluginIntlAiOptions | undefined> = (options) => {
  return {
    name: "@intl-ai/unplugin",
    async buildStart() {
      try {
        const { runFill } = await import("@intl-ai/api");
        const config = await loadConfig();
        await runFill(config);
      } catch (error) {
        console.warn(`[intl-ai] Skipping translation fill due to error: ${error}`);
      }
    },
  };
};

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);
export default unplugin;
