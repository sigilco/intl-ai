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

// --- Quality estimation (issue #5) + retry loop (issue #14) ---

export const QUALITY_ERROR_TYPES = [
  "accuracy",
  "fluency",
  "terminology",
  "style",
  "locale_convention",
] as const;
export type QualityErrorType = (typeof QUALITY_ERROR_TYPES)[number];

export const QUALITY_SEVERITIES = ["critical", "major", "minor"] as const;
export type QualitySeverity = (typeof QUALITY_SEVERITIES)[number];

export interface QualityError {
  type: QualityErrorType;
  severity: QualitySeverity;
  span?: { start: number; end: number };
  description?: string;
}

export interface QualityResult {
  /** 0..1; below `threshold` triggers a refill attempt. */
  score: number;
  /** Coarse bucket derived from score unless explicitly set by a custom assessor. */
  riskTier: "low" | "medium" | "high";
  /** True for any entry the fill loop should surface to humans. */
  needsReview: boolean;
  /** Granular errors. Empty when the assessor only emits a score. */
  errorTypes?: QualityError[];
  /** Free-text explanation from the assessor. */
  reason?: string;
  /** Identifier of the assessor that produced this result. */
  assessorName: string;
}

export interface TranslationContext {
  key: string;
  source: string;
  translation: string;
  locale: string;
  sourceHash: string;
  origin: "ai" | "human";
  model: string;
  provider: string;
}

/**
 * Escape hatch for users who want bespoke judgment logic. The default
 * quality assessor uses the configured provider with an adversarial
 * prompt; supplying an instance here replaces that path entirely.
 */
export interface QualityAssessorInstance {
  /** Stable identifier written to `LockfileEntry.quality.assessorName`. */
  name: string;
  assess(ctx: TranslationContext): Promise<QualityResult>;
}

/**
 * Settings for the quality-aware fill loop. Lives both on
 * `IntlAiConfig.quality` (from JSON config) and on `RunFillOptions.quality`
 * (programmatic override).
 *
 * `failOnLowQuality` is intentionally absent from the JSON schema: it is a
 * runtime-only switch that build plugins set when they want a low-quality
 * build to throw.
 */
export interface QualityOptions {
  /** 0..1. Default 0.8. */
  threshold?: number;
  /** 0..5. Default 2. */
  maxRetries?: number;
  /** When true, `runFill` throws if any key remains below threshold after retries. */
  failOnLowQuality?: boolean;
}

export function isQualityAssessorInstance(
  value: unknown,
): value is QualityAssessorInstance {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as { name?: unknown }).name === "string" &&
    typeof (value as { assess?: unknown }).assess === "function"
  );
}
