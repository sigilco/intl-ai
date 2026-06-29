import { z } from "zod";
import type { AIProvider } from "../../ports/provider";
import type { TranslationHook } from "../../ports/hook";
import type {
  ApiKeyValue,
  QualityAssessorInstance,
  QualityResult,
  TranslationContext,
} from "../../core/types";
import { resolveProvider } from "../../adapters/providers/registry";

export { ADVERSARIAL_SYSTEM_PROMPT } from "./prompts";

export interface JudgeBatchOptions {
  provider: AIProvider | string;
  modelId?: string;
  baseURL: string;
  apiKey: ApiKeyValue;
  contexts: TranslationContext[];
  hook?: TranslationHook;
  modelParams?: Record<string, unknown>;
}

const JudgeResponseSchema = z.object({
  judgements: z.array(
    z.object({
      key: z.string(),
      score: z.number().min(0).max(1),
      reason: z.string().optional(),
      errors: z.array(z.string()).optional(),
    }),
  ),
});

const ASSESSOR_NAME = "default-adversarial-judge";
const DEFAULT_THRESHOLD = 0.8;

function tierFor(score: number): QualityResult["riskTier"] {
  if (score < 0.6) return "high";
  if (score < 0.8) return "medium";
  return "low";
}

function resolveApiKey(value: ApiKeyValue): string {
  return value.replace(/\$\{?(\w+)\}?/g, (_, name) => {
    const val = process.env[name];
    if (val === undefined) throw new Error(`Environment variable ${name} is not set for apiKey`);
    return val;
  });
}

function buildJudgePrompt(opts: { contexts: TranslationContext[] }): string {
  const entriesText = opts.contexts
    .map(
      (c, i) =>
        `${i + 1}. key="${c.key}" locale=${c.locale}\n   source: ${c.source}\n   translation: ${c.translation}`,
    )
    .join("\n\n");

  return `Evaluate each translation for accuracy, fluency, terminology, style, and locale convention. Be a strict adversarial reviewer: assume any translation has issues until you have evidence otherwise.\n\n${entriesText}\n\nRespond with JSON: { "judgements": [{ "key": "...", "score": 0..1, "reason": "...", "errors": ["..."] }] }`;
}

/**
 * Default quality assessor. Sends one batched chat completion to the
 * configured provider with an adversarial system prompt and parses per-key
 * scores. Returns one `QualityResult` per input context, in order.
 */
export async function judgeBatch(opts: JudgeBatchOptions): Promise<QualityResult[]> {
  if (opts.contexts.length === 0) return [];

  const provider = resolveProvider(opts.provider);
  const apiKey = resolveApiKey(opts.apiKey);
  const modelId = opts.modelId ?? "gpt-4o-mini";
  const threshold = DEFAULT_THRESHOLD;

  const { ADVERSARIAL_SYSTEM_PROMPT } = await import("./prompts");
  const userPrompt = buildJudgePrompt({ contexts: opts.contexts });

  const req = provider.buildRequest({
    model: modelId,
    systemPrompt: ADVERSARIAL_SYSTEM_PROMPT,
    userPrompt,
    temperature: 0,
    modelParams: opts.modelParams,
  });

  const startTime =
    typeof performance !== "undefined" && typeof performance.now === "function"
      ? performance.now()
      : Date.now();

  opts.hook?.onRequest?.({
    provider: provider.id,
    model: modelId,
    locale: opts.contexts[0].locale,
    entryCount: opts.contexts.length,
  });

  const res = await fetch(`${opts.baseURL}${req.url}`, {
    method: "POST",
    headers: { ...req.headers, Authorization: `Bearer ${apiKey}` },
    body: JSON.stringify(req.body),
  });

  if (!res.ok) {
    const msg = `HTTP ${res.status}: ${await res.text()}`;
    opts.hook?.onError?.({
      provider: provider.id,
      model: modelId,
      locale: opts.contexts[0].locale,
      error: msg,
      attempt: 1,
    });
    throw new Error(msg);
  }

  const data = await res.json();
  const { content } = provider.parseResponse(data);
  if (!content) {
    const msg = "Empty response from judge";
    opts.hook?.onError?.({
      provider: provider.id,
      model: modelId,
      locale: opts.contexts[0].locale,
      error: msg,
      attempt: 1,
    });
    throw new Error(msg);
  }

  let parsed;
  try {
    parsed = JudgeResponseSchema.parse(JSON.parse(content));
  } catch (err) {
    const msg = `Invalid judge schema: ${err instanceof Error ? err.message : String(err)}`;
    opts.hook?.onError?.({
      provider: provider.id,
      model: modelId,
      locale: opts.contexts[0].locale,
      error: msg,
      attempt: 1,
    });
    throw new Error(msg);
  }

  const byKey = new Map(parsed.judgements.map((j) => [j.key, j]));

  const durationMs =
    typeof performance !== "undefined" && typeof performance.now === "function"
      ? performance.now() - startTime
      : 0;

  const results: QualityResult[] = opts.contexts.map((ctx) => {
    const j = byKey.get(ctx.key);
    if (!j) {
      return {
        score: 0,
        riskTier: "high",
        needsReview: true,
        reason: "Judge omitted this key from response",
        assessorName: ASSESSOR_NAME,
      };
    }
    return {
      score: j.score,
      riskTier: tierFor(j.score),
      needsReview: j.score < threshold,
      reason: j.reason,
      errorTypes: j.errors?.map((description) => ({
        type: "accuracy" as const,
        severity: "major" as const,
        description,
      })),
      assessorName: ASSESSOR_NAME,
    };
  });

  opts.hook?.onSuccess?.({
    provider: provider.id,
    model: modelId,
    locale: opts.contexts[0].locale,
    results: results.map((r) => ({
      key: r.reason ?? "(no reason)",
      translated: r.score.toString(),
      success: true,
    })),
    durationMs,
  });

  return results;
}

/**
 * Per-key wrapper around `judgeBatch`. Lets users plug the default judge
 * into the `QualityAssessorInstance` escape hatch.
 */
export function createDefaultAssessor(
  opts: Omit<JudgeBatchOptions, "contexts">,
): QualityAssessorInstance {
  return {
    name: ASSESSOR_NAME,
    async assess(ctx: TranslationContext): Promise<QualityResult> {
      const [result] = await judgeBatch({ ...opts, contexts: [ctx] });
      if (!result) {
        return {
          score: 0,
          riskTier: "high",
          needsReview: true,
          reason: "Judge returned no result",
          assessorName: ASSESSOR_NAME,
        };
      }
      return result;
    },
  };
}
