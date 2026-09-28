import type { ConfigPlugin, ExportedConfigWithProps } from "@expo/config-plugins";
import { createRunOncePlugin, withDangerousMod } from "@expo/config-plugins";
import type { IntlAiPluginOptions } from "@intl-ai/unplugin/runner";
import { runIntlAiPipeline } from "@intl-ai/unplugin/runner";

export type IntlAiExpoOptions = IntlAiPluginOptions;

/**
 * Runs `intl-ai fill` (+ the optional `check --fail-on` gate) while Expo
 * evaluates config plugins during `expo prebuild`, `eas build`, and
 * `expo run:*`. Registered on both platforms with a run-once guard so the
 * pipeline fires exactly once whichever platform evaluates first.
 */
const withIntlAiPlugin: ConfigPlugin<IntlAiExpoOptions | void> = (config, rawOptions) => {
  const options: IntlAiExpoOptions = rawOptions ?? {};
  let ran = false;

  const runOnce = async (
    modConfig: ExportedConfigWithProps<unknown>,
  ): Promise<ExportedConfigWithProps<unknown>> => {
    if (ran) return modConfig;
    ran = true;
    if (options.dev === false && process.env.NODE_ENV !== "production") return modConfig;
    await runIntlAiPipeline(options, options.cwd ?? modConfig.modRequest.projectRoot);
    return modConfig;
  };

  config = withDangerousMod(config, ["ios", runOnce]);
  config = withDangerousMod(config, ["android", runOnce]);
  return config;
};

export const withIntlAi = createRunOncePlugin(withIntlAiPlugin, "@intl-ai/expo");

export default withIntlAi;
