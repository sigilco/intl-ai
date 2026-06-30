import { describe, it, expect } from "vitest";
import { mkdtemp, rm, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "pathe";
import { jsonFormat } from "./json";
import { yamlFormat } from "./yaml";

async function createTempDir(): Promise<string> {
  return mkdtemp(join(tmpdir(), "intl-ai-formats-"));
}

describe("jsonFormat", () => {
  it("round-trips flat dot-notation keys", async () => {
    const dir = await createTempDir();
    try {
      const data = { "a.b": "x", c: "y" };
      await jsonFormat.writeLocale(dir, "en", data);
      const read = await jsonFormat.readLocale(dir, "en");
      expect(read).toEqual(data);

      const raw = await readFile(join(dir, "en.json"), "utf-8");
      expect(JSON.parse(raw)).toEqual({ a: { b: "x" }, c: "y" });
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});

describe("yamlFormat", () => {
  it("round-trips flat dot-notation keys", async () => {
    const dir = await createTempDir();
    try {
      const data = { "a.b": "x", c: "y" };
      await yamlFormat.writeLocale(dir, "en", data);
      const read = await yamlFormat.readLocale(dir, "en");
      expect(read).toEqual(data);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
