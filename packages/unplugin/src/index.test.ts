import { mkdtempSync, writeFileSync, readFileSync, chmodSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, test, vi } from "vitest";
import { unplugin } from "./index";
import { buildCheckArgs, buildFillArgs, normalizeFailOn, shouldSkipInDev } from "./run";

const tmp = mkdtempSync(join(tmpdir(), "intl-ai-unplugin-"));
afterAll(() => rmSync(tmp, { recursive: true, force: true }));

function stubBin(): string {
  const script = join(tmp, "intl-ai-stub.cjs");
  writeFileSync(
    script,
    [
      "#!/usr/bin/env node",
      'const fs = require("node:fs");',
      "const argv = process.argv.slice(2);",
      'fs.appendFileSync(process.env.INTL_AI_STUB_LOG, argv.join(" ") + "\\n");',
      'const code = argv.includes("check")',
      "  ? Number(process.env.INTL_AI_STUB_CHECK_EXIT || 0)",
      "  : Number(process.env.INTL_AI_STUB_EXIT || 0);",
      "process.exit(code);",
    ].join("\n"),
  );
  chmodSync(script, 0o755);
  return script;
}

async function invokeBuildStart(options: Record<string, unknown>, log: string) {
  process.env.INTL_AI_STUB_LOG = log;
  const plugin = unplugin.raw(options as never, { framework: "rollup" } as never);
  const warnings: string[] = [];
  const errors: string[] = [];
  const warnSpy = vi.spyOn(console, "warn").mockImplementation((m?: unknown) => {
    warnings.push(String(m));
  });
  const hook = (plugin as { buildStart: unknown }).buildStart as
    | ((this: unknown) => Promise<void>)
    | { handler: (this: unknown) => Promise<void> };
  const fn = typeof hook === "function" ? hook : hook.handler;
  try {
    await fn.call({});
  } catch (e) {
    errors.push((e as Error).message);
  } finally {
    warnSpy.mockRestore();
  }
  return { warnings, errors };
}

describe("@intl-ai/unplugin shim", () => {
  test("exposes the same adapter surface as before", () => {
    for (const name of [
      "vite",
      "webpack",
      "rollup",
      "esbuild",
      "rspack",
      "rolldown",
      "farm",
      "bun",
    ]) {
      expect(typeof unplugin[name as keyof typeof unplugin]).toBe("function");
    }
  });

  test("buildFillArgs maps options to the CLI surface", () => {
    expect(buildFillArgs({})).toEqual(["fill"]);
    expect(buildFillArgs({ validate: false })).toEqual(["fill", "--no-validate"]);
    expect(buildFillArgs({ validate: ["icu", "judge"] })).toEqual([
      "fill",
      "--validate",
      "icu,judge",
    ]);
    expect(buildFillArgs({ config: "conf/intl-ai.toml", judgeThreshold: 0.9 })).toEqual([
      "--config",
      "conf/intl-ai.toml",
      "fill",
      "--judge-threshold",
      "0.9",
    ]);
    expect(buildFillArgs({ args: ["--locale", "fr"] })).toEqual(["fill", "--locale", "fr"]);
  });

  test("failOn builds a check gate", () => {
    expect(normalizeFailOn("missing, stale")).toEqual(["missing", "stale"]);
    expect(normalizeFailOn(["invalid"])).toEqual(["invalid"]);
    expect(normalizeFailOn(undefined)).toEqual([]);
    expect(buildCheckArgs({}, ["missing", "invalid"])).toEqual([
      "check",
      "--fail-on",
      "missing,invalid",
    ]);
  });

  test("dev: false skips outside production", () => {
    const prev = process.env.NODE_ENV;
    process.env.NODE_ENV = "development";
    expect(shouldSkipInDev({ dev: false })).toBe(true);
    expect(shouldSkipInDev({ dev: true })).toBe(false);
    process.env.NODE_ENV = "production";
    expect(shouldSkipInDev({ dev: false })).toBe(false);
    process.env.NODE_ENV = prev;
  });

  test("buildStart runs fill then check and streams argv to the binary", async () => {
    const log = join(tmp, "run1.log");
    const { errors } = await invokeBuildStart(
      {
        bin: stubBin(),
        failOn: ["missing", "invalid"],
        validate: ["icu"],
        judgeThreshold: 0.9,
      },
      log,
    );
    expect(errors).toEqual([]);
    const lines = readFileSync(log, "utf8").trim().split("\n");
    expect(lines).toEqual([
      "fill --validate icu --judge-threshold 0.9",
      "check --fail-on missing,invalid",
    ]);
  });

  test("nonzero check exit fails the build via a thrown error", async () => {
    const log = join(tmp, "run2.log");
    process.env.INTL_AI_STUB_CHECK_EXIT = "10";
    const { errors } = await invokeBuildStart({ bin: stubBin(), failOn: "missing" }, log);
    delete process.env.INTL_AI_STUB_CHECK_EXIT;
    expect(errors).toHaveLength(1);
    expect(errors[0]).toContain("intl-ai check --fail-on missing exited with code 10");
    expect(errors[0]).toContain("@intl-ai");
    expect(readFileSync(log, "utf8").trim().split("\n")).toEqual([
      "fill",
      "check --fail-on missing",
    ]);
  });

  test("strict: false degrades failures to warnings", async () => {
    const log = join(tmp, "run3.log");
    process.env.INTL_AI_STUB_EXIT = "1";
    const { warnings, errors } = await invokeBuildStart({ bin: stubBin(), strict: false }, log);
    delete process.env.INTL_AI_STUB_EXIT;
    expect(errors).toEqual([]);
    expect(warnings[0]).toContain("intl-ai fill exited with code 1");
  });
});
