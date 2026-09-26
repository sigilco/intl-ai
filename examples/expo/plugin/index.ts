import { ConfigPlugin, createRunOncePlugin, withDangerousMod } from "@expo/config-plugins";
import { execSync } from "node:child_process";
import { join } from "pathe";

export interface IntlAiExpoOptions {
  /** Path to the intl-ai JSON config file, relative to the project root. */
  configPath?: string;
  /** When true, prints the CLI output instead of swallowing it. */
  verbose?: boolean;
  /** When true, do not fail the prebuild if the CLI exits non-zero. */
  continueOnError?: boolean;
}

const DEFAULT_CONFIG_PATH = "intl-ai.config.json";

const withIntlAiInternal: ConfigPlugin<IntlAiExpoOptions | void> = (config, rawOptions) => {
  const options: IntlAiExpoOptions = rawOptions ?? {};
  const configPath = options.configPath ?? DEFAULT_CONFIG_PATH;

  return withDangerousMod(config, [
    "ios",
    async (config) => {
      await runFill({
        configPath,
        projectRoot: config.modRequest.projectRoot,
        verbose: options.verbose,
        continueOnError: options.continueOnError,
      });
      return config;
    },
  ]);
};

interface RunFillOptions extends IntlAiExpoOptions {
  configPath: string;
  projectRoot: string;
}

async function runFill(options: RunFillOptions): Promise<void> {
  const absoluteConfigPath = join(options.projectRoot, options.configPath);
  const command = `npx intl-ai fill --config "${absoluteConfigPath}"`;

  try {
    execSync(command, {
      cwd: options.projectRoot,
      stdio: options.verbose ? "inherit" : "pipe",
      encoding: "utf-8",
    });
  } catch (error) {
    if (options.continueOnError) {
      // eslint-disable-next-line no-console
      console.warn(
        `[intl-ai] Translation step failed but continueOnError is enabled: ${error instanceof Error ? error.message : String(error)}`,
      );
      return;
    }
    throw error;
  }
}

export const withIntlAi = createRunOncePlugin(withIntlAiInternal, "intl-ai-expo-example", "0.2.0");

export default withIntlAi;
