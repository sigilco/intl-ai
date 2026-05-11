export interface NextLoaderOptions {
  debug?: boolean;
}

export default function intlAiLoader(this: any, source: string): string {
  return `export default ${source}`;
}
