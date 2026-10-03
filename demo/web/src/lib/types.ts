export type Finding = { key: string; check: string; message: string };
export type Judgement = {
  key: string;
  score: number;
  reason?: string;
  errors?: string[];
};

export type RowStatus = "existing" | "queued" | "filling" | "filled" | "failed";

export type GenRow = {
  key: string;
  source: string;
  translation: string;
  status: RowStatus;
  findings: Finding[];
  judge?: Judgement;
};

export type ProviderPreset = "illo" | "openrouter" | "openai" | "custom";

export type ProviderConfig = {
  preset: ProviderPreset;
  baseUrl: string;
  apiKey: string;
  model: string;
  batchSize: number;
  runJudge: boolean;
  stream: boolean;
};

export type Phase = "idle" | "flatten" | "fill" | "check" | "judge" | "done" | "error";
