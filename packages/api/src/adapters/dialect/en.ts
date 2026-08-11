export type DialectVariant = "american" | "british";

export interface DialectHit {
  term: string;
  suggestion: string;
  offset: number;
}

const WORD_RE = /[A-Za-z]+(?:'[A-Za-z]+)?/g;

let americanToBritishPromise: Promise<Record<string, string>> | undefined;

async function loadAmericanToBritish(): Promise<Record<string, string>> {
  americanToBritishPromise ??= import("./data/en.json").then(
    (mod) => (mod.default ?? mod) as Record<string, string>,
  );
  return americanToBritishPromise;
}

function invert(map: Record<string, string>): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [american, british] of Object.entries(map)) {
    out[british] ??= american;
  }
  return out;
}

let britishToAmericanCache: Record<string, string> | undefined;

function matchCase(word: string, suggestion: string): string {
  if (word === word.toUpperCase()) return suggestion.toUpperCase();
  if (word[0] === word[0]?.toUpperCase()) {
    return suggestion.charAt(0).toUpperCase() + suggestion.slice(1);
  }
  return suggestion;
}

function overlapsAnySpan(offset: number, length: number, spans: Array<[number, number]>): boolean {
  const end = offset + length;
  return spans.some(([start, spanEnd]) => offset < spanEnd && end > start);
}

/**
 * Finds every occurrence of each token in `text` and returns their spans,
 * so dialect matches inside ICU/placeholder syntax (from processor.extractTokens)
 * can be skipped rather than flagged.
 */
export function tokenSpans(text: string, tokens: string[]): Array<[number, number]> {
  const spans: Array<[number, number]> = [];
  for (const token of tokens) {
    if (!token) continue;
    let fromIndex = 0;
    let idx: number;
    while ((idx = text.indexOf(token, fromIndex)) !== -1) {
      spans.push([idx, idx + token.length]);
      fromIndex = idx + token.length;
    }
  }
  return spans;
}

/**
 * Detects words in `text` that belong to the opposite English dialect from
 * `variant`. Word matching is case-insensitive on Unicode word boundaries;
 * matches inside `excludeSpans` (e.g. ICU tokens) are skipped.
 */
export async function detectDialect(
  text: string,
  variant: DialectVariant,
  excludeSpans: Array<[number, number]> = [],
): Promise<DialectHit[]> {
  const americanToBritish = await loadAmericanToBritish();
  let lookup: Record<string, string>;
  if (variant === "american") {
    britishToAmericanCache ??= invert(americanToBritish);
    lookup = britishToAmericanCache;
  } else {
    lookup = americanToBritish;
  }

  const hits: DialectHit[] = [];
  for (const match of text.matchAll(WORD_RE)) {
    const word = match[0];
    const offset = match.index;
    if (overlapsAnySpan(offset, word.length, excludeSpans)) continue;

    const suggestion = lookup[word.toLowerCase()];
    if (!suggestion) continue;

    hits.push({ term: word, suggestion: matchCase(word, suggestion), offset });
  }
  return hits;
}
