import { hashSha1 } from "./hash";
import type { LockfileEntry, StaleEntry } from "../lockfile/types";
import type {
  MissingTranslationEntry,
  FindMissingTranslationsOptions,
  FindMissingTranslationsResult,
} from "./types";

export type {
  MissingTranslationEntry,
  FindMissingTranslationsOptions,
  FindMissingTranslationsResult,
};

export async function findMissingTranslations(
  options: FindMissingTranslationsOptions,
  force = false,
): Promise<FindMissingTranslationsResult> {
  const { sourceLocale, targetLocale, locale, lockfileEntries } = options;
  // ponytail: safe — flattenObject is idempotent on already-flat string-value records.
  const sourceFlat = flattenObject(sourceLocale);
  const targetFlat = flattenObject(targetLocale);

  const missing: MissingTranslationEntry[] = [];
  const stale: StaleEntry[] = [];

  for (const [key, source] of Object.entries(sourceFlat)) {
    const target = targetFlat[key];
    const sourceHash = await hashSha1(source);

    // An empty string is a valid translation; only undefined means missing.
    if (target === undefined) {
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

export function unflattenObject(flat: Record<string, string>): Record<string, unknown> {
  const out = Object.create(null) as Record<string, unknown>;
  for (const [key, value] of Object.entries(flat)) {
    const parts = key.split(".");
    let cur = out;
    for (let i = 0; i < parts.length - 1; i++) {
      const p = parts[i]!;
      // ponytail: guard against prototype pollution via malicious flat keys.
      if (p === "__proto__" || p === "constructor" || p === "prototype") break;
      if (cur[p] === undefined || typeof cur[p] !== "object") {
        cur[p] = {};
      }
      cur = cur[p] as Record<string, unknown>;
    }
    const lastPart = parts[parts.length - 1]!;
    if (lastPart !== "__proto__" && lastPart !== "constructor" && lastPart !== "prototype") {
      cur[lastPart] = value;
    }
  }
  return out;
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
    // locale never contains '||' by construction; split on first occurrence.
    const idx = compositeKey.indexOf("||");
    if (idx === -1) continue;
    const entryLocale = compositeKey.slice(0, idx);
    if (entryLocale === locale) {
      const key = compositeKey.slice(idx + 2);
      m.set(key, { sourceHash: entry.sourceHash });
    }
  }
  return m;
}
