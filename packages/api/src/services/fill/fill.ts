import { LockfileManager } from "../../lockfile/manager";
import { findMissingTranslations, lockfileEntryToMap } from "../../core/diff";
import { translateBatch } from "./translator";
import { runQualityLoop, type RefillRequest } from "./loop";
import { judgeBatch } from "./judge";
import type { AIProvider } from "../../ports/provider";
import type { ResolvedIntlAiConfig } from "../../infrastructure/config/loader";
import type {
  ErrorType,
  QualityAssessorInstance,
  QualityOptions,
  QualityResult,
  TranslationContext,
  TranslationResult,
} from "../../core/types";
import type { LockfileQuality } from "../../lockfile/types";
import type { TranslationHook } from "../../ports/hook";

export interface RunFillProgressInfo {
  locale: string;
  completed: number; // keys completed so far in this locale
  total: number; // total keys for this locale
}

export interface RunFillOptions {
  locale?: string;
  force?: boolean;
  dryRun?: boolean;
  /**
   * Quality-aware fill loop. Threshold, maxRetries, and (optionally) a
   * custom assessor are merged with `config.quality`. When the loop is
   * enabled, every translation is judged, below-threshold keys are refilled
   * with the judge's feedback, and `lockfile` entries get a `quality` record.
   */
  quality?: QualityOptions;
  /** Fires after each batch completes. Allows callers to implement progress UI. */
  onProgress?: (info: RunFillProgressInfo) => void;
  /** Translation hook for request/success/error events. */
  hook?: TranslationHook;
}

export interface FillFailure {
  locale: string;
  key: string;
  source: string;
  error: string;
  errorType?: ErrorType;
}

export interface RunFillResult {
  locales: string[];
  translated: number;
  skipped: number;
  errors: number;
  needsReview: number;
  failures: FillFailure[];
}

export class IntlAiQualityError extends Error {
  readonly locale: string;
  readonly items: Array<{ key: string; source: string; score: number }>;

  constructor(
    locale: string,
    items: Array<{ key: string; source: string; quality: QualityResult }>,
  ) {
    const summary = items
      .map((i) => `  [${locale}] ${i.key}: score=${i.quality.score.toFixed(2)}`)
      .join("\n");
    super(
      `intl-ai: ${items.length} key(s) still below quality threshold in "${locale}":\n${summary}`,
    );
    this.name = "IntlAiQualityError";
    this.locale = locale;
    this.items = items.map((i) => ({
      key: i.key,
      source: i.source,
      score: i.quality.score,
    }));
  }
}

const DEFAULT_THRESHOLD = 0.8;
const DEFAULT_MAX_RETRIES = 2;

export async function runFill(
  config: ResolvedIntlAiConfig,
  options?: RunFillOptions,
): Promise<RunFillResult> {
  const { force = false, dryRun = false } = options ?? {};
  const {
    defaultLocale,
    locales,
    localeDir,
    provider,
    model,
    baseURL,
    apiKey,
    glossary,
    maxRetries,
    processor,
    hook,
    modelParams,
    batchSize,
    quality: configQuality,
    format: fmt,
  } = config;
  const effectiveHook = options?.hook ?? hook;
  // Loader resolves the format string to a LocaleFormat object before runFill is called.
  const format = fmt;

  const qualityOptions: QualityOptions | undefined = options?.quality ?? configQuality;
  const qualityEnabled = qualityOptions !== undefined;
  const threshold = qualityOptions?.threshold ?? DEFAULT_THRESHOLD;
  const maxQualityRetries = qualityOptions?.maxRetries ?? DEFAULT_MAX_RETRIES;
  const failOnLowQuality = qualityOptions?.failOnLowQuality === true;

  const customAssessor: QualityAssessorInstance | undefined = configQuality?.assessor;

  const targetLocales = options?.locale
    ? locales.filter((l) => l === options.locale)
    : locales.filter((l) => l !== defaultLocale);

  if (targetLocales.length === 0) {
    return { locales: [], translated: 0, skipped: 0, errors: 0, needsReview: 0, failures: [] };
  }

  const lockfileManager = new LockfileManager(localeDir);
  await lockfileManager.load();

  const sourceFlat = await format.readLocale(localeDir, defaultLocale);
  if (Object.keys(sourceFlat).length === 0) {
    return { locales: [], translated: 0, skipped: 0, errors: 0, needsReview: 0, failures: [] };
  }

  let translated = 0;
  let skipped = 0;
  let errors = 0;
  let needsReview = 0;
  const failures: FillFailure[] = [];

  for (const targetLocale of targetLocales) {
    try {
      let targetFlat = await format.readLocale(localeDir, targetLocale);

      const lockfileEntries = lockfileEntryToMap(lockfileManager.getAllEntries(), targetLocale);

      const diff = await findMissingTranslations(
        {
          sourceLocale: sourceFlat,
          targetLocale: targetFlat,
          locale: targetLocale,
          lockfileEntries,
        },
        force,
      );

      if (diff.missing.length === 0 && diff.stale.length === 0) {
        skipped += Object.keys(targetFlat).length;
        continue;
      }

      const entriesToTranslate: Array<{ key: string; source: string }> = [
        ...diff.missing,
        ...diff.stale.map((s) => ({ key: s.key, source: s.source })),
      ];

      const results = await translateBatch({
        provider,
        modelId: model,
        entries: entriesToTranslate.map((e) => ({ key: e.key, source: e.source })),
        targetLocale,
        sourceLocale: defaultLocale,
        glossary,
        maxRetries,
        processor,
        baseURL,
        apiKey,
        hook: effectiveHook,
        modelParams,
      });

      const successResults = results.filter((r) => r.success && r.translated !== undefined);
      const failedResults = results.filter((r) => !r.success || r.translated === undefined);
      errors += failedResults.length;

      for (const r of failedResults) {
        failures.push({
          locale: targetLocale,
          key: r.key,
          source: sourceFlat[r.key] ?? "",
          error: r.error ?? "Unknown error",
          errorType: r.errorType,
        });
      }

      // No quality loop: original behavior.
      if (!qualityEnabled || successResults.length === 0) {
        for (const result of successResults) {
          targetFlat[result.key] = result.translated!;
          const source = sourceFlat[result.key] ?? "";
          const sourceHash = await lockfileManager.hashSource(source);
          lockfileManager.setEntry(result.key, targetLocale, {
            key: result.key,
            locale: targetLocale,
            sourceHash,
            translated: result.translated!,
            origin: "ai",
            model: model,
            timestamp: new Date().toISOString(),
          });
          translated++;
        }

        options?.onProgress?.({
          locale: targetLocale,
          completed: translated,
          total: entriesToTranslate.length,
        });

        if (!dryRun) {
          await format.writeLocale(localeDir, targetLocale, targetFlat);
          await lockfileManager.save();
        }
        continue;
      }

      // Quality-aware path.
      const sourceHashByKey = new Map<string, string>();
      for (const r of successResults) {
        const source = sourceFlat[r.key] ?? "";
        sourceHashByKey.set(r.key, await lockfileManager.hashSource(source));
      }

      const judge = customAssessor
        ? undefined
        : (ctxs: TranslationContext[]) =>
            judgeBatch({
              provider,
              modelId: model,
              baseURL,
              apiKey,
              hook: effectiveHook,
              modelParams,
              contexts: ctxs,
            });

      const loopResult = await runQualityLoop({
        initialTranslations: successResults.map((r) => ({
          key: r.key,
          source: sourceFlat[r.key] ?? "",
          translated: r.translated!,
        })),
        sourceHashByKey,
        targetLocale,
        provider: modelToString(provider),
        model: model,
        judge,
        assessor: customAssessor,
        refill: async (reqs: RefillRequest[]) => {
          const refillBatches = chunk(reqs, batchSize ?? Infinity);
          const result: TranslationResult[] = [];
          for (const batch of refillBatches) {
            const batchResult = await translateBatch({
              provider,
              modelId: model,
              entries: batch.map((e) => ({ key: e.key, source: e.source })),
              targetLocale,
              sourceLocale: defaultLocale,
              glossary,
              maxRetries,
              processor,
              baseURL,
              apiKey,
              hook: effectiveHook,
              modelParams,
              feedback: Object.fromEntries(batch.map((r) => [r.key, summarizeRefillPrompt(r)])),
            });
            result.push(...batchResult);
          }
          return result;
        },
        options: { threshold, maxRetries: maxQualityRetries },
      });

      needsReview += loopResult.belowThreshold.length;

      if (failOnLowQuality && loopResult.belowThreshold.length > 0) {
        throw new IntlAiQualityError(targetLocale, loopResult.belowThreshold);
      }

      for (const [key, text] of loopResult.finalByKey) {
        targetFlat[key] = text;
        const source = sourceFlat[key] ?? "";
        const sourceHash = await lockfileManager.hashSource(source);
        const quality = loopResult.qualityByKey.get(key);
        const record: LockfileQuality | undefined = quality
          ? {
              score: quality.score,
              riskTier: quality.riskTier,
              needsReview: quality.needsReview,
              errorTypes: quality.errorTypes,
              reason: quality.reason,
              assessorName: quality.assessorName,
              assessedAt: new Date().toISOString(),
            }
          : undefined;
        lockfileManager.setEntry(key, targetLocale, {
          key,
          locale: targetLocale,
          sourceHash,
          translated: text,
          origin: "ai",
          model: model,
          timestamp: new Date().toISOString(),
          quality: record,
        });
        translated++;
      }

      if (!dryRun) {
        await format.writeLocale(localeDir, targetLocale, targetFlat);
        await lockfileManager.save();
      }
    } catch (err) {
      if (err instanceof IntlAiQualityError) {
        throw err;
      }
      errors++;
    }
  }

  return { locales: targetLocales, translated, skipped, errors, needsReview, failures };
}

function summarizeRefillPrompt(req: RefillRequest): string {
  const parts: string[] = [];
  if (req.feedback) parts.push(req.feedback);
  parts.push(`Previous attempt: ${req.previousTranslation}`);
  return parts.join("\n");
}

function modelToString(model: AIProvider | string): string {
  if (typeof model === "string") return model;
  return model.id;
}

function chunk<T>(arr: T[], size: number): T[][] {
  // ponytail: Infinity (the default) collapses to a single all-in-one batch.
  if (!Number.isFinite(size)) return [arr];
  const out: T[][] = [];
  for (let i = 0; i < arr.length; i += size) out.push(arr.slice(i, i + size));
  return out;
}
