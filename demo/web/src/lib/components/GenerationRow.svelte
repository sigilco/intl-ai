<script lang="ts">
  import type { GenRow } from "../types";

  let {
    row,
    expanded = false,
    ontoggle,
  }: {
    row: GenRow;
    expanded?: boolean;
    ontoggle: () => void;
  } = $props();

  function chipClass(r: GenRow): string {
    if (r.findings.length) return "badge-error";
    if (r.judge) {
      if (r.judge.score >= 0.7) return "badge-success";
      if (r.judge.score >= 0.4) return "badge-warning";
      return "badge-error";
    }
    switch (r.status) {
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

  function chipText(r: GenRow): string {
    if (r.findings.length) return `${r.findings.length} finding(s)`;
    if (r.judge) return `judge ${r.judge.score.toFixed(2)}`;
    switch (r.status) {
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
</script>

<li>
  <button
    class="grid w-full grid-cols-[minmax(0,1fr)_auto] items-center gap-2 px-3 py-1.5 text-left hover:bg-base-200/60"
    onclick={ontoggle}
  >
    <span class="min-w-0">
      <span class="block truncate font-mono text-xs text-base-content/70">
        {row.key}
      </span>
      <span class="block truncate font-mono text-sm">
        {row.translation || " "}
      </span>
    </span>
    <span class="badge badge-sm font-mono {chipClass(row)}">
      {chipText(row)}
    </span>
  </button>
  {#if expanded}
    <div class="space-y-1 px-3 pb-2 text-xs">
      <p class="text-base-content/70">
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
        <p class="text-base-content/70">no findings</p>
      {/if}
    </div>
  {/if}
</li>
