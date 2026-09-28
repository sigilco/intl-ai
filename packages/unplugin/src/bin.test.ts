// Binary-resolution tests for PATH fallback and the not-found error.
// createRequire is mocked out so the `intl-ai` npm-package resolution
// branch (which succeeds inside this workspace) is skipped; the npm
// branch itself is covered by real installs of the package.
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, test, vi } from "vitest";

vi.mock("node:module", async (importOriginal) => {
  const orig = await importOriginal<typeof import("node:module")>();
  return {
    ...orig,
    createRequire: () => ({
      resolve: () => {
        throw new Error("mocked: npm resolution unavailable");
      },
    }),
  };
});

const { resolveIntlAiBin } = await import("./run");

const tmp = mkdtempSync(join(tmpdir(), "intl-ai-bin-"));
afterAll(() => rmSync(tmp, { recursive: true, force: true }));

describe("resolveIntlAiBin", () => {
  test("explicit bin option wins", () => {
    expect(resolveIntlAiBin("/opt/intl-ai").command).toBe("/opt/intl-ai");
  });

  test("INTL_AI_BIN env wins over PATH", () => {
    process.env.INTL_AI_BIN = "/env/intl-ai";
    expect(resolveIntlAiBin(undefined, tmp).command).toBe("/env/intl-ai");
    delete process.env.INTL_AI_BIN;
  });

  test("falls back to PATH", () => {
    const dir = join(tmp, "path-bin");
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, "intl-ai"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    const prev = process.env.PATH;
    process.env.PATH = dir;
    try {
      expect(resolveIntlAiBin(undefined, tmp).command).toBe(join(dir, "intl-ai"));
    } finally {
      process.env.PATH = prev;
    }
  });

  test("throws an actionable error when nothing resolves", () => {
    const prev = process.env.PATH;
    process.env.PATH = join(tmp, "empty-path");
    try {
      expect(() => resolveIntlAiBin(undefined, tmp)).toThrow(/intl-ai binary not found/);
    } finally {
      process.env.PATH = prev;
    }
  });
});
