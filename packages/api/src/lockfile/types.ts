export interface LockfileEntry {
  key: string;
  locale: string;
  sourceHash: string;
  translated: string;
  origin: "ai" | "human";
  model?: string;
  timestamp: string;
}

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
