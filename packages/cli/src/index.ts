#!/usr/bin/env node
import { defineCommand, runMain } from "citty";

const main = defineCommand({
  meta: {
    name: "intl-ai",
    version: "0.2.0",
    description:
      "AI-powered i18n translation. Config schema: https://www.schemastore.org/intl-ai.json",
  },
  subCommands: {
    fill: () => import("./commands/fill").then((m) => m.fillCommand),
    check: () => import("./commands/check").then((m) => m.checkCommand),
  },
});

runMain(main);
