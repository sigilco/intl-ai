<script lang="ts">
  import { marked } from "marked";
  import { runDocPipeline } from "../docs";
  import { SAMPLE_DOC } from "../docsample";
  import { config } from "../config.svelte";
  import LocaleSelect from "./LocaleSelect.svelte";
  import type { DocExtractResult, GenRow, Phase } from "../types";
  import { call } from "../worker";

  let { wasmReady = false }: { wasmReady?: boolean } = $props();

  let running = $state(false);

  let docText = $state("");
  let targetLocale = $state("es");
  let localeInstruction = $state("");
  let rewriteLinks = $state(true);

  let rows = $state<GenRow[]>([]);
  let kinds = $state<Record<string, string>>({});
  let warnings = $state<string[]>([]);
  let merged = $state("");
  let fellBack = $state<string[]>([]);
  let phase = $state<Phase>("idle");
  let progress = $state({ done: 0, total: 0 });
  let statusLine = $state("");

  let dragging = $state(false);
  let copied = $state(false);
  let viewTab = $state<"preview" | "markdown" | "segments">("preview");
  let fileInput: HTMLInputElement;

  const pct = $derived(
    progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0,
  );

  const previewHtml = $derived.by(() => {
    if (!merged) return "";
    // Frontmatter is metadata, not body: keep it out of the rendered
    // preview (the markdown tab shows the verbatim merged document).
    const body = merged.replace(/^---\n[\s\S]*?\n---\n?/, "");
    return marked.parse(body, { async: false }) as string;
  });

  function accept(file: File | undefined) {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      docText = String(reader.result ?? "");
      statusLine = `loaded ${file.name}`;
    };
    reader.readAsText(file);
  }

  async function previewExtract() {
    const extracted = await call<DocExtractResult>("docExtract", docText, null);
    kinds = extracted.kinds;
    warnings = extracted.warnings;
    rows = extracted.order.map((seg) => ({
      key: seg,
      source: extracted.segments[seg],
      translation: "",
      status: "queued",
      findings: [],
    }));
  }

  async function translate() {
    if (running) return;
    running = true;
    rows = [];
    merged = "";
    fellBack = [];
    progress = { done: 0, total: 0 };
    phase = "extract";
    statusLine = "";
    try {
      const result = await runDocPipeline(
        {
          docText,
          sourceLocale: "en",
          targetLocale,
          localeInstruction: localeInstruction || null,
          linkPrefix: rewriteLinks ? `/${targetLocale}` : "",
          cfg: config,
        },
        {
          onPhase: (p, note) => {
            phase = p;
            statusLine = note;
          },
          onRows: (r) => (rows = r),
          onProgress: (done, total) => (progress = { done, total }),
        },
      );
      kinds = result.kinds;
      warnings = result.warnings;
      merged = result.merged;
      fellBack = result.fellBack;
      statusLine =
        `${result.rows.filter((r) => r.status === "filled").length} segment(s) translated` +
        (fellBack.length ? `, ${fellBack.length} kept source text` : "") +
        `; ${result.findings.length} check finding(s)`;
      viewTab = "preview";
    } catch (e) {
      phase = "error";
      statusLine = `error: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      running = false;
    }
  }

  async function copyMerged() {
    await navigator.clipboard.writeText(merged);
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }
</script>

<main
  class="grid min-h-0 flex-1 grid-cols-1 gap-3 px-4 pt-3 pb-2 lg:grid-cols-2"
>
  <section
    class="card flex min-h-0 flex-col border border-base-300 bg-base-200"
    class:border-primary={dragging}
  >
    <div class="flex items-center justify-between gap-2 px-4 pt-3">
      <h2 class="text-sm font-semibold tracking-wide text-base-content/70">
        SOURCE DOCUMENT
      </h2>
      <button
        class="btn btn-ghost btn-xs"
        onclick={() => fileInput.click()}
        title="load a markdown file"
      >
        open file
      </button>
      <input
        bind:this={fileInput}
        type="file"
        class="hidden"
        accept=".md,.mdx,.markdown"
        onchange={(e) => accept(e.currentTarget.files?.[0])}
      />
    </div>

    <textarea
      class="m-3 min-h-0 flex-1 resize-none rounded-field border border-base-300 bg-base-100 p-3 font-mono text-xs leading-relaxed outline-none focus:border-primary/60"
      placeholder="# Title&#10;&#10;Paste markdown/MDX, or drop a file here"
      spellcheck="false"
      bind:value={docText}
      ondragover={(e) => {
        e.preventDefault();
        dragging = true;
      }}
      ondragleave={() => (dragging = false)}
      ondrop={(e) => {
        e.preventDefault();
        dragging = false;
        accept(e.dataTransfer?.files?.[0]);
      }}
    ></textarea>

    <div class="flex items-center gap-2 px-3 pb-3">
      <button
        class="btn btn-primary btn-sm flex-1"
        disabled={running || !wasmReady || !docText.trim()}
        onclick={translate}
      >
        {#if running}
          <span class="loading loading-spinner loading-xs"></span> running
        {:else}
          translate document
        {/if}
      </button>
      <button
        class="btn btn-ghost btn-sm"
        disabled={running}
        onclick={() => {
          docText = SAMPLE_DOC;
          statusLine = "loaded bundled example";
        }}
      >
        load example
      </button>
    </div>
  </section>

  <section
    class="card flex min-h-0 flex-col border border-base-300 bg-base-200"
  >
    <div class="flex items-center justify-between gap-2 px-4 pt-3">
      <h2 class="text-sm font-semibold tracking-wide text-base-content/70">
        TRANSLATED DOCUMENT
      </h2>
      <div class="flex items-center gap-2">
        {#if rows.length}
          <div class="tabs tabs-xs tabs-border">
            <button
              class={`tab ${viewTab === "preview" ? "tab-active text-base-content" : "text-base-content/70"}`}
              onclick={() => (viewTab = "preview")}>preview</button
            >
            <button
              class={`tab ${viewTab === "markdown" ? "tab-active text-base-content" : "text-base-content/70"}`}
              onclick={() => (viewTab = "markdown")}>markdown</button
            >
            <button
              class={`tab ${viewTab === "segments" ? "tab-active text-base-content" : "text-base-content/70"}`}
              onclick={() => (viewTab = "segments")}>segments</button
            >
          </div>
        {/if}
        <LocaleSelect bind:value={targetLocale} />
      </div>
    </div>

    <details class="group mx-3 mt-2">
      <summary
        class="cursor-pointer font-mono text-[11px] text-base-content/70 select-none hover:text-base-content"
      >
        options
      </summary>
      <div class="mt-1 flex items-center gap-3">
        <label class="flex items-center gap-1.5 font-mono text-[11px]">
          <input
            type="checkbox"
            class="checkbox checkbox-xs"
            bind:checked={rewriteLinks}
          />
          rewrite / links to /{targetLocale}
        </label>
        <input
          class="input input-bordered input-xs flex-1 font-mono"
          placeholder="e.g. informal tone"
          bind:value={localeInstruction}
        />
        <button
          class="btn btn-ghost btn-xs"
          disabled={running || !wasmReady || !docText.trim()}
          onclick={previewExtract}
          title="extract segments without translating"
        >
          extract only
        </button>
      </div>
    </details>

    {#if running}
      <progress
        class="progress progress-primary mx-3 mt-2 w-auto"
        value={phase === "fill" ? progress.done : undefined}
        max={progress.total || 100}
        aria-label="pipeline progress"
      ></progress>
    {/if}

    <div class="m-3 min-h-0 flex-1 overflow-hidden rounded-field border border-base-300 bg-base-100">
      {#if rows.length === 0 && !merged}
        <div
          class="flex h-full flex-col items-center justify-center gap-1 text-center text-base-content/70"
        >
          <p class="text-sm">Nothing translated yet</p>
          <p class="text-xs">
            paste a document, set a provider, press translate
          </p>
        </div>
      {:else if viewTab === "preview"}
        {#if merged}
          <article
            class="prose prose-sm h-full max-w-none overflow-y-auto p-3 text-xs"
          >
            {@html previewHtml}
          </article>
        {:else}
          <div
            class="flex h-full items-center justify-center text-sm text-base-content/70"
          >
            merged document appears here
          </div>
        {/if}
      {:else if viewTab === "markdown"}
        <pre
          class="h-full overflow-y-auto p-3 font-mono text-xs leading-relaxed whitespace-pre-wrap">{merged}</pre>
      {:else}
        <div class="h-full overflow-y-auto">
          <table class="table table-xs table-pin-rows">
            <thead>
              <tr>
                <th>segment</th>
                <th>source</th>
                <th>translation</th>
              </tr>
            </thead>
            <tbody>
              {#each rows as row (row.key)}
                <tr>
                  <td class="align-top">
                    <div class="flex flex-col gap-0.5">
                      <span class="badge badge-ghost badge-xs font-mono">
                        {kinds[row.key] ?? "seg"}
                      </span>
                      <span
                        class={`font-mono text-[10px] ${
                          row.status === "filled"
                            ? "text-success"
                            : row.status === "failed"
                              ? "text-error"
                              : "text-base-content/50"
                        }`}
                      >
                        {row.status}
                      </span>
                    </div>
                  </td>
                  <td class="max-w-64 align-top whitespace-pre-wrap"
                    >{row.source}</td
                  >
                  <td class="max-w-64 align-top whitespace-pre-wrap">
                    {row.translation}
                    {#each row.findings as f (f.check + f.message)}
                      <div class="mt-0.5 font-mono text-[10px] text-warning">
                        {f.check}: {f.message}
                      </div>
                    {/each}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <div class="flex items-center gap-2 px-3 pb-3">
      <button
        class="btn btn-ghost btn-sm"
        disabled={!merged}
        onclick={copyMerged}
      >
        {copied ? "copied" : "copy"}
      </button>
      <span class="truncate font-mono text-xs text-base-content/70">
        {statusLine}{progress.total > 0 && phase === "fill" ? ` ${pct}%` : ""}
        {#each warnings as w (w)}
          <span class="text-warning" title={w}>warning</span>
        {/each}
      </span>
    </div>
  </section>
</main>
