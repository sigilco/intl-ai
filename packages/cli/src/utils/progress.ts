import ora from "ora";

export interface ProgressOptions {
  silent?: boolean;
}

export const createProgressReporter = (options: ProgressOptions = {}) => {
  const { silent = false } = options;

  return {
    startSpinner: (message: string) => {
      if (silent) return null;
      return ora(message).start();
    },

    updateSpinner: (spinner: any, message: string) => {
      if (!spinner || silent) return;
      spinner.text = message;
    },

    succeedSpinner: (spinner: any, message: string) => {
      if (!spinner || silent) return;
      spinner.succeed(message);
    },

    failSpinner: (spinner: any, message: string) => {
      if (!spinner || silent) return;
      spinner.fail(message);
    },

    stopSpinner: (spinner: any) => {
      if (!spinner || silent) return;
      spinner.stop();
    },

    log: (message: string) => {
      if (!silent) {
        console.log(message);
      }
    },

    error: (message: string) => {
      if (!silent) {
        console.error(message);
      }
    },

    logSummary: (summary: {
      locale: string;
      total: number;
      successful: number;
      failed: number;
    }) => {
      if (silent) return;

      console.log("\n" + "=".repeat(50));
      console.log(`Summary for ${summary.locale}:`);
      console.log(`  Total entries: ${summary.total}`);
      console.log(`  Successful: ${summary.successful}`);
      console.log(`  Failed: ${summary.failed}`);
      console.log("=".repeat(50) + "\n");
    },
  };
};

export type ProgressReporter = ReturnType<typeof createProgressReporter>;
