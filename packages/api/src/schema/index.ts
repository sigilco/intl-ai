export { IntlAiJsonConfigSchema } from "./json-config";

import schemaJson from "./intl-ai.schema.json" with { type: "json" };
import type { IntlAiConfig } from "../types";
import { icuProcessor } from "../adapters/processors/icu";
import { passthroughProcessor } from "../adapters/processors/index";
import type { QualityOptions } from "../core/types";

export const INTL_AI_SCHEMA_URL = "https://www.schemastore.org/intl-ai.json";

export function getIntlAiSchema(): Record<string, unknown> {
  return schemaJson as Record<string, unknown>;
}

export interface IntlAiJsonConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  provider: string;
  model: string;
  apiKey: string;
  baseURL?: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: "passthrough" | "icu";
  modelParams?: Record<string, unknown>;
  batchSize?: number;
  quality?: Pick<QualityOptions, "threshold" | "maxRetries">;
  format?: string;
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
    provider: json.provider,
    model: json.model,
    apiKey: json.apiKey,
    baseURL: json.baseURL ?? "https://api.openai.com/v1",
    glossary: json.glossary,
    maxRetries: json.maxRetries ?? 3,
    processor: json.processor === "icu" ? icuProcessor : passthroughProcessor,
    modelParams: json.modelParams,
    batchSize: json.batchSize,
    quality: json.quality
      ? {
          threshold: json.quality.threshold,
          maxRetries: json.quality.maxRetries,
        }
      : undefined,
    format: json.format,
  };
}
