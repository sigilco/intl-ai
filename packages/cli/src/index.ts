#!/usr/bin/env node
import { cli } from "cleye";
import { createRequire } from "node:module";
import { fillCommand } from "./commands/fill";
import { checkCommand } from "./commands/check";

const { version } = createRequire(import.meta.url)("../package.json") as { version: string };

cli({
  name: "intl-ai",
  version,
  commands: [fillCommand, checkCommand],
});
