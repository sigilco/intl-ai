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
  model: z.string().min(1),
  apiKey: z.string().min(1),
  baseURL: z.url().optional(),
  glossary: z.record(z.string(), z.string()).optional(),
  maxRetries: z.number().int().min(0).max(10).optional(),
  processor: z.enum(["passthrough", "icu"]).optional(),
  modelParams: z.record(z.string(), z.unknown()).optional(),
  /**
   * Max source entries per translation request. Omit to send all entries in
   * one request (default, efficient for many short UI strings). Set to 1 when
   * each key holds a large value, e.g. whole documents in a page-as-key setup.
   */
  batchSize: z.number().int().min(1).optional(),
  /**
   * Quality-aware fill loop settings. `failOnLowQuality` and `assessor`
   * are intentionally absent here: they are runtime-only and are not
   * representable in JSON.
   */
  quality: z
    .object({
      threshold: z.number().min(0).max(1).optional(),
      maxRetries: z.number().int().min(0).max(5).optional(),
    })
    .optional(),
  format: z.string().optional(),
  // JSON Schema meta-key. Editors read it for autocomplete. Stripped from
  // runtime config (see jsonConfigToIntlAiConfig). Other JSON Schema meta-
  // keys (e.g. $id, $defs, $ref) are not allowed because `.strict()` is
  // kept; add them inline only if a real consumer needs them.
  $schema: z.string().min(1).optional(),
}).strict();
