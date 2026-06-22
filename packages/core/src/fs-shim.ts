import { readFile, writeFile, mkdir, readdir, stat, rm, access } from "node:fs/promises";
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
