// Core type definitions for intl-ai package
import { z } from "zod";
import type { LanguageModel } from "ai";

// IntlAiConfig - Main configuration interface
export interface IntlAiConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string; // Directory containing locale files
  model: LanguageModel; // Vercel AI SDK LanguageModel type
  processor?: IntlAiProcessor; // Optional syntax processor
  glossary?: Record<string, string>; // Optional translation glossary
  maxRetries?: number; // Default 3
}

// IntlAiProcessor - For handling ICU, i18next syntax
export interface IntlAiProcessor {
  name: string;
  extractTokens(message: string): string[];
  validate(source: string, translated: string): ValidationResult;
  getSyntaxHint(): string;
}

export interface ValidationResult {
  valid: boolean;
  errors?: string[];
}

// TranslationEntry - Single translation item
export interface TranslationEntry {
  key: string; // Dot-notation key path
  source: string; // Source text (default locale)
  target?: string; // Translated text
  locale: string; // Target locale
}

// TranslationResult - Result of translation operation
export interface TranslationResult {
  key: string;
  source: string;
  translated: string;
  locale: string;
  success: boolean;
  error?: string;
}

// Zod schemas for config validation
export const IntlAiConfigSchema = z.object({
  defaultLocale: z.string().min(1),
  locales: z.array(z.string().min(1)).min(1),
  localeDir: z.string().min(1),
  // model is validated at runtime (LanguageModel interface)
  model: z.custom<LanguageModel>((val) => val !== undefined, {
    message: "model must be a valid LanguageModel",
  }),
  processor: z.custom<IntlAiProcessor>().optional(),
  glossary: z.record(z.string(), z.string()).optional(),
  maxRetries: z.number().int().min(0).max(10).optional().default(3),
});

export const ValidationResultSchema = z.object({
  valid: z.boolean(),
  errors: z.array(z.string()).optional(),
});

export const TranslationEntrySchema = z.object({
  key: z.string(),
  source: z.string(),
  target: z.string().optional(),
  locale: z.string(),
});

export const TranslationResultSchema = z.object({
  key: z.string(),
  source: z.string(),
  translated: z.string(),
  locale: z.string(),
  success: z.boolean(),
  error: z.string().optional(),
});
