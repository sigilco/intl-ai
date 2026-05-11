// Lockfile Entry - Per-translation tracking
export interface LockfileEntry {
  key: string; // Dot-notation key path
  locale: string; // Target locale
  sourceHash: string; // SHA-1 hash of source text
  translated: string; // The translated text
  origin: "ai" | "human"; // Who last edited
  model?: string; // Model ID if AI-generated
  timestamp: string; // ISO timestamp of last change
}

// Root lockfile structure
export interface Lockfile {
  version: 1; // Always 1 for v1 schema
  entries: Record<string, LockfileEntry>; // key -> entry
}

// Stale entry for staleness detection
export interface StaleEntry {
  key: string;
  locale: string;
  reason: "source_changed" | "source_deleted";
  currentHash?: string;
  lockedHash: string;
}

// Lockfile file name
export const LOCKFILE_NAME = "intl-ai.lock.json";
