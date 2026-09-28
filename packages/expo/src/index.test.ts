import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, afterEach, describe, expect, test, vi } from "vitest";
import { withIntlAi } from "./index";

const tmp = mkdtempSync(join(tmpdir(), "intl-ai-expo-"));
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

function baseConfig() {
  return { name: "App", slug: "app" };
}

function modContext(platform: "ios" | "android") {
  return {
    name: "App",
    slug: "app",
    modRequest: {
      projectRoot: tmp,
      platformProjectRoot: tmp,
      modName: "dangerous",
      platform,
      introspect: false,
    },
  } as never;
}

async function runMod(
  config: ReturnType<typeof baseConfig>,
  options: Record<string, unknown>,
  platform: "ios" | "android",
) {
  const wrapped = withIntlAi(config as never, options as never) as typeof config & {
    mods: Record<string, { dangerous: (c: unknown) => Promise<unknown> }>;
  };
  const mod = wrapped.mods?.[platform]?.dangerous;
  if (!mod) throw new Error(`no ${platform} dangerous mod registered`);
  await mod(modContext(platform));
  return wrapped;
}

const log = join(tmp, "stub.log");
afterEach(() => {
  rmSync(log, { force: true });
  delete process.env.INTL_AI_STUB_LOG;
  delete process.env.INTL_AI_STUB_EXIT;
  delete process.env.INTL_AI_STUB_CHECK_EXIT;
  delete process.env.NODE_ENV;
});

describe("@intl-ai/expo shim", () => {
  test("registers dangerous mods on ios and android", () => {
    const wrapped = withIntlAi(baseConfig() as never, {}) as typeof baseConfig & {
      mods: Record<string, unknown>;
    };
    expect(wrapped.name).toBe("App");
    expect(wrapped.mods?.ios?.dangerous).toBeTypeOf("function");
    expect(wrapped.mods?.android?.dangerous).toBeTypeOf("function");
  });

  test("runs intl-ai fill with the app project root as cwd", async () => {
    process.env.INTL_AI_STUB_LOG = log;
    await runMod(baseConfig(), { bin: stubBin() }, "ios");
    expect(readFileSync(log, "utf8").trim()).toBe("fill");
  });

  test("runs the pipeline once across platform mods", async () => {
    process.env.INTL_AI_STUB_LOG = log;
    const bin = stubBin();
    const wrapped = withIntlAi(baseConfig() as never, { bin } as never) as typeof baseConfig & {
      mods: Record<string, { dangerous: (c: unknown) => Promise<unknown> }>;
    };
    await wrapped.mods.ios.dangerous(modContext("ios"));
    await wrapped.mods.android.dangerous(modContext("android"));
    const lines = readFileSync(log, "utf8").trim().split("\n");
    expect(lines).toEqual(["fill"]);
  });

  test("passes quality-gate options through to the binary", async () => {
    process.env.INTL_AI_STUB_LOG = log;
    await runMod(
      baseConfig(),
      { bin: stubBin(), validate: ["icu", "judge"], judgeThreshold: 0.9, failOn: "missing" },
      "android",
    );
    const lines = readFileSync(log, "utf8").trim().split("\n");
    expect(lines).toEqual([
      "fill --validate icu,judge --judge-threshold 0.9",
      "check --fail-on missing",
    ]);
  });

  test("throws on nonzero exit by default, warns with strict: false", async () => {
    process.env.INTL_AI_STUB_LOG = log;
    process.env.INTL_AI_STUB_EXIT = "1";
    const bin = stubBin();
    await expect(runMod(baseConfig(), { bin }, "ios")).rejects.toThrow(/exited with code 1/);
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    await runMod(baseConfig(), { bin, strict: false }, "ios");
    expect(warn).toHaveBeenCalled();
    warn.mockRestore();
  });

  test("dev: false skips outside production", async () => {
    process.env.INTL_AI_STUB_LOG = log;
    process.env.NODE_ENV = "development";
    await runMod(baseConfig(), { bin: stubBin(), dev: false }, "ios");
    expect(() => readFileSync(log, "utf8")).toThrow();
  });
});
