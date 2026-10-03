<script lang="ts">
  import type { GenRow, Phase } from "../types";
  import LocaleSelect from "./LocaleSelect.svelte";

  let {
    rows,
    targetLocale = $bindable("fr"),
    phase,
    progress,
    statusLine,
    outText,
    running = false,
    targetText = $bindable(""),
    localeInstruction = $bindable(""),
  }: {
    rows: GenRow[];
    targetLocale: string;
    phase: Phase;
    progress: { done: number; total: number };
    statusLine: string;
    outText: string;
    running?: boolean;
    targetText?: string;
    localeInstruction?: string;
  } = $props();

  let expanded = $state<Record<string, boolean>>({});
  let copied = $state(false);
  let viewTab = $state<"rows" | "file">("rows");

  const pct = $derived(
    progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0,
  );

  function chipClass(row: GenRow): string {
    if (row.findings.length) return "badge-error";
    if (row.judge) {
      if (row.judge.score >= 0.7) return "badge-success";
      if (row.judge.score >= 0.4) return "badge-warning";
      return "badge-error";
    }
    switch (row.status) {
      case "existing":
        return "badge-neutral";
      case "filled":
        return "badge-success badge-outline";
      case "filling":
        return "badge-info badge-outline";
      case "failed":
        return "badge-error";
      default:
        return "badge-ghost";
    }
  }

  function chipText(row: GenRow): string {
    if (row.findings.length) return `${row.findings.length} finding(s)`;
    if (row.judge) return `judge ${row.judge.score.toFixed(2)}`;
    switch (row.status) {
      case "existing":
        return "kept";
      case "filled":
        return "new";
      case "filling":
        return "…";
      case "failed":
        return "failed";
      default:
        return "queued";
    }
  }

  async function copy() {
    await navigator.clipboard.writeText(outText);
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }
</script>

<section
  class="card flex min-h-0 flex-col border border-base-300 bg-base-200"
>
  <div class="flex items-center justify-between gap-2 px-4 pt-3">
    <h2 class="text-sm font-semibold tracking-wide text-base-content/70">
      GENERATION
    </h2>
    <div class="flex items-center gap-2">
      {#if rows.length}
        <div class="tabs tabs-xs tabs-border">
          <button
            class="tab"
            class:tab-active={viewTab === "rows"}
            onclick={() => (viewTab = "rows")}>rows</button
          >
          <button
            class="tab"
            class:tab-active={viewTab === "file"}
            onclick={() => (viewTab = "file")}>file</button
          >
        </div>
      {/if}
      <LocaleSelect bind:value={targetLocale} />
    </div>
  </div>

  <details class="group mx-3 mt-2">
    <summary
      class="cursor-pointer font-mono text-[11px] text-base-content/60 select-none hover:text-base-content/70"
    >
      existing translations + instruction (optional)
    </summary>
    <div class="mt-1 grid grid-cols-[1fr_auto] gap-2">
      <textarea
        class="h-16 resize-none rounded-field border border-base-300 bg-base-100 p-2 font-mono text-xs outline-none focus:border-primary/60"
        placeholder={'{"nav":{"home":"Accueil"}} (kept as-is; missing keys filled)'}
        spellcheck="false"
        bind:value={targetText}
      ></textarea>
      <input
        class="input input-bordered input-xs h-16 w-44 font-mono"
        placeholder="e.g. informal tone"
        bind:value={localeInstruction}
      />
    </div>
  </details>

  {#if running || phase === "judge"}
    <progress
      class="progress progress-primary mx-3 mt-2 w-auto"
      value={phase === "judge" ? undefined : progress.done}
      max={progress.total || 100}
      aria-label="pipeline progress"
    ></progress>
  {/if}

  <div class="m-3 min-h-0 flex-1 overflow-y-auto rounded-field border border-base-300 bg-base-100">
    {#if rows.length === 0}
      <div
        class="flex h-full flex-col items-center justify-center gap-1 text-center text-base-content/60"
      >
        <p class="text-sm">Nothing generated yet</p>
        <p class="text-xs">
          paste a source locale, set a provider, press generate
        </p>
      </div>
    {:else if viewTab === "file"}
      <pre
        class="p-3 font-mono text-xs leading-relaxed whitespace-pre-wrap">{outText}</pre>
    {:else}
      <ul class="divide-y divide-base-300/60">
        {#each rows as row (row.key)}
          <li>
            <button
              class="grid w-full grid-cols-[minmax(0,1fr)_auto] items-center gap-2 px-3 py-1.5 text-left hover:bg-base-200/60"
              onclick={() =>
                (expanded[row.key] = !expanded[row.key])}
            >
              <span class="min-w-0">
                <span class="block truncate font-mono text-xs text-base-content/60"
                  >{row.key}</span
                >
                <span class="block truncate font-mono text-sm"
                  >{row.translation || " "}</span
                >
              </span>
              <span class="badge badge-sm font-mono {chipClass(row)}">
                {chipText(row)}
              </span>
            </button>
            {#if expanded[row.key]}
              <div class="space-y-1 px-3 pb-2 text-xs">
                <p class="text-base-content/60">
                  source: <span class="font-mono">{row.source}</span>
                </p>
                {#if row.judge}
                  <p>
                    <span class="font-semibold">judge:</span>
                    {row.judge.reason || "no reason given"}
                  </p>
                  {#each row.judge.errors ?? [] as err (err)}
                    <p class="text-error">· {err}</p>
                  {/each}
                {/if}
                {#each row.findings as f (f.check)}
                  <p class="text-warning">
                    <span class="font-semibold">{f.check}:</span>
                    {f.message}
                  </p>
                {/each}
                {#if !row.judge && row.findings.length === 0}
                  <p class="text-base-content/60">no findings</p>
                {/if}
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="flex items-center gap-2 px-3 pb-3">
    <button
      class="btn btn-ghost btn-sm"
      disabled={!outText}
      onclick={copy}
    >
      {copied ? "copied" : "copy"}
    </button>
    <span class="truncate font-mono text-xs text-base-content/60">
      {statusLine}{progress.total > 0 && phase === "fill" ? ` ${pct}%` : ""}
    </span>
  </div>
</section>
