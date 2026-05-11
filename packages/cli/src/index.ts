#!/usr/bin/env node
import { Command } from "commander";
import { fillCommand } from "./commands/fill";
import { checkCommand } from "./commands/check";

const program = new Command();

program
  .name("intl-ai")
  .version("0.0.1")
  .description("AI-powered i18n translation CLI")
  .option("--silent", "Suppress all output except errors")
  .addCommand(fillCommand)
  .addCommand(checkCommand)
  .parse(process.argv);
