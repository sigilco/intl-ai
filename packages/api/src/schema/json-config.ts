import { z } from "zod";

/**
 * IntlAiJsonConfigSchema — Zod schema for the JSON config file shape.
 *
 * This is the source of truth for the published JSON Schema. The runtime
 * `IntlAiConfigSchema` (in `types.ts`) has a different shape (model is `unknown`,
 * processor is an object) because SDK consumers pass live Vercel AI SDK instances.
 * The JSON config references a provider by string ID and optionally passes extra
 * model parameters via `modelParams`. It uses simple strings and is mapped to the
 * runtime config by
 * `jsonConfigToIntlAiConfig`.
 */
export const IntlAiJsonConfigSchema = z.object({
  defaultLocale: z.string().min(1),
  locales: z.array(z.string().min(1)).min(1),
  localeDir: z.string().min(1),
  provider: z.string().min(1).default("openai"),
  apiKey: z.string().min(1),
  baseURL: z.string().url().optional(),
  glossary: z.record(z.string(), z.string()).optional(),
  maxRetries: z.number().int().min(0).max(10).optional(),
  processor: z.enum(["passthrough", "icu"]).optional(),
  modelParams: z.record(z.string(), z.unknown()).optional(),
});
