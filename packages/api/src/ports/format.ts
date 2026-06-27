export interface LocaleFormat {
  extension: string;
  read(path: string): Promise<Record<string, unknown>>;
  write(path: string, data: Record<string, unknown>): Promise<void>;
}
