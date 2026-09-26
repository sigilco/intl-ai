/** @see https://www.schemastore.org/intl-ai.json */
import { resolveProvider } from "@intl-ai/api/internal";

const lmstudio = resolveProvider("openai");

export default {
  provider: lmstudio,
  model: "qwen3.5-4b-instruct",
  apiKey: "${LMSTUDIO_API_KEY}",
  baseURL: "http://127.0.0.1:1234/v1",
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./locales",
};
