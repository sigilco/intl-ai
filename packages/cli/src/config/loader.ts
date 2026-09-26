import {
  loadConfigFromPath,
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  type IntlAiJsonConfig,
  type ResolvedIntlAiConfig,
} from "@intl-ai/api/internal";

export type { ResolvedIntlAiConfig };

export interface LoadConfigOptions {
  validate?: boolean;
}

export async function loadConfig(
  path: string,
  options: LoadConfigOptions = {},
): Promise<ResolvedIntlAiConfig> {
  return loadConfigFromPath(path, options);
}

export function validateJsonConfig(json: unknown): asserts json is IntlAiJsonConfig {
  const result = IntlAiJsonConfigSchema.safeParse(json);
  if (!result.success) {
    const errors = result.error.issues
      .map((issue) => {
        const p = issue.path.length > 0 ? issue.path.join(".") : "/";
        return `${p}: ${issue.message}`;
      })
      .join("\n  ");
    throw new Error(`Config validation failed:\n  ${errors}`);
  }
}

export { jsonConfigToIntlAiConfig };
