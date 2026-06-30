import type { TranslationResult, ErrorType } from "../core/types";

export type { TranslationResult, ErrorType };

export interface TranslationHook {
  onRequest?: (info: {
    provider: string;
    model: string;
    locale: string;
    entryCount: number;
  }) => void;
  onSuccess?: (info: {
    provider: string;
    model: string;
    locale: string;
    results: TranslationResult[];
    durationMs: number;
  }) => void;
  /** Fires after each retry attempt fails (including the final one). */
  onAttemptFailure?: (info: {
    provider: string;
    model: string;
    locale: string;
    errorType: ErrorType;
    error: string;
    attempt: number;
    maxRetries: number;
    statusCode?: number;
    durationMs: number;
  }) => void;
  /** Fires when all retries are exhausted for a batch. */
  onError?: (info: {
    provider: string;
    model: string;
    locale: string;
    errorType: ErrorType;
    error: string;
    attempt: number;
    maxRetries: number;
    statusCode?: number;
    durationMs: number;
    attempts: Array<{
      attempt: number;
      errorType: ErrorType;
      durationMs: number;
      statusCode?: number;
    }>;
    rawResponse?: string;
  }) => void;
}
