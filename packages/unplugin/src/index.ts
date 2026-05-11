import { createUnplugin } from "unplugin";
import type { UnpluginFactory } from "unplugin";
import { runFill, loadConfig, type IntlAiConfig } from "@intl-ai/core";

export interface UnpluginIntlAiOptions {
  debug?: boolean;
}

const unpluginFactory: UnpluginFactory<UnpluginIntlAiOptions | undefined> = (options) => {
  return {
    name: "unplugin-intl-ai",
    async buildStart() {
      const config = await loadConfig();
      if (config) {
        await runFill({ debug: options?.debug ?? false });
      }
    },
  };
};

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);
export default unplugin;
