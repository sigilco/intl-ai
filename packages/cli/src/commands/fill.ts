import { defineCommand } from "citty";
import { consola } from "consola";
import { runFill } from "@intl-ai/api";
import { loadConfig } from "../config/loader";

export const fillCommand = defineCommand({
  meta: {
    name: "fill",
    description: "Fill missing translations using AI",
  },
  args: {
    config: {
      type: "string",
      default: "intl-ai.config.json",
      description: "Path to JSON config file",
    },
    locale: {
      type: "string",
      description: "Target a specific locale only",
    },
    force: {
      type: "boolean",
      default: false,
      description: "Overwrite human-edited entries",
    },
    silent: {
      type: "boolean",
      default: false,
      description: "Suppress all output except errors",
    },
    "dry-run": {
      type: "boolean",
      default: false,
      description: "Preview changes without modifying files",
    },
  },
  async run({ args }) {
    if (args.silent) {
      consola.level = 0;
    }

    const config = await loadConfig(args.config);
    consola.start("Translating missing keys...");

    try {
      const result = await runFill(config, {
        locale: args.locale,
        force: args.force,
        dryRun: args["dry-run"],
      });

      consola.success(`Translated ${result.translated} keys across ${result.locales.join(", ")}`);

      if (result.skipped > 0) {
        consola.info(`${result.skipped} keys already up to date`);
      }
      if (result.errors > 0) {
        consola.warn(`${result.errors} translation errors occurred`);
      }
      if (args["dry-run"]) {
        consola.info("Dry-run: no files were modified");
      }

      if (result.errors > 0) {
        process.exit(1);
      }
    } catch (error) {
      consola.fail("Translation failed");
      consola.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    }
  },
});
