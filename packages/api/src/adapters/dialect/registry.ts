import { detectDialect, type DialectHit, type DialectVariant } from "./en";

export const builtInDialects = {
  en: detectDialect,
} as const;

export type BuiltInDialect = keyof typeof builtInDialects;

export type DialectDetector = (
  text: string,
  variant: DialectVariant,
  excludeSpans?: Array<[number, number]>,
) => Promise<DialectHit[]>;

/**
 * Resolve a language subtag (e.g. "en") into its dialect detector.
 * The config loader / check service is responsible for deriving the subtag
 * from a full locale (e.g. "en-GB" -> "en") before calling this.
 */
export function resolveDialect(id: string): DialectDetector {
  const detector = (builtInDialects as Record<string, DialectDetector>)[id];
  if (!detector)
    throw new Error(
      `Unknown dialect: ${id}. Built-ins: ${Object.keys(builtInDialects).join(", ")}`,
    );
  return detector;
}

/**
 * Maps a locale's region subtag to the English dialect variant it implies.
 * Returns undefined when the region is absent or not recognized, in which
 * case the caller cannot pick a variant and should surface a clear error.
 */
export function variantForLocale(locale: string): DialectVariant | undefined {
  const region = locale.split("-")[1]?.toUpperCase();
  if (region === "US") return "american";
  if (region === "GB" || region === "UK") return "british";
  return undefined;
}
