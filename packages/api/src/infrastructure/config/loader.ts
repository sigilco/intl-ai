import { readFile } from "node:fs/promises";
import { existsSync } from "fs";
import { resolve as resolvePath, dirname } from "pathe";
import { createJiti } from "jiti";
import {
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  type IntlAiJsonConfig,
} from "../../schema/index";
import { resolveFormat } from "../../adapters/formats/registry";
import {
  IntlAiConfigSchema,
  type IntlAiConfig,
  type HttpIntlAiConfig,
  type AgentIntlAiConfig,
} from "../../types";
import type { LocaleFormat } from "../../ports/format";
import type { AITransport } from "../../ports/provider";
import { createCommandTransport } from "../transports/command";
import { resolveAgentPreset } from "../transports/presets";

export type { IntlAiConfig };

export type ResolvedIntlAiConfig =
  | (HttpIntlAiConfig & { format: LocaleFormat })
  | (AgentIntlAiConfig & { format: LocaleFormat; transport: AITransport });

const CONFIG_FILE_NAMES = ["intl-ai.config.ts", "intl-ai.config.json"] as const;

async function findConfigFile(cwd: string): Promise<string | null> {
  for (const name of CONFIG_FILE_NAMES) {
    const p = resolvePath(cwd, name);
    if (existsSync(p)) return p;
  }
  return null;
}

async function loadJsonRaw(path: string): Promise<unknown> {
  const content = await readFile(path, "utf-8");
  try {
    return JSON.parse(content);
  } catch (error) {
    throw new Error(
      `Invalid JSON in config file ${path}: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

async function loadTsRaw(path: string): Promise<unknown> {
  const j = createJiti(process.cwd(), { interopDefault: true });
  const mod = (await j.import(path, { default: true })) as { default?: unknown } | unknown;
  if (mod && typeof mod === "object" && "default" in (mod as object)) {
    return (mod as { default: unknown }).default;
  }
  return mod;
}

function parseAndConvert(
  raw: unknown,
  sourcePath: string,
  schema: "json" | "runtime",
): IntlAiConfig {
  const zodSchema = schema === "json" ? IntlAiJsonConfigSchema : IntlAiConfigSchema;
  const parsed = zodSchema.safeParse(raw);
  if (!parsed.success) {
    const errors = parsed.error.issues
      .map((issue) => {
        const p = issue.path.length > 0 ? issue.path.join(".") : "/";
        return `${p}: ${issue.message}`;
      })
      .join("\n  ");
    throw new Error(`Config validation failed (${sourcePath}):\n  ${errors}`);
  }
  if (schema === "json") {
    return jsonConfigToIntlAiConfig(parsed.data as IntlAiJsonConfig);
  }
  return parsed.data as IntlAiConfig;
}

function resolveTransport(config: AgentIntlAiConfig, configPath: string): AITransport {
  const cwd = config.cwd ? resolvePath(dirname(configPath), config.cwd) : dirname(configPath);
  if (config.command) {
    return createCommandTransport({
      id: config.command,
      command: config.command,
      args: config.args ?? [],
      cwd,
    });
  }
  if (config.agent) {
    return resolveAgentPreset(config.agent, cwd);
  }
  throw new Error(
    `Agent config requires either "agent" (a preset name) or "command" (a custom binary), in ${configPath}`,
  );
}

/**
 * Resolve a relative `localeDir` against the config file's directory so configs
 * are relocatable (standard behavior, like tsconfig/eslint paths). Absolute
 * paths pass through unchanged. Also resolves `config.format` from a string
 * identifier to a concrete LocaleFormat adapter, and (for `kind: "agent"`)
 * the preset name or command/args escape hatch to a concrete `AITransport`.
 */
function resolveConfig(config: IntlAiConfig, configPath: string): ResolvedIntlAiConfig {
  const localeDir = resolvePath(dirname(configPath), config.localeDir);
  // ponytail: resolve format here so services receive a LocaleFormat object and
  // never need to import the registry (hexagonal boundary compliance).
  const format = resolveFormat(config.format);
  if (config.kind === "agent") {
    const transport = resolveTransport(config, configPath);
    return { ...config, localeDir, format, transport };
  }
  return { ...config, localeDir, format };
}

/**
 * Load config from an explicit file path (`.ts` or `.json`).
 * Skips schema validation when `validate` is false (useful in tests).
 * A relative `localeDir` is resolved against the config file's directory.
 */
export async function loadConfigFromPath(
  path: string,
  options: { validate?: boolean } = {},
): Promise<ResolvedIntlAiConfig> {
  if (!existsSync(path)) {
    throw new Error(`Config file not found: ${path}`);
  }
  const isTs = path.endsWith(".ts");
  const raw = isTs ? await loadTsRaw(path) : await loadJsonRaw(path);
  const config = isTs
    ? options.validate === false
      ? (raw as IntlAiConfig)
      : parseAndConvert(raw, path, "runtime")
    : parseAndConvert(raw, path, "json");
  return resolveConfig(config, path);
}

/**
 * Search `cwd` for `intl-ai.config.ts` or `intl-ai.config.json` and load it.
 * Throws if no config is found. A relative `localeDir` is resolved against the
 * config file's directory.
 */
export async function loadConfig(cwd: string = process.cwd()): Promise<ResolvedIntlAiConfig> {
  const configPath = await findConfigFile(cwd);
  if (!configPath) {
    throw new Error(
      `No intl-ai config found in ${cwd}. Expected one of: ${CONFIG_FILE_NAMES.join(", ")}`,
    );
  }
  const raw = configPath.endsWith(".ts")
    ? await loadTsRaw(configPath)
    : await loadJsonRaw(configPath);
  const isTs = configPath.endsWith(".ts");
  return resolveConfig(
    isTs ? parseAndConvert(raw, configPath, "runtime") : parseAndConvert(raw, configPath, "json"),
    configPath,
  );
}
