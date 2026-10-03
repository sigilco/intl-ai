import type { ProviderConfig, ProviderPreset } from "./types";

const KEY = "intl-ai-demo:config";

export const PRESETS: Record<
  ProviderPreset,
  { label: string; baseUrl: string; model: string; needsKey: boolean }
> = {
  illo: {
    label: "illo (api.illo.fyi)",
    baseUrl: "https://api.illo.fyi/v1",
    model: "illo-demo",
    needsKey: false,
  },
  openrouter: {
    label: "OpenRouter",
    baseUrl: "https://openrouter.ai/api/v1",
    model: "openrouter/free",
    needsKey: true,
  },
  openai: {
    label: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    needsKey: true,
  },
  custom: {
    label: "Custom (OpenAI-compatible)",
    baseUrl: "http://localhost:8787/v1",
    model: "",
    needsKey: false,
  },
};

const DEFAULTS: ProviderConfig = {
  preset: "illo",
  baseUrl: PRESETS.illo.baseUrl,
  apiKey: "",
  model: PRESETS.illo.model,
  batchSize: 20,
  runJudge: true,
  stream: true,
};

function load(): ProviderConfig {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    return { ...DEFAULTS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULTS };
  }
}

export const config = $state<ProviderConfig>(load());

export function saveConfig() {
  localStorage.setItem(KEY, JSON.stringify(config));
}

export function applyPreset(preset: ProviderPreset) {
  config.preset = preset;
  config.baseUrl = PRESETS[preset].baseUrl;
  if (PRESETS[preset].model) config.model = PRESETS[preset].model;
  saveConfig();
}

export type TestState = { kind: "idle" | "busy" | "ok" | "fail"; detail: string };

export async function testProvider(): Promise<TestState> {
  const url = `${config.baseUrl.replace(/\/+$/, "")}/models`;
  try {
    const headers: Record<string, string> = {};
    if (config.apiKey) headers.authorization = `Bearer ${config.apiKey}`;
    const res = await fetch(url, { headers });
    if (!res.ok) return { kind: "fail", detail: `HTTP ${res.status}` };
    const data = await res.json();
    const ids = Array.isArray(data?.data)
      ? data.data.map((m: { id?: string }) => m.id).filter(Boolean)
      : [];
    return {
      kind: "ok",
      detail: ids.length ? `${ids.length} model(s)` : "reachable",
    };
  } catch (e) {
    return { kind: "fail", detail: e instanceof Error ? e.message : String(e) };
  }
}

export async function fetchModels(): Promise<string[]> {
  const url = `${config.baseUrl.replace(/\/+$/, "")}/models`;
  const headers: Record<string, string> = {};
  if (config.apiKey) headers.authorization = `Bearer ${config.apiKey}`;
  const res = await fetch(url, { headers });
  if (!res.ok) return [];
  const data = await res.json();
  if (!Array.isArray(data?.data)) return [];
  return data.data
    .map((m: { id?: string }) => m.id)
    .filter((id: unknown): id is string => typeof id === "string");
}
