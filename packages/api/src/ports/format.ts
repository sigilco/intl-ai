export interface LocaleFormat {
  readonly name: string;
  readLocale(localeDir: string, locale: string): Promise<Record<string, string>>;
  writeLocale(localeDir: string, locale: string, data: Record<string, string>): Promise<void>;
}
