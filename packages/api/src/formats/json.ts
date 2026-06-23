import { readText, writeText, pathExists } from "../utils/fs";

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
