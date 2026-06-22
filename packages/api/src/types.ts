import { z } from "zod";

/**
 * IntlAiProcessor — optional token extractor / validator for a syntax family
 * (e.g. ICU MessageFormat, i18next `{{var}}`, Vue I18n, plain).
 */
export interface IntlAiProcessor {
  name: string;
  extractTokens(message: string): string[];
  validate(source: string, translated: string): ValidationResult;
  getSyntaxHint(): string;
}

export interface ValidationResult {
  valid: boolean;
  errors?: string[];
}

export interface TranslationEntry {
  key: string;
  source: string;
}

export interface TranslationResult {
  key: string;
  translated?: string;
  success: boolean;
  error?: string;
}

export interface TranslationStaleEntry {
  key: string;
  source: string;
  previous: string;
  sourceHash: string;
}

/**
 * IntlAiConfig — runtime-agnostic configuration.
 *
 * The `model` field is intentionally typed as `unknown` in this package:
 * - CLI passes a simple duck-typed object `{ modelId, config: { baseURL, apiKey } }`
 * - SDK users can pass a Vercel AI SDK `LanguageModel` (typed as `unknown`
 *   here; cast to the proper type in `@intl-ai/core` if needed)
 * - The translator uses duck-typing to read `modelId`, `config.baseURL`,
 *   `config.apiKey` from the model object.
 */
export interface IntlAiConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  model: unknown;
  processor?: IntlAiProcessor;
  glossary?: Record<string, string>;
  maxRetries?: number;
}

export const IntlAiConfigSchema = z.object({
  defaultLocale: z.string().min(1),
  locales: z.array(z.string().min(1)).min(1),
  localeDir: z.string().min(1),
  model: z.unknown(),
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
}) satisfies z.ZodType<IntlAiConfig>;

export const ValidationResultSchema = z.object({
  valid: z.boolean(),
  errors: z.array(z.string()).optional(),
});

export const TranslationEntrySchema = z.object({
  key: z.string(),
  source: z.string(),
});

export const TranslationResultSchema = z.object({
  key: z.string(),
  translated: z.string().optional(),
  success: z.boolean(),
  error: z.string().optional(),
});

/**
 * Extract `baseURL` and `apiKey` from a duck-typed `model` value.
 * Works for both plain JSON-shape config objects and Vercel AI SDK `LanguageModel` instances.
 */
export function extractModelConfig(model: unknown): {
  baseURL: string;
  apiKey: string;
  modelId?: string;
} {
  const m = model as {
    modelId?: string;
    config?: { baseURL?: string; apiKey?: string };
  } | null;
  return {
    baseURL: m?.config?.baseURL ?? "https://api.openai.com/v1",
    apiKey: m?.config?.apiKey ?? "lm-studio",
    modelId: m?.modelId,
  };
}
