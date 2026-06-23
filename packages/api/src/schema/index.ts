export { IntlAiJsonConfigSchema } from "./json-config";

import schemaJson from "./intl-ai.schema.json" with { type: "json" };
import type { IntlAiConfig } from "../types";
import { icuProcessor } from "../processor/icu";
import { passthroughProcessor } from "../processor";

export const INTL_AI_SCHEMA_URL = "https://www.schemastore.org/intl-ai.json";

export function getIntlAiSchema(): Record<string, unknown> {
  return schemaJson as Record<string, unknown>;
}

export interface IntlAiJsonConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  model: string;
  apiKey: string;
  baseURL?: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: "passthrough" | "icu";
}

/**
 * Convert a validated JSON config into a runtime `IntlAiConfig` object.
 * Maps the `processor` string to a concrete processor implementation.
 */
export function jsonConfigToIntlAiConfig(json: IntlAiJsonConfig): IntlAiConfig {
  return {
    defaultLocale: json.defaultLocale,
    locales: json.locales,
    localeDir: json.localeDir,
    model: {
      modelId: json.model,
      config: {
        baseURL: json.baseURL ?? "https://api.openai.com/v1",
        apiKey: json.apiKey,
      },
    },
    glossary: json.glossary,
    maxRetries: json.maxRetries ?? 3,
    processor: json.processor === "icu" ? icuProcessor : passthroughProcessor,
  };
}
