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

const $ = (id) => document.getElementById(id);
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

let targetFormat = "json";
let wasmReady = false;

function status(message, cls) {
  const el = $("progress");
  if (cls) {
    const span = document.createElement("span");
    span.className = cls;
    span.textContent = message;
    el.append(span, "\n");
  } else {
    el.append(message + "\n");
  }
}

function readFileText(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error);
    reader.readAsText(file);
  });
}

function wireDrop(dropId, inputId, textId, onExt) {
  const drop = $(dropId);
  const input = $(inputId);
  const text = $(textId);
  const accept = async (file) => {
    text.value = await readFileText(file);
    const ext = file.name.split(".").pop().toLowerCase();
    if (onExt) onExt(ext);
    status(`loaded ${file.name}`);
  };
  drop.addEventListener("click", () => input.click());
  input.addEventListener("change", () => input.files[0] && accept(input.files[0]));
  for (const evt of ["dragover", "dragenter"]) {
    drop.addEventListener(evt, (e) => {
      e.preventDefault();
      drop.classList.add("hover");
    });
  }
  drop.addEventListener("dragleave", () => drop.classList.remove("hover"));
  drop.addEventListener("drop", (e) => {
    e.preventDefault();
    drop.classList.remove("hover");
    if (e.dataTransfer.files[0]) accept(e.dataTransfer.files[0]);
  });
}

async function chatComplete(baseUrl, apiKey, body) {
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
        status(`HTTP ${res.status}, retrying in ${Math.round(waitMs / 1000)}s…`);
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

const SAMPLE_SOURCE = `{
  "nav": {
    "home": "Home",
    "settings": "Settings",
    "signOut": "Sign out"
  },
  "checkout": {
    "title": "Checkout",
    "items": "{count, plural, one {# item} other {# items}}",
    "total": "Total: {total, number, ::currency/EUR}"
  },
  "errors": {
    "offline": "You are offline. Changes will sync when you reconnect."
  }
}
`;

const SAMPLE_TARGET = `{
  "nav": {
    "home": "Accueil"
  }
}
`;

$("sampleBtn").addEventListener("click", () => {
  $("sourceText").value = SAMPLE_SOURCE;
  $("targetText").value = SAMPLE_TARGET;
  $("targetLocale").value = "fr";
  targetFormat = "json";
  status("loaded bundled example");
});

wireDrop("sourceDrop", "sourceFile", "sourceText", null);
wireDrop("targetDrop", "targetFile", "targetText", (ext) => {
  targetFormat = ext === "yaml" || ext === "yml" ? "yaml" : "json";
});

function checkIdsFor(targetLocale) {
  const ids = ["icu", "placeholder-parity"];
  if (/^en-(us|gb|uk)$/i.test(targetLocale)) {
    ids.push(`dialect:${targetLocale.toLowerCase()}`);
  }
  return ids;
}

async function run() {
  const progress = $("progress");
  progress.textContent = "";
  $("results").classList.add("hidden");
  $("download").classList.add("hidden");

  const sourceLocale = $("sourceLocale").value.trim() || "en";
  const targetLocale = $("targetLocale").value.trim() || "fr";
  const localeInstruction = $("localeInstruction").value.trim() || null;
  const baseUrl = $("baseUrl").value.trim();
  const model = $("model").value.trim();
  const apiKey = $("apiKey").value.trim();
  const batchSize = Math.max(1, Math.min(100, Number($("batchSize").value) || 20));

  const sourceText = $("sourceText").value.trim();
  if (!sourceText) throw new Error("paste or drop a source locale file first");

  status("flattening source…");
  const sourceFlat = flatten(sourceText);
  const targetText = $("targetText").value.trim();
  const targetFlat = targetText ? flatten(targetText) : null;
  const missing = missingKeys(sourceFlat, targetFlat);
  status(
    `${Object.keys(sourceFlat).length} source keys, ` +
      `${missing.length} missing in ${targetLocale}`,
  );

  const filled = {};
  for (let i = 0; i < missing.length; i += batchSize) {
    const batch = missing.slice(i, i + batchSize);
    const entries = batch.map((key) => ({ key, source: sourceFlat[key] }));
    const body = buildTranslateBody({
      sourceLocale,
      targetLocale,
      entries,
      localeInstruction,
      model,
      modelParams: {},
    });
    status(`translating keys ${i + 1} to ${i + batch.length} via ${baseUrl}…`);
    const content = await chatComplete(baseUrl, apiKey, body);
    Object.assign(filled, parseTranslations(content));
    status(`translated ${Math.min(i + batch.length, missing.length)}/${missing.length}`);
  }

  const outText = unflatten(targetText || null, filled, targetFormat);
  const finalFlat = targetFlat ? { ...targetFlat, ...filled } : { ...filled };

  const ctx = { sourceLocale, targetLocale, localeInstruction };
  const items = Object.entries(finalFlat).map(([key, target]) => ({
    key,
    source: sourceFlat[key] ?? null,
    target,
  }));
  const ids = checkIdsFor(targetLocale);
  status(`running checks: ${ids.join(", ")}`);
  const findings = runChecks(items, ids, ctx);

  let judgements = null;
  if ($("runJudge").checked && Object.keys(filled).length > 0) {
    const judgeItems = Object.entries(filled).map(([key, translation]) => ({
      key,
      locale: targetLocale,
      source: sourceFlat[key] ?? "",
      translation,
    }));
    const body = buildJudgeBody({
      items: judgeItems,
      localeInstruction,
      model,
      modelParams: {},
    });
    status("judging translations…");
    const content = await chatComplete(baseUrl, apiKey, body);
    judgements = parseJudgements(content);
  }

  render({ filled, findings, judgements, outText, targetLocale });
}

function render({ filled, findings, judgements, outText, targetLocale }) {
  const results = $("results");
  results.classList.remove("hidden");
  $("summary").textContent =
    `${Object.keys(filled).length} keys filled for ${targetLocale}; ` +
    `${findings.length} check finding(s).`;

  const wrap = $("findingsWrap");
  const tbody = $("findingsBody");
  tbody.textContent = "";
  if (findings.length > 0) {
    wrap.classList.remove("hidden");
    for (const f of findings) {
      const tr = document.createElement("tr");
      for (const cell of [f.key, f.check, f.message]) {
        const td = document.createElement("td");
        td.textContent = cell;
        tr.append(td);
      }
      tbody.append(tr);
    }
  } else {
    wrap.classList.add("hidden");
  }

  const jwrap = $("judgeWrap");
  const jbody = $("judgeBody");
  jbody.textContent = "";
  if (judgements && judgements.length > 0) {
    jwrap.classList.remove("hidden");
    for (const j of judgements) {
      const tr = document.createElement("tr");
      const reason = [j.reason, (j.errors || []).join(", ")].filter(Boolean).join("; ");
      for (const cell of [j.key, j.score.toFixed(2), reason]) {
        const td = document.createElement("td");
        td.textContent = cell;
        tr.append(td);
      }
      jbody.append(tr);
    }
  } else {
    jwrap.classList.add("hidden");
  }

  $("output").textContent = outText;
  const blob = new Blob([outText], { type: "text/plain" });
  const a = $("download");
  a.href = URL.createObjectURL(blob);
  a.download = `${targetLocale}.${targetFormat === "yaml" ? "yaml" : "json"}`;
  a.classList.remove("hidden");
  status("done", "ok");
}

$("runBtn").addEventListener("click", () => {
  $("runBtn").disabled = true;
  run()
    .catch((e) => status(`error: ${e.message}`, "err"))
    .finally(() => {
      $("runBtn").disabled = !wasmReady;
    });
});

init()
  .then(() => {
    wasmReady = true;
    $("wasmStatus").textContent = "wasm module ready";
    $("runBtn").disabled = false;
  })
  .catch((e) => {
    $("wasmStatus").textContent = `failed to load wasm: ${e.message}`;
  });
