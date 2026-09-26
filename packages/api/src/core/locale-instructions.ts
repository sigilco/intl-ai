export function resolveLocaleInstruction(
  instructions: Record<string, string> | undefined,
  locale: string,
): string | undefined {
  if (!instructions) return undefined;
  if (instructions[locale] !== undefined) return instructions[locale];
  const subtag = locale.split("-")[0]!;
  if (instructions[subtag] !== undefined) return instructions[subtag];
  return instructions["*"];
}

export function findUnmatchedInstructionKeys(
  instructions: Record<string, string> | undefined,
  locales: string[],
): string[] {
  if (!instructions) return [];
  const subtags = new Set(locales.map((locale) => locale.split("-")[0]!));
  return Object.keys(instructions).filter((key) => {
    if (key === "*") return false;
    return !locales.includes(key) && !subtags.has(key);
  });
}
