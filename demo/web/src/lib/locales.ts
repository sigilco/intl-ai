const COMMON = [
  "en",
  "en-GB",
  "en-US",
  "es",
  "fr",
  "de",
  "it",
  "pt",
  "pt-BR",
  "nl",
  "pl",
  "sv",
  "da",
  "no",
  "fi",
  "cs",
  "sk",
  "hu",
  "ro",
  "bg",
  "hr",
  "el",
  "tr",
  "ru",
  "uk",
  "ja",
  "ko",
  "zh-CN",
  "zh-TW",
  "ar",
  "he",
  "hi",
  "th",
  "vi",
  "id",
];

const names = new Intl.DisplayNames(["en"], { type: "language" });

export const LOCALES = COMMON.map((code) => ({
  code,
  label: `${code} — ${names.of(code) ?? code}`,
}));

const LOCALE_RE = /(?:^|[.\-_/])([a-z]{2,3}(?:[-_][a-zA-Z]{2,4})?)(?:\.|_|$)/i;

/** Guess a locale from a dropped filename (en.json -> en, app.en-GB.yaml -> en-GB). */
export function detectLocale(fileName: string): string | null {
  const m = LOCALE_RE.exec(fileName);
  if (!m) return null;
  const code = m[1].replace(/_/g, "-");
  const [lang] = code.split("-");
  return /^[a-z]{2,3}$/i.test(lang) ? code : null;
}
