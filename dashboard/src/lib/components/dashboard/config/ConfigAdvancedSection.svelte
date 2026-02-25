<script>
  import { formatUnknownForDisplay } from '../../../domain/core/strings.js';
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import TextareaField from '../primitives/TextareaField.svelte';

  export let writable = false;
  export let advancedDirty = false;
  export let advancedConfigJson = '{}';
  export let advancedValidationPending = false;
  export let advancedInvalidMessage = '';
  export let advancedValidationIssues = [];
  export let advancedValid = false;
</script>

<ConfigPanel writable={writable} dirty={advancedDirty}>
  <h3>Advanced Config <abbr title="JavaScript Object Notation">JSON</abbr></h3>
  <p class="control-desc text-muted">Directly edit writable config keys as a <abbr title="JavaScript Object Notation">JSON</abbr> object. This editor reflects the last loaded snapshot and does not auto-sync while you change controls above.</p>
  <div class="admin-controls">
    <TextareaField
      id="advanced-config-json"
      label='<abbr title="JavaScript Object Notation">JSON</abbr> Patch'
      rows="8"
      ariaLabel="Advanced config JavaScript Object Notation patch"
      ariaInvalid={advancedValid ? 'false' : 'true'}
      spellcheck={false}
      monospace={true}
      showLineNumbers={true}
      bind:value={advancedConfigJson}
    />
    {#if advancedValidationPending}
      <p id="advanced-config-json-validating" class="text-muted">Validating Advanced <abbr title="JavaScript Object Notation">JSON</abbr>...</p>
    {/if}
    {#if advancedInvalidMessage}
      <div id="advanced-config-json-error" class="message error">
        <p>{advancedInvalidMessage}</p>
        {#if advancedValidationIssues.length > 0}
          <ul id="advanced-config-json-issue-list" class="validation-issue-list">
            {#each advancedValidationIssues as issue, issueIndex}
              <li id={`advanced-config-json-issue-${issueIndex}`}>
                {#if Number.isFinite(Number(issue.line)) && Number(issue.line) > 0}
                  <span class="validation-issue-line">Line {Number(issue.line)}:</span>
                {/if}
                {#if issue.field}
                  <code>{issue.field}</code>:&nbsp;
                {/if}
                {issue.message}
                {#if issue.expected}
                  <span class="validation-issue-expected">Expected: {issue.expected}</span>
                {/if}
                {#if issue.received !== undefined}
                  <span class="validation-issue-received">Received: <code>{formatUnknownForDisplay(issue.received)}</code></span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
        <a
          id="advanced-config-json-docs-link"
          href="https://github.com/atomless/Shuma-Gorath/blob/main/docs/configuration.md"
          target="_blank"
          rel="noopener noreferrer"
        >Configuration docs</a>
      </div>
    {/if}
  </div>
</ConfigPanel>
