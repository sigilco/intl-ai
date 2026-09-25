import { describe, it, expect } from "vitest";
import { translateBatch } from "./translator";
import {
  resolveAgentPreset,
  agentPresetIds,
  type AgentPreset,
} from "../../infrastructure/transports/presets";
import { icuProcessor } from "../../adapters/processors/icu";

/**
 * Drives a real local coding-agent subprocess via the preset table in
 * presets.ts, selected by AGENT_PRESET (default "opencode"). Exercises the
 * actual command transport used in production, including the crush argv
 * prompt path: `AGENT_PRESET=crush pnpm test:integration`. Local-only: run
 * via `pnpm test:integration`, never part of `pnpm test` (excluded in
 * vitest.config.ts) or CI (no agent installed there, and CI also
 * short-circuits below as a second guard).
 */
const hasCommand = Boolean(process.env.AGENT_PRESET ?? "opencode");
const preset = (process.env.AGENT_PRESET ?? "opencode") as AgentPreset;

describe.skipIf(!hasCommand || Boolean(process.env.CI))(
  "translateBatch (command transport, live)",
  () => {
    it(`translates a batch through a real agent subprocess (preset: ${preset})`, async () => {
      expect(agentPresetIds).toContain(preset);
      const transport = resolveAgentPreset(preset);

      const result = await translateBatch({
        transport,
        processor: icuProcessor,
        entries: [
          { key: "greeting", source: "Hello {name}, welcome back." },
          { key: "farewell", source: "See you soon." },
        ],
        targetLocale: "es",
        sourceLocale: "en",
      });

      expect(result).toHaveLength(2);
      for (const entry of result) {
        expect(entry.success).toBe(true);
        expect(entry.translated).toBeTruthy();
      }
      expect(result.find((r) => r.key === "greeting")?.translated).toContain("{name}");
    }, 300_000);
  },
);
