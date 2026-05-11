import { existsSync } from "fs";
import { join } from "path";
import type { IntlAiConfig } from "../types";
import { IntlAiConfigSchema } from "../types";

const CONFIG_FILES = ["intl-ai.config.ts", "intl-ai.config.js", ".intl-airc"];

export async function loadConfig(cwd: string = process.cwd()): Promise<IntlAiConfig> {
  const configPath = findConfigFile(cwd);
  if (!configPath) {
    throw new Error("No intl-ai config file found");
  }

  const config = await loadConfigFile(configPath);
  return IntlAiConfigSchema.parse(config);
}

function findConfigFile(cwd: string): string | null {
  for (const file of CONFIG_FILES) {
    const path = join(cwd, file);
    if (existsSync(path)) return path;
  }
  return null;
}

async function loadConfigFile(path: string): Promise<unknown> {
  const jiti = (await import("jiti")).default;
  const loader = jiti(path);
  const config = loader(path);
  return config?.default || config;
}
