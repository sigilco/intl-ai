// All wasm calls run inside a Web Worker so the UI stays responsive
// while the pipeline parses, fetches, and checks. The streaming drivers
// (runFill/runCheck) own the batch loop in wasm and post every core
// ProgressEvent back to the page as it fires.
const worker = new Worker("./worker.mjs", { type: "module" });

const $ = (id) => document.getElementById(id);

let targetFormat = "json";
let wasmReady = false;

let nextCallId = 0;
const pendingCalls = new Map();
worker.onmessage = (event) => {
  const { id, ready, ok, result, error, event: ev, status: note } = event.data;
  if (ready !== undefined) {
    if (ready) {
      wasmReady = true;
      $("wasmStatus").textContent = "wasm module ready (worker)";
      $("runBtn").disabled = false;
    } else {
      $("wasmStatus").textContent = `failed to load wasm: ${error}`;
    }
    return;
  }
  const pending = pendingCalls.get(id);
  if (!pending) return;
  if (ev !== undefined) {
    pending.onEvent?.(ev);
    return;
  }
  if (note !== undefined) {
    status(note);
    return;
  }
  pendingCalls.delete(id);
  if (ok) pending.resolve(result);
  else pending.reject(new Error(error));
};

const call = (fn, ...args) =>
  new Promise((resolve, reject) => {
    nextCallId += 1;
    pendingCalls.set(nextCallId, { resolve, reject });
    worker.postMessage({ id: nextCallId, fn, args });
  });

// runFill/runCheck: (spec, provider, onEvent) — provider is read by the
// worker's fetch loop, onEvent receives each ProgressEvent object.
const callStream = (fn, spec, provider, onEvent) =>
  new Promise((resolve, reject) => {
    nextCallId += 1;
    pendingCalls.set(nextCallId, { resolve, reject, onEvent });
    worker.postMessage({ id: nextCallId, fn, args: [spec, provider] });
  });

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

// key -> row element for the run in flight, so translations can be
// filled in once the report arrives.
const keyRows = new Map();

function addKeyRow(e) {
  $("keysWrap").classList.remove("hidden");
  const tr = document.createElement("tr");
  tr.dataset.key = e.key;
  const written = e.outcome.status === "written";
  const cells = [e.key, written ? "filled" : `failed: ${e.outcome.message}`, ""];
  for (const cell of cells) {
    const td = document.createElement("td");
    td.textContent = cell;
    tr.append(td);
  }
  tr.cells[1].className = written ? "ok" : "err";
  keyRows.set(e.key, tr);
  $("keysBody").append(tr);
}

function addFindingRow(f) {
  $("findingsWrap").classList.remove("hidden");
  const tr = document.createElement("tr");
  for (const cell of [f.key, f.check || f.kind, f.message]) {
    const td = document.createElement("td");
    td.textContent = cell;
    tr.append(td);
  }
  $("findingsBody").append(tr);
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
  $("progress").textContent = "";
  keyRows.clear();
  $("results").classList.add("hidden");
  $("keysWrap").classList.add("hidden");
  $("findingsWrap").classList.add("hidden");
  $("judgeWrap").classList.add("hidden");
  $("keysBody").textContent = "";
  $("findingsBody").textContent = "";
  $("judgeBody").textContent = "";
  $("summary").textContent = "";
  $("output").textContent = "";
  $("download").classList.add("hidden");
  $("runMeta").classList.add("hidden");

  const sourceLocale = $("sourceLocale").value.trim() || "en";
  const targetLocale = $("targetLocale").value.trim() || "fr";
  const localeInstruction = $("localeInstruction").value.trim() || null;
  const baseUrl = $("baseUrl").value.trim();
  const model = $("model").value.trim();
  const apiKey = $("apiKey").value.trim();
  const batchSize = Math.max(1, Math.min(100, Number($("batchSize").value) || 20));

  const sourceText = $("sourceText").value.trim();
  if (!sourceText) throw new Error("paste or drop a source locale file first");
  const targetText = $("targetText").value.trim();

  status("flattening source…");
  const sourceFlat = await call("flatten", sourceText);
  const targetFlat = targetText ? await call("flatten", targetText) : null;
  const missing = await call("missingKeys", sourceFlat, targetFlat);
  status(
    `${Object.keys(sourceFlat).length} source keys, ` +
      `${missing.length} missing in ${targetLocale}`,
  );

  // Results render live: the card opens now, rows land as events arrive.
  $("results").classList.remove("hidden");
  $("runMeta").classList.remove("hidden");
  const bar = $("fillBar");
  bar.max = Math.max(missing.length, 1);
  bar.value = 0;
  const localeStats = $("localeStats");
  let pipeline = "";
  const onEvent = (e) => {
    switch (e.type) {
      case "run_started":
        pipeline = e.pipeline;
        localeStats.textContent =
          e.pipeline === "fill"
            ? `${e.locales[0]}: 0/${missing.length} keys`
            : `${e.locales[0]}: checking…`;
        status(`${e.pipeline}: started for ${e.locales.join(", ")}`);
        break;
      case "batch_started":
        status(
          `${pipeline}: ${pipeline === "check" ? "judging" : "translating"} ${e.keys} key(s)…`,
        );
        break;
      case "batch_finished":
        status(`batch done: ${e.answered} answered, ${e.failed} failed`);
        break;
      case "key_done":
        addKeyRow(e);
        bar.value += 1;
        localeStats.textContent = `${e.locale}: ${bar.value}/${missing.length} keys`;
        break;
      case "finding":
        addFindingRow(e);
        break;
      case "locale_finished":
        status(`${e.pipeline}: ${e.locale} finished`);
        break;
      case "run_finished":
        localeStats.textContent = `${e.pipeline} done`;
        status(`${e.pipeline}: done, ${e.failures} failure(s)`, e.failures ? "err" : "ok");
        break;
    }
  };

  const provider = { baseUrl, apiKey };
  const fill = await callStream(
    "runFill",
    {
      source: sourceText,
      target: targetText || null,
      targetFormat,
      sourceLocale,
      targetLocale,
      localeInstruction,
      model,
      modelParams: {},
      batchSize,
    },
    provider,
    onEvent,
  );
  const { filled, output: outText, failures } = fill;

  const check = await callStream(
    "runCheck",
    {
      source: sourceText,
      target: outText,
      sourceLocale,
      targetLocale,
      localeInstruction,
      checkIds: checkIdsFor(targetLocale),
      judgeItems:
        $("runJudge").checked && Object.keys(filled).length > 0
          ? Object.entries(filled).map(([key, translation]) => ({
              key,
              locale: targetLocale,
              source: sourceFlat[key] ?? "",
              translation,
            }))
          : [],
      model,
      modelParams: {},
    },
    provider,
    onEvent,
  );

  render({
    filled,
    failures,
    findings: check.findings,
    judgements: check.judgements,
    errors: check.errors,
    outText,
    targetLocale,
  });
}

function render({ filled, failures, findings, judgements, errors, outText, targetLocale }) {
  // Fill in the translation cells of the streamed key rows.
  for (const [key, tr] of keyRows) {
    tr.cells[2].textContent = filled[key] ?? "";
  }
  $("summary").textContent =
    `${Object.keys(filled).length} keys filled for ${targetLocale}, ` +
    `${failures.length} failed; ${findings.length} check finding(s).`;

  // Rows already arrived live via finding events; rebuild from the
  // report so the final table is authoritative.
  const wrap = $("findingsWrap");
  const tbody = $("findingsBody");
  tbody.textContent = "";
  if (findings.length > 0) {
    wrap.classList.remove("hidden");
    for (const f of findings) addFindingRow(f);
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

  for (const e of errors || []) {
    status(`check error: ${e}`, "err");
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

worker.onerror = (e) => {
  $("wasmStatus").textContent = `wasm worker failed: ${e.message || e.type}`;
};
