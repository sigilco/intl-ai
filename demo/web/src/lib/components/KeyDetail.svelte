<script lang="ts">
  import type { GenRow } from "../types";

  let { row }: { row: GenRow } = $props();
</script>

<div class="min-h-0 space-y-3 overflow-y-auto p-3 text-xs">
  <p class="font-mono font-semibold break-all">{row.key}</p>

  <div>
    <p class="mb-0.5 text-base-content/70">source</p>
    <p class="font-mono break-words whitespace-pre-wrap">{row.source}</p>
  </div>

  <div>
    <p class="mb-0.5 text-base-content/70">translation</p>
    {#if row.translation}
      <p class="font-mono break-words whitespace-pre-wrap">
        {row.translation}
      </p>
    {:else}
      <p class="text-base-content/70 italic">
        {row.status === "filling" ? "filling…" : row.status}
      </p>
    {/if}
  </div>

  {#if row.judge}
    <div>
      <p class="mb-0.5 text-base-content/70">judge</p>
      <p>
        <span
          class="badge badge-sm font-mono"
          class:badge-success={row.judge.score >= 0.7}
          class:badge-warning={row.judge.score >= 0.4 &&
            row.judge.score < 0.7}
          class:badge-error={row.judge.score < 0.4}
        >
          {row.judge.score.toFixed(2)}
        </span>
        {row.judge.reason || "no reason given"}
      </p>
      {#each row.judge.errors ?? [] as err (err)}
        <p class="mt-0.5 text-error">· {err}</p>
      {/each}
    </div>
  {/if}

  {#if row.findings.length}
    <div>
      <p class="mb-0.5 text-base-content/70">findings</p>
      {#each row.findings as f (f.check)}
        <p class="text-warning">
          <span class="font-semibold">{f.check}:</span>
          {f.message}
        </p>
      {/each}
    </div>
  {/if}

  {#if !row.judge && row.findings.length === 0}
    <p class="text-base-content/70">no findings</p>
  {/if}
</div>
