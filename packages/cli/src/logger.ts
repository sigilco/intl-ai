import { configure, getAnsiColorFormatter, getConsoleSink, getLogger } from "@logtape/logtape";

export async function configureLogger(silent: boolean): Promise<void> {
  await configure({
    reset: true,
    sinks: { console: getConsoleSink({ formatter: getAnsiColorFormatter() }) },
    loggers: [
      {
        category: ["intl-ai"],
        lowestLevel: silent ? "error" : "info",
        sinks: ["console"],
      },
      {
        category: ["logtape", "meta"],
        lowestLevel: "warning",
        sinks: ["console"],
      },
    ],
  });
}

export const logger = getLogger(["intl-ai", "cli"]);
