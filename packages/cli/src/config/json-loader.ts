import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import {
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  type IntlAiJsonConfig,
} from "@intl-ai/api/internal";
import type { IntlAiConfig } from "@intl-ai/api";

export interface LoadJsonConfigOptions {
  validate?: boolean;
}

export async function loadJsonConfig(
  path: string,
  options: LoadJsonConfigOptions = {},
): Promise<IntlAiConfig> {
  const { validate = true } = options;

  if (!existsSync(path)) {
    throw new Error(`Config file not found: ${path}`);
  }

  const content = await readFile(path, "utf-8");
  let json: unknown;
  try {
    json = JSON.parse(content);
  } catch (error) {
    throw new Error(
      `Invalid JSON in config file ${path}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }

  if (validate) {
    validateJsonConfig(json);
  }

  return jsonConfigToIntlAiConfig(json as IntlAiJsonConfig);
}

export function validateJsonConfig(json: unknown): asserts json is IntlAiJsonConfig {
  const result = IntlAiJsonConfigSchema.safeParse(json);
  if (!result.success) {
    const errors = result.error.issues
      .map((issue) => {
        const path = issue.path.length > 0 ? issue.path.join(".") : "/";
        return `${path}: ${issue.message}`;
      })
      .join("\n  ");
    throw new Error(`Config validation failed:\n  ${errors}`);
  }
}
