import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";
import { createHash } from "crypto";
import type { Lockfile, LockfileEntry } from "./types";
import { LOCKFILE_NAME } from "./types";

export class LockfileManager {
  private lockfile: Lockfile;
  private filePath: string;

  constructor(localeDir: string) {
    this.filePath = join(localeDir, LOCKFILE_NAME);
    this.lockfile = this.load();
  }

  private load(): Lockfile {
    if (!existsSync(this.filePath)) {
      return { version: 1, entries: {} };
    }
    const content = readFileSync(this.filePath, "utf-8");
    return JSON.parse(content) as Lockfile;
  }

  save(): void {
    writeFileSync(this.filePath, JSON.stringify(this.lockfile, null, 2), "utf-8");
  }

  getEntry(key: string, locale: string): LockfileEntry | undefined {
    const entryKey = `${locale}:${key}`;
    return this.lockfile.entries[entryKey];
  }

  setEntry(key: string, locale: string, entry: LockfileEntry): void {
    const entryKey = `${locale}:${key}`;
    this.lockfile.entries[entryKey] = entry;
  }

  isHumanEdited(key: string, locale: string): boolean {
    const entry = this.getEntry(key, locale);
    return entry?.origin === "human";
  }

  isStale(key: string, locale: string, currentSource: string): boolean {
    const entry = this.getEntry(key, locale);
    if (!entry) return false;
    const currentHash = this.hashSource(currentSource);
    return entry.sourceHash !== currentHash;
  }

  hashSource(text: string): string {
    return createHash("sha1").update(text).digest("hex");
  }

  getAllEntries(): Record<string, LockfileEntry> {
    return { ...this.lockfile.entries };
  }

  getStaleEntries(sourceLocale: Record<string, unknown>): Array<{ key: string; locale: string }> {
    const stale: Array<{ key: string; locale: string }> = [];

    for (const [entryKey, entry] of Object.entries(this.lockfile.entries)) {
      const [locale, key] = entryKey.split(":");
      const sourceValue = this.getNestedValue(sourceLocale, key);

      if (sourceValue !== undefined) {
        const currentHash = this.hashSource(String(sourceValue));
        if (currentHash !== entry.sourceHash) {
          stale.push({ key, locale });
        }
      }
    }

    return stale;
  }

  private getNestedValue(obj: Record<string, unknown>, path: string): unknown {
    const parts = path.split(".");
    let current: unknown = obj;

    for (const part of parts) {
      if (current && typeof current === "object") {
        current = (current as Record<string, unknown>)[part];
      } else {
        return undefined;
      }
    }

    return current;
  }
}
