// Re-export types from @intl-ai/core
export type { IntlAiConfig } from "@intl-ai/core";
export type { Lockfile as TranslationLockfile } from "@intl-ai/core";

// Unplugin-specific options
export interface UnpluginIntlAiOptions {
  /** Enable debug logging */
  debug?: boolean;
}
