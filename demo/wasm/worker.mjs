import init, {
  flatten,
  missingKeys,
  buildTranslateBody,
  parseTranslations,
  buildJudgeBody,
  parseJudgements,
  runChecks,
  unflatten,
} from "./pkg/intl_ai_wasm.js";

const fns = {
  flatten,
  missingKeys,
  buildTranslateBody,
  parseTranslations,
  buildJudgeBody,
  parseJudgements,
  runChecks,
  unflatten,
};

self.onmessage = (event) => {
  const { id, fn, args } = event.data;
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

init()
  .then(() => self.postMessage({ ready: true }))
  .catch((error) =>
    self.postMessage({
      ready: false,
      error: error instanceof Error ? error.message : String(error),
    }),
  );
