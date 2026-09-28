import { spawn } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { delimiter, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export interface IntlAiPluginOptions {
  /** Run `intl-ai fill` on build start. Default: true. */
  fill?: boolean;
  /**
   * Finding kinds that fail the build, mapped to
   * `intl-ai check --fail-on` (comma-separated or array), e.g.
   * "missing,stale,invalid". Default: no check gate.
   */
  failOn?: string | string[];
  /**
   * Fill-time validation gate. `true` (default) defers to the config's
   * `[fill].validate`; an array of check ids overrides it; `false`
   * passes `--no-validate`.
   */
  validate?: boolean | string[];
  /** Override the judge check's 0..1 threshold inside the fill gate. */
  judgeThreshold?: number;
  /**
   * Set `false` to skip the plugin outside production builds
   * (NODE_ENV !== "production"). Default: true (runs everywhere).
   */
  dev?: boolean;
  /**
   * Turn intl-ai failures into build errors. Default: true; set `false`
   * to warn and continue the build.
   */
  strict?: boolean;
  /** Path to intl-ai.toml/.json/.yaml. Default: discovered by the binary. */
  config?: string;
  /** Working directory for the intl-ai invocation. Default: process.cwd(). */
  cwd?: string;
  /** Explicit path to the intl-ai binary. */
  bin?: string;
  /** Extra arguments appended to `intl-ai fill`. */
  args?: string[];
}

export interface ResolvedBin {
  command: string;
  argsPrefix: string[];
}

function npmPackageBin(fromDir: string): string | undefined {
  try {
    const req = createRequire(join(fromDir, "noop.cjs"));
    const pkgPath = req.resolve("intl-ai/package.json");
    const pkg = JSON.parse(readFileSync(pkgPath, "utf8")) as {
      bin?: string | Record<string, string>;
    };
    const rel = typeof pkg.bin === "string" ? pkg.bin : pkg.bin?.["intl-ai"];
    if (!rel) return undefined;
    const binPath = join(dirname(pkgPath), rel);
    return existsSync(binPath) ? binPath : undefined;
  } catch {
    return undefined;
  }
}

/**
 * Resolution order: `bin` option, INTL_AI_BIN env, the `intl-ai` npm
 * package (this shim's dependency, resolved first relative to the shim
 * and then the project cwd), then PATH.
 */
export function resolveIntlAiBin(explicit?: string, cwd = process.cwd()): ResolvedBin {
  const direct = explicit ?? process.env.INTL_AI_BIN;
  if (direct) return { command: direct, argsPrefix: [] };
  const here = dirname(fileURLToPath(import.meta.url));
  for (const dir of [here, cwd]) {
    const script = npmPackageBin(dir);
    if (script) return { command: process.execPath, argsPrefix: [script] };
  }
  const names =
    process.platform === "win32" ? ["intl-ai.exe", "intl-ai.cmd", "intl-ai"] : ["intl-ai"];
  for (const dir of (process.env.PATH ?? "").split(delimiter)) {
    if (!dir) continue;
    for (const name of names) {
      const p = join(dir, name);
      if (existsSync(p)) return { command: p, argsPrefix: [] };
    }
  }
  throw new Error(
    "intl-ai binary not found: install the `intl-ai` npm package or binary " +
      "(https://intl-ai.pages.dev), or pass the `bin` option / INTL_AI_BIN",
  );
}

export function buildFillArgs(o: IntlAiPluginOptions): string[] {
  const argv: string[] = [];
  if (o.config) argv.push("--config", o.config);
  argv.push("fill");
  if (o.validate === false) {
    argv.push("--no-validate");
  } else if (Array.isArray(o.validate) && o.validate.length) {
    argv.push("--validate", o.validate.join(","));
  }
  if (o.judgeThreshold != null) argv.push("--judge-threshold", String(o.judgeThreshold));
  if (o.args?.length) argv.push(...o.args);
  return argv;
}

export function normalizeFailOn(f?: string | string[]): string[] {
  if (!f) return [];
  return (Array.isArray(f) ? f : f.split(",")).map((s) => s.trim()).filter(Boolean);
}

export function buildCheckArgs(o: IntlAiPluginOptions, failOn: string[]): string[] {
  const argv: string[] = [];
  if (o.config) argv.push("--config", o.config);
  argv.push("check", "--fail-on", failOn.join(","));
  return argv;
}

export function shouldSkipInDev(o: IntlAiPluginOptions): boolean {
  return o.dev === false && process.env.NODE_ENV !== "production";
}

export function runIntlAi(bin: ResolvedBin, argv: string[], cwd: string): Promise<number> {
  return new Promise((resolve, reject) => {
    const child = spawn(bin.command, [...bin.argsPrefix, ...argv], {
      cwd,
      stdio: "inherit",
      shell: process.platform === "win32",
    });
    child.on("error", reject);
    child.on("close", (code) => resolve(code ?? 1));
  });
}
