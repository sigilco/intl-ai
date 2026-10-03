<script lang="ts">
  import { onMount } from "svelte";
  import GenerationCard from "./lib/components/GenerationCard.svelte";
  import ProviderConfig from "./lib/components/ProviderConfig.svelte";
  import SourceCard from "./lib/components/SourceCard.svelte";
  import { config } from "./lib/config.svelte";
  import { detectLocale } from "./lib/locales";
  import { runPipeline } from "./lib/pipeline";
  import { SAMPLE_SOURCE, SAMPLE_TARGET } from "./lib/sample";
  import { initTheme, theme, toggleTheme } from "./lib/theme.svelte";
  import { startWorker } from "./lib/worker";
  import type { GenRow, Phase } from "./lib/types";

  let wasmStatus = $state("loading wasm…");
  let wasmOk = $state(false);
  let running = $state(false);

  let sourceText = $state("");
  let targetText = $state("");
  let sourceLocale = $state("auto");
  let detectedSource = $state<string | null>(null);
  let targetLocale = $state("fr");
  let localeInstruction = $state("");
  let targetFormat = $state("json");

  let rows = $state<GenRow[]>([]);
  let outText = $state("");
  let phase = $state<Phase>("idle");
  let progress = $state({ done: 0, total: 0 });
  let statusLine = $state("");

  onMount(() => {
    initTheme();
    startWorker((err) => {
      wasmOk = !err;
      wasmStatus = err ? `wasm failed: ${err}` : "wasm ready";
    });
  });

  function onSourceFile(name: string, content: string) {
    if (name === "example") {
      sourceText = SAMPLE_SOURCE;
      targetText = SAMPLE_TARGET;
      targetLocale = "fr";
      targetFormat = "json";
      statusLine = "loaded bundled example";
      return;
    }
    sourceText = content;
    detectedSource = detectLocale(name);
    statusLine = `loaded ${name}`;
  }

  async function generate() {
    if (running) return;
    running = true;
    rows = [];
    outText = "";
    progress = { done: 0, total: 0 };
    phase = "flatten";
    statusLine = "";

    const src = sourceLocale === "auto" ? (detectedSource ?? "en") : sourceLocale;
    const targetTrimmed = targetText.trim();
    const format = targetTrimmed.startsWith("{") ? "json" : targetTrimmed ? "yaml" : targetFormat;
    try {
      const result = await runPipeline(
        {
          sourceText,
          targetText,
          sourceLocale: src,
          targetLocale,
          localeInstruction: localeInstruction || null,
          targetFormat: format,
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
      outText = result.outText;
      const filled = result.rows.filter((r) => r.status === "filled").length;
      statusLine =
        `${filled} key(s) filled for ${targetLocale}; ` +
        `${result.findings.length} check finding(s)`;
    } catch (e) {
      phase = "error";
      statusLine = `error: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      running = false;
    }
  }
</script>

<div class="flex h-dvh flex-col bg-base-100 text-base-content">
  <header class="flex items-baseline gap-3 px-4 pt-3">
    <h1 class="text-base font-semibold tracking-tight">intl-ai studio</h1>
    <span class="font-mono text-xs text-base-content/70">
      wasm pipeline · web demo
    </span>
    <span
      class="ms-auto font-mono text-xs"
      class:text-success={wasmOk}
      class:text-error={!wasmOk && !wasmStatus.startsWith("loading")}
    >
      {wasmStatus}
    </span>
    <button
      class="btn btn-ghost btn-xs btn-circle"
      title={theme.mode === "dark" ? "switch to light" : "switch to dark"}
      aria-label="toggle theme"
      onclick={toggleTheme}
    >
      {#if theme.mode === "dark"}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="h-4 w-4"
        >
          <circle cx="12" cy="12" r="4" />
          <path
            d="M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"
          />
        </svg>
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="h-4 w-4"
        >
          <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z" />
        </svg>
      {/if}
    </button>
  </header>

  <section aria-label="Provider configuration" class="px-4 pt-2">
    <ProviderConfig />
  </section>

  <main
    class="grid min-h-0 flex-1 grid-cols-1 gap-3 px-4 pt-3 pb-2 lg:grid-cols-2"
  >
    <SourceCard
      bind:text={sourceText}
      bind:locale={sourceLocale}
      detected={detectedSource}
      {running}
      wasmReady={wasmOk}
      ongenerate={generate}
      onfile={onSourceFile}
    />
    <GenerationCard
      {rows}
      bind:targetLocale
      bind:targetText
      bind:localeInstruction
      {phase}
      {progress}
      {statusLine}
      {outText}
      {running}
    />
  </main>

  <footer
    class="flex items-center border-t border-base-300 px-4 py-1.5 font-mono text-[11px] text-base-content/70"
  >
    <span>intl-ai · crates/intl-ai-wasm · {phase}</span>
  </footer>
</div>
