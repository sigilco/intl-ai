import type { LocaleFormat } from "../../ports/format";
import { jsonFormat } from "./json";
import { yamlFormat } from "./yaml";

export const builtInFormats = {
  json: jsonFormat,
  yaml: yamlFormat,
} as const;

export type BuiltInFormat = keyof typeof builtInFormats;

/**
 * Resolve a `format` config value (string | LocaleFormat | undefined) into a
 * concrete LocaleFormat. Strings select a built-in from the registry.
 * `undefined` defaults to the JSON format (backward compatibility).
 *
 * The config loader is responsible for calling this before passing config to
 * services; services should never import the registry.
 */
export function resolveFormat(format: LocaleFormat | string | undefined): LocaleFormat {
  if (format === undefined) return builtInFormats.json;
  if (typeof format === "string") {
    const f = (builtInFormats as Record<string, LocaleFormat>)[format];
    if (!f)
      throw new Error(
        `Unknown locale format: ${format}. Built-ins: ${Object.keys(builtInFormats).join(", ")}`,
      );
    return f;
  }
  return format;
}
