export interface ValidationResult {
  valid: boolean;
  errors?: string[];
}

export interface TranslationEntry {
  key: string;
  source: string;
}

export interface TranslationResult {
  key: string;
  translated?: string;
  success: boolean;
  error?: string;
}

export interface TranslationStaleEntry {
  key: string;
  source: string;
  previous: string;
  sourceHash: string;
}

export type ApiKeyValue = string;

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
  stale: import("../lockfile/types").StaleEntry[];
  extra: string[];
}
