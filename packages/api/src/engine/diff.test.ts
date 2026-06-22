import { describe, it, expect } from "vitest";
import { findMissingTranslations, flattenObject } from "./diff";

describe("findMissingTranslations (api)", () => {
  it("returns empty result when target is fully populated", async () => {
    const source = { greeting: "Hello", farewell: "Bye" };
    const target = { greeting: "Hola", farewell: "Adiós" };
    const r = await findMissingTranslations({
      sourceLocale: source,
      targetLocale: target,
      locale: "es",
      lockfileEntries: new Map(),
    });
    expect(r.missing).toHaveLength(0);
    expect(r.stale).toHaveLength(0);
  });

  it("detects missing keys", async () => {
    const source = { greeting: "Hello", farewell: "Bye" };
    const target = { greeting: "Hola" };
    const r = await findMissingTranslations({
      sourceLocale: source,
      targetLocale: target,
      locale: "es",
      lockfileEntries: new Map(),
    });
    expect(r.missing.map((m) => m.key)).toEqual(["farewell"]);
  });

  it("detects empty string in target as missing", async () => {
    const source = { greeting: "Hello" };
    const target = { greeting: "" };
    const r = await findMissingTranslations({
      sourceLocale: source,
      targetLocale: target,
      locale: "es",
      lockfileEntries: new Map(),
    });
    expect(r.missing).toHaveLength(1);
  });

  it("detects stale entries when source hash differs from lockfile", async () => {
    const source = { greeting: "Hello v2" };
    const target = { greeting: "Hola" };
    const lockfileEntries = new Map<string, { sourceHash: string }>([
      ["greeting", { sourceHash: "DIFFERENT_HASH" }],
    ]);
    const r = await findMissingTranslations({
      sourceLocale: source,
      targetLocale: target,
      locale: "es",
      lockfileEntries,
    });
    expect(r.stale).toHaveLength(1);
    expect(r.stale[0].key).toBe("greeting");
  });

  it("force=true treats all keys as missing", async () => {
    const source = { greeting: "Hello" };
    const target = { greeting: "Hola" };
    const r = await findMissingTranslations(
      {
        sourceLocale: source,
        targetLocale: target,
        locale: "es",
        lockfileEntries: new Map(),
      },
      true,
    );
    expect(r.missing).toHaveLength(1);
  });

  it("flattenObject handles nested keys", () => {
    expect(flattenObject({ a: { b: { c: "x" } } })).toEqual({ "a.b.c": "x" });
  });
});
