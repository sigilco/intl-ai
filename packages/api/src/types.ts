import { z } from "zod";
import type { AIProvider } from "./ports/provider";
import type { IntlAiProcessor } from "./ports/processor";
import type { TranslationHook } from "./ports/hook";
import type { ApiKeyValue, QualityAssessorInstance, QualityOptions } from "./core/types";

export type { ApiKeyValue };

export interface IntlAiConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  model: AIProvider | string;
  apiKey: ApiKeyValue;
  baseURL: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: IntlAiProcessor;
  modelParams?: Record<string, unknown>;
  hook?: TranslationHook;
  /** Quality-aware fill loop settings + optional custom assessor. */
  quality?: QualityOptions & { assessor?: QualityAssessorInstance };
}

export const IntlAiConfigSchema = z.object({
  defaultLocale: z.string().min(1),
  locales: z.array(z.string().min(1)).min(1),
  localeDir: z.string().min(1),
  model: z.union([
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
  apiKey: z.string().min(1),
  baseURL: z.string().min(1),
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
  maxRetries: z.number().int().min(0).max(10).default(3),
  modelParams: z.record(z.string(), z.unknown()).optional(),
  hook: z.custom<TranslationHook>().optional(),
  quality: z
    .object({
      threshold: z.number().min(0).max(1).optional(),
      maxRetries: z.number().int().min(0).max(5).optional(),
      failOnLowQuality: z.boolean().optional(),
      assessor: z.custom<QualityAssessorInstance>().optional(),
    })
    .optional(),
}) satisfies z.ZodType<IntlAiConfig>;
