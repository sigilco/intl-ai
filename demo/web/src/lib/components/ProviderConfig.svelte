<script lang="ts">
  import { onMount } from "svelte";
  import {
    PRESETS,
    applyPreset,
    config,
    fetchModels,
    saveConfig,
    testProvider,
    type TestState,
  } from "../config.svelte";

  let open = $state(false);
  let models = $state<string[]>([]);
  let testing = $state<TestState>({ kind: "idle", detail: "" });
  let modelsBusy = $state(false);

  const presetEntries = Object.entries(PRESETS) as [
    keyof typeof PRESETS,
    (typeof PRESETS)[keyof typeof PRESETS],
  ][];

  let modelsTimer: ReturnType<typeof setTimeout> | undefined;

  async function refreshModels() {
    modelsBusy = true;
    try {
      models = await fetchModels();
    } finally {
      modelsBusy = false;
    }
  }

  /** Debounced combobox load: typing a URL or key refetches /models. */
  function queueModels() {
    clearTimeout(modelsTimer);
    modelsTimer = setTimeout(() => void refreshModels(), 500);
  }

  async function test() {
    testing = { kind: "busy", detail: "testing…" };
    testing = await testProvider();
    if (testing.kind === "ok") await refreshModels();
  }

  function persist() {
    saveConfig();
  }

  onMount(() => {
    void refreshModels();
  });
</script>

<details
  class="collapse collapse-arrow rounded-box border border-base-300 bg-base-200"
  bind:open
>
  <summary class="collapse-title flex min-h-0 items-center gap-3 py-2 pe-10">
    <span class="badge badge-sm badge-primary font-mono">provider</span>
    <span class="text-sm font-medium">{PRESETS[config.preset].label}</span>
    <span class="truncate font-mono text-xs text-base-content/70">
      {config.baseUrl}
    </span>
    {#if testing.kind === "ok"}
      <span class="badge badge-xs badge-success">ok</span>
    {:else if testing.kind === "fail"}
      <span class="badge badge-xs badge-error">fail</span>
    {/if}
  </summary>

  <div class="collapse-content">
    <div class="grid grid-cols-2 gap-x-4 gap-y-3 pb-1 md:grid-cols-4">
      <label class="form-control">
        <span class="label-text mb-1 text-xs">Preset</span>
        <select
          class="select select-bordered select-sm"
          value={config.preset}
          onchange={(e) => {
            applyPreset(e.currentTarget.value as keyof typeof PRESETS);
            queueModels();
          }}
        >
          {#each presetEntries as [id, p] (id)}
            <option value={id}>{p.label}</option>
          {/each}
        </select>
      </label>

      <label class="form-control">
        <span class="label-text mb-1 text-xs">Base URL</span>
        <input
          class="input input-bordered input-sm font-mono"
          bind:value={config.baseUrl}
          onchange={() => {
            persist();
            queueModels();
          }}
          placeholder="https://…/v1"
        />
      </label>

      <label class="form-control">
        <span class="label-text mb-1 text-xs">
          API key{PRESETS[config.preset].needsKey ? "" : " (optional)"}
        </span>
        <input
          class="input input-bordered input-sm font-mono"
          type="password"
          bind:value={config.apiKey}
          onchange={() => {
            persist();
            queueModels();
          }}
          placeholder="sk-…"
          autocomplete="off"
        />
      </label>

      <label class="form-control">
        <span class="label-text mb-1 text-xs">
          Model{modelsBusy ? " · loading…" : ""}
        </span>
        <input
          class="input input-bordered input-sm font-mono"
          bind:value={config.model}
          onchange={persist}
          list="provider-models"
          placeholder="model id"
        />
        <datalist id="provider-models">
          {#each models as id (id)}
            <option value={id}></option>
          {/each}
        </datalist>
      </label>

      <label class="form-control">
        <span class="label-text mb-1 text-xs">Batch size</span>
        <input
          class="input input-bordered input-sm font-mono"
          type="number"
          min="1"
          max="100"
          bind:value={config.batchSize}
          onchange={persist}
        />
      </label>

      <label class="form-control">
        <span class="label-text mb-1 text-xs">Stream fill</span>
        <input
          type="checkbox"
          class="toggle toggle-sm toggle-primary"
          bind:checked={config.stream}
          onchange={persist}
        />
      </label>

      <label class="form-control">
        <span class="label-text mb-1 text-xs">Quality judge</span>
        <input
          type="checkbox"
          class="toggle toggle-sm toggle-primary"
          bind:checked={config.runJudge}
          onchange={persist}
        />
      </label>

      <div class="form-control">
        <span class="label-text mb-1 text-xs opacity-0" aria-hidden="true">
          Test
        </span>
        <div class="flex items-center gap-2">
          <button
            class="btn btn-primary btn-sm"
            onclick={test}
            disabled={testing.kind === "busy"}
          >
            {testing.kind === "busy" ? "testing…" : "test"}
          </button>
          {#if testing.detail}
            <span
              class="truncate text-xs"
              class:text-success={testing.kind === "ok"}
              class:text-error={testing.kind === "fail"}
            >
              {testing.detail}
            </span>
          {/if}
        </div>
      </div>
    </div>

    {#if config.preset === "illo"}
      <p class="mt-1 text-xs text-base-content/70">
        api.illo.fyi only answers CORS from *.illo.fyi origins — local runs
        should point Base URL elsewhere (see demo/wasm/mock-provider.mjs).
      </p>
    {/if}
  </div>
</details>
