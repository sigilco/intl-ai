import { parseArgs } from "node:util";
import { $ } from "bun";
import task from "tasuku";

interface TaskError {
  errors?: string;
  warnings?: string;
}
interface Task {
  display: string;
  cmd: string;
  filter?: string;
  onError?: (_: $.ShellError) => TaskError;
}

const defaultOnError = ({ stderr }: $.ShellError): TaskError => ({
  errors: stderr.toString("utf8"),
});

const getLintErrors = ({ stdout, stderr }: $.ShellError): TaskError => {
  const output = stdout.toString("utf-8") + stderr.toString("utf-8");
  const lintResults = [...output.matchAll(/Found (\d+) warnings? and (\d+) errors?/g)];
  if (lintResults.length > 0) {
    const totalErrors = lintResults.reduce((sum, match) => sum + parseInt(match[2]!, 10), 0);
    const totalWarnings = lintResults.reduce((sum, match) => sum + parseInt(match[1]!, 10), 0);
    if (totalErrors === 0 && totalWarnings > 0)
      return {
        warnings: `WARN: (${totalWarnings} linter warning${totalWarnings === 1 ? "" : "s"})`,
      };
  }
  return {};
};

const TASKS: Task[] = [
  { display: "TypeScript", cmd: "typecheck" },
  { display: "Lint", cmd: "lint", onError: getLintErrors },
  { display: "Format", cmd: "format" },
  { display: "Test", cmd: "test" },
];

const quietTaskRunner = async (t: Task, packageName?: string): Promise<void> => {
  const filter = packageName ? `--filter=${packageName}` : "";
  const fullCmd = `bunx turbo ${t.cmd} ${filter}`.trim();
  try {
    await $`bash -c ${fullCmd}`;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (error: any) {
    const shellError = error as $.ShellError;
    if (t.onError) {
      const result = t.onError(shellError);
      if (result.warnings) {
        throw new Error(`WARN: ${result.warnings}`);
      }
      if (result.errors) {
        throw new Error(result.errors);
      }
    }
    throw new Error(shellError.stderr?.toString("utf8") || shellError.message || "Command failed");
  }
};

const getPackageName = (cwd: string): string | undefined => {
  const pkgPath = `${cwd}/package.json`;
  try {
    const pkg = JSON.parse(require("fs").readFileSync(pkgPath, "utf-8"));
    return pkg.name;
  } catch {
    return undefined;
  }
};

const getInput = (): { cwd?: string } => {
  const { values } = parseArgs({
    args: process.argv.slice(2),
    options: { cwd: { type: "string" } },
    strict: false,
  });
  return values as { cwd?: string };
};

const main = async (): Promise<void> => {
  const input = getInput();
  const packageName = input.cwd ? getPackageName(input.cwd) : undefined;

  await task.group(
    (task) =>
      TASKS.map((t) =>
        task(t.display, async () => {
          await quietTaskRunner(t, packageName);
        }),
      ),
    { concurrency: Infinity, stopOnError: false },
  );
};

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
