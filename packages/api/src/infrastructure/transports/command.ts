import { spawn } from "node:child_process";
import { stripVTControlCharacters } from "node:util";
import type { AITransport } from "../../ports/provider";

export interface CommandTransportOptions {
  id: string;
  command: string;
  args: string[];
  cwd?: string;
  /** Wall-clock budget before SIGTERM, then SIGKILL after a grace period. Default 300_000ms. */
  timeoutMs?: number;
  /** Buffered stdout cap; the process is killed and an error thrown past this. Default 10MB. */
  maxStdoutBytes?: number;
  /**
   * Where the prompt travels. Default "stdin". "argv" appends it as the last
   * arg instead (needed for CLIs whose non-interactive reader ignores stdin,
   * e.g. crush) but is capped by the OS ARG_MAX (~128KB+ depending on
   * platform), a real ceiling stdin doesn't have — opt in only where required.
   */
  promptVia?: "stdin" | "argv";
}

const DEFAULT_TIMEOUT_MS = 300_000;
const DEFAULT_MAX_STDOUT_BYTES = 10 * 1024 * 1024;
const KILL_GRACE_MS = 5_000;

/**
 * Wraps a local headless coding agent (subprocess) as an AITransport.
 * Never uses `shell: true`: command and args are passed as an array so
 * nothing is shell-expanded. The prompt goes in on stdin by default; set
 * `promptVia: "argv"` for agents whose non-interactive reader ignores stdin.
 */
export function createCommandTransport(opts: CommandTransportOptions): AITransport {
  const {
    id,
    command,
    args,
    cwd,
    timeoutMs = DEFAULT_TIMEOUT_MS,
    maxStdoutBytes = DEFAULT_MAX_STDOUT_BYTES,
    promptVia = "stdin",
  } = opts;

  return {
    id,
    async complete({ systemPrompt, userPrompt, signal }) {
      const input = `${systemPrompt}\n\n---\n\n${userPrompt}`;
      const raw = await runCommand({
        command,
        args: promptVia === "argv" ? [...args, input] : args,
        cwd,
        input: promptVia === "argv" ? undefined : input,
        timeoutMs,
        maxStdoutBytes,
        signal,
      });
      const cleaned = stripVTControlCharacters(raw);
      return { content: extractJsonPayload(cleaned) ?? cleaned };
    },
  };
}

interface RunCommandOptions {
  command: string;
  args: string[];
  cwd?: string;
  /** Written to stdin when set; when undefined stdin is closed with no data (argv-only prompt). */
  input?: string;
  timeoutMs: number;
  maxStdoutBytes: number;
  signal: AbortSignal;
}

function runCommand(opts: RunCommandOptions): Promise<string> {
  const { command, args, cwd, input, timeoutMs, maxStdoutBytes, signal } = opts;

  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd, stdio: ["pipe", "pipe", "pipe"] });

    let stdout = "";
    let stderr = "";
    let stdoutBytes = 0;
    let capped = false;
    let settled = false;
    let killTimer: ReturnType<typeof setTimeout> | undefined;

    const cleanup = () => {
      clearTimeout(timer);
      clearTimeout(killTimer);
      signal.removeEventListener("abort", onAbort);
    };

    const finish = (fn: () => void) => {
      if (settled) return;
      settled = true;
      cleanup();
      fn();
    };

    const killEscalating = () => {
      child.kill("SIGTERM");
      killTimer = setTimeout(() => child.kill("SIGKILL"), KILL_GRACE_MS);
    };

    const timer = setTimeout(() => {
      killEscalating();
      finish(() =>
        reject(new Error(`command transport: ${command} timed out after ${timeoutMs}ms`)),
      );
    }, timeoutMs);

    const onAbort = () => {
      killEscalating();
      finish(() => reject(new Error(`command transport: ${command} aborted`)));
    };
    signal.addEventListener("abort", onAbort);

    child.stdout?.on("data", (chunk: Buffer) => {
      if (capped) return;
      stdoutBytes += chunk.length;
      if (stdoutBytes > maxStdoutBytes) {
        capped = true;
        killEscalating();
        finish(() =>
          reject(
            new Error(`command transport: ${command} stdout exceeded ${maxStdoutBytes} bytes`),
          ),
        );
        return;
      }
      stdout += chunk.toString("utf-8");
    });

    child.stderr?.on("data", (chunk: Buffer) => {
      stderr += chunk.toString("utf-8");
    });

    child.on("error", (err) => {
      finish(() => reject(err));
    });

    child.on("close", (code) => {
      finish(() => {
        if (code !== 0) {
          const stderrPreview = stderr.trim().slice(0, 2000);
          reject(
            new Error(
              `command transport: ${command} exited with code ${code}${stderrPreview ? `: ${stderrPreview}` : ""}`,
            ),
          );
          return;
        }
        resolve(stdout);
      });
    });

    child.stdin?.end(input ?? "");
  });
}

/**
 * Agents rarely emit bare JSON. Tries a fenced code block first, then the
 * first balanced (string-aware) `{...}` span. Returns undefined on a miss
 * so the caller falls through to the existing JSON.parse retry.
 */
export function extractJsonPayload(text: string): string | undefined {
  const fenced = text.match(/```(?:json)?\s*\n?([\s\S]*?)```/);
  if (fenced?.[1]?.trim()) return fenced[1].trim();

  const start = text.indexOf("{");
  if (start === -1) return undefined;

  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let i = start; i < text.length; i++) {
    const ch = text[i];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === '"') {
        inString = false;
      }
      continue;
    }
    if (ch === '"') {
      inString = true;
    } else if (ch === "{") {
      depth++;
    } else if (ch === "}") {
      depth--;
      if (depth === 0) return text.slice(start, i + 1);
    }
  }
  return undefined;
}
