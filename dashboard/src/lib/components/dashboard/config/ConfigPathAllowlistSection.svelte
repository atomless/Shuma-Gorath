<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';
  import TextareaField from '../primitives/TextareaField.svelte';

  export let writable = false;
  export let pathAllowlistDirty = false;
  export let pathAllowlistEnabled = true;
  export let pathAllowlist = '';
  export let onFieldChange = null;

  const handleFieldChange = (field, value) => {
    if (typeof onFieldChange === 'function') {
      onFieldChange(field, value);
    }
  };
</script>

<ConfigPanel writable={writable} dirty={pathAllowlistDirty}>
  <ConfigPanelHeading title="Path Allowlist">
    <label class="toggle-switch" for="path-allowlist-enabled-toggle">
      <input
        type="checkbox"
        id="path-allowlist-enabled-toggle"
        aria-label="Enable path allowlist bypass"
        checked={pathAllowlistEnabled}
        disabled={!writable}
        on:change={(event) => handleFieldChange('pathAllowlistEnabled', event?.currentTarget?.checked === true)}
      >
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">
    Use this list to bypass bot defenses for trusted machine paths such as payment webhooks and partner callbacks.
  </p>
  <p class="control-desc text-muted">
    Rules: each entry must be a path, not a full URL. Exact paths must match exactly (example: <code>/webhook/stripe</code>). Prefix rules must end with <code>*</code> (example: <code>/api/integrations/*</code>).
  </p>
  <p class="control-desc text-muted">
    You must not enter hostnames, query strings, or fragments (for example, do not enter <code>https://example.com/hook</code>, <code>/hook?token=1</code>, or <code>/hook#frag</code>).
  </p>
  {#if !pathAllowlistEnabled}
    <p class="message warning">
      Path allowlist bypass is disabled. Existing entries are preserved and will be applied again when you re-enable this toggle.
    </p>
  {/if}
  <TextareaField
    id="path-allowlist"
    label="Path Allowlist"
    rows="5"
    ariaLabel="Path allowlist"
    spellcheck={false}
    disabled={!writable || !pathAllowlistEnabled}
    value={pathAllowlist}
    onInput={(event) => handleFieldChange('pathAllowlist', event?.currentTarget?.value ?? '')}
  />
</ConfigPanel>
