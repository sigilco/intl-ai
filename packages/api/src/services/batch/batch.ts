import { runFill } from "../fill/fill";
import type { RunFillOptions, RunFillResult, FillFailure } from "../fill/fill";
import { ensureDir, writeText } from "../../infrastructure/fs";
import { join as pathJoin } from "pathe";
import type { ResolvedIntlAiConfig } from "../../infrastructure/config/loader";

export interface BatchedFillOptions extends RunFillOptions {
  /** Max parallel locale processing. Defaults to 4. */
  concurrency?: number;
}

export interface BatchedFillResult extends RunFillResult {}

export async function batchedFill(
  config: ResolvedIntlAiConfig,
  options?: BatchedFillOptions,
): Promise<BatchedFillResult> {
  const { concurrency = 4, ...runFillOptions } = options ?? {};
  const { defaultLocale, locales, localeDir } = config;
  const targetLocales = runFillOptions.locale
    ? locales.filter((l) => l === runFillOptions.locale)
    : locales.filter((l) => l !== defaultLocale);

  if (targetLocales.length === 0) {
    return { locales: [], translated: 0, skipped: 0, errors: 0, needsReview: 0, failures: [] };
  }

  const results = await mapPool(targetLocales, concurrency, (locale) =>
    runFill(config, { ...runFillOptions, locale }),
  );

  const aggregated: BatchedFillResult = {
    locales: targetLocales,
    translated: 0,
    skipped: 0,
    errors: 0,
    needsReview: 0,
    failures: [] as FillFailure[],
  };

  for (const r of results) {
    aggregated.translated += r.translated;
    aggregated.skipped += r.skipped;
    aggregated.errors += r.errors;
    aggregated.needsReview += r.needsReview;
    aggregated.failures.push(...r.failures);
  }

  if (aggregated.failures.length > 0 && !runFillOptions.dryRun) {
    const reportDir = pathJoin(localeDir, ".intl-ai");
    await ensureDir(reportDir);
    const ts = new Date().toISOString().replace(/[:.]/g, "-");
    const reportPath = pathJoin(reportDir, `report-${ts}.json`);
    const report = {
      version: 1,
      generatedAt: new Date().toISOString(),
      locales: targetLocales,
      summary: {
        total: aggregated.failures.length,
        byType: groupBy(aggregated.failures.map((f) => f.errorType ?? "unknown")),
      },
      failures: aggregated.failures,
    };
    await writeText(reportPath, JSON.stringify(report, null, 2));
  }

  return aggregated;
}

function groupBy(items: string[]): Record<string, number> {
  const out: Record<string, number> = {};
  for (const item of items) out[item] = (out[item] ?? 0) + 1;
  return out;
}

async function mapPool<T, R>(
  items: T[],
  concurrency: number,
  fn: (item: T) => Promise<R>,
): Promise<R[]> {
  if (concurrency <= 1 || items.length <= 1) {
    const out: R[] = [];
    for (const item of items) out.push(await fn(item));
    return out;
  }
  const results: R[] = Array.from({ length: items.length });
  let next = 0;
  async function worker(): Promise<void> {
    while (next < items.length) {
      const idx = next++;
      results[idx] = await fn(items[idx]!);
    }
  }
  const workers = Array.from({ length: Math.min(concurrency, items.length) }, () => worker());
  await Promise.all(workers);
  return results;
}
