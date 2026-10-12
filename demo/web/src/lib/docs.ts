import { checkIdsFor, complete } from "./pipeline";
import type { ChatBody } from "./pipeline";
import { call } from "./worker";
import type {
  DocExtractResult,
  DocMergeResult,
  Finding,
  GenRow,
  Phase,
  ProviderConfig,
} from "./types";

export interface DocRunResult {
  rows: GenRow[];
  kinds: Record<string, string>;
  warnings: string[];
  merged: string;
  fellBack: string[];
  findings: Finding[];
}

export interface DocRunHooks {
  onPhase: (phase: Phase, note: string) => void;
  onRows: (rows: GenRow[]) => void;
  onProgress: (done: number, total: number) => void;
}

/** `intl-ai-docs` demo cut: extract the document's segment FlatMap,
 * fill every segment through the provider, run the builtin checks, and
 * merge translations back into the untouched skeleton. */
export async function runDocPipeline(
  input: {
    docText: string;
    sourceLocale: string;
    targetLocale: string;
    localeInstruction: string | null;
    linkPrefix: string;
    cfg: ProviderConfig;
  },
  hooks: DocRunHooks,
): Promise<DocRunResult> {
  const { cfg } = input;

  hooks.onPhase("extract", "extracting segments");
  const extracted = await call<DocExtractResult>("docExtract", input.docText, null);

  const rows: GenRow[] = extracted.order.map((seg) => ({
    key: seg,
    source: extracted.segments[seg],
    translation: "",
    status: "queued",
    findings: [],
  }));
  const rowByKey = new Map(rows.map((r) => [r.key, r]));
  const emit = () => hooks.onRows([...rows]);
  emit();

  hooks.onPhase("fill", `${rows.length} segments`);

  for (let i = 0; i < rows.length; i += cfg.batchSize) {
    const batch = rows.slice(i, i + cfg.batchSize);
    for (const r of batch) r.status = "filling";
    emit();

    const body = await call<ChatBody>("buildTranslateBody", {
      sourceLocale: input.sourceLocale,
      targetLocale: input.targetLocale,
      entries: batch.map((r) => ({ key: r.key, source: r.source })),
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
      for (const r of batch) {
        if (translated[r.key] !== undefined) {
          r.translation = translated[r.key];
          r.status = "filled";
        } else {
          r.status = "failed";
        }
      }
    } catch (e) {
      for (const r of batch) r.status = "failed";
      throw e instanceof Error ? e : new Error(String(e));
    } finally {
      hooks.onProgress(Math.min(i + cfg.batchSize, rows.length), rows.length);
      emit();
    }
  }

  hooks.onPhase("check", `running ${checkIdsFor(input.targetLocale).join(", ")}`);
  const findings = await call<Finding[]>(
    "runChecks",
    rows.map((r) => ({ key: r.key, source: r.source, target: r.translation })),
    checkIdsFor(input.targetLocale),
    {
      sourceLocale: input.sourceLocale,
      targetLocale: input.targetLocale,
      localeInstruction: input.localeInstruction,
    },
  );
  for (const f of findings) rowByKey.get(f.key)?.findings.push(f);
  emit();

  hooks.onPhase("merge", "splicing the skeleton");
  const targetMap = Object.fromEntries(
    rows.filter((r) => r.status === "filled").map((r) => [r.key, r.translation]),
  );
  const merged = await call<DocMergeResult>(
    "docMerge",
    input.docText,
    targetMap,
    input.linkPrefix,
    null,
  );

  hooks.onPhase("done", "done");
  emit();

  return {
    rows,
    kinds: extracted.kinds,
    warnings: extracted.warnings,
    merged: merged.output,
    fellBack: merged.fellBack,
    findings,
  };
}
