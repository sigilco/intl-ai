import type {
  QualityAssessorInstance,
  QualityOptions,
  QualityResult,
  TranslationContext,
  TranslationResult,
} from "../../core/types";

export interface RefillRequest {
  key: string;
  source: string;
  previousTranslation: string;
  feedback?: string;
}

export interface QualityLoopOptions {
  initialTranslations: Array<{ key: string; source: string; translated: string }>;
  sourceHashByKey: Map<string, string>;
  targetLocale: string;
  provider: string;
  model: string;
  /**
   * Batched judge. Receives contexts, returns same-length `QualityResult[]`.
   * Maps to `judgeBatch` for the default implementation. Wins over `assessor`.
   */
  judge?: (ctxs: TranslationContext[]) => Promise<QualityResult[]>;
  /** Per-key assessor fallback when no batched judge is available. */
  assessor?: QualityAssessorInstance;
  /** Refill callback for below-threshold entries. */
  refill: (entries: RefillRequest[]) => Promise<TranslationResult[]>;
  options: Pick<QualityOptions, "threshold" | "maxRetries"> & {
    threshold: number;
    maxRetries: number;
  };
}

export interface QualityLoopBelowThreshold {
  key: string;
  source: string;
  quality: QualityResult;
}

export interface QualityLoopResult {
  finalByKey: Map<string, string>;
  qualityByKey: Map<string, QualityResult>;
  belowThreshold: QualityLoopBelowThreshold[];
  /** Number of refill rounds executed (0..maxRetries). */
  attempts: number;
}

function needsReview(result: QualityResult, threshold: number): QualityResult {
  if (typeof result.needsReview === "boolean") return result;
  return { ...result, needsReview: result.score < threshold };
}

function summarizeFeedback(result: QualityResult): string {
  const parts: string[] = [];
  if (result.reason) parts.push(result.reason);
  if (result.errorTypes?.length) {
    parts.push(
      ...result.errorTypes.map(
        (e) => `${e.type}${e.severity ? ` (${e.severity})` : ""}: ${e.description ?? ""}`.trim(),
      ),
    );
  }
  if (parts.length === 0) parts.push(`Score ${result.score.toFixed(2)} below threshold.`);
  return parts.join("; ");
}

/**
 * Quality-aware retry loop. Pure orchestration: caller wires `judge` and
 * `refill`. After `maxRetries` rounds the remaining below-threshold keys
 * are surfaced through `belowThreshold` for the caller to fail or record.
 */
export async function runQualityLoop(opts: QualityLoopOptions): Promise<QualityLoopResult> {
  const { threshold, maxRetries } = opts.options;
  const finalByKey = new Map<string, string>();
  const qualityByKey = new Map<string, QualityResult>();
  const belowThreshold: QualityLoopBelowThreshold[] = [];

  const judgeOnce = opts.judge
    ? (items: Array<{ key: string; source: string; translated?: string }>) => {
        const contexts: TranslationContext[] = items.map((it) => ({
          key: it.key,
          source: it.source,
          translation: it.translated ?? "",
          locale: opts.targetLocale,
          sourceHash: opts.sourceHashByKey.get(it.key) ?? "",
          origin: "ai",
          model: opts.model,
          provider: opts.provider,
        }));
        return opts.judge!(contexts);
      }
    : opts.assessor
      ? async (items: Array<{ key: string; source: string; translated?: string }>) => {
          const contexts: TranslationContext[] = items.map((it) => ({
            key: it.key,
            source: it.source,
            translation: it.translated ?? "",
            locale: opts.targetLocale,
            sourceHash: opts.sourceHashByKey.get(it.key) ?? "",
            origin: "ai",
            model: opts.model,
            provider: opts.provider,
          }));
          return Promise.all(contexts.map((c) => opts.assessor!.assess(c)));
        }
      : () => {
          throw new Error("runQualityLoop requires `judge` or `assessor`");
        };

  // Round 0 = initial translations
  let current = opts.initialTranslations.map((e) => ({
    key: e.key,
    source: e.source,
    translated: e.translated,
  }));
  let attempts = 0;

  while (current.length > 0) {
    const judgements = await judgeOnce(current);
    const passing: typeof current = [];
    const rejected: typeof current = [];

    for (let i = 0; i < current.length; i++) {
      const item = current[i];
      const raw = judgements[i] ?? {
        score: 0,
        riskTier: "high" as const,
        needsReview: true,
        reason: "Judge returned no result",
        assessorName: "unknown",
      };
      const result = needsReview(raw, threshold);
      qualityByKey.set(item.key, result);
      if (result.score < threshold) rejected.push(item);
      else {
        passing.push(item);
        finalByKey.set(item.key, item.translated ?? "");
      }
    }

    if (rejected.length === 0) break;
    if (attempts >= maxRetries) {
      for (const item of rejected) {
        finalByKey.set(item.key, item.translated ?? "");
        belowThreshold.push({
          key: item.key,
          source: item.source,
          quality: qualityByKey.get(item.key) as QualityResult,
        });
      }
      break;
    }

    const refillReqs: RefillRequest[] = rejected.map((r) => ({
      key: r.key,
      source: r.source,
      previousTranslation: r.translated,
      feedback: summarizeFeedback(qualityByKey.get(r.key) as QualityResult),
    }));

    const refilled = await opts.refill(refillReqs);
    const byKey = new Map(refilled.map((r) => [r.key, r]));
    const nextCurrent: typeof current = [];
    for (const r of rejected) {
      const result = byKey.get(r.key);
      if (!result || !result.success || result.translated === undefined) {
        // Refill call itself failed; preserve last translation as final
        // and surface as below-threshold for the caller to decide.
        finalByKey.set(r.key, r.translated ?? "");
        belowThreshold.push({
          key: r.key,
          source: r.source,
          quality: qualityByKey.get(r.key) as QualityResult,
        });
        continue;
      }
      nextCurrent.push({ key: r.key, source: r.source, translated: result.translated });
    }
    current = nextCurrent;
    attempts++;
  }

  return {
    finalByKey,
    qualityByKey,
    belowThreshold,
    attempts,
  };
}
