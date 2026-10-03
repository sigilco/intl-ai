<script lang="ts">
  import { LOCALES } from "../locales";

  let {
    value = $bindable("en"),
    auto = false,
    detected = null,
  }: {
    value: string;
    auto?: boolean;
    detected?: string | null;
  } = $props();
</script>

<select
  class="select select-bordered select-sm w-44 font-mono"
  bind:value
  aria-label="locale"
>
  {#if auto}
    <option value="auto">auto{detected ? ` (${detected})` : ""}</option>
  {/if}
  {#each LOCALES as l (l.code)}
    <option value={l.code}>{l.label}</option>
  {/each}
</select>
