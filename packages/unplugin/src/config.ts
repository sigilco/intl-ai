import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { resolve as resolvePath } from "pathe";
import { createJiti } from "jiti";
import {
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  type IntlAiJsonConfig,
} from "@intl-ai/api/internal";
import type { IntlAiConfig } from "@intl-ai/api";

const CONFIG_FILE_NAMES = ["intl-ai.config.ts", "intl-ai.config.json"];

/**
 * Find the nearest `intl-ai.config.ts` or `intl-ai.config.json` in `cwd`.
 *
 * Only these two filenames are supported. The runtime-agnostic refactor
 * dropped the legacy `.js`, `.mjs`, `.cjs`, and `.intl-airc` filenames.
 */
async function findConfigFile(cwd: string): Promise<string | null> {
  for (const name of CONFIG_FILE_NAMES) {
    const p = resolvePath(cwd, name);
    if (existsSync(p)) return p;
  }
  return null;
}

/**
 * Load an `IntlAiConfig` from the project root.
 *
 * Searches for `intl-ai.config.ts` then `intl-ai.config.json`. The format
 * is inferred from the file extension. Throws if no config is found.
 */
export async function loadConfig(cwd: string = process.cwd()): Promise<IntlAiConfig> {
  const configPath = await findConfigFile(cwd);
  if (!configPath) {
    throw new Error(
      `No intl-ai config found in ${cwd}. Expected one of: ${CONFIG_FILE_NAMES.join(", ")}`,
    );
  }
  return loadConfigFile(configPath);
}

async function loadConfigFile(configPath: string): Promise<IntlAiConfig> {
  const raw = configPath.endsWith(".ts")
    ? await loadTsConfig(configPath)
    : await loadJsonConfig(configPath);

  const parsed = IntlAiJsonConfigSchema.safeParse(raw);
  if (!parsed.success) {
    throw new Error(
      `Invalid config at ${configPath}: ${parsed.error.issues.map((i) => i.message).join(", ")}`,
    );
  }
  return jsonConfigToIntlAiConfig(parsed.data as IntlAiJsonConfig);
}

async function loadJsonConfig(path: string): Promise<unknown> {
  const content = await readFile(path, "utf-8");
  try {
    return JSON.parse(content);
  } catch (error) {
    throw new Error(
      `Invalid JSON in config file ${path}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

async function loadTsConfig(path: string): Promise<unknown> {
  const j = createJiti(process.cwd(), { interopDefault: true });
  const mod = (await j.import(path, { default: true })) as { default?: unknown } | unknown;
  if (mod && typeof mod === "object" && "default" in (mod as object)) {
    return (mod as { default: unknown }).default;
  }
  return mod;
}
