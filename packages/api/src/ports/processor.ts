import type { ValidationResult } from "../core/types";

export type { ValidationResult };

export interface IntlAiProcessor {
  name: string;
  extractTokens(message: string): string[];
  validate(source: string, translated: string): ValidationResult;
  getSyntaxHint(): string;
}
