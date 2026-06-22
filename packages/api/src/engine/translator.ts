import { z } from "zod";
import {
  extractModelConfig,
  type IntlAiProcessor,
  type TranslationEntry,
  type TranslationResult,
} from "../types";

export interface TranslateBatchOptions {
  model: unknown;
  entries: TranslationEntry[];
  targetLocale: string;
  sourceLocale: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: IntlAiProcessor;
  baseURL?: string;
  apiKey?: string;
}

const TranslationResponseSchema = z.object({
  translations: z.array(
    z.object({
      key: z.string(),
      translated: z.string(),
    }),
  ),
});

const DEFAULT_PROMPT_HINT = "Preserve any placeholders like {variable} exactly as they appear.";

export async function translateBatch(options: TranslateBatchOptions): Promise<TranslationResult[]> {
  const {
    model,
    entries,
    targetLocale,
    sourceLocale,
    glossary,
    maxRetries = 3,
    processor,
    baseURL: baseURLOverride,
    apiKey: apiKeyOverride,
  } = options;

  if (entries.length === 0) return [];

  const modelConfig = extractModelConfig(model);
  const baseURL = baseURLOverride ?? modelConfig.baseURL;
  const apiKey = apiKeyOverride ?? modelConfig.apiKey;
  const modelId = modelConfig.modelId ?? "gpt-4o-mini";

  const prompt = buildTranslationPrompt({
    entries,
    targetLocale,
    sourceLocale,
    glossary,
    processor,
  });

  const body = {
    model: modelId,
    messages: [
      {
        role: "system",
        content:
          "You are a professional translation engine. You respond only with valid JSON matching the requested schema.",
      },
      { role: "user", content: prompt },
    ],
    response_format: {
      type: "json_schema",
      json_schema: {
        name: "translations",
        schema: {
          type: "object",
          properties: {
            translations: {
              type: "array",
              items: {
                type: "object",
                properties: {
                  key: { type: "string" },
                  translated: { type: "string" },
                },
                required: ["key", "translated"],
                additionalProperties: false,
              },
            },
          },
          required: ["translations"],
          additionalProperties: false,
        },
      },
    },
    temperature: 0.3,
  };

  let lastError: string | undefined;
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const res = await fetch(`${baseURL}/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${apiKey}`,
        },
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        lastError = `HTTP ${res.status}: ${await res.text()}`;
        continue;
      }

      const data = (await res.json()) as {
        choices?: Array<{ message?: { content?: string } }>;
      };
      const content = data.choices?.[0]?.message?.content;
      if (!content) {
        lastError = "Empty response from model";
        continue;
      }

      const parsed = TranslationResponseSchema.safeParse(JSON.parse(content));
      if (!parsed.success) {
        lastError = `Invalid response schema: ${parsed.error.message}`;
        continue;
      }

      const map = new Map(parsed.data.translations.map((t) => [t.key, t.translated]));
      return entries.map((entry) => {
        const translated = map.get(entry.key);
        if (translated === undefined) {
          return {
            key: entry.key,
            success: false,
            error: "No translation returned for key",
          };
        }
        if (processor) {
          const validation = processor.validate(entry.source, translated);
          if (!validation.valid) {
            console.warn(
              `Translation validation failed for key '${entry.key}': ${(validation.errors ?? []).join("; ")}`,
            );
            return {
              key: entry.key,
              success: false,
              error: `Validation failed: ${(validation.errors ?? []).join("; ")}`,
            };
          }
        }
        return { key: entry.key, translated, success: true };
      });
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
    }
  }

  return entries.map((entry) => ({
    key: entry.key,
    success: false,
    error: lastError ?? "Unknown error",
  }));
}

function buildTranslationPrompt(opts: {
  entries: TranslationEntry[];
  targetLocale: string;
  sourceLocale: string;
  glossary?: Record<string, string>;
  processor?: IntlAiProcessor;
}): string {
  const syntaxHint = opts.processor?.getSyntaxHint() ?? DEFAULT_PROMPT_HINT;
  const glossaryText = opts.glossary
    ? `\n\nGlossary (use these exact translations):\n${Object.entries(opts.glossary)
        .map(([k, v]) => `- ${k} → ${v}`)
        .join("\n")}`
    : "";

  const entriesText = opts.entries
    .map((e, i) => `${i + 1}. "${e.source}" (key: ${e.key})`)
    .join("\n");

  return `Translate the following ${opts.sourceLocale} strings to ${opts.targetLocale}.
${syntaxHint}${glossaryText}

Input strings:
${entriesText}

Respond with JSON: { "translations": [{ "key": "...", "translated": "..." }] }`;
}
