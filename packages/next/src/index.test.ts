import { mkdtempSync, writeFileSync, readFileSync, chmodSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, test, vi } from "vitest";
import type { NextConfig } from "next";
import {
  PHASE_DEVELOPMENT_SERVER,
  PHASE_PRODUCTION_BUILD,
  PHASE_PRODUCTION_SERVER,
} from "next/constants";
import { withIntlAi } from "./index";
import type { NextConfigContext } from "./index";

const tmp = mkdtempSync(join(tmpdir(), "intl-ai-next-"));
afterAll(() => rmSync(tmp, { recursive: true, force: true }));

const ctx: NextConfigContext = { defaultConfig: {} };

function stubBin(): string {
  const script = join(tmp, "intl-ai-stub.cjs");
  writeFileSync(
    script,
    [
      "#!/usr/bin/env node",
      'const fs = require("node:fs");',
      "const argv = process.argv.slice(2);",
      'fs.appendFileSync(process.env.INTL_AI_STUB_LOG, argv.join(" ") + "\\n");',
      "process.exit(Number(process.env.INTL_AI_STUB_EXIT || 0));",
    ].join("\n"),
  );
  chmodSync(script, 0o755);
  return script;
}

function stubLog(name: string): string {
  const log = join(tmp, name);
  writeFileSync(log, "");
  process.env.INTL_AI_STUB_LOG = log;
  return log;
}

describe("withIntlAi", () => {
  test("returns a phase-aware config function that preserves the config", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrapped = withIntlAi(config, { bin: stubBin() });
    expect(typeof wrapped).toBe("function");
    const resolved = await wrapped(PHASE_PRODUCTION_SERVER, ctx);
    expect(resolved.reactStrictMode).toBe(true);
  });

  test("wraps sync and async function configs", async () => {
    for (const input of [
      () => ({ reactStrictMode: true }),
      async () => ({ reactStrictMode: true }),
    ] as const) {
      const wrapped = withIntlAi(input, { bin: stubBin() });
      const resolved = await wrapped(PHASE_PRODUCTION_SERVER, ctx);
      expect(resolved.reactStrictMode).toBe(true);
    }
  });

  test("runs intl-ai fill on production build", async () => {
    const log = stubLog("build.log");
    const wrapped = withIntlAi({}, { bin: stubBin(), judgeThreshold: 0.9 });
    await wrapped(PHASE_PRODUCTION_BUILD, ctx);
    expect(readFileSync(log, "utf8").trim()).toBe("fill --judge-threshold 0.9");
  });

  test("runs on dev server unless dev: false", async () => {
    const log = stubLog("dev.log");
    await withIntlAi({}, { bin: stubBin() })(PHASE_DEVELOPMENT_SERVER, ctx);
    expect(readFileSync(log, "utf8").trim()).toBe("fill");

    const log2 = stubLog("dev-off.log");
    await withIntlAi({}, { bin: stubBin(), dev: false })(PHASE_DEVELOPMENT_SERVER, ctx);
    expect(readFileSync(log2, "utf8")).toBe("");
  });

  test("skips phases outside dev/build (e.g. next start)", async () => {
    const log = stubLog("start.log");
    await withIntlAi({}, { bin: stubBin() })(PHASE_PRODUCTION_SERVER, ctx);
    expect(readFileSync(log, "utf8")).toBe("");
    expect(existsSync(log)).toBe(true);
  });

  test("failOn adds a check gate after fill", async () => {
    const log = stubLog("gate.log");
    await withIntlAi({}, { bin: stubBin(), failOn: ["missing", "invalid"] })(
      PHASE_PRODUCTION_BUILD,
      ctx,
    );
    expect(readFileSync(log, "utf8").trim().split("\n")).toEqual([
      "fill",
      "check --fail-on missing,invalid",
    ]);
  });

  test("nonzero exit throws; strict: false warns", async () => {
    stubLog("strict.log");
    process.env.INTL_AI_STUB_EXIT = "7";
    await expect(withIntlAi({}, { bin: stubBin() })(PHASE_PRODUCTION_BUILD, ctx)).rejects.toThrow(
      "intl-ai fill exited with code 7",
    );

    const warnings: string[] = [];
    const spy = vi.spyOn(console, "warn").mockImplementation((m?: unknown) => {
      warnings.push(String(m));
    });
    await withIntlAi({}, { bin: stubBin(), strict: false })(PHASE_PRODUCTION_BUILD, ctx);
    spy.mockRestore();
    delete process.env.INTL_AI_STUB_EXIT;
    expect(warnings[0]).toContain("intl-ai fill exited with code 7");
  });
});
