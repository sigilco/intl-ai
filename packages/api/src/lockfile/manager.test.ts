import { describe, it, expect } from "vitest";
import { LockfileManager } from "./manager";
import { mkdtemp, rm, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "pathe";

async function createTempDir(): Promise<string> {
  return mkdtemp(join(tmpdir(), "intl-ai-test-"));
}

describe("LockfileManager (api)", () => {
  it("loads empty lockfile when file does not exist", async () => {
    const dir = await createTempDir();
    try {
      const mgr = new LockfileManager(dir);
      const lockfile = await mgr.load();
      expect(lockfile).toEqual({ version: 2, entries: {} });
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("saves and reloads entries", async () => {
    const dir = await createTempDir();
    try {
      const mgr = new LockfileManager(dir);
      await mgr.load();
      mgr.setEntry("greeting", "es", {
        key: "greeting",
        locale: "es",
        sourceHash: "abc",
        translated: "Hola",
        origin: "ai",
        timestamp: new Date().toISOString(),
      });
      await mgr.save();

      const raw = await readFile(join(dir, "intl-ai.lock.json"), "utf-8");
      const parsed = JSON.parse(raw);
      expect(parsed.version).toBe(2);
      expect(parsed.entries["es||greeting"].translated).toBe("Hola");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("migrates v1 lockfiles with ':' separator to v2 '||' separator", async () => {
    const dir = await createTempDir();
    try {
      const v1 = {
        version: 1,
        entries: {
          "es:greeting": {
            key: "greeting",
            locale: "es",
            sourceHash: "abc",
            translated: "Hola",
            origin: "ai",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        },
      };
      await writeFile(join(dir, "intl-ai.lock.json"), JSON.stringify(v1), "utf-8");

      const mgr = new LockfileManager(dir);
      const lockfile = await mgr.load();
      expect(lockfile.version).toBe(2);
      expect(lockfile.entries["es||greeting"].translated).toBe("Hola");
      expect(lockfile.migratedAt).toBeDefined();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("hashSource is stable for same input", async () => {
    const dir = await createTempDir();
    try {
      const mgr = new LockfileManager(dir);
      const h1 = await mgr.hashSource("Hello");
      const h2 = await mgr.hashSource("Hello");
      expect(h1).toBe(h2);
      expect(h1).toMatch(/^[0-9a-f]{40}$/);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("isStale returns true when source hash differs", async () => {
    const dir = await createTempDir();
    try {
      const mgr = new LockfileManager(dir);
      await mgr.load();
      mgr.setEntry("greeting", "es", {
        key: "greeting",
        locale: "es",
        sourceHash: "stale-hash",
        translated: "Hola",
        origin: "ai",
        timestamp: new Date().toISOString(),
      });
      expect(mgr.isStale("greeting", "es", "new-hash")).toBe(true);
      expect(mgr.isStale("greeting", "es", "stale-hash")).toBe(false);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("skips malformed v1 entries (missing locale or key) during migration", async () => {
    const dir = await createTempDir();
    try {
      const v1 = {
        version: 1,
        entries: {
          "es:greeting": {
            key: "greeting",
            locale: "es",
            sourceHash: "abc",
            translated: "Hola",
            origin: "ai",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
          // malformed: no locale
          "es:broken": {
            key: undefined,
            locale: undefined,
            sourceHash: "abc",
            translated: "Hola",
            origin: "ai",
            timestamp: "2024-01-01T00:00:00.000Z",
          },
        },
      };
      await writeFile(join(dir, "intl-ai.lock.json"), JSON.stringify(v1), "utf-8");

      const mgr = new LockfileManager(dir);
      const lockfile = await mgr.load();
      // Valid entry migrated.
      expect(lockfile.entries["es||greeting"]).toBeDefined();
      // Malformed entry skipped (no crash, no ghost key).
      const allKeys = Object.keys(lockfile.entries);
      expect(allKeys).toHaveLength(1);
      expect(allKeys[0]).toBe("es||greeting");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
