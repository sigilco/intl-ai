import type { AIProvider } from "../../ports/provider";
import { openaiProvider } from "./openai";
import { anthropicProvider } from "./anthropic";

const BUILTIN_PROVIDERS: Record<string, AIProvider> = {
  openai: openaiProvider,
  anthropic: anthropicProvider,
};

export function resolveProvider(id: AIProvider | string): AIProvider {
  if (typeof id === "string") {
    const found = BUILTIN_PROVIDERS[id];
    if (!found) {
      throw new Error(
        `Unknown provider: ${id}. Available: ${Object.keys(BUILTIN_PROVIDERS).join(", ")}`,
      );
    }
    return found;
  }
  return id;
}

export { openaiProvider, anthropicProvider };
