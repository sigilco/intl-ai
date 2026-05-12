import { z } from "zod";
import type { LanguageModel } from "ai";
import type { IntlAiProcessor } from "../types";

const BatchTranslationSchema = z.object({
  translations: z.array(
    z.object({
      key: z.string(),
      translated: z.string(),
    }),
  ),
});

export interface TranslateBatchOptions {
  model: LanguageModel;
  entries: Array<{ key: string; source: string }>;
  targetLocale: string;
  sourceLocale: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: IntlAiProcessor;
}

export async function translateBatch(
  options: TranslateBatchOptions,
): Promise<Array<{ key: string; translated: string; success: boolean; error?: string }>> {
  const {
    model,
    entries,
    targetLocale,
    sourceLocale,
    glossary,
    maxRetries = 3,
    processor,
  } = options;

  const prompt = buildTranslationPrompt(entries, sourceLocale, targetLocale, glossary, processor);

  const modelId = (model as { modelId?: string }).modelId || "unknown";
  const baseURL = extractBaseURL(model);
  const apiKey = extractApiKey(model);

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const url = baseURL + "/chat/completions";
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: "Bearer " + apiKey,
        },
        body: JSON.stringify({
          model: modelId,
          messages: [
            {
              role: "system",
              content: "You are a translator. Translate the user text and return ONLY valid JSON.",
            },
            { role: "user", content: prompt },
          ],
          response_format: {
            type: "json_schema",
            json_schema: {
              name: "translation_response",
              strict: "true",
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
                    },
                  },
                },
                required: ["translations"],
              },
            },
          },
          temperature: 0.3,
          max_tokens: 500,
        }),
      });

      if (!response.ok) {
        throw new Error("HTTP error! status: " + response.status);
      }

      const data = (await response.json()) as {
        choices?: Array<{ message?: { content?: string } }>;
      };
      const content = data.choices?.[0]?.message?.content;

      if (!content) {
        throw new Error("No content in response");
      }

      const parsed = JSON.parse(content);
      const validated = BatchTranslationSchema.parse(parsed);

      const results = validated.translations.map((t) => {
        const entry = entries.find((e) => e.key === t.key);
        if (processor && entry) {
          const validation = processor.validate(entry.source, t.translated);
          if (!validation.valid) {
            console.warn(
              `[intl-ai] Translation validation failed for key '${t.key}': ${validation.errors?.join(", ")}`,
            );
            return {
              key: t.key,
              translated: t.translated,
              success: false,
              error: `Validation failed: ${validation.errors?.join(", ")}`,
            };
          }
        }
        return { key: t.key, translated: t.translated, success: true };
      });
      return results;
    } catch (error) {
      console.warn("[intl-ai] Translation attempt " + (attempt + 1) + " failed:", error);
      if (attempt === maxRetries - 1) {
        return entries.map((e) => ({
          key: e.key,
          translated: "",
          success: false,
          error: error instanceof Error ? error.message : "Translation failed",
        }));
      }
    }
  }

  return [];
}

function extractBaseURL(model: LanguageModel): string {
  const config = (model as { config?: { baseURL?: string } }).config || {};
  return config.baseURL || "http://127.0.0.1:1234/v1";
}

function extractApiKey(model: LanguageModel): string {
  const config = (model as { config?: { apiKey?: string } }).config || {};
  return config.apiKey || "lm-studio";
}

function buildTranslationPrompt(
  entries: Array<{ key: string; source: string }>,
  sourceLocale: string,
  targetLocale: string,
  glossary?: Record<string, string>,
  processor?: IntlAiProcessor,
): string {
  const syntaxHint = processor
    ? processor.getSyntaxHint()
    : "Preserve any placeholders like {variable} exactly as they appear.";

  let prompt =
    "Translate the following " +
    entries.length +
    " text strings from " +
    sourceLocale +
    " to " +
    targetLocale +
    ".\n" +
    syntaxHint +
    '\nReturn ONLY a JSON object in this exact format:\n{\n  "translations": [\n    { "key": "original_key", "translated": "translated_text" },\n    ...\n  ]\n}\n\nTexts to translate:\n' +
    entries.map((e, i) => i + 1 + '. "' + e.source + '" (key: ' + e.key + ")").join("\n");

  if (glossary && Object.keys(glossary).length > 0) {
    prompt +=
      "\n\nGlossary (use these translations where applicable):\n" +
      Object.entries(glossary)
        .map(([k, v]) => '- "' + k + '" → "' + v + '"')
        .join("\n");
  }

  return prompt;
}
