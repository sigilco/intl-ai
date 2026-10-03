import { createOpenAICompatible } from "@ai-sdk/openai-compatible";
import { generateObject, streamObject } from "ai";
import { z } from "zod";
import { call } from "./worker";
import type { Finding, GenRow, Judgement, Phase, ProviderConfig } from "./types";

const translationsSchema = z.object({
  translations: z.array(z.object({ key: z.string(), translated: z.string() })),
});

const judgementsSchema = z.object({
  judgements: z.array(
    z.object({
      key: z.string(),
      score: z.number(),
      reason: z.string().optional(),
      errors: z.array(z.string()).optional(),
    }),
  ),
});

type Message = { role: "system" | "user"; content: string };

/** Split system text out of the wire body: AI SDK v7 wants it via the
 * `instructions` option, not as a system message inside `messages`. */
function promptParts(messages: Message[]) {
  const system = messages
    .filter((m) => m.role === "system")
    .map((m) => m.content)
    .join("\n\n");
  return {
    instructions: system || undefined,
    messages: messages.filter((m) => m.role !== "system"),
  };
}

type ChatBody = {
  model: string;
  messages: Message[];
  response_format: { json_schema: { name: string; schema: unknown } };
  temperature: number;
};

export type RunResult = {
  rows: GenRow[];
  outText: string;
  findings: Finding[];
};

export type RunHooks = {
  onPhase: (phase: Phase, note: string) => void;
  onRows: (rows: GenRow[]) => void;
  onProgress: (done: number, total: number) => void;
};

function providerOf(cfg: ProviderConfig) {
  return createOpenAICompatible({
    name: "intl-ai-demo",
    baseURL: cfg.baseUrl.replace(/\/+$/, ""),
    apiKey: cfg.apiKey || undefined,
    supportsStructuredOutputs: true,
  });
}

/** Structured call: stream partial objects when enabled, else one shot. */
async function complete(
  cfg: ProviderConfig,
  body: ChatBody,
  onPartial?: (translations: Record<string, string>) => void,
): Promise<Record<string, string>> {
  const provider = providerOf(cfg);
  const model = provider.chatModel(body.model);
  const { instructions, messages } = promptParts(body.messages);
  const args = {
    model,
    schema: translationsSchema,
    schemaName: "translations",
    instructions,
    messages,
    temperature: body.temperature,
  };

  if (cfg.stream) {
    try {
      const { partialObjectStream, object } = await streamObject(args);
      for await (const partial of partialObjectStream) {
        const acc: Record<string, string> = {};
        for (const t of partial?.translations ?? []) {
          if (t?.key && typeof t.translated === "string") acc[t.key] = t.translated;
        }
        if (Object.keys(acc).length) onPartial?.(acc);
      }
      const final = await object;
      const out: Record<string, string> = {};
      for (const t of final.translations) out[t.key] = t.translated;
      return out;
    } catch {
      // Some compatible endpoints lack SSE or json_schema support; fall
      // back to a plain request rather than failing the run.
    }
  }
  const { object } = await generateObject(args);
  const out: Record<string, string> = {};
  for (const t of object.translations) out[t.key] = t.translated;
  return out;
}

async function judge(cfg: ProviderConfig, body: ChatBody): Promise<Judgement[]> {
  const provider = providerOf(cfg);
  const { instructions, messages } = promptParts(body.messages);
  const { object } = await generateObject({
    model: provider.chatModel(body.model),
    schema: judgementsSchema,
    schemaName: "judgements",
    instructions,
    messages,
    temperature: body.temperature,
  });
  return object.judgements;
}

function checkIdsFor(targetLocale: string): string[] {
  const ids = ["icu", "placeholder-parity"];
  if (/^en-(us|gb|uk)$/i.test(targetLocale)) ids.push(`dialect:${targetLocale.toLowerCase()}`);
  return ids;
}

export async function runPipeline(
  input: {
    sourceText: string;
    targetText: string;
    sourceLocale: string;
    targetLocale: string;
    localeInstruction: string | null;
    targetFormat: string;
    cfg: ProviderConfig;
  },
  hooks: RunHooks,
): Promise<RunResult> {
  const { cfg } = input;

  hooks.onPhase("flatten", "flattening locales");
  const sourceFlat = await call<Record<string, string>>("flatten", input.sourceText);
  const targetFlat = input.targetText
    ? await call<Record<string, string>>("flatten", input.targetText)
    : null;
  const missing = await call<string[]>("missingKeys", sourceFlat, targetFlat);

  const rows: GenRow[] = Object.keys(sourceFlat).map((key) => ({
    key,
    source: sourceFlat[key],
    translation: targetFlat?.[key] ?? "",
    status: targetFlat?.[key] !== undefined ? "existing" : "queued",
    findings: [],
  }));
  const rowByKey = new Map(rows.map((r) => [r.key, r]));
  const emit = () => hooks.onRows([...rows]);
  emit();

  hooks.onPhase("fill", `${Object.keys(sourceFlat).length} keys, ${missing.length} missing`);

  let done = 0;
  for (let i = 0; i < missing.length; i += cfg.batchSize) {
    const batch = missing.slice(i, i + cfg.batchSize);
    for (const key of batch) {
      const r = rowByKey.get(key);
      if (r) r.status = "filling";
    }
    emit();

    const body = await call<ChatBody>("buildTranslateBody", {
      sourceLocale: input.sourceLocale,
      targetLocale: input.targetLocale,
      entries: batch.map((key) => ({ key, source: sourceFlat[key] })),
      localeInstruction: input.localeInstruction,
      model: cfg.model,
      modelParams: {},
    });

    try {
      const translated = await complete(cfg, body, (acc) => {
        for (const [key, value] of Object.entries(acc)) {
          const r = rowByKey.get(key);
          if (r) r.translation = value;
        }
        emit();
      });
      for (const key of batch) {
        const r = rowByKey.get(key);
        if (!r) continue;
        if (translated[key] !== undefined) {
          r.translation = translated[key];
          r.status = "filled";
        } else {
          r.status = "failed";
        }
      }
    } catch (e) {
      for (const key of batch) {
        const r = rowByKey.get(key);
        if (r) r.status = "failed";
      }
      throw e instanceof Error ? e : new Error(String(e));
    } finally {
      done = Math.min(i + cfg.batchSize, missing.length);
      hooks.onProgress(done, missing.length);
      emit();
    }
  }

  const outText = await call<string>(
    "unflatten",
    input.targetText || null,
    Object.fromEntries(
      rows.filter((r) => r.status === "filled").map((r) => [r.key, r.translation]),
    ),
    input.targetFormat,
  );

  hooks.onPhase("check", `running ${checkIdsFor(input.targetLocale).join(", ")}`);
  const items = rows.map((r) => ({
    key: r.key,
    source: r.source,
    target: r.translation,
  }));
  const findings = await call<Finding[]>("runChecks", items, checkIdsFor(input.targetLocale), {
    sourceLocale: input.sourceLocale,
    targetLocale: input.targetLocale,
    localeInstruction: input.localeInstruction,
  });
  for (const f of findings) rowByKey.get(f.key)?.findings.push(f);

  const filledRows = rows.filter((r) => r.status === "filled");
  if (cfg.runJudge && filledRows.length > 0) {
    hooks.onPhase("judge", `judging ${filledRows.length} translation(s)`);
    const body = await call<ChatBody>("buildJudgeBody", {
      items: filledRows.map((r) => ({
        key: r.key,
        locale: input.targetLocale,
        source: r.source,
        translation: r.translation,
      })),
      localeInstruction: input.localeInstruction,
      model: cfg.model,
      modelParams: {},
    });
    try {
      const judgements = await judge(cfg, body);
      for (const j of judgements) {
        const r = rowByKey.get(j.key);
        if (r) r.judge = j;
      }
    } catch (e) {
      hooks.onPhase("judge", `judge failed: ${e instanceof Error ? e.message : String(e)}`);
    }
    emit();
  }

  hooks.onPhase("done", "done");
  emit();
  return { rows, outText, findings };
}
