import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";

export interface JsonFormatOptions {
  pretty?: boolean;
  indent?: number;
}

export function readJsonFile<T = Record<string, unknown>>(filePath: string): T {
  if (!existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }
  const content = readFileSync(filePath, "utf-8");
  return JSON.parse(content) as T;
}

export function writeJsonFile<T = Record<string, unknown>>(
  filePath: string,
  data: T,
  options: JsonFormatOptions = {},
): void {
  const pretty = options.pretty ?? true;
  const indent = options.indent ?? 2;

  writeFileSync(
    filePath,
    pretty ? JSON.stringify(data, null, indent) : JSON.stringify(data),
    "utf-8",
  );
}
