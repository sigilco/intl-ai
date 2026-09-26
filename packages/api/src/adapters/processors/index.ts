import type { IntlAiProcessor } from "../../ports/processor";
import type { ValidationResult } from "../../core/types";

export { icuProcessor } from "./icu";

export const passthroughProcessor: IntlAiProcessor = {
  name: "passthrough",
  extractTokens: () => [],
  validate: (_source: string, _translated: string): ValidationResult => ({ valid: true }),
  getSyntaxHint: () => "Preserve the source text exactly; do not add or remove any markers.",
};

export function createProcessor(overrides: Partial<IntlAiProcessor>): IntlAiProcessor {
  return {
    name: overrides.name ?? "custom",
    extractTokens: overrides.extractTokens ?? (() => []),
    validate: overrides.validate ?? (() => ({ valid: true })),
    getSyntaxHint: overrides.getSyntaxHint ?? (() => ""),
  };
}
