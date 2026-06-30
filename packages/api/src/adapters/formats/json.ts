import { readText, writeText, pathExists, join } from "../../infrastructure/fs";
import { flattenObject, unflattenObject } from "../../core/diff";
import type { LocaleFormat } from "../../ports/format";

export async function readJsonFile<T = Record<string, unknown>>(path: string): Promise<T> {
  if (!(await pathExists(path))) {
    throw new Error(`File not found: ${path}`);
  }
  return JSON.parse(await readText(path)) as T;
}

export async function writeJsonFile<T = Record<string, unknown>>(
  path: string,
  data: T,
  options?: { pretty?: boolean; indent?: number },
): Promise<void> {
  const { pretty = true, indent = 2 } = options ?? {};
  const serialized = pretty ? JSON.stringify(data, null, indent) : JSON.stringify(data);
  await writeText(path, serialized);
}

export const jsonFormat: LocaleFormat = {
  name: "json",
  async readLocale(localeDir, locale) {
    const path = join(localeDir, `${locale}.json`);
    if (!(await pathExists(path))) return {};
    const raw = JSON.parse(await readText(path)) as Record<string, unknown>;
    return flattenObject(raw);
  },
  async writeLocale(localeDir, locale, data) {
    const path = join(localeDir, `${locale}.json`);
    await writeText(path, `${JSON.stringify(unflattenObject(data), null, 2)}\n`);
  },
};
