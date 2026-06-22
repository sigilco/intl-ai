import { hashSha1 } from "../utils/hash";
import type { LockfileEntry, StaleEntry } from "../lockfile/types";

export interface MissingTranslationEntry {
  key: string;
  source: string;
}

export interface FindMissingTranslationsOptions {
  sourceLocale: Record<string, unknown>;
  targetLocale: Record<string, unknown>;
  locale: string;
  lockfileEntries: Map<string, { sourceHash: string }>;
}

export interface FindMissingTranslationsResult {
  locale: string;
  missing: MissingTranslationEntry[];
  stale: StaleEntry[];
  extra: string[];
}

/**
 * Walk both locale objects, detect keys present in source but missing in target
 * (or empty in target), and keys present in target but not in source.
 *
 * @param force  if true, ignore lockfile staleness and treat every key as missing
 */
export async function findMissingTranslations(
  options: FindMissingTranslationsOptions,
  force = false,
): Promise<FindMissingTranslationsResult> {
  const { sourceLocale, targetLocale, locale, lockfileEntries } = options;
  const sourceFlat = flattenObject(sourceLocale);
  const targetFlat = flattenObject(targetLocale);

  const missing: MissingTranslationEntry[] = [];
  const stale: StaleEntry[] = [];

  for (const [key, source] of Object.entries(sourceFlat)) {
    const target = targetFlat[key];
    const sourceHash = await hashSha1(source);

    if (target === undefined || target === "") {
      missing.push({ key, source });
      continue;
    }

    if (force) {
      missing.push({ key, source });
      continue;
    }

    const entry = lockfileEntries.get(key);
    if (entry && entry.sourceHash !== sourceHash) {
      stale.push({ key, source, previous: target, sourceHash });
    }
  }

  const extra = Object.keys(targetFlat).filter((key) => !(key in sourceFlat));

  return { locale, missing, stale, extra };
}

/**
 * Flatten a nested object to dot-notation keys.
 * Arrays are NOT flattened — they preserve as JSON arrays in their parent.
 */
export function flattenObject(obj: unknown, prefix = ""): Record<string, string> {
  const result: Record<string, string> = {};
  if (obj === null || obj === undefined) return result;
  if (typeof obj !== "object") {
    if (prefix) result[prefix] = String(obj);
    return result;
  }
  if (Array.isArray(obj)) {
    if (prefix) result[prefix] = JSON.stringify(obj);
    return result;
  }
  for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (v !== null && typeof v === "object" && !Array.isArray(v)) {
      Object.assign(result, flattenObject(v, key));
    } else {
      result[key] = Array.isArray(v) ? JSON.stringify(v) : String(v);
    }
  }
  return result;
}

export async function hashSource(text: string): Promise<string> {
  return hashSha1(text);
}

export function lockfileEntryToMap(
  entries: Record<string, LockfileEntry>,
  locale: string,
): Map<string, { sourceHash: string }> {
  const m = new Map<string, { sourceHash: string }>();
  for (const [compositeKey, entry] of Object.entries(entries)) {
    const [entryLocale, key] = compositeKey.split(":");
    if (entryLocale === locale) {
      m.set(key, { sourceHash: entry.sourceHash });
    }
  }
  return m;
}
