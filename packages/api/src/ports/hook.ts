import type { TranslationResult } from "../core/types";

export type { TranslationResult };

export interface TranslationHook {
  onRequest?: (info: { provider: string; model: string; locale: string; entryCount: number }) => void;
  onSuccess?: (info: {
    provider: string;
    model: string;
    locale: string;
    results: TranslationResult[];
    durationMs: number;
  }) => void;
  onError?: (info: { provider: string; model: string; locale: string; error: string; attempt: number }) => void;
}
