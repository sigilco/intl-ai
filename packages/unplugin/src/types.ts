// Re-export types from @intl-ai/core
export type { IntlAiConfig, TranslationLockfile } from "@intl-ai/core";

// Unplugin-specific options
export interface UnpluginIntlAiOptions {
  /** Enable debug logging */
  debug?: boolean;
}
