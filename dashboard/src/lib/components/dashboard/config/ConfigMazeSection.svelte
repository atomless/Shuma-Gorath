<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';

  export let writable = false;
  export let mazeDirty = false;
  export let tarpitDirty = false;
  export let mazeEnabled = true;
  export let mazeAutoBan = true;
  export let mazeThreshold = 50;
  export let mazeThresholdValid = true;
  export let tarpitEnabled = true;
</script>

<ConfigPanel writable={writable} dirty={mazeDirty}>
  <ConfigPanelHeading title="Maze">
    <label class="toggle-switch" for="maze-enabled-toggle">
      <input type="checkbox" id="maze-enabled-toggle" aria-label="Enable maze" bind:checked={mazeEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Toggle whether identified bot traffic gets routed into the Maze. Set the threshold of Maze pages visited before the IP of the visitor is added to the ban list. Low thresholds increase potential of false positives. You may click here to preview the <a id="preview-maze-link" href="/admin/maze/preview" target="_blank" rel="noopener noreferrer">Maze</a> without risk of being banned (admin session required).</p>
  <div class="admin-controls">
    <div class="toggle-row">
      <label class="control-label" for="maze-auto-ban-toggle">Enable Auto-ban</label>
      <label class="toggle-switch">
        <input type="checkbox" id="maze-auto-ban-toggle" aria-label="Enable maze auto-ban" bind:checked={mazeAutoBan}>
        <span class="toggle-slider"></span>
      </label>
    </div>
    <div class="input-row" class:input-row--disabled={!mazeAutoBan} aria-disabled={!mazeAutoBan}>
      <label class="control-label" class:text-muted={!mazeAutoBan} for="maze-threshold">Auto-ban Threshold (maze pages visited)</label>
      <input class="input-field" type="number" id="maze-threshold" min="5" max="500" step="1" inputmode="numeric" aria-label="Maze ban threshold in pages" aria-invalid={mazeThresholdValid ? 'false' : 'true'} bind:value={mazeThreshold} disabled={!mazeAutoBan}>
    </div>
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={tarpitDirty}>
  <ConfigPanelHeading title="Tarpit">
    <label class="toggle-switch" for="tarpit-enabled-toggle">
      <input type="checkbox" id="tarpit-enabled-toggle" aria-label="Enable tarpit" bind:checked={tarpitEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Enable progression-gated tarpit defence for confirmed challenge attacks. Tarpit uses bounded work and deterministic fallback to keep host cost controlled while increasing attacker cost. You may click here to preview the <a id="preview-tarpit-link" href="/admin/tarpit/preview" target="_blank" rel="noopener noreferrer">Tarpit</a> without mutating runtime state (admin session required).</p>
  {#if !mazeEnabled}
    <p class="message warning">Maze is currently disabled, so tarpit cannot be served until Maze is enabled.</p>
  {/if}
</ConfigPanel>
