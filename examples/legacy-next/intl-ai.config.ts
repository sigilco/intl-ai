/** @see https://www.schemastore.org/intl-ai.json */
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";

const lmstudio = createOpenAICompatible({
  name: "lmstudio",
  baseURL: "http://127.0.0.1:1234/v1",
});

export default {
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./messages",
  model: lmstudio("lfm2.5-1.2b-instruct"),
};
