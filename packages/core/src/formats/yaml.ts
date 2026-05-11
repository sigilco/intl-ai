import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";
import { parse, stringify } from "yaml";

export interface YamlFormatOptions {
  pretty?: boolean;
  lineWidth?: number;
}

export function readYamlFile<T = Record<string, unknown>>(filePath: string): T {
  if (!existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }
  const content = readFileSync(filePath, "utf-8");
  return parse(content) as T;
}

export function writeYamlFile<T = Record<string, unknown>>(
  filePath: string,
  data: T,
  options: YamlFormatOptions = {},
): void {
  const pretty = options.pretty ?? true;
  const lineWidth = options.lineWidth ?? 80;

  writeFileSync(filePath, pretty ? stringify(data, { lineWidth }) : stringify(data), "utf-8");
}
