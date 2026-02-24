<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';
  import NumericInputRow from '../primitives/NumericInputRow.svelte';
  import TextareaField from '../primitives/TextareaField.svelte';
  import ToggleRow from '../primitives/ToggleRow.svelte';

  export let writable = false;

  export let honeypotDirty = false;
  export let honeypotEnabled = true;
  export let honeypotPaths = '';

  export let ipRangeDirty = false;
  export let ipRangePolicyMode = 'off';
  export let ipRangeEmergencyAllowlist = '';
  export let ipRangeCustomRulesJson = '[]';
  export let ipRangeManagedPoliciesJson = '[]';
  export let ipRangeManagedMaxStalenessHours = 168;
  export let ipRangeAllowStaleManagedEnforce = false;
  export let ipRangeManagedStalenessMin = 1;
  export let ipRangeManagedStalenessMax = 2160;
  export let ipRangeManagedSetRows = [];
  export let ipRangeCatalogVersion = '-';
  export let ipRangeCatalogGeneratedAt = '-';
  export let ipRangeCatalogAgeHours = null;
  export let ipRangeManagedSetStaleCount = 0;
  export let ipRangeCatalogStale = false;

  export let browserPolicyDirty = false;
  export let browserPolicyEnabled = true;
  export let browserBlockRules = '';
  export let browserWhitelistRules = '';

  export let whitelistDirty = false;
  export let bypassAllowlistsEnabled = true;
  export let networkWhitelist = '';
  export let pathWhitelist = '';

  export let edgeModeDirty = false;
  export let edgeIntegrationMode = 'off';
</script>

<ConfigPanel writable={writable} dirty={honeypotDirty}>
  <ConfigPanelHeading title="Honeypot Paths">
    <label class="toggle-switch" for="honeypot-enabled-toggle">
      <input type="checkbox" id="honeypot-enabled-toggle" aria-label="Enable honeypot" bind:checked={honeypotEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">One path per line. Requests that hit these paths are treated as high-confidence bot behavior. Paths must start with <code>/</code>.</p>
  <div class="admin-controls">
    <TextareaField id="honeypot-paths" label="Paths" rows="3" ariaLabel="Honeypot paths" spellcheck={false} bind:value={honeypotPaths} />
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={ipRangeDirty}>
  <ConfigPanelHeading title='<abbr title="Internet Protocol">IP</abbr> Range Policy'>
    <select class="input-field panel-heading-select" id="ip-range-policy-mode" aria-label="Internet Protocol range policy mode" bind:value={ipRangePolicyMode}>
      <option value="off">off</option>
      <option value="advisory">advisory</option>
      <option value="enforce">enforce</option>
    </select>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">
    Configure <abbr title="Classless Inter-Domain Routing">CIDR</abbr> policy mode, emergency allowlist, custom rules, managed set policies, and managed-catalog staleness safeguards.
  </p>
  <div class="admin-controls">
    <TextareaField id="ip-range-emergency-allowlist" label='Emergency Allowlist <abbr title="Classless Inter-Domain Routing">CIDRs</abbr>' rows="3" ariaLabel="Internet Protocol range emergency allowlist" spellcheck={false} bind:value={ipRangeEmergencyAllowlist} />
    <TextareaField id="ip-range-custom-rules-json" label='Custom Rules <abbr title="JavaScript Object Notation">JSON</abbr>' rows="8" ariaLabel="Internet Protocol range custom rules JavaScript Object Notation" spellcheck={false} textareaClass="input-field geo-textarea input-field--mono" bind:value={ipRangeCustomRulesJson} />
    <TextareaField id="ip-range-managed-policies-json" label='Managed Policies <abbr title="JavaScript Object Notation">JSON</abbr>' rows="6" ariaLabel="Internet Protocol range managed policies JavaScript Object Notation" spellcheck={false} textareaClass="input-field geo-textarea input-field--mono" bind:value={ipRangeManagedPoliciesJson} />
    <NumericInputRow id="ip-range-managed-max-staleness" label="Managed Max Staleness (hours)" labelClass="control-label control-label--wide" min={ipRangeManagedStalenessMin} max={ipRangeManagedStalenessMax} step="1" inputmode="numeric" ariaLabel="Internet Protocol range managed max staleness hours" bind:value={ipRangeManagedMaxStalenessHours} />
    <ToggleRow id="ip-range-allow-stale-enforce" label="Allow stale managed enforce" labelClass="control-label control-label--wide" ariaLabel="Allow stale managed enforce" bind:checked={ipRangeAllowStaleManagedEnforce} />
  </div>
  <div class="info-panel">
    <h4>Managed Catalog Snapshot</h4>
    <div class="info-row">
      <span class="info-label text-muted">Version</span>
      <span><code>{ipRangeCatalogVersion}</code></span>
    </div>
    <div class="info-row">
      <span class="info-label text-muted">Generated At</span>
      <span>{ipRangeCatalogGeneratedAt}</span>
    </div>
    <div class="info-row">
      <span class="info-label text-muted">Catalog Age</span>
      <span>
        {#if Number.isFinite(ipRangeCatalogAgeHours)}
          {ipRangeCatalogAgeHours}h
        {:else}
          -
        {/if}
      </span>
    </div>
    <div class="info-row">
      <span class="info-label text-muted">Managed Sets (stale)</span>
      <span>{ipRangeManagedSetRows.length} ({ipRangeManagedSetStaleCount})</span>
    </div>
  </div>
  {#if ipRangeCatalogStale}
    <p class="message warning">Managed catalog is stale under current max staleness policy.</p>
  {/if}
  {#if ipRangeManagedSetRows.length > 0}
    <div class="table-wrapper">
      <table id="ip-range-config-managed-sets-table">
        <thead>
          <tr>
            <th>Set</th>
            <th>Provider</th>
            <th>Version</th>
            <th>Entries</th>
            <th>Stale</th>
          </tr>
        </thead>
        <tbody>
          {#each ipRangeManagedSetRows as set}
            <tr>
              <td><code>{set?.set_id || '-'}</code></td>
              <td>{set?.provider || '-'}</td>
              <td><code>{set?.version || '-'}</code></td>
              <td>{set?.entry_count ?? 0}</td>
              <td>{set?.stale === true ? 'YES' : 'NO'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
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
    <TextareaField id="browser-block-rules" label="Minimum Versions (Block)" rows="3" ariaLabel="Browser block minimum versions" spellcheck={false} bind:value={browserBlockRules} />
    <TextareaField id="browser-whitelist-rules" label="Allowlist Exceptions" rows="2" ariaLabel="Browser allowlist exceptions" spellcheck={false} bind:value={browserWhitelistRules} />
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={whitelistDirty}>
  <ConfigPanelHeading title="Bypass Allowlists">
    <label class="toggle-switch" for="bypass-allowlists-toggle">
      <input type="checkbox" id="bypass-allowlists-toggle" bind:checked={bypassAllowlistsEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Define trusted bypass entries. Use one entry per line.</p>
  <div class="admin-controls">
    <TextareaField id="network-whitelist" label='<abbr title="Internet Protocol">IP</abbr>/<abbr title="Classless Inter-Domain Routing">CIDR</abbr> Allowlist' rows="3" ariaLabel="Internet Protocol and Classless Inter-Domain Routing allowlist" spellcheck={false} bind:value={networkWhitelist} />
    <TextareaField id="path-whitelist" label="Path Allowlist" rows="3" ariaLabel="Path allowlist" spellcheck={false} bind:value={pathWhitelist} />
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={edgeModeDirty}>
  <ConfigPanelHeading title="Edge Integration Mode">
    <select class="input-field panel-heading-select" id="edge-integration-mode-select" aria-label="Edge integration mode" bind:value={edgeIntegrationMode}>
      <option value="off">off</option>
      <option value="advisory">advisory</option>
      <option value="authoritative">authoritative</option>
    </select>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Control how external edge bot outcomes influence local policy: off ignores edge outcomes, advisory records them without direct enforcement, authoritative allows strong edge outcomes to short-circuit.</p>
</ConfigPanel>
