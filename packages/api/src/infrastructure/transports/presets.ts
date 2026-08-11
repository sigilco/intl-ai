import { createCommandTransport } from "./command";
import type { AITransport } from "../../ports/provider";

export const agentPresetIds = ["claude-code", "opencode", "codex", "crush", "gemini"] as const;
export type AgentPreset = (typeof agentPresetIds)[number];

interface PresetDefinition {
  command: string;
  args: string[];
}

/**
 * Command + args per preset, each carrying the flag that stops the agent
 * from blocking an unattended build on a permission prompt. Re-verified
 * against installed CLIs (not just published docs) on 2026-08-11:
 * - opencode: `--yolo` does not exist on this CLI; the real flag is `--auto`.
 * - codex: `--approve-for-me` cannot be combined with `--sandbox`.
 * - crush: `-y`/`--yolo` errors as an unknown flag on v0.88.0 despite being
 *   documented in `--help`. Omitted; a pure text-generation prompt does not
 *   trigger crush's permission system anyway.
 * Agent CLI surfaces drift fast — re-verify before trusting this table.
 */
const presets: Record<AgentPreset, PresetDefinition> = {
  "claude-code": {
    command: "claude",
    args: ["-p", "--dangerously-skip-permissions", "--output-format", "text"],
  },
  opencode: { command: "opencode", args: ["run", "--auto"] },
  codex: { command: "codex", args: ["exec", "--approve-for-me"] },
  crush: { command: "crush", args: ["run", "-q"] },
  // gemini's -p takes a value and appends stdin to it; pass "" so the full
  // prompt travels on stdin like every other preset.
  gemini: { command: "gemini", args: ["-p", "", "-y"] },
};

export function resolveAgentPreset(preset: AgentPreset, cwd?: string): AITransport {
  const def = presets[preset];
  return createCommandTransport({ id: preset, command: def.command, args: def.args, cwd });
}
