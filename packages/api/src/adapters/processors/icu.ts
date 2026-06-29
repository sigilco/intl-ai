import { parse } from "@formatjs/icu-messageformat-parser";
import type {
  MessageFormatElement,
  PluralOrSelectOption,
} from "@formatjs/icu-messageformat-parser";
import type { IntlAiProcessor } from "../../ports/processor";
import type { ValidationResult } from "../../core/types";

export const icuProcessor: IntlAiProcessor = {
  name: "icu",

  extractTokens(message: string): string[] {
    try {
      const result = parse(message);
      return extractTokensFromAst(result);
    } catch {
      return [];
    }
  },

  validate(source: string, translated: string): ValidationResult {
    const sourceTokens = this.extractTokens(source);
    const translatedTokens = this.extractTokens(translated);

    const missing = sourceTokens.filter((t) => !translatedTokens.includes(t));
    const extra = translatedTokens.filter((t) => !sourceTokens.includes(t));

    if (missing.length > 0 || extra.length > 0) {
      return {
        valid: false,
        errors: [
          ...(missing.length > 0 ? [`Missing tokens: ${missing.join(", ")}`] : []),
          ...(extra.length > 0 ? [`Extra tokens: ${extra.join(", ")}`] : []),
        ],
      };
    }

    return { valid: true };
  },

  getSyntaxHint(): string {
    return 'ICU MessageFormat: Use {variable} for placeholders, e.g., "Hello {name}". Supports plural/select syntax.';
  },
};

function extractTokensFromAst(
  nodes: MessageFormatElement | MessageFormatElement[],
  parentType?: number,
): string[] {
  if (Array.isArray(nodes)) {
    return nodes.flatMap((n) => extractTokensFromAst(n, parentType));
  }

  const tokens: string[] = [];

  if ("value" in nodes && typeof nodes.value === "string" && nodes.type >= 1 && nodes.type <= 6) {
    tokens.push(nodes.value);
  }

  if (nodes.type === 7 && parentType !== 6 && parentType !== 7) {
    const n = nodes as { type: number; value?: string };
    if (n.value) tokens.push(n.value);
  }

  if ("children" in nodes && Array.isArray(nodes.children)) {
    for (const child of nodes.children) {
      tokens.push(...extractTokensFromAst(child, nodes.type));
    }
  }

  if ("options" in nodes && nodes.options) {
    for (const option of Object.values(nodes.options as Record<string, PluralOrSelectOption>)) {
      tokens.push(...extractTokensFromAst(option.value, nodes.type));
    }
  }

  return tokens;
}
