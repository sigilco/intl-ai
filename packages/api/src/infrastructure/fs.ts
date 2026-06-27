import { readFile, writeFile, access, mkdir, rm, readdir, stat } from "node:fs/promises";
import { dirname, join, isAbsolute, relative, resolve } from "pathe";

export { join, dirname, isAbsolute, relative, resolve };

export async function readText(path: string): Promise<string> {
  return readFile(path, "utf-8");
}

export async function writeText(path: string, content: string): Promise<void> {
  await writeFile(path, content, "utf-8");
}

export async function pathExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

export async function ensureDir(path: string): Promise<void> {
  await mkdir(path, { recursive: true });
}

export async function removeDir(path: string): Promise<void> {
  await rm(path, { recursive: true, force: true });
}

export async function listFiles(path: string): Promise<string[]> {
  return readdir(path);
}

export async function fileSize(path: string): Promise<number> {
  const s = await stat(path);
  return s.size;
}

export function getNestedValue(obj: unknown, path: string): string {
  const parts = path.split(".");
  let current: unknown = obj;
  for (const part of parts) {
    if (current && typeof current === "object") {
      current = (current as Record<string, unknown>)[part];
    } else {
      return "";
    }
  }
  return String(current ?? "");
}

export function setNestedValue(obj: Record<string, unknown>, path: string, value: unknown): void {
  const parts = path.split(".");
  const lastPart = parts.pop()!;
  let current: Record<string, unknown> = obj;
  for (const part of parts) {
    if (!(part in current)) current[part] = {};
    current = current[part] as Record<string, unknown>;
  }
  current[lastPart] = value;
}
