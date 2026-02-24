<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';

  export let writable = false;
  export let exportConfigDisabled = true;
  export let exportConfigStatus = '';
  export let exportConfigStatusKind = 'info';
  export let onExportCurrentConfigJson = null;

  const handleExport = (event) => {
    if (typeof onExportCurrentConfigJson === 'function') {
      void onExportCurrentConfigJson(event);
    }
  };
</script>

<ConfigPanel writable={writable} variant="export">
  <button
    id="export-current-config-json"
    class="btn btn-subtle"
    disabled={exportConfigDisabled}
    on:click={handleExport}
  >Export a JSON copy of the above configuration</button>
  {#if exportConfigStatus}
    <p id="export-current-config-status" class={`message ${exportConfigStatusKind}`}>{exportConfigStatus}</p>
  {/if}
</ConfigPanel>
