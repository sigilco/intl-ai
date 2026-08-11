import { z } from "zod";
import { getLogger } from "@logtape/logtape";

const logger = getLogger(["intl-ai", "fill", "translator"]);

import type { AIProvider, AITransport } from "../../ports/provider";
import type { IntlAiProcessor } from "../../ports/processor";
import type { ErrorType, TranslationHook } from "../../ports/hook";
import type { TranslationEntry, TranslationResult, ApiKeyValue } from "../../core/types";
import { resolveProvider } from "../../adapters/providers/registry";

export type { TranslationEntry, TranslationResult };

/** Thrown by runOnce so classifyError can inspect the failed HTTP response. */
class HttpResponseError extends Error {
  constructor(
    message: string,
    readonly response: Response,
  ) {
    super(message);
    this.name = "HttpResponseError";
  }
}

function classifyError(err: Error, res?: Response): { errorType: ErrorType; statusCode?: number } {
  const response = res ?? (err instanceof HttpResponseError ? err.response : undefined);
  if (response) {
    if (response.status === 429) return { errorType: "rate_limit", statusCode: 429 };
    if (response.status === 401 || response.status === 403)
      return { errorType: "http", statusCode: response.status };
    if (response.status >= 500) return { errorType: "http", statusCode: response.status };
    if (response.status >= 400) return { errorType: "http", statusCode: response.status };
  }
  if (err.message.includes("ENOENT")) return { errorType: "spawn_failure" };
  if (
    err.name === "TimeoutError" ||
    err.message.includes("aborted") ||
    err.message.includes("timeout") ||
    err.message.includes("SIGTERM") ||
    err.message.includes("SIGKILL")
  )
    return { errorType: "timeout" };
  if (err.message.includes("exited with code")) return { errorType: "process_exit" };
  if (
    err.message.includes("JSON") ||
    err.message.includes("Unexpected token") ||
    err.message.includes("parse")
  )
    return { errorType: "parse_error" };
  if (err.message === "Empty response from model") return { errorType: "empty" };
  return { errorType: "unknown" };
}

export interface TranslateBatchOptions {
  /** Required unless `transport` is set. */
  provider?: AIProvider | string;
  /** Required unless `transport` is set. */
  modelId?: string;
  entries: TranslationEntry[];
  targetLocale: string;
  sourceLocale: string;
  glossary?: Record<string, string>;
  maxRetries?: number;
  processor?: IntlAiProcessor;
  /** Required unless `transport` is set. */
  baseURL?: string;
  /** Required unless `transport` is set. */
  apiKey?: ApiKeyValue;
  hook?: TranslationHook;
  modelParams?: Record<string, unknown>;
  /**
   * Drive a local headless coding agent instead of the HTTP path. When set,
   * `provider`, `baseURL`, `apiKey`, and `modelId` are not used.
   */
  transport?: AITransport;
  /**
   * Per-key reviewer notes to inject into the prompt when refilling a
   * rejected entry. Used by the quality loop, ignored on the first pass.
   */
  feedback?: Record<string, string>;
  /**
   * Freeform style/dialect rule for the target locale, already resolved by
   * the caller (see core/locale-instructions.ts). Applies to the whole batch.
   */
  localeInstruction?: string;
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

function buildSystemPrompt(localeInstruction?: string): string {
  const base =
    "You are a professional translation engine. You respond only with valid JSON matching the requested schema.";
  if (!localeInstruction) return base;
  return `${base}\n\nLocale style instruction: ${localeInstruction}`;
}

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
    feedback,
    localeInstruction,
    transport,
  } = options;

  if (entries.length === 0) return [];

  if (!transport && (!providerInput || !modelIdInput || !baseURL || !apiKeyInput)) {
    throw new Error(
      "translateBatch: provider, modelId, baseURL, and apiKey are required when no transport is set",
    );
  }

  const provider = transport ? undefined : resolveProvider(providerInput!);
  const apiKey = transport ? undefined : await resolveApiKey(apiKeyInput!);
  const modelId = modelIdInput;
  const dispatcherId = transport?.id ?? provider!.id;
  // Hooks require a model name; a transport has none, so fall back to its id.
  const hookModel = modelId ?? dispatcherId;

  const systemPrompt = buildSystemPrompt(localeInstruction);
  const userPrompt = buildTranslationPrompt({
    entries,
    targetLocale,
    sourceLocale,
    glossary,
    processor,
    feedback,
  });

  const req = provider?.buildRequest({
    model: modelId!,
    systemPrompt,
    userPrompt,
    temperature: 0.3,
    modelParams,
  });

  // ponytail: 3min cap; local models and headless agents both stall without it
  async function runOnce(): Promise<{ content: string }> {
    if (transport) {
      return transport.complete({
        systemPrompt,
        userPrompt,
        signal: AbortSignal.timeout(300_000),
      });
    }

    const res = await fetch(`${baseURL}${req!.url}`, {
      method: "POST",
      headers: {
        ...req!.headers,
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify(req!.body),
      signal: AbortSignal.timeout(300_000),
    });

    if (!res.ok) {
      throw new HttpResponseError(`HTTP ${res.status}: ${await res.text()}`, res);
    }

    const data = await res.json();
    return provider!.parseResponse(data);
  }

  let lastError: string | undefined;
  let lastErrorType: ErrorType = "unknown";
  let lastStatusCode: number | undefined;
  let rawResponse: string | undefined;
  const attemptHistory: Array<{
    attempt: number;
    errorType: ErrorType;
    durationMs: number;
    statusCode?: number;
  }> = [];

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    const startTime = performance.now();
    try {
      hook?.onRequest?.({
        provider: dispatcherId,
        model: hookModel,
        locale: targetLocale,
        entryCount: entries.length,
      });

      const { content } = await runOnce();
      if (!content) {
        const errMsg = "Empty response from model";
        const { errorType, statusCode } = classifyError(new Error(errMsg));
        lastError = errMsg;
        lastErrorType = errorType;
        lastStatusCode = statusCode;
        const durationMs = performance.now() - startTime;
        attemptHistory.push({ attempt: attempt + 1, errorType, durationMs, statusCode });
        logger.debug(
          `[${targetLocale}] Attempt ${attempt + 1}/${maxRetries} failed (${errorType}): ${errMsg}`,
        );
        hook?.onAttemptFailure?.({
          provider: dispatcherId,
          model: hookModel,
          locale: targetLocale,
          errorType,
          error: errMsg,
          attempt: attempt + 1,
          maxRetries,
          statusCode,
          durationMs,
        });
        continue;
      }

      let parsedData: unknown;
      try {
        parsedData = JSON.parse(content);
      } catch (parseErr) {
        const errMsg = parseErr instanceof Error ? parseErr.message : String(parseErr);
        const { errorType, statusCode } = classifyError(new Error(errMsg));
        rawResponse = content.length > 500 ? content.slice(0, 500) + "..." : content;
        lastError = errMsg;
        lastErrorType = errorType;
        lastStatusCode = statusCode;
        const durationMs = performance.now() - startTime;
        attemptHistory.push({ attempt: attempt + 1, errorType, durationMs, statusCode });
        logger.debug(
          `[${targetLocale}] Attempt ${attempt + 1}/${maxRetries} failed (${errorType}): ${errMsg} — raw response captured for diagnosis`,
        );
        hook?.onAttemptFailure?.({
          provider: dispatcherId,
          model: hookModel,
          locale: targetLocale,
          errorType,
          error: errMsg,
          attempt: attempt + 1,
          maxRetries,
          statusCode,
          durationMs,
        });
        continue;
      }

      const parsed = TranslationResponseSchema.safeParse(parsedData);
      if (!parsed.success) {
        const errMsg = `Invalid response schema: ${parsed.error.message}`;
        const { errorType, statusCode } = classifyError(new Error(errMsg));
        rawResponse = content.length > 500 ? content.slice(0, 500) + "..." : content;
        lastError = errMsg;
        lastErrorType = errorType;
        lastStatusCode = statusCode;
        const durationMs = performance.now() - startTime;
        attemptHistory.push({ attempt: attempt + 1, errorType, durationMs, statusCode });
        logger.debug(
          `[${targetLocale}] Attempt ${attempt + 1}/${maxRetries} failed (${errorType}): ${errMsg} — raw response captured for diagnosis`,
        );
        hook?.onAttemptFailure?.({
          provider: dispatcherId,
          model: hookModel,
          locale: targetLocale,
          errorType,
          error: errMsg,
          attempt: attempt + 1,
          maxRetries,
          statusCode,
          durationMs,
        });
        continue;
      }

      const durationMs = performance.now() - startTime;

      const map = new Map(parsed.data.translations.map((t) => [t.key, t.translated]));
      const results: TranslationResult[] = entries.map((entry) => {
        const translated = map.get(entry.key);
        if (translated === undefined) {
          // max_tokens truncation: model output was cut before this key appeared
          logger.debug(
            `Key '${entry.key}' not in model response (output likely truncated at max_tokens)`,
          );
          return {
            key: entry.key,
            success: false,
            error: "No translation returned for key",
            errorType: "output_truncated",
          };
        }
        if (processor) {
          const validation = processor.validate(entry.source, translated);
          if (!validation.valid) {
            logger.warn(
              `Translation validation failed for key '${entry.key}': ${(validation.errors ?? []).join("; ")}`,
            );
            return {
              key: entry.key,
              success: false,
              error: `Validation failed: ${(validation.errors ?? []).join("; ")}`,
              errorType: "validation",
            };
          }
        }
        return { key: entry.key, translated, success: true };
      });

      hook?.onSuccess?.({
        provider: dispatcherId,
        model: hookModel,
        locale: targetLocale,
        results,
        durationMs,
      });

      return results;
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      const { errorType, statusCode } = classifyError(
        err instanceof Error ? err : new Error(errMsg),
      );
      lastError = errMsg;
      lastErrorType = errorType;
      lastStatusCode = statusCode;
      const durationMs = performance.now() - startTime;
      attemptHistory.push({ attempt: attempt + 1, errorType, durationMs, statusCode });

      // Log attempt failure: debug for known types (retry noise), warn for unknown (actionable)
      if (errorType === "unknown") {
        logger.warn(
          `[${targetLocale}] Attempt ${attempt + 1}/${maxRetries} failed (${errorType}): ${errMsg} — raw error, classifyError may need updating`,
        );
      } else {
        logger.debug(
          `[${targetLocale}] Attempt ${attempt + 1}/${maxRetries} failed (${errorType}${statusCode ? `, HTTP ${statusCode}` : ""}): ${errMsg}`,
        );
      }

      hook?.onAttemptFailure?.({
        provider: dispatcherId,
        model: hookModel,
        locale: targetLocale,
        errorType,
        error: errMsg,
        attempt: attempt + 1,
        maxRetries,
        statusCode,
        durationMs,
      });
    }
  }

  const totalDurationMs = attemptHistory.reduce((sum, a) => sum + a.durationMs, 0);

  hook?.onError?.({
    provider: dispatcherId,
    model: hookModel,
    locale: targetLocale,
    errorType: lastErrorType,
    error: lastError ?? "Unknown error",
    attempt: maxRetries,
    maxRetries,
    statusCode: lastStatusCode,
    durationMs: totalDurationMs,
    attempts: attemptHistory,
    rawResponse,
  });

  return entries.map((entry) => ({
    key: entry.key,
    success: false,
    error: lastError ?? "Unknown error",
    errorType: lastErrorType,
  }));
}

function buildTranslationPrompt(opts: {
  entries: TranslationEntry[];
  targetLocale: string;
  sourceLocale: string;
  glossary?: Record<string, string>;
  processor?: IntlAiProcessor;
  feedback?: Record<string, string>;
}): string {
  const syntaxHint = opts.processor?.getSyntaxHint() ?? DEFAULT_PROMPT_HINT;
  const glossaryText = opts.glossary
    ? `\n\nGlossary (use these exact translations):\n${Object.entries(opts.glossary)
        .map(([k, v]) => `- ${k} → ${v}`)
        .join("\n")}`
    : "";

  const feedbackEntries = opts.feedback
    ? Object.entries(opts.feedback).filter(([, v]) => v?.trim())
    : [];

  const feedbackBlock = feedbackEntries.length
    ? `\n\nThe following entries were rejected by a quality reviewer. Address each note in your new translation.\n${feedbackEntries
        .map(([key, note]) => `- ${key}: ${note}`)
        .join("\n")}`
    : "";

  const entriesText = opts.entries
    .map((e, i) => `${i + 1}. "${e.source}" (key: ${e.key})`)
    .join("\n");

  return `Translate the following ${opts.sourceLocale} strings to ${opts.targetLocale}.
${syntaxHint}${glossaryText}${feedbackBlock}

Input strings:
${entriesText}

Respond with JSON: { "translations": [{ "key": "...", "translated": "..." }] }`;
}
