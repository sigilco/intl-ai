import init, {
  flatten,
  missingKeys,
  buildTranslateBody,
  parseTranslations,
  buildJudgeBody,
  parseJudgements,
  runChecks,
  unflatten,
  runFill,
  runCheck,
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

// Streaming drivers take (spec, transport, onEvent): the transport is
// the host-owned fetch loop below, and every core ProgressEvent is
// forwarded to the page as `{id, event}`.
const streamFns = { runFill, runCheck };

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function chatComplete(baseUrl, apiKey, body, onStatus) {
  const url = `${baseUrl.replace(/\/+$/, "")}/chat/completions`;
  let lastError = null;
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const headers = { "content-type": "application/json" };
    // api.illo.fyi's CORS policy does not allow Authorization; only send
    // it when the user pasted a key for a BYO endpoint.
    if (apiKey) headers.authorization = `Bearer ${apiKey}`;
    try {
      const res = await fetch(url, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
      });
      if (res.ok) {
        const data = await res.json();
        const content = data?.choices?.[0]?.message?.content;
        if (typeof content !== "string") {
          throw new Error("response missing choices[0].message.content");
        }
        return content;
      }
      const retryable = res.status === 429 || res.status >= 500;
      const text = await res.text();
      lastError = new Error(`provider HTTP ${res.status}: ${text.slice(0, 300)}`);
      if (!retryable) throw lastError;
      const retryAfter = Number(res.headers.get("retry-after"));
      const waitMs = Number.isFinite(retryAfter)
        ? Math.min(retryAfter * 1000, 20000)
        : 2000 * (attempt + 1);
      if (attempt < 2) {
        onStatus?.(`HTTP ${res.status}, retrying in ${Math.round(waitMs / 1000)}s…`);
        await sleep(waitMs);
      }
    } catch (e) {
      if (e instanceof Error && e.message.startsWith("provider HTTP")) throw e;
      lastError = e;
      if (attempt < 2) await sleep(2000 * (attempt + 1));
    }
  }
  throw new Error(
    `provider unavailable after 3 attempts: ${lastError ? lastError.message : "unknown error"}`,
  );
}

self.onmessage = async (event) => {
  const { id, fn, args } = event.data;
  try {
    let result;
    if (fn in streamFns) {
      const [spec, provider] = args;
      const transport = (body) =>
        chatComplete(provider.baseUrl, provider.apiKey, body, (m) =>
          self.postMessage({ id, status: m }),
        );
      result = await streamFns[fn](spec, transport, (e) => self.postMessage({ id, event: e }));
    } else {
      result = fns[fn](...args);
    }
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
