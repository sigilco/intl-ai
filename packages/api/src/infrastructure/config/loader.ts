import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { resolve as resolvePath } from "pathe";
import { createJiti } from "jiti";
import {
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  type IntlAiJsonConfig,
} from "../../schema/index";
import type { IntlAiConfig } from "../../types";

export type { IntlAiConfig };

const CONFIG_FILE_NAMES = ["intl-ai.config.ts", "intl-ai.config.json"] as const;

async function findConfigFile(cwd: string): Promise<string | null> {
  for (const name of CONFIG_FILE_NAMES) {
    const p = resolvePath(cwd, name);
    if (existsSync(p)) return p;
  }
  return null;
}

async function loadJsonRaw(path: string): Promise<unknown> {
  const content = await readFile(path, "utf-8");
  try {
    return JSON.parse(content);
  } catch (error) {
    throw new Error(
      `Invalid JSON in config file ${path}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

async function loadTsRaw(path: string): Promise<unknown> {
  const j = createJiti(process.cwd(), { interopDefault: true });
  const mod = (await j.import(path, { default: true })) as { default?: unknown } | unknown;
  if (mod && typeof mod === "object" && "default" in (mod as object)) {
    return (mod as { default: unknown }).default;
  }
  return mod;
}

function parseAndConvert(raw: unknown, sourcePath: string): IntlAiConfig {
  const parsed = IntlAiJsonConfigSchema.safeParse(raw);
  if (!parsed.success) {
    const errors = parsed.error.issues
      .map((issue) => {
        const p = issue.path.length > 0 ? issue.path.join(".") : "/";
        return `${p}: ${issue.message}`;
      })
      .join("\n  ");
    throw new Error(`Config validation failed (${sourcePath}):\n  ${errors}`);
  }
  return jsonConfigToIntlAiConfig(parsed.data as IntlAiJsonConfig);
}

/**
 * Load config from an explicit file path (`.ts` or `.json`).
 * Skips schema validation when `validate` is false (useful in tests).
 */
export async function loadConfigFromPath(
  path: string,
  options: { validate?: boolean } = {},
): Promise<IntlAiConfig> {
  if (!existsSync(path)) {
    throw new Error(`Config file not found: ${path}`);
  }
  const raw = path.endsWith(".ts") ? await loadTsRaw(path) : await loadJsonRaw(path);
  if (options.validate === false) {
    return jsonConfigToIntlAiConfig(raw as IntlAiJsonConfig);
  }
  return parseAndConvert(raw, path);
}

/**
 * Search `cwd` for `intl-ai.config.ts` or `intl-ai.config.json` and load it.
 * Throws if no config is found.
 */
export async function loadConfig(cwd: string = process.cwd()): Promise<IntlAiConfig> {
  const configPath = await findConfigFile(cwd);
  if (!configPath) {
    throw new Error(
      `No intl-ai config found in ${cwd}. Expected one of: ${CONFIG_FILE_NAMES.join(", ")}`,
    );
  }
  const raw = configPath.endsWith(".ts") ? await loadTsRaw(configPath) : await loadJsonRaw(configPath);
  return parseAndConvert(raw, configPath);
}
