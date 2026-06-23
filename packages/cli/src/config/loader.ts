import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { createJiti } from "jiti";
import {
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  type IntlAiJsonConfig,
} from "@intl-ai/api/internal";
import type { IntlAiConfig } from "@intl-ai/api";

export interface LoadConfigOptions {
  validate?: boolean;
}

/**
 * Load an `IntlAiConfig` from a `.json` or `.ts` config file.
 *
 * Only `intl-ai.config.ts` and `intl-ai.config.json` are supported.
 * The format is inferred from the file extension.
 */
export async function loadConfig(
  path: string,
  options: LoadConfigOptions = {},
): Promise<IntlAiConfig> {
  if (!existsSync(path)) {
    throw new Error(`Config file not found: ${path}`);
  }

  const isTs = path.endsWith(".ts");
  const raw = isTs ? await loadTsConfig(path) : await loadJsonConfigFile(path);

  if (options.validate !== false) {
    validateJsonConfig(raw);
  }

  return jsonConfigToIntlAiConfig(raw as IntlAiJsonConfig);
}

async function loadJsonConfigFile(path: string): Promise<unknown> {
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
  const j = createJiti(path, { interopDefault: true });
  const mod = (await j.import(path, { default: true })) as { default?: unknown } | unknown;
  if (mod && typeof mod === "object" && "default" in (mod as object)) {
    return (mod as { default: unknown }).default;
  }
  return mod;
}

export function validateJsonConfig(json: unknown): asserts json is IntlAiJsonConfig {
  const result = IntlAiJsonConfigSchema.safeParse(json);
  if (!result.success) {
    const errors = result.error.issues
      .map((issue) => {
        const p = issue.path.length > 0 ? issue.path.join(".") : "/";
        return `${p}: ${issue.message}`;
      })
      .join("\n  ");
    throw new Error(`Config validation failed:\n  ${errors}`);
  }
}
