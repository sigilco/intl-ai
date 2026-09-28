import type { NextConfig } from "next";
import { PHASE_DEVELOPMENT_SERVER, PHASE_EXPORT, PHASE_PRODUCTION_BUILD } from "next/constants";
import type { IntlAiPluginOptions } from "@intl-ai/unplugin";
import { runIntlAiPipeline } from "@intl-ai/unplugin";

/** Same option surface as `@intl-ai/unplugin`; see its README. */
export type IntlAiNextOptions = IntlAiPluginOptions;

export interface NextConfigContext {
  defaultConfig: NextConfig;
}

export type NextConfigResult = NextConfig | Promise<NextConfig>;
export type NextConfigInput =
  | NextConfig
  | ((phase: string, ctx: NextConfigContext) => NextConfigResult);

function shouldRunInPhase(phase: string, o: IntlAiNextOptions): boolean {
  if (phase === PHASE_PRODUCTION_BUILD || phase === PHASE_EXPORT) return true;
  if (phase === PHASE_DEVELOPMENT_SERVER) return o.dev !== false;
  return false;
}

/**
 * Wrap a `next.config` so the `intl-ai` binary runs while Next.js
 * evaluates the config (before either webpack or Turbopack starts):
 *
 * ```ts
 * // next.config.ts
 * import { withIntlAi } from "@intl-ai/next";
 * export default withIntlAi(nextConfig, { failOn: ["missing"] });
 * ```
 *
 * `intl-ai fill` runs on `next build` and `next export`, and on
 * `next dev` unless `dev: false`. A nonzero exit throws (fails the
 * build) unless `strict: false`, which downgrades to a warning.
 */
export function withIntlAi(nextConfig: NextConfigInput = {}, options: IntlAiNextOptions = {}) {
  return async function intlAiNextConfig(
    phase: string,
    ctx: NextConfigContext,
  ): Promise<NextConfig> {
    const resolved = typeof nextConfig === "function" ? await nextConfig(phase, ctx) : nextConfig;
    if (shouldRunInPhase(phase, options)) {
      await runIntlAiPipeline(options, options.cwd ?? process.cwd());
    }
    return resolved;
  };
}

export default withIntlAi;
