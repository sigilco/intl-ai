import { rename } from "node:fs/promises";
import { join, readText, writeText, pathExists } from "../infrastructure/fs";
import { hashSha1 } from "../core/hash";
import { LOCKFILE_NAME, type Lockfile, type LockfileEntry } from "./types";

const EMPTY_LOCKFILE: Lockfile = { version: 2, entries: {} };

function compositeKey(locale: string, key: string): string {
  return `${locale}||${key}`;
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

        if (typeof parsed !== "object" || parsed === null) {
          throw new Error("lockfile must be an object");
        }

        if (parsed.version === 1) {
          const migrated: Record<string, LockfileEntry> = {};
          for (const [, entry] of Object.entries(parsed.entries ?? {})) {
            // ponytail: skip malformed entries (hand-edited lockfiles)
            if (!entry.locale || !entry.key) continue;
            migrated[compositeKey(entry.locale, entry.key)] = entry;
          }
          this.data = {
            version: 2,
            entries: migrated,
            migratedAt: new Date().toISOString(),
          };
          // B4: persist migration immediately
          await this.#atomicSave();
        } else if (parsed.version === 2) {
          this.data = { version: 2, entries: parsed.entries ?? {} };
        } else {
          throw new Error(`Unknown lockfile version: ${parsed.version}`);
        }
      } catch (err) {
        // B2: do not silently wipe on parse/migration error — fail safe.
        // Leave this.data at EMPTY and loaded=false so save() is a no-op.
        // Re-throw so caller knows the lockfile is corrupt.
        throw new Error(
          `Failed to load lockfile (${path}): ${err instanceof Error ? err.message : String(err)}`,
        );
      }
    } else {
      this.data = { ...EMPTY_LOCKFILE };
    }
    this.loaded = true;
    return this.data;
  }

  async save(): Promise<void> {
    if (!this.loaded) return;
    await this.#atomicSave();
  }

  /** Atomic write: write to temp file then rename (POSIX atomic, Windows near-atomic). */
  async #atomicSave(): Promise<void> {
    const path = join(this.localeDir, LOCKFILE_NAME);
    const tmp = path + ".tmp";
    await writeText(tmp, JSON.stringify(this.data, null, 2));
    await rename(tmp, path);
  }

  private ensureLoaded(): void {
    if (!this.loaded) {
      this.data = { ...EMPTY_LOCKFILE };
    }
  }

  getEntry(key: string, locale: string): LockfileEntry | undefined {
    this.ensureLoaded();
    return this.data.entries[compositeKey(locale, key)];
  }

  setEntry(key: string, locale: string, entry: LockfileEntry): void {
    this.ensureLoaded();
    this.data.entries[compositeKey(locale, key)] = entry;
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
}
