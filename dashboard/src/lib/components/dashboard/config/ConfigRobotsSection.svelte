<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';
  import NumericInputRow from '../primitives/NumericInputRow.svelte';
  import ToggleRow from '../primitives/ToggleRow.svelte';

  export let writable = false;
  export let robotsDirty = false;
  export let aiPolicyDirty = false;
  export let robotsEnabled = true;
  export let robotsCrawlDelay = 2;
  export let robotsCrawlDelayValid = true;
  export let robotsBlockTraining = true;
  export let robotsBlockSearch = false;
  export let restrictSearchEngines = false;
  export let robotsPreviewOpen = false;
  export let robotsPreviewLoading = false;
  export let robotsPreviewContent = '';
  export let onRobotsPreviewControlChanged = null;
  export let onToggleRobotsPreview = null;
  export let onFieldChange = null;

  const handlePreviewControlChange = (event) => {
    if (typeof onRobotsPreviewControlChanged === 'function') {
      onRobotsPreviewControlChanged(event);
    }
  };

  const handleFieldChange = (field, value) => {
    if (typeof onFieldChange === 'function') {
      onFieldChange(field, value);
    }
  };

  const handlePreviewToggle = () => {
    if (typeof onToggleRobotsPreview === 'function') {
      onToggleRobotsPreview();
    }
  };

  const handleCheckboxChange = (field, event) => {
    handleFieldChange(field, event?.currentTarget?.checked === true);
    handlePreviewControlChange(event);
  };

  const handleCrawlDelayInput = (event) => {
    handleFieldChange('robotsCrawlDelay', event?.currentTarget?.value ?? '');
    handlePreviewControlChange(event);
  };
</script>

<ConfigPanel writable={writable} dirty={robotsDirty || aiPolicyDirty}>
  <ConfigPanelHeading title="Serve a Robots.txt Specifying Bot Policy">
    <label class="toggle-switch" for="robots-enabled-toggle">
      <input
        type="checkbox"
        id="robots-enabled-toggle"
        aria-label="Serve robots.txt"
        checked={robotsEnabled}
        on:change={(event) => handleCheckboxChange('robotsEnabled', event)}
      >
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">View the current policy as configured below: <a id="open-robots-txt-link" href="/robots.txt" target="_blank" rel="noopener noreferrer">robots.txt</a></p>
  <div class="admin-controls">
    <NumericInputRow
      id="robots-crawl-delay"
      label="Crawl Delay (seconds)"
      labelClass="control-label control-label--wide"
      min="0"
      max="60"
      step="1"
      inputmode="numeric"
      ariaLabel="Robots crawl delay in seconds"
      ariaInvalid={robotsCrawlDelayValid ? 'false' : 'true'}
      value={robotsCrawlDelay}
      onInput={handleCrawlDelayInput}
    />
    <h4 class="control-subtitle"><abbr title="Large Language Models">AI</abbr> Bot Policy</h4>
    <ToggleRow
      id="robots-block-training-toggle"
      label='Opt-out <abbr title="Large Language Models">AI</abbr> Training'
      labelClass="control-label control-label--wide"
      ariaLabel="Opt-out Large Language Models training"
      hint="GPTBot, CCBot, ClaudeBot"
      checked={robotsBlockTraining}
      onChange={(event) => handleCheckboxChange('robotsBlockTraining', event)}
    />
    <ToggleRow
      id="robots-block-search-toggle"
      label='Opt-out <abbr title="Large Language Models">AI</abbr> Search'
      labelClass="control-label control-label--wide"
      ariaLabel="Opt-out Large Language Models search"
      hint="PerplexityBot, etc."
      checked={robotsBlockSearch}
      onChange={(event) => handleCheckboxChange('robotsBlockSearch', event)}
    />
    <ToggleRow
      id="robots-allow-search-toggle"
      label="Restrict Search Engines"
      labelClass="control-label control-label--wide"
      ariaLabel="Restrict search engines"
      hint="Google, Bing, etc."
      checked={restrictSearchEngines}
      onChange={(event) => handleCheckboxChange('restrictSearchEngines', event)}
    />
  </div>
  <button id="preview-robots" class="btn btn-subtle" on:click={handlePreviewToggle}>{robotsPreviewOpen ? 'Hide robots.txt' : 'Show robots.txt'}</button>
  <div id="robots-preview" class="robots-preview panel pad-sm" class:hidden={!robotsPreviewOpen}>
    <h4>robots.txt Preview</h4>
    <pre id="robots-preview-content">{robotsPreviewLoading ? 'Loading...' : robotsPreviewContent}</pre>
  </div>
</ConfigPanel>
