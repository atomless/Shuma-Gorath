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
  export let onFieldChange = null;

  const handleFieldChange = (field, value) => {
    if (typeof onFieldChange === 'function') {
      onFieldChange(field, value);
    }
  };
</script>

<ConfigPanel writable={writable} dirty={mazeDirty}>
  <ConfigPanelHeading title="Maze">
    <label class="toggle-switch" for="maze-enabled-toggle">
      <input
        type="checkbox"
        id="maze-enabled-toggle"
        aria-label="Enable maze"
        checked={mazeEnabled}
        on:change={(event) => handleFieldChange('mazeEnabled', event?.currentTarget?.checked === true)}
      >
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Toggle whether identified bot traffic gets routed into the Maze. Set the threshold of Maze pages visited before the IP of the visitor is added to the ban list. Low thresholds increase potential of false positives. You may click here to preview the <a id="preview-maze-link" href="/shuma/admin/maze/preview" target="_blank" rel="noopener noreferrer">Maze</a> without risk of being banned (admin session required).</p>
  <div class="admin-controls">
    <div class="toggle-row">
      <label class="control-label" for="maze-auto-ban-toggle">Enable Auto-ban</label>
      <label class="toggle-switch">
        <input
          type="checkbox"
          id="maze-auto-ban-toggle"
          aria-label="Enable maze auto-ban"
          checked={mazeAutoBan}
          on:change={(event) => handleFieldChange('mazeAutoBan', event?.currentTarget?.checked === true)}
        >
        <span class="toggle-slider"></span>
      </label>
    </div>
    <div class="input-row" class:input-row--disabled={!mazeAutoBan} aria-disabled={!mazeAutoBan}>
      <label class="control-label" class:text-muted={!mazeAutoBan} for="maze-threshold">Auto-ban Threshold (maze pages visited)</label>
      <input
        class="input-field"
        type="number"
        id="maze-threshold"
        min="5"
        max="500"
        step="1"
        inputmode="numeric"
        aria-label="Maze ban threshold in pages"
        aria-invalid={mazeThresholdValid ? 'false' : 'true'}
        value={mazeThreshold}
        disabled={!mazeAutoBan}
        on:input={(event) => handleFieldChange('mazeThreshold', event?.currentTarget?.value ?? '')}
      >
    </div>
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={tarpitDirty}>
  <ConfigPanelHeading title="Tarpit">
    <label class="toggle-switch" for="tarpit-enabled-toggle">
      <input
        type="checkbox"
        id="tarpit-enabled-toggle"
        aria-label="Enable tarpit"
        checked={tarpitEnabled}
        on:change={(event) => handleFieldChange('tarpitEnabled', event?.currentTarget?.checked === true)}
      >
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Enable progression-gated tarpit defence for confirmed challenge attacks. Tarpit uses bounded work and deterministic fallback to keep host cost controlled while increasing attacker cost. You may click here to preview the <a id="preview-tarpit-link" href="/shuma/admin/tarpit/preview" target="_blank" rel="noopener noreferrer">Tarpit</a> without mutating runtime state (admin session required).</p>
  {#if !mazeEnabled}
    <p class="message warning">Tarpit depends on Maze being enabled.</p>
  {/if}
</ConfigPanel>
