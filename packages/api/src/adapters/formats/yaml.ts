import { readText, writeText, pathExists, dirname, join } from "../../infrastructure/fs";
import { parse, stringify } from "yaml";
import { flattenObject, unflattenObject } from "../../core/diff";
import type { LocaleFormat } from "../../ports/format";

export { dirname };

export async function readYamlFile<T = Record<string, unknown>>(path: string): Promise<T> {
  if (!(await pathExists(path))) {
    throw new Error(`File not found: ${path}`);
  }
  return parse(await readText(path)) as T;
}

export async function writeYamlFile(path: string, data: Record<string, unknown>): Promise<void> {
  await writeText(path, stringify(data));
}

export const yamlFormat: LocaleFormat = {
  name: "yaml",
  async readLocale(localeDir, locale) {
    const path = join(localeDir, `${locale}.yaml`);
    if (!(await pathExists(path))) return {};
    const raw = parse(await readText(path)) as Record<string, unknown>;
    return flattenObject(raw);
  },
  async writeLocale(localeDir, locale, data) {
    const path = join(localeDir, `${locale}.yaml`);
    await writeText(path, stringify(unflattenObject(data)));
  },
};
