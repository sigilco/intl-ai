import { describe, it, expect } from "vitest";
import { LockfileManager } from "./manager";
import { mkdtemp, rm, readFile } from "node:fs/promises";
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
      expect(lockfile).toEqual({ version: 1, entries: {} });
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
      expect(parsed.version).toBe(1);
      expect(parsed.entries["es:greeting"].translated).toBe("Hola");
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

  it("getStaleEntries detects changed sources", async () => {
    const dir = await createTempDir();
    try {
      const mgr = new LockfileManager(dir);
      await mgr.load();
      const oldHash = await mgr.hashSource("Hello v1");
      mgr.setEntry("greeting", "es", {
        key: "greeting",
        locale: "es",
        sourceHash: oldHash,
        translated: "Hola",
        origin: "ai",
        timestamp: new Date().toISOString(),
      });
      const stale = await mgr.getStaleEntries("es", { greeting: "Hello v2" });
      expect(stale).toHaveLength(1);
      expect(stale[0].key).toBe("greeting");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
