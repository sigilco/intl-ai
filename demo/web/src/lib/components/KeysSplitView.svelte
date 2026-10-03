<script lang="ts">
  import type { GenRow } from "../types";
  import KeyDetail from "./KeyDetail.svelte";

  let { rows }: { rows: GenRow[] } = $props();

  let selectedKey = $state("");
  let userPicked = $state(false);
  let query = $state("");
  let issuesOnly = $state(false);

  const hasIssue = (r: GenRow) =>
    r.findings.length > 0 ||
    r.status === "failed" ||
    (r.judge !== undefined && r.judge.score < 0.7);

  const shownRows = $derived(
    rows.filter(
      (r) =>
        (!issuesOnly || hasIssue(r)) &&
        (!query || r.key.toLowerCase().includes(query.toLowerCase())),
    ),
  );

  const selected = $derived(rows.find((r) => r.key === selectedKey));

  // Keep selection valid as rows stream in: until the user picks a key,
  // prefer the first flagged one once issues attach; otherwise fall back
  // to the first visible row.
  $effect(() => {
    const firstIssue = shownRows.find(hasIssue);
    if (selected && shownRows.includes(selected)) {
      if (userPicked || !firstIssue || hasIssue(selected)) return;
      selectedKey = firstIssue.key;
      return;
    }
    const next = firstIssue ?? shownRows[0];
    selectedKey = next?.key ?? "";
    if (!next) userPicked = false;
  });

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
    if (r.findings.length) return `${r.findings.length} !`;
    if (r.judge) return r.judge.score.toFixed(2);
    switch (r.status) {
      case "existing":
        return "kept";
      case "filled":
        return "new";
      case "filling":
        return "…";
      case "failed":
        return "fail";
      default:
        return "…";
    }
  }
</script>

<div class="grid h-full min-h-0 sm:grid-cols-[11rem_minmax(0,1fr)]">
  <div
    class="flex max-h-40 min-h-0 flex-col border-b border-base-300 sm:max-h-none sm:border-e sm:border-b-0"
  >
    <div class="flex items-center gap-2 border-b border-base-300 px-2 py-1.5">
      <input
        class="input input-xs w-full font-mono"
        placeholder="filter keys"
        bind:value={query}
      />
      <label
        class="flex shrink-0 cursor-pointer items-center gap-1 font-mono text-[11px] text-base-content/70"
        title="only rows with findings, low scores or failures"
      >
        <input
          type="checkbox"
          class="toggle toggle-xs toggle-primary"
          bind:checked={issuesOnly}
        />
        issues
      </label>
    </div>
    <ul class="min-h-0 flex-1 divide-y divide-base-300/60 overflow-y-auto">
      {#each shownRows as row (row.key)}
        <li>
          <button
            class="flex w-full items-center justify-between gap-1 px-2 py-1 text-left hover:bg-base-200/60"
            class:bg-base-200={row.key === selectedKey}
            onclick={() => {
              selectedKey = row.key;
              userPicked = true;
            }}
          >
            <span class="truncate font-mono text-xs">{row.key}</span>
            <span class="badge badge-xs font-mono {chipClass(row)}">
              {chipText(row)}
            </span>
          </button>
        </li>
      {/each}
      {#if shownRows.length === 0}
        <li class="px-2 py-4 text-center text-[11px] text-base-content/70">
          no matching rows
        </li>
      {/if}
    </ul>
  </div>

  {#if selected}
    <KeyDetail row={selected} />
  {:else}
    <div
      class="flex items-center justify-center p-4 text-xs text-base-content/70"
    >
      select a key
    </div>
  {/if}
</div>
