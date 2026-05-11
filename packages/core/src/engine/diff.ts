import { createHash } from "crypto";

export interface TranslationDiff {
  locale: string;
  missing: Array<{ key: string; source: string; locale: string }>;
  stale: Array<{
    key: string;
    locale: string;
    reason: string;
    currentHash?: string;
    lockedHash: string;
  }>;
  extra: string[];
}

export interface DiffOptions {
  sourceLocale: Record<string, unknown>;
  targetLocale: Record<string, unknown>;
  locale: string;
  lockfileEntries?: Map<string, { sourceHash: string }>;
}

export function findMissingTranslations(options: DiffOptions): TranslationDiff {
  const { sourceLocale, targetLocale, locale, lockfileEntries } = options;

  const sourceKeys = flattenObject(sourceLocale);
  const targetKeys = flattenObject(targetLocale);

  const missing: Array<{ key: string; source: string; locale: string }> = [];
  const stale: Array<{
    key: string;
    locale: string;
    reason: string;
    currentHash?: string;
    lockedHash: string;
  }> = [];
  const extra: string[] = [];

  for (const [key, value] of Object.entries(sourceKeys)) {
    const targetValue = targetKeys[key];
    const isMissing = !(key in targetKeys);
    const isEmpty = typeof targetValue === "string" && targetValue.trim() === "";

    if (isMissing || isEmpty) {
      missing.push({ key, source: String(value), locale });
    } else if (lockfileEntries?.has(key)) {
      const entry = lockfileEntries.get(key)!;
      const currentHash = hashSource(String(value));
      if (currentHash !== entry.sourceHash) {
        stale.push({
          key,
          locale,
          reason: "source_changed",
          currentHash,
          lockedHash: entry.sourceHash,
        });
      }
    }
  }

  for (const key of Object.keys(targetKeys)) {
    if (!(key in sourceKeys)) {
      extra.push(key);
    }
  }

  return { locale, missing, stale, extra };
}

export function flattenObject(obj: Record<string, unknown>, prefix = ""): Record<string, unknown> {
  const result: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === "object" && !Array.isArray(value)) {
      Object.assign(result, flattenObject(value as Record<string, unknown>, fullKey));
    } else {
      result[fullKey] = value;
    }
  }

  return result;
}

export function hashSource(text: string): string {
  return createHash("sha1").update(text).digest("hex");
}
