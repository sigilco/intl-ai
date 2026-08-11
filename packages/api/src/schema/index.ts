export { IntlAiJsonConfigSchema } from "./json-config";

import schemaJson from "./intl-ai.schema.json" with { type: "json" };
import type { IntlAiConfig } from "../types";
import type { AgentPreset } from "../infrastructure/transports/presets";
import { icuProcessor } from "../adapters/processors/icu";
import { passthroughProcessor } from "../adapters/processors/index";
import type { QualityOptions } from "../core/types";

export const INTL_AI_SCHEMA_URL = "https://www.schemastore.org/intl-ai.json";

export function getIntlAiSchema(): Record<string, unknown> {
  return schemaJson as Record<string, unknown>;
}

interface IntlAiJsonConfigShared {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  glossary?: Record<string, string>;
  localeInstructions?: Record<string, string>;
  maxRetries?: number;
  processor?: "passthrough" | "icu";
  batchSize?: number;
  quality?: Pick<QualityOptions, "threshold" | "maxRetries">;
  format?: string;
}

export interface IntlAiHttpJsonConfig extends IntlAiJsonConfigShared {
  kind?: "http";
  provider: string;
  model: string;
  apiKey: string;
  baseURL?: string;
  modelParams?: Record<string, unknown>;
}

export interface IntlAiAgentJsonConfig extends IntlAiJsonConfigShared {
  kind: "agent";
  agent: AgentPreset;
}

export type IntlAiJsonConfig = IntlAiHttpJsonConfig | IntlAiAgentJsonConfig;

/**
 * Convert a validated JSON config into a runtime `IntlAiConfig` object.
 * Maps the `processor` string to a concrete processor implementation. The
 * agent preset name is carried through unresolved: `loadConfigFromPath`
 * resolves it (and the TS-config `command`/`args` escape hatch) to a
 * concrete `AITransport` once, for both config formats alike.
 */
export function jsonConfigToIntlAiConfig(json: IntlAiJsonConfig): IntlAiConfig {
  const shared = {
    defaultLocale: json.defaultLocale,
    locales: json.locales,
    localeDir: json.localeDir,
    glossary: json.glossary,
    localeInstructions: json.localeInstructions,
    maxRetries: json.maxRetries ?? 3,
    processor: json.processor === "icu" ? icuProcessor : passthroughProcessor,
    batchSize: json.batchSize,
    quality: json.quality
      ? {
          threshold: json.quality.threshold,
          maxRetries: json.quality.maxRetries,
        }
      : undefined,
    format: json.format,
  };

  if (json.kind === "agent") {
    return { ...shared, kind: "agent", agent: json.agent };
  }

  return {
    ...shared,
    kind: "http",
    provider: json.provider,
    model: json.model,
    apiKey: json.apiKey,
    baseURL: json.baseURL ?? "https://api.openai.com/v1",
    modelParams: json.modelParams,
  };
}
