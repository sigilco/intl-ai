import { join, readText, writeText, pathExists, getNestedValue } from "../utils/fs";
import { hashSha1 } from "../utils/hash";
import { LOCKFILE_NAME, type Lockfile, type LockfileEntry, type StaleEntry } from "./types";

const EMPTY_LOCKFILE: Lockfile = { version: 1, entries: {} };

function lockfileKey(locale: string, key: string): string {
  return `${locale}:${key}`;
}

export class LockfileManager {
  private localeDir: string;
  private data: Lockfile = { ...EMPTY_LOCKFILE };
  private loaded = false;

  constructor(localeDir: string) {
    this.localeDir = localeDir;
  }

  async load(): Promise<Lockfile> {
    const path = join(this.localeDir, LOCKFILE_NAME);
    if (await pathExists(path)) {
      try {
        const raw = await readText(path);
        const parsed = JSON.parse(raw) as Lockfile;
        if (parsed && typeof parsed === "object" && parsed.version === 1) {
          this.data = { version: 1, entries: parsed.entries ?? {} };
        }
      } catch {
        this.data = { ...EMPTY_LOCKFILE };
      }
    } else {
      this.data = { ...EMPTY_LOCKFILE };
    }
    this.loaded = true;
    return this.data;
  }

  async save(): Promise<void> {
    const path = join(this.localeDir, LOCKFILE_NAME);
    await writeText(path, JSON.stringify(this.data, null, 2));
  }

  private ensureLoaded(): void {
    if (!this.loaded) {
      this.data = { ...EMPTY_LOCKFILE };
    }
  }

  getEntry(key: string, locale: string): LockfileEntry | undefined {
    this.ensureLoaded();
    return this.data.entries[lockfileKey(locale, key)];
  }

  setEntry(key: string, locale: string, entry: LockfileEntry): void {
    this.ensureLoaded();
    this.data.entries[lockfileKey(locale, key)] = entry;
  }

  isHumanEdited(key: string, locale: string): boolean {
    return this.getEntry(key, locale)?.origin === "human";
  }

  isStale(key: string, locale: string, sourceHash: string): boolean {
    const entry = this.getEntry(key, locale);
    return !!entry && entry.sourceHash !== sourceHash;
  }

  async hashSource(text: string): Promise<string> {
    return hashSha1(text);
  }

  getAllEntries(): Record<string, LockfileEntry> {
    this.ensureLoaded();
    return this.data.entries;
  }

  async getStaleEntries(
    targetLocale: string,
    sourceLocaleData: Record<string, unknown>,
  ): Promise<StaleEntry[]> {
    this.ensureLoaded();
    const stale: StaleEntry[] = [];
    for (const [compositeKey, entry] of Object.entries(this.data.entries)) {
      const [entryLocale, entryKey] = compositeKey.split(":");
      if (entryLocale !== targetLocale) continue;

      const source = getNestedValue(sourceLocaleData, entryKey);
      const currentHash = await hashSha1(source);
      if (currentHash !== entry.sourceHash) {
        stale.push({
          key: entryKey,
          source,
          previous: entry.translated,
          sourceHash: entry.sourceHash,
        });
      }
    }
    return stale;
  }
}
