<script>
  export let id = '';
  export let controlsLabel = 'These controls';
  export let loading = false;
  export let hasConfigSnapshot = false;
  export let writable = false;
</script>

<p id={id || undefined} class="admin-group-subtitle text-muted">
  {#if loading}
    {controlsLabel} are loading.
  {:else if hasConfigSnapshot}
    {#if writable}
      {controlsLabel} are editable. Saved changes persist across builds.
      Set <code class="env-var">SHUMA_ADMIN_CONFIG_WRITE_ENABLED</code> to false in your deployment env to disable runtime configuration.
    {:else}
      {controlsLabel} are read-only.
      Set <code class="env-var">SHUMA_ADMIN_CONFIG_WRITE_ENABLED</code> to true in your deployment env to enable runtime configuration of these settings here.
    {/if}
  {:else}
    {controlsLabel} loaded, but the config snapshot is empty.
  {/if}
</p>
