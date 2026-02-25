<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';
  import TextareaField from '../primitives/TextareaField.svelte';

  export let writable = false;

  export let honeypotDirty = false;
  export let honeypotEnabled = true;
  export let honeypotPaths = '';
  export let honeypotPathsValid = true;
  export let honeypotInvalidMessage = '';

  export let browserPolicyDirty = false;
  export let browserPolicyEnabled = true;
  export let browserBlockRules = '';
  export let browserWhitelistRules = '';
  export let browserBlockRulesValid = true;
  export let browserWhitelistRulesValid = true;
</script>

<ConfigPanel writable={writable} dirty={honeypotDirty}>
  <ConfigPanelHeading title="Honeypot Paths">
    <label class="toggle-switch" for="honeypot-enabled-toggle">
      <input type="checkbox" id="honeypot-enabled-toggle" aria-label="Enable honeypot" bind:checked={honeypotEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">One path per line. Each path must start with <code>/</code>. Unencoded path characters are limited to letters, digits, <code>/ - . _ ~ ! $ &amp; ' ( ) * + , ; = : @</code>. Query (<code>?</code>) and fragment (<code>#</code>) are not allowed. Any other character must be percent-encoded as <code>%HH</code>.</p>
  <div class="admin-controls">
    <TextareaField id="honeypot-paths" label="Paths" rows="3" ariaLabel="Honeypot paths" spellcheck={false} ariaInvalid={honeypotPathsValid ? 'false' : 'true'} bind:value={honeypotPaths} />
    {#if !honeypotPathsValid && honeypotInvalidMessage}
      <p id="honeypot-paths-error" class="field-error visible">{honeypotInvalidMessage}</p>
    {/if}
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={browserPolicyDirty}>
  <ConfigPanelHeading title="Browser Policy">
    <label class="toggle-switch" for="browser-policy-toggle">
      <input type="checkbox" id="browser-policy-toggle" bind:checked={browserPolicyEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Use one rule per line in <code>BrowserName,min_major</code> format (for example <code>Chrome,120</code>).</p>
  <div class="admin-controls">
    <TextareaField id="browser-block-rules" label="Minimum Versions (Block)" rows="3" ariaLabel="Browser block minimum versions" spellcheck={false} ariaInvalid={browserBlockRulesValid ? 'false' : 'true'} bind:value={browserBlockRules} />
    <TextareaField id="browser-whitelist-rules" label="Allowlist Exceptions" rows="2" ariaLabel="Browser allowlist exceptions" spellcheck={false} ariaInvalid={browserWhitelistRulesValid ? 'false' : 'true'} bind:value={browserWhitelistRules} />
  </div>
</ConfigPanel>
