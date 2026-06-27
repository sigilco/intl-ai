import { z } from "zod";
import type { AIProvider } from "../../ports/provider";
import type { IntlAiProcessor } from "../../ports/processor";
import type { TranslationHook } from "../../ports/hook";
import type { TranslationEntry, TranslationResult, ApiKeyValue } from "../../core/types";
import { resolveProvider } from "../../adapters/providers/registry";

export type { TranslationEntry, TranslationResult };

export interface TranslateBatchOptions {
  provider: AIProvider | string;
  modelId?: string;
  entries: TranslationEntry[];
  targetLocale: string;
  sourceLocale: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: IntlAiProcessor;
  baseURL: string;
  apiKey: ApiKeyValue;
  hook?: TranslationHook;
  modelParams?: Record<string, unknown>;
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

async function resolveApiKey(value: ApiKeyValue): Promise<string> {
  return value.replace(/\$\{?(\w+)\}?/g, (_, name) => {
    const val = process.env[name];
    if (val === undefined) throw new Error(`Environment variable ${name} is not set for apiKey`);
    return val;
  });
}

export async function translateBatch(options: TranslateBatchOptions): Promise<TranslationResult[]> {
  const {
    provider: providerInput,
    modelId: modelIdInput,
    entries,
    targetLocale,
    sourceLocale,
    glossary,
    maxRetries = 3,
    processor,
    baseURL,
    apiKey: apiKeyInput,
    hook,
    modelParams,
  } = options;

  if (entries.length === 0) return [];

  const provider = resolveProvider(providerInput);
  const apiKey = await resolveApiKey(apiKeyInput);
  const modelId = modelIdInput ?? "gpt-4o-mini";

  const systemPrompt =
    "You are a professional translation engine. You respond only with valid JSON matching the requested schema.";
  const userPrompt = buildTranslationPrompt({
    entries,
    targetLocale,
    sourceLocale,
    glossary,
    processor,
  });

  const req = provider.buildRequest({
    model: modelId,
    systemPrompt,
    userPrompt,
    temperature: 0.3,
    modelParams,
  });

  let lastError: string | undefined;
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      hook?.onRequest?.({
        provider: provider.id,
        model: modelId,
        locale: targetLocale,
        entryCount: entries.length,
      });

      const startTime = performance.now();

      const res = await fetch(`${baseURL}${req.url}`, {
        method: "POST",
        headers: {
          ...req.headers,
          Authorization: `Bearer ${apiKey}`,
        },
        body: JSON.stringify(req.body),
      });

      if (!res.ok) {
        lastError = `HTTP ${res.status}: ${await res.text()}`;
        continue;
      }

      const data = await res.json();
      const { content } = provider.parseResponse(data);
      if (!content) {
        lastError = "Empty response from model";
        continue;
      }

      const parsed = TranslationResponseSchema.safeParse(JSON.parse(content));
      if (!parsed.success) {
        lastError = `Invalid response schema: ${parsed.error.message}`;
        continue;
      }

      const durationMs = performance.now() - startTime;

      const map = new Map(parsed.data.translations.map((t) => [t.key, t.translated]));
      const results: TranslationResult[] = entries.map((entry) => {
        const translated = map.get(entry.key);
        if (translated === undefined) {
          return { key: entry.key, success: false, error: "No translation returned for key" };
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

      hook?.onSuccess?.({
        provider: provider.id,
        model: modelId,
        locale: targetLocale,
        results,
        durationMs,
      });

      return results;
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
    }
  }

  hook?.onError?.({
    provider: provider.id,
    model: modelId,
    locale: targetLocale,
    error: lastError ?? "Unknown error",
    attempt: maxRetries,
  });

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
