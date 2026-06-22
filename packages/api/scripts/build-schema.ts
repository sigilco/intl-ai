import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";
import { IntlAiJsonConfigSchema } from "../src/schema/json-config";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "../../..");

const baseSchema = z.toJSONSchema(IntlAiJsonConfigSchema);

const output = {
  ...baseSchema,
  $id: "https://www.schemastore.org/intl-ai.json",
  title: "intl-ai config",
  description: "Configuration for the intl-ai translation build step",
} as Record<string, unknown>;

const json = JSON.stringify(output, null, 2) + "\n";

// Write to packages/api/src/schema/intl-ai.schema.json (source)
const srcPath = resolve(__dirname, "../src/schema/intl-ai.schema.json");
writeFileSync(srcPath, json);

// Write to docs/public/schema/v1.json (served by GitHub Pages at /schema/v1.json)
const docsPath = resolve(repoRoot, "docs/public/schema/v1.json");
mkdirSync(dirname(docsPath), { recursive: true });
writeFileSync(docsPath, json);

console.log(`Schema written to:`);
console.log(`  ${srcPath}`);
console.log(`  ${docsPath}`);
