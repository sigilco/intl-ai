import type { IntlAiProcessor } from "../types";

export function createProcessor(overrides: Partial<IntlAiProcessor>): IntlAiProcessor {
  return {
    name: overrides.name || "custom-processor",
    extractTokens: overrides.extractTokens || (() => []),
    validate: overrides.validate || (() => ({ valid: true })),
    getSyntaxHint: overrides.getSyntaxHint || (() => ""),
  };
}

export const passthroughProcessor: IntlAiProcessor = {
  name: "passthrough",
  extractTokens: () => [],
  validate: () => ({ valid: true }),
  getSyntaxHint: () => "",
};
