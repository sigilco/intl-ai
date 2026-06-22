/**
 * @intl-ai/core — DEPRECATED. Import from @intl-ai/api instead.
 *
 * @intl-ai/core will be removed in v1.0.
 */

let warned = false;
function warnDeprecation(): void {
  if (warned) return;
  warned = true;
  // eslint-disable-next-line no-console
  console.warn(
    "\n[intl-ai] @intl-ai/core is deprecated. Import directly from @intl-ai/api instead.\n" +
      "  See: https://github.com/sigilco/intl-ai/blob/main/docs/migration.md\n",
  );
}

import { createJiti } from "jiti";
import { resolve as resolvePath } from "pathe";
import { pathExists } from "./fs-shim";
import { IntlAiConfigSchema } from "@intl-ai/api";
import type { IntlAiConfig as ApiIntlAiConfig } from "@intl-ai/api";

// Re-export the lean public API surface from @intl-ai/api
export { runFill } from "@intl-ai/api";
export type { RunFillOptions, RunFillResult } from "@intl-ai/api";
export { IntlAiConfigSchema } from "@intl-ai/api";

// Back-compat: existing `IntlAiConfig` users that pass a Vercel AI SDK
// `LanguageModel` need to keep getting `model: any` (not `model: unknown`).
// This is type-only — runtime values are unchanged.
export type IntlAiConfig = Omit<ApiIntlAiConfig, "model"> & { model: any };

const CONFIG_FILE_NAMES = [
  "intl-ai.config.ts",
  "intl-ai.config.js",
  "intl-ai.config.mjs",
  "intl-ai.config.cjs",
  ".intl-airc",
  ".intl-airc.json",
];

async function findConfigFile(cwd: string): Promise<string | null> {
  for (const name of CONFIG_FILE_NAMES) {
    const p = resolvePath(cwd, name);
    if (await pathExists(p)) return p;
  }
  return null;
}

/**
 * Load an `IntlAiConfig` from a TypeScript/JavaScript/JSON config file.
 * This is preserved for backward compatibility with bundler plugins
 * (`@intl-ai/unplugin`, `@intl-ai/next`) that used to call it from `core`.
 *
 * New code should use `@intl-ai/api` + JSON config.
 */
export async function loadConfig(cwd: string = process.cwd()): Promise<IntlAiConfig> {
  warnDeprecation();
  const configPath = await findConfigFile(cwd);
  if (!configPath) {
    throw new Error(
      `No intl-ai config found in ${cwd}. Expected one of: ${CONFIG_FILE_NAMES.join(", ")}`,
    );
  }
  return loadConfigFile(configPath);
}

async function loadConfigFile(configPath: string): Promise<IntlAiConfig> {
  warnDeprecation();
  const j = createJiti(process.cwd(), { interopDefault: true });
  const mod = (await j.import(configPath, { default: true })) as { default?: unknown } | unknown;
  const raw = (
    mod && typeof mod === "object" && "default" in (mod as object)
      ? (mod as { default: unknown }).default
      : mod
  ) as unknown;

  const parsed = IntlAiConfigSchema.safeParse(raw);
  if (!parsed.success) {
    throw new Error(
      `Invalid config at ${configPath}: ${parsed.error.issues.map((i: { message: string }) => i.message).join(", ")}`,
    );
  }
  return parsed.data as IntlAiConfig;
}
