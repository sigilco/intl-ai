import { createUnplugin } from "unplugin";
import type { UnpluginFactory } from "unplugin";
import { runIntlAiPipeline, shouldSkipInDev } from "./run.js";
import type { IntlAiPluginOptions } from "./run.js";

export type { IntlAiPluginOptions } from "./run.js";
export { resolveIntlAiBin, runIntlAiPipeline } from "./run.js";

/** @deprecated Use {@link IntlAiPluginOptions}. */
export type UnpluginIntlAiOptions = IntlAiPluginOptions;

const unpluginFactory: UnpluginFactory<IntlAiPluginOptions | undefined> = (options) => {
  const o = options ?? {};
  return {
    name: "@intl-ai/unplugin",
    async buildStart() {
      if (shouldSkipInDev(o)) {
        console.warn("@intl-ai/unplugin: skipped in dev (pass dev: true to enable)");
        return;
      }
      await runIntlAiPipeline(o, o.cwd ?? process.cwd());
    },
  };
};

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);
export default unplugin;
