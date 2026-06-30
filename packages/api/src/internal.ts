// Core domain
export {
  findMissingTranslations,
  flattenObject,
  unflattenObject,
  lockfileEntryToMap,
  hashSource,
} from "./core/diff";
export { hashSha1 } from "./core/hash";
export type {
  ValidationResult,
  TranslationEntry,
  TranslationResult,
  TranslationStaleEntry,
  ApiKeyValue,
  MissingTranslationEntry,
  FindMissingTranslationsOptions,
  FindMissingTranslationsResult,
  QualityError,
  QualityErrorType,
  QualitySeverity,
  QualityResult,
  TranslationContext,
  QualityAssessorInstance,
  QualityOptions,
} from "./core/types";
export { QUALITY_ERROR_TYPES, QUALITY_SEVERITIES, isQualityAssessorInstance } from "./core/types";

// Ports
export type { AIProvider } from "./ports/provider";
export type { IntlAiProcessor } from "./ports/processor";
export type { TranslationHook } from "./ports/hook";
export type { LocaleFormat } from "./ports/format";

// Services
export { runFill } from "./services/fill/fill";
export type { RunFillOptions, RunFillResult } from "./services/fill/fill";
export { batchedFill } from "./services/batch/index";
export type { BatchedFillOptions, BatchedFillResult } from "./services/batch/index";
export { translateBatch } from "./services/fill/translator";
export type { TranslateBatchOptions } from "./services/fill/translator";
export { runCheck } from "./services/check/check";
export type { RunCheckOptions, RunCheckResult, CheckLocaleResult } from "./services/check/check";

// Adapters: providers
export { openaiProvider, anthropicProvider, resolveProvider } from "./adapters/providers/registry";

// Adapters: processors
export { icuProcessor } from "./adapters/processors/icu";
export { passthroughProcessor, createProcessor } from "./adapters/processors/index";

// Adapters: formats
export { readJsonFile, writeJsonFile, jsonFormat } from "./adapters/formats/json";
export { readYamlFile, writeYamlFile, yamlFormat, dirname } from "./adapters/formats/yaml";
export { builtInFormats, resolveFormat } from "./adapters/formats/registry";
export type { BuiltInFormat } from "./adapters/formats/registry";

// Infrastructure
export {
  readText,
  writeText,
  pathExists,
  ensureDir,
  removeDir,
  listFiles,
  fileSize,
  join,
  isAbsolute,
  relative,
  resolve,
} from "./infrastructure/fs";
export { loadConfig, loadConfigFromPath } from "./infrastructure/config/loader";
export type { ResolvedIntlAiConfig } from "./infrastructure/config/loader";

// Lockfile
export { LockfileManager } from "./lockfile/manager";
export type {
  LockfileEntry,
  Lockfile,
  LockfileQuality,
  QualityRecord,
  StaleEntry,
} from "./lockfile/types";
export { LOCKFILE_NAME } from "./lockfile/types";

// Quality-aware fill loop (issue #14)
export {
  judgeBatch,
  createDefaultAssessor,
  ADVERSARIAL_SYSTEM_PROMPT,
} from "./services/fill/judge";
export type { JudgeBatchOptions } from "./services/fill/judge";
export { runQualityLoop } from "./services/fill/loop";
export type { QualityLoopOptions, QualityLoopResult } from "./services/fill/loop";

// Schema
export {
  IntlAiJsonConfigSchema,
  jsonConfigToIntlAiConfig,
  getIntlAiSchema,
  INTL_AI_SCHEMA_URL,
} from "./schema/index";
export type { IntlAiJsonConfig } from "./schema/index";
