<script lang="ts">
  import LocaleSelect from "./LocaleSelect.svelte";

  let {
    text = $bindable(""),
    locale = $bindable("auto"),
    detected = null,
    running = false,
    wasmReady = false,
    ongenerate,
    onfile,
  }: {
    text: string;
    locale: string;
    detected?: string | null;
    running?: boolean;
    wasmReady?: boolean;
    ongenerate: () => void;
    onfile: (name: string, content: string) => void;
  } = $props();

  let dragging = $state(false);
  let fileInput: HTMLInputElement;

  function accept(file: File | undefined) {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => onfile(file.name, String(reader.result ?? ""));
    reader.readAsText(file);
  }
</script>

<section
  class="card flex min-h-0 flex-col border border-base-300 bg-base-200"
  class:border-primary={dragging}
>
  <div class="flex items-center justify-between gap-2 px-4 pt-3">
    <h2 class="text-sm font-semibold tracking-wide text-base-content/70">
      SOURCE
    </h2>
    <div class="flex items-center gap-2">
      <LocaleSelect bind:value={locale} auto {detected} />
      <button
        class="btn btn-ghost btn-xs"
        onclick={() => fileInput.click()}
        title="load a locale file"
      >
        open file
      </button>
      <input
        bind:this={fileInput}
        type="file"
        class="hidden"
        accept=".json,.yaml,.yml"
        onchange={(e) => accept(e.currentTarget.files?.[0])}
      />
    </div>
  </div>

  <textarea
    class="m-3 min-h-0 flex-1 resize-none rounded-field border border-base-300 bg-base-100 p-3 font-mono text-xs leading-relaxed outline-none focus:border-primary/60"
    placeholder={'{"nav":{"home":"Home","about":"About"}} or drop a file here'}
    spellcheck="false"
    bind:value={text}
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
      disabled={running || !wasmReady || !text.trim()}
      onclick={ongenerate}
    >
      {#if running}
        <span class="loading loading-spinner loading-xs"></span> running
      {:else}
        generate
      {/if}
    </button>
    <button class="btn btn-ghost btn-sm" onclick={() => onfile("example", "")}>
      load example
    </button>
  </div>
</section>
