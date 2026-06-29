import { readText, writeText, pathExists, dirname } from "../../infrastructure/fs";
import { parse, stringify } from "yaml";
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
  extension: ".yaml",
  read: (path) => readYamlFile(path),
  write: writeYamlFile,
};
