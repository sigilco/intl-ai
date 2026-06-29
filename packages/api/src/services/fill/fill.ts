import {
  join,
  readText,
  writeText,
  pathExists,
  ensureDir,
  setNestedValue,
  getNestedValue,
} from "../../infrastructure/fs";
import { LockfileManager } from "../../lockfile/manager";
import { findMissingTranslations, lockfileEntryToMap } from "../../core/diff";
import { translateBatch } from "./translator";
import { runQualityLoop, type RefillRequest } from "./loop";
import { judgeBatch } from "./judge";
import type { AIProvider } from "../../ports/provider";
import type { IntlAiConfig } from "../../types";
import type {
  QualityAssessorInstance,
  QualityOptions,
  QualityResult,
  TranslationContext,
} from "../../core/types";
import type { LockfileQuality } from "../../lockfile/types";

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
}

export interface RunFillResult {
  locales: string[];
  translated: number;
  skipped: number;
  errors: number;
  needsReview: number;
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
  config: IntlAiConfig,
  options?: RunFillOptions,
): Promise<RunFillResult> {
  const { force = false, dryRun = false } = options ?? {};
  const {
    defaultLocale,
    locales,
    localeDir,
    model: provider,
    baseURL,
    apiKey,
    glossary,
    maxRetries,
    processor,
    hook,
    modelParams,
    quality: configQuality,
  } = config;

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
    return { locales: [], translated: 0, skipped: 0, errors: 0, needsReview: 0 };
  }

  const lockfileManager = new LockfileManager(localeDir);
  await lockfileManager.load();

  const sourceLocalePath = join(localeDir, `${defaultLocale}.json`);
  if (!(await pathExists(sourceLocalePath))) {
    return { locales: [], translated: 0, skipped: 0, errors: 0, needsReview: 0 };
  }
  const sourceLocaleData = JSON.parse(await readText(sourceLocalePath)) as Record<string, unknown>;

  let translated = 0;
  let skipped = 0;
  let errors = 0;
  let needsReview = 0;

  for (const targetLocale of targetLocales) {
    try {
      const targetLocalePath = join(localeDir, `${targetLocale}.json`);
      let targetLocaleData: Record<string, unknown> = {};
      if (await pathExists(targetLocalePath)) {
        targetLocaleData = JSON.parse(await readText(targetLocalePath));
      }

      const lockfileEntries = lockfileEntryToMap(lockfileManager.getAllEntries(), targetLocale);

      const diff = await findMissingTranslations(
        {
          sourceLocale: sourceLocaleData,
          targetLocale: targetLocaleData,
          locale: targetLocale,
          lockfileEntries,
        },
        force,
      );

      if (diff.missing.length === 0 && diff.stale.length === 0) {
        skipped += Object.keys(flattenKeys(targetLocaleData)).length;
        continue;
      }

      const entriesToTranslate: Array<{ key: string; source: string }> = [
        ...diff.missing,
        ...diff.stale.map((s) => ({ key: s.key, source: s.source })),
      ];

      const results = await translateBatch({
        provider,
        entries: entriesToTranslate.map((e) => ({ key: e.key, source: e.source })),
        targetLocale,
        sourceLocale: defaultLocale,
        glossary,
        maxRetries,
        processor,
        baseURL,
        apiKey,
        hook,
        modelParams,
      });

      const successResults = results.filter(
        (r) => r.success && r.translated !== undefined,
      );
      const failedResults = results.filter(
        (r) => !r.success || r.translated === undefined,
      );
      errors += failedResults.length;

      // No quality loop: original behavior.
      if (!qualityEnabled || successResults.length === 0) {
        for (const result of successResults) {
          setNestedValue(targetLocaleData, result.key, result.translated!);
          const source = getNestedValue(sourceLocaleData, result.key);
          const sourceHash = await lockfileManager.hashSource(source);
          lockfileManager.setEntry(result.key, targetLocale, {
            key: result.key,
            locale: targetLocale,
            sourceHash,
            translated: result.translated!,
            origin: "ai",
            model: modelToString(provider),
            timestamp: new Date().toISOString(),
          });
          translated++;
        }

        if (!dryRun) {
          await ensureDir(localeDir);
          await writeText(targetLocalePath, JSON.stringify(targetLocaleData, null, 2));
          await lockfileManager.save();
        }
        continue;
      }

      // Quality-aware path.
      const sourceHashByKey = new Map<string, string>();
      for (const r of successResults) {
        const source = getNestedValue(sourceLocaleData, r.key);
        sourceHashByKey.set(r.key, await lockfileManager.hashSource(source));
      }

      const judge = customAssessor
        ? undefined
        : (ctxs: TranslationContext[]) =>
            judgeBatch({
              provider,
              baseURL,
              apiKey,
              hook,
              modelParams,
              contexts: ctxs,
            });

      const loopResult = await runQualityLoop({
        initialTranslations: successResults.map((r) => ({
          key: r.key,
          source: getNestedValue(sourceLocaleData, r.key),
          translated: r.translated!,
        })),
        sourceHashByKey,
        targetLocale,
        provider: modelToString(provider),
        model: modelToString(provider),
        judge,
        assessor: customAssessor,
        refill: async (reqs: RefillRequest[]) => {
          const refillResults = await translateBatch({
            provider,
            entries: reqs.map((e) => ({ key: e.key, source: e.source })),
            targetLocale,
            sourceLocale: defaultLocale,
            glossary,
            maxRetries,
            processor,
            baseURL,
            apiKey,
            hook,
            modelParams,
            feedback: Object.fromEntries(
              reqs.map((r) => [r.key, summarizeRefillPrompt(r)]),
            ),
          });
          return refillResults;
        },
        options: { threshold, maxRetries: maxQualityRetries },
      });

      for (const r of loopResult.belowThreshold) {
        needsReview++;
      }

      if (failOnLowQuality && loopResult.belowThreshold.length > 0) {
        throw new IntlAiQualityError(targetLocale, loopResult.belowThreshold);
      }

      for (const [key, text] of loopResult.finalByKey) {
        setNestedValue(targetLocaleData, key, text);
        const source = getNestedValue(sourceLocaleData, key);
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
          model: modelToString(provider),
          timestamp: new Date().toISOString(),
          quality: record,
        });
        translated++;
      }

      if (!dryRun) {
        await ensureDir(localeDir);
        await writeText(targetLocalePath, JSON.stringify(targetLocaleData, null, 2));
        await lockfileManager.save();
      }
    } catch (err) {
      if (err instanceof IntlAiQualityError) {
        throw err;
      }
      errors++;
    }
  }

  return { locales: targetLocales, translated, skipped, errors, needsReview };
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

function flattenKeys(obj: Record<string, unknown>): Record<string, string> {
  const out: Record<string, string> = {};
  function walk(o: unknown, prefix: string) {
    if (o === null || o === undefined) return;
    if (typeof o !== "object") {
      if (prefix) out[prefix] = String(o);
      return;
    }
    if (Array.isArray(o)) {
      if (prefix) out[prefix] = JSON.stringify(o);
      return;
    }
    for (const [k, v] of Object.entries(o as Record<string, unknown>)) {
      const key = prefix ? `${prefix}.${k}` : k;
      if (v !== null && typeof v === "object" && !Array.isArray(v)) walk(v, key);
      else out[key] = Array.isArray(v) ? JSON.stringify(v) : String(v);
    }
  }
  walk(obj, "");
  return out;
}
