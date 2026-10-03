<script lang="ts">
  import type { GenRow, Phase } from "../types";
  import GenerationRow from "./GenerationRow.svelte";
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
  let issuesOnly = $state(false);

  const pct = $derived(
    progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0,
  );

  const hasIssue = (r: GenRow) =>
    r.findings.length > 0 ||
    r.status === "failed" ||
    (r.judge !== undefined && r.judge.score < 0.7);

  const shownRows = $derived(
    issuesOnly ? rows.filter(hasIssue) : rows,
  );

  /** Raw output, pretty-printed when it parses as JSON. */
  const prettyOut = $derived.by(() => {
    if (!outText) return "";
    try {
      return JSON.stringify(JSON.parse(outText), null, 2);
    } catch {
      return outText;
    }
  });

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
        {#if viewTab === "rows"}
          <label class="flex cursor-pointer items-center gap-1.5 font-mono text-xs text-base-content/60">
            <input
              type="checkbox"
              class="toggle toggle-xs toggle-primary"
              bind:checked={issuesOnly}
            />
            issues only
          </label>
        {/if}
        <div class="tabs tabs-xs tabs-border">
          <button
            class={`tab ${viewTab === "rows" ? "tab-active text-base-content" : "text-base-content/60"}`}
            onclick={() => (viewTab = "rows")}>rows</button
          >
          <button
            class={`tab ${viewTab === "file" ? "tab-active text-base-content" : "text-base-content/60"}`}
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
        class="p-3 font-mono text-xs leading-relaxed whitespace-pre-wrap">{prettyOut}</pre>
    {:else}
      <ul class="divide-y divide-base-300/60">
        {#each shownRows as row (row.key)}
          <GenerationRow
            {row}
            expanded={expanded[row.key] === true}
            ontoggle={() => (expanded[row.key] = !expanded[row.key])}
          />
        {/each}
        {#if shownRows.length === 0}
          <li class="px-3 py-6 text-center text-xs text-base-content/60">
            no rows with issues
          </li>
        {/if}
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
