import type { QualityResult } from "../core/types";

/**
 * Per-key quality record persisted in `intl-ai.lock.json`. Shape mirrors
 * `QualityResult` from `@intl-ai/api` but adds `assessedAt` and snapshots
 * `errorTypes` as a plain field rather than optional.
 */
export interface LockfileQuality {
  score: number;
  riskTier: QualityResult["riskTier"];
  needsReview: boolean;
  errorTypes: QualityResult["errorTypes"];
  reason?: string;
  assessorName: string;
  assessedAt: string;
}

export type QualityRecord = LockfileQuality;

export interface LockfileEntry {
  key: string;
  locale: string;
  sourceHash: string;
  translated: string;
  origin: "ai" | "human";
  model?: string;
  timestamp: string;
  quality?: LockfileQuality;
}

export type LockfileQualityRecord = LockfileQuality;

export interface Lockfile {
  version: 1;
  entries: Record<string, LockfileEntry>;
}

export interface StaleEntry {
  key: string;
  source: string;
  previous: string;
  sourceHash: string;
}

export const LOCKFILE_NAME = "intl-ai.lock.json";
