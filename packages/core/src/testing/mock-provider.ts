// packages/core/src/testing/mock-provider.ts

import type {
  LanguageModelV3,
  LanguageModelV3CallOptions,
  LanguageModelV3GenerateResult,
  LanguageModelV3StreamResult,
  LanguageModelV3StreamPart,
  LanguageModelV3TextPart,
  SharedV3Warning,
} from "@ai-sdk/provider";

export interface MockProviderOptions {
  translations?: Record<string, Record<string, string>>;
  shouldFail?: boolean;
  failureCount?: number;
}

export class MockLanguageModel implements LanguageModelV3 {
  readonly specificationVersion = "v3";
  readonly provider = "mock";
  readonly modelId = "mock-model";

  private callCount = 0;
  private options: MockProviderOptions;

  constructor(options: MockProviderOptions = {}) {
    this.options = options;
  }

  getCallCount(): number {
    return this.callCount;
  }

  reset(): void {
    this.callCount = 0;
  }

  get supportedUrls(): Record<string, RegExp[]> {
    return {};
  }

  async doGenerate(options: LanguageModelV3CallOptions): Promise<LanguageModelV3GenerateResult> {
    this.callCount++;

    if (this.options.shouldFail && this.callCount <= (this.options.failureCount || 1)) {
      throw new Error("Mock provider failure for testing");
    }

    const inputText = this.extractTextFromPrompt(options.prompt);
    const targetLocale = this.extractTargetLocale(inputText) || "en";
    const translation = this.getTranslation(inputText, targetLocale);

    return {
      content: [
        {
          type: "text" as const,
          text: this.formatTranslationResponse(inputText, translation),
        },
      ],
      finishReason: { unified: "stop" as const, raw: undefined },
      usage: {
        inputTokens: {
          total: 50,
          noCache: 50,
          cacheRead: undefined,
          cacheWrite: undefined,
        },
        outputTokens: {
          total: 20,
          text: 20,
          reasoning: undefined,
        },
      },
      warnings: [],
    };
  }

  private extractTextFromPrompt(
    prompt: Array<{
      role: string;
      content: unknown;
      providerOptions?: unknown;
    }>,
  ): string {
    const lastMessage = prompt[prompt.length - 1];
    if (!lastMessage) return "";

    if (lastMessage.role === "system") {
      return lastMessage.content as string;
    }

    if (lastMessage.role === "user" && Array.isArray(lastMessage.content)) {
      const parts = lastMessage.content as Array<LanguageModelV3TextPart>;
      return parts
        .filter((part): part is LanguageModelV3TextPart => part.type === "text")
        .map((part) => part.text)
        .join("");
    }

    if (lastMessage.role === "assistant" && Array.isArray(lastMessage.content)) {
      const parts = lastMessage.content as Array<LanguageModelV3TextPart>;
      return parts
        .filter((part): part is LanguageModelV3TextPart => part.type === "text")
        .map((part) => part.text)
        .join("");
    }

    return "";
  }

  private extractTargetLocale(input: string): string | null {
    const match = input.match(/Translate.*?to (\w+)/i);
    return match ? match[1].toLowerCase() : null;
  }

  private formatTranslationResponse(input: string, translation: string): string {
    const entries = this.extractTranslationEntries(input);
    if (entries.length === 0) {
      return translation;
    }

    const translations = entries.map((entry) => {
      const translated = this.getTranslation(entry.source, entry.locale || "en");
      return {
        key: entry.key,
        translated,
      };
    });

    return JSON.stringify({ translations });
  }

  private extractTranslationEntries(
    input: string,
  ): Array<{ key: string; source: string; locale?: string }> {
    const entries: Array<{ key: string; source: string; locale?: string }> = [];

    const localeMatch = input.match(/to (\w+)/i);
    const targetLocale = localeMatch ? localeMatch[1].toLowerCase() : "en";

    const lines = input.split("\n");
    for (const line of lines) {
      const match = line.match(/(\d+)\. "([^"]+)" \(key: ([^)]+)\)/);
      if (match) {
        entries.push({
          key: match[3],
          source: match[2],
          locale: targetLocale,
        });
      }
    }

    return entries;
  }

  async doStream(options: LanguageModelV3CallOptions): Promise<LanguageModelV3StreamResult> {
    this.callCount++;

    if (this.options.shouldFail && this.callCount <= (this.options.failureCount || 1)) {
      throw new Error("Mock provider failure for testing");
    }

    const inputText = this.extractTextFromPrompt(options.prompt);
    const translation = this.getTranslation(inputText, options.headers?.["x-locale"] || "en");

    const stream = new ReadableStream<LanguageModelV3StreamPart>({
      start(controller) {
        controller.enqueue({ type: "text-start" as const, id: "1" });
        for (const char of translation) {
          controller.enqueue({ type: "text-delta" as const, id: "1", delta: char });
        }
        controller.enqueue({ type: "text-end" as const, id: "1" });
        controller.close();
      },
    });

    return { stream };
  }

  private getTranslation(input: string, locale: string): string {
    if (!this.options.translations || !this.options.translations[locale]) {
      return input;
    }

    const localeTranslations = this.options.translations[locale];

    if (localeTranslations[input]) {
      return localeTranslations[input];
    }

    for (const [key, value] of Object.entries(localeTranslations)) {
      if (input.includes(key) || key.includes(input)) {
        return value as string;
      }
    }

    return input;
  }
}

export const DEFAULT_TRANSLATIONS: Record<string, Record<string, string>> = {
  en: {
    Hello: "Hello",
    "Good morning": "Good morning",
    "Good evening": "Good evening",
    Goodbye: "Goodbye",
    "Thank you": "Thank you",
    Please: "Please",
    Yes: "Yes",
    No: "No",
    Welcome: "Welcome",
    Success: "Success",
  },
  fr: {
    Hello: "Bonjour",
    "Good morning": "Bonjour",
    "Good evening": "Bonne soirée",
    Goodbye: "Au revoir",
    "Thank you": "Merci",
    Please: "S'il vous plaît",
    Yes: "Oui",
    No: "Non",
    Welcome: "Bienvenue",
    Success: "Succès",
  },
  es: {
    Hello: "Hola",
    "Good morning": "Buenos días",
    "Good evening": "Buenas noches",
    Goodbye: "Adiós",
    "Thank you": "Gracias",
    Please: "Por favor",
    Yes: "Sí",
    No: "No",
    Welcome: "Bienvenido",
    Success: "Éxito",
  },
  de: {
    Hello: "Hallo",
    "Good morning": "Guten Morgen",
    "Good evening": "Guten Abend",
    Goodbye: "Auf Wiedersehen",
    "Thank you": "Danke",
    Please: "Bitte",
    Yes: "Ja",
    No: "Nein",
    Welcome: "Willkommen",
    Success: "Erfolg",
  },
};
