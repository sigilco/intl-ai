import { readText, writeText, pathExists } from "../../infrastructure/fs";
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
  extension: ".json",
  read: (path) => readJsonFile(path),
  write: (path, data) => writeJsonFile(path, data),
};
