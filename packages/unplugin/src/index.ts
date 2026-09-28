import { createUnplugin } from "unplugin";
import type { UnpluginFactory } from "unplugin";
import {
  buildCheckArgs,
  buildFillArgs,
  normalizeFailOn,
  resolveIntlAiBin,
  runIntlAi,
  shouldSkipInDev,
} from "./run.js";
import type { IntlAiPluginOptions, ResolvedBin } from "./run.js";

export type { IntlAiPluginOptions } from "./run.js";
export { resolveIntlAiBin } from "./run.js";

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
      const cwd = o.cwd ?? process.cwd();
      const strict = o.strict !== false;
      const fail = (message: string) => {
        if (strict) {
          throw new Error(`@intl-ai/unplugin: ${message}`);
        }
        console.warn(`@intl-ai/unplugin: ${message}`);
      };
      let bin: ResolvedBin;
      try {
        bin = resolveIntlAiBin(o.bin, cwd);
      } catch (error) {
        fail((error as Error).message);
        return;
      }
      if (o.fill !== false) {
        let code: number;
        try {
          code = await runIntlAi(bin, buildFillArgs(o), cwd);
        } catch (error) {
          fail(`intl-ai fill failed to start: ${(error as Error).message}`);
          return;
        }
        if (code !== 0) {
          fail(`intl-ai fill exited with code ${code}`);
          return;
        }
      }
      const failOn = normalizeFailOn(o.failOn);
      if (failOn.length) {
        let code: number;
        try {
          code = await runIntlAi(bin, buildCheckArgs(o, failOn), cwd);
        } catch (error) {
          fail(`intl-ai check failed to start: ${(error as Error).message}`);
          return;
        }
        if (code !== 0) {
          fail(`intl-ai check --fail-on ${failOn.join(",")} exited with code ${code}`);
          return;
        }
      }
    },
  };
};

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);
export default unplugin;
