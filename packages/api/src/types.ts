import { z } from "zod";
import type { AIProvider } from "./ports/provider";
import type { IntlAiProcessor } from "./ports/processor";
import type { LocaleFormat } from "./ports/format";
import type { TranslationHook } from "./ports/hook";
import type { ApiKeyValue, QualityAssessorInstance, QualityOptions } from "./core/types";
import { agentPresetIds, type AgentPreset } from "./infrastructure/transports/presets";

export type { ApiKeyValue };
export type { AgentPreset };

interface IntlAiBaseConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  glossary?: Record<string, string>;
  /**
   * Freeform style/dialect instruction per locale (e.g. "en-GB": "Use British spelling.").
   * Keys resolve exact locale, then language subtag, then "*" as a catch-all.
   */
  localeInstructions?: Record<string, string>;
  maxRetries?: number;
  processor?: IntlAiProcessor;
  /**
   * Max source entries per translation request. Defaults to all-in-one
   * (backward compatible). For formats that batch multiple keys per request,
   * set to 1 when each key holds a large value such as a full document body.
   */
  batchSize?: number;
  hook?: TranslationHook;
  /** Quality-aware fill loop settings + optional custom assessor. */
  quality?: QualityOptions & { assessor?: QualityAssessorInstance };
  /** Locale format adapter. String selects a built-in; object is a custom impl. */
  format?: LocaleFormat | string;
}

export interface HttpIntlAiConfig extends IntlAiBaseConfig {
  kind?: "http";
  provider: AIProvider | string;
  model: string;
  apiKey: ApiKeyValue;
  baseURL: string;
  modelParams?: Record<string, unknown>;
}

export interface AgentIntlAiConfig extends IntlAiBaseConfig {
  kind: "agent";
  /** Preset name for a known headless coding agent CLI. */
  agent?: AgentPreset;
  /** TS-config-only escape hatch: run an arbitrary binary as the agent. */
  command?: string;
  args?: string[];
  cwd?: string;
}

export type IntlAiConfig = HttpIntlAiConfig | AgentIntlAiConfig;

const baseFields = {
  defaultLocale: z.string().min(1),
  locales: z.array(z.string().min(1)).min(1),
  localeDir: z.string().min(1),
  processor: z
    .object({
      name: z.string(),
      extractTokens: z
        .function()
        .input(z.tuple([z.string()]))
        .output(z.array(z.string())),
      validate: z
        .function()
        .input(z.tuple([z.string(), z.string()]))
        .output(z.object({ valid: z.boolean(), errors: z.array(z.string()).optional() })),
      getSyntaxHint: z.function().input(z.tuple([])).output(z.string()),
    })
    .optional(),
  glossary: z.record(z.string(), z.string()).optional(),
  localeInstructions: z.record(z.string(), z.string()).optional(),
  maxRetries: z.number().int().min(0).max(10).default(3),
  batchSize: z.number().int().min(1).optional(),
  hook: z.custom<TranslationHook>().optional(),
  quality: z
    .object({
      threshold: z.number().min(0).max(1).optional(),
      maxRetries: z.number().int().min(0).max(5).optional(),
      failOnLowQuality: z.boolean().optional(),
      assessor: z.custom<QualityAssessorInstance>().optional(),
    })
    .optional(),
  format: z.union([z.custom<LocaleFormat>(), z.string()]).optional(),
};

const httpConfigSchema = z.object({
  kind: z.literal("http").default("http"),
  ...baseFields,
  provider: z.union([
    z.custom<AIProvider>(
      (v): v is AIProvider =>
        v !== null &&
        typeof v === "object" &&
        "id" in v &&
        typeof (v as AIProvider).id === "string" &&
        "buildRequest" in v &&
        typeof (v as AIProvider).buildRequest === "function" &&
        "parseResponse" in v &&
        typeof (v as AIProvider).parseResponse === "function",
    ),
    z.string(),
  ]),
  model: z.string().min(1),
  apiKey: z.string().min(1),
  baseURL: z.string().min(1),
  modelParams: z.record(z.string(), z.unknown()).optional(),
}) satisfies z.ZodType<HttpIntlAiConfig>;

const agentConfigSchema = z.object({
  kind: z.literal("agent"),
  ...baseFields,
  agent: z.enum(agentPresetIds).optional(),
  command: z.string().min(1).optional(),
  args: z.array(z.string()).optional(),
  cwd: z.string().min(1).optional(),
}) satisfies z.ZodType<AgentIntlAiConfig>;

export const IntlAiConfigSchema = z.preprocess(
  (val) => (val && typeof val === "object" && !("kind" in val) ? { ...val, kind: "http" } : val),
  z.discriminatedUnion("kind", [httpConfigSchema, agentConfigSchema]),
) satisfies z.ZodType<IntlAiConfig>;

/**
 * TypeScript helper for intl-ai.config.ts files.
 * Provides LSP autocomplete and validation for the config object.
 * This is purely compile-time; the function returns the input unchanged.
 */
export function defineConfig(config: IntlAiConfig): IntlAiConfig {
  return config;
}
