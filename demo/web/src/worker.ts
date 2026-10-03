/// <reference lib="webworker" />
// Hosts the intl-ai wasm bundle inside a module worker so pipeline
// parse/check work never blocks the page.

import init, * as wasmModule from "./lib/pkg/intl_ai_wasm.js";
import wasmUrl from "./lib/pkg/intl_ai_wasm_bg.wasm?url";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type WasmFns = Record<string, (...args: any[]) => any>;

const fns: WasmFns = {
  flatten: wasmModule.flatten,
  missingKeys: wasmModule.missingKeys,
  buildTranslateBody: wasmModule.buildTranslateBody,
  parseTranslations: wasmModule.parseTranslations,
  buildJudgeBody: wasmModule.buildJudgeBody,
  parseJudgements: wasmModule.parseJudgements,
  runChecks: wasmModule.runChecks,
  unflatten: wasmModule.unflatten,
};

self.onmessage = (event: MessageEvent) => {
  const { id, fn, args } = event.data as {
    id: number;
    fn: string;
    args: unknown[];
  };
  try {
    const result = fns[fn](...args);
    self.postMessage({ id, ok: true, result });
  } catch (error) {
    self.postMessage({
      id,
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    });
  }
};

init({ module_or_path: wasmUrl })
  .then(() => self.postMessage({ ready: true }))
  .catch((error: unknown) =>
    self.postMessage({
      ready: false,
      error: error instanceof Error ? error.message : String(error),
    }),
  );
