// packages/core/src/testing/mock-provider.ts

import type {
  LanguageModelV1,
  LanguageModelV1CallOptions,
  LanguageModelV1GenerateMessageResult,
  LanguageModelV1StreamMessagePart,
  LanguageModelV1CallWarning,
} from "@ai-sdk/provider";

export interface MockProviderOptions {
  translations?: Record<string, Record<string, string>>;
  shouldFail?: boolean;
  failureCount?: number;
}

export class MockLanguageModel implements LanguageModelV1 {
  readonly specificationVersion = "v1";
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

  async doGenerate(
    options: LanguageModelV1CallOptions,
  ): Promise<LanguageModelV1GenerateMessageResult> {
    this.callCount++;

    if (this.options.shouldFail && this.callCount <= (this.options.failureCount || 1)) {
      throw new Error("Mock provider failure for testing");
    }

    const messages = options.messages;
    const lastMessage = messages[messages.length - 1];
    const inputText =
      typeof lastMessage === "string" ? lastMessage : (lastMessage.content as string);

    const targetLocale = this.extractTargetLocale(inputText) || "en";
    const translation = this.getTranslation(inputText, targetLocale);

    return {
      text: this.formatTranslationResponse(inputText, translation),
      finishReason: "stop",
      usage: { promptTokens: 1, completionTokens: 1, totalTokens: 2 },
      responseId: `mock-response-${this.callCount}`,
    };
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

  doStream(
    options: LanguageModelV1CallOptions,
  ): AsyncGenerator<LanguageModelV1StreamMessagePart, void, unknown> {
    this.callCount++;

    if (this.options.shouldFail && this.callCount <= (this.options.failureCount || 1)) {
      throw new Error("Mock provider failure for testing");
    }

    const messages = options.messages;
    const lastMessage = messages[messages.length - 1];
    const inputText =
      typeof lastMessage === "string" ? lastMessage : (lastMessage.content as string);

    const translation = this.getTranslation(inputText, options.headers?.["x-locale"] || "en");

    return (async function* () {
      yield {
        role: "assistant",
        content: [{ type: "text", text: translation }],
      };
    })();
  }

  get warnings(): LanguageModelV1CallWarning[] {
    return [];
  }

  get embeddingModel(): LanguageModelV1 | undefined {
    return undefined;
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
