<script>
  import { onDestroy, onMount } from 'svelte';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import ConfigRobotsSection from './config/ConfigRobotsSection.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let onFetchRobotsPreview = null;

  const ROBOTS_PREVIEW_AUTO_REFRESH_DEBOUNCE_MS = 120;

  let writable = false;
  let savingRobots = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let robotsEnabled = true;
  let robotsCrawlDelay = 2;
  let robotsBlockTraining = true;
  let robotsBlockSearch = false;
  let restrictSearchEngines = false;

  let robotsPreviewOpen = false;
  let robotsPreviewLoading = false;
  let robotsPreviewContent = '';
  let robotsPreviewPatchKey = '';
  let robotsPreviewRequestId = 0;
  let robotsPreviewAutoRefreshTimer = null;

  let baseline = {
    enabled: true,
    crawlDelay: 2,
    blockTraining: true,
    blockSearch: false,
    restrictSearchEngines: false
  };

  const readBool = (value) => value === true;

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  const clearRobotsPreviewAutoRefreshTimer = () => {
    if (robotsPreviewAutoRefreshTimer) {
      clearTimeout(robotsPreviewAutoRefreshTimer);
      robotsPreviewAutoRefreshTimer = null;
    }
  };

  const buildRobotsPreviewPatch = () => {
    const crawlDelayValue = Number(robotsCrawlDelay);
    return {
      robots_enabled: robotsEnabled === true,
      robots_crawl_delay: Number.isFinite(crawlDelayValue) ? Math.max(0, Math.floor(crawlDelayValue)) : 0,
      ai_policy_block_training: robotsBlockTraining === true,
      ai_policy_block_search: robotsBlockSearch === true,
      ai_policy_allow_search_engines: restrictSearchEngines !== true
    };
  };

  const applyConfig = (config = {}) => {
    writable = config.admin_config_write_enabled !== false;
    robotsEnabled = config.robots_enabled !== false;
    robotsCrawlDelay = parseInteger(config.robots_crawl_delay, 2);
    robotsBlockTraining = config.ai_policy_block_training !== false;
    robotsBlockSearch = config.ai_policy_block_search === true;
    const aiAllowSearchEngines = config.ai_policy_allow_search_engines;
    restrictSearchEngines = aiAllowSearchEngines === undefined ? false : aiAllowSearchEngines !== true;
    baseline = {
      enabled: robotsEnabled === true,
      crawlDelay: Number(robotsCrawlDelay),
      blockTraining: robotsBlockTraining === true,
      blockSearch: robotsBlockSearch === true,
      restrictSearchEngines: restrictSearchEngines === true
    };
  };

  async function saveRobotsConfig() {
    if (saveRobotsDisabled || typeof onSaveConfig !== 'function') return;
    savingRobots = true;
    const payload = {
      robots_enabled: robotsEnabled === true,
      robots_crawl_delay: Number(robotsCrawlDelay),
      ai_policy_block_training: robotsBlockTraining === true,
      ai_policy_block_search: robotsBlockSearch === true,
      ai_policy_allow_search_engines: !restrictSearchEngines
    };
    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Robots policy saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      } else {
        baseline = {
          enabled: robotsEnabled === true,
          crawlDelay: Number(robotsCrawlDelay),
          blockTraining: robotsBlockTraining === true,
          blockSearch: robotsBlockSearch === true,
          restrictSearchEngines: restrictSearchEngines === true
        };
      }
    } finally {
      savingRobots = false;
    }
  }

  async function refreshRobotsPreview(previewPatch = null) {
    if (typeof onFetchRobotsPreview !== 'function') return;
    const patch = previewPatch && typeof previewPatch === 'object'
      ? previewPatch
      : buildRobotsPreviewPatch();
    const requestId = ++robotsPreviewRequestId;
    robotsPreviewLoading = true;
    try {
      const payload = await onFetchRobotsPreview(patch);
      if (requestId !== robotsPreviewRequestId) return;
      robotsPreviewContent = payload && typeof payload.content === 'string'
        ? payload.content
        : '# No preview available';
    } catch (error) {
      if (requestId !== robotsPreviewRequestId) return;
      robotsPreviewContent = `# Error loading preview: ${error && error.message ? error.message : 'Unknown error'}`;
    } finally {
      if (requestId === robotsPreviewRequestId) {
        robotsPreviewLoading = false;
      }
    }
  }

  function scheduleRobotsPreviewRefresh() {
    clearRobotsPreviewAutoRefreshTimer();
    robotsPreviewAutoRefreshTimer = setTimeout(() => {
      void refreshRobotsPreview();
    }, ROBOTS_PREVIEW_AUTO_REFRESH_DEBOUNCE_MS);
  }

  function onRobotsPreviewControlChanged() {
    if (!robotsPreviewOpen) return;
    scheduleRobotsPreviewRefresh();
  }

  function toggleRobotsPreview() {
    if (robotsPreviewOpen) {
      clearRobotsPreviewAutoRefreshTimer();
      robotsPreviewRequestId += 1;
      robotsPreviewOpen = false;
      robotsPreviewLoading = false;
      return;
    }
    robotsPreviewOpen = true;
    scheduleRobotsPreviewRefresh();
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  onDestroy(() => {
    clearRobotsPreviewAutoRefreshTimer();
  });

  $: robotsCrawlDelayValid = inRange(robotsCrawlDelay, 0, 60);
  $: robotsValid = robotsCrawlDelayValid;
  $: robotsDirty = (
    readBool(robotsEnabled) !== baseline.enabled ||
    Number(robotsCrawlDelay) !== baseline.crawlDelay
  );
  $: aiPolicyDirty = (
    readBool(robotsBlockTraining) !== baseline.blockTraining ||
    readBool(robotsBlockSearch) !== baseline.blockSearch ||
    readBool(restrictSearchEngines) !== baseline.restrictSearchEngines
  );
  $: hasUnsavedChanges = robotsDirty || aiPolicyDirty;
  $: saveRobotsDisabled = !writable || !hasUnsavedChanges || !robotsValid || savingRobots;
  $: saveRobotsLabel = savingRobots ? 'Saving...' : 'Save robots policy';
  $: saveRobotsSummary = hasUnsavedChanges
    ? 'Robots policy has unsaved changes'
    : 'No unsaved changes';
  $: saveRobotsInvalidText = robotsValid
    ? ''
    : 'Crawl delay must be between 0 and 60 seconds.';
  $: warnOnUnload = writable && hasUnsavedChanges;
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;
  $: robotsPreviewPatchKey = JSON.stringify(buildRobotsPreviewPatch());

  $: if (robotsPreviewOpen) {
    void robotsPreviewPatchKey;
    scheduleRobotsPreviewRefresh();
  }

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingRobots) {
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }
</script>

<section
  id="dashboard-panel-robots"
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={hasUnsavedChanges}
  data-dashboard-tab-panel="robots"
  aria-labelledby="dashboard-tab-robots"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="robots" status={tabStatus} />
  <div class="controls-grid controls-grid--config">
    <ConfigRobotsSection
      bind:writable
      robotsDirty={robotsDirty}
      aiPolicyDirty={aiPolicyDirty}
      bind:robotsEnabled
      bind:robotsCrawlDelay
      robotsCrawlDelayValid={robotsCrawlDelayValid}
      bind:robotsBlockTraining
      bind:robotsBlockSearch
      bind:restrictSearchEngines
      bind:robotsPreviewOpen
      bind:robotsPreviewLoading
      bind:robotsPreviewContent
      onRobotsPreviewControlChanged={onRobotsPreviewControlChanged}
      onToggleRobotsPreview={toggleRobotsPreview}
    />
    <SaveChangesBar
      containerId="robots-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="robots-unsaved-summary"
      summaryText={saveRobotsSummary}
      summaryClass="text-unsaved-changes"
      invalidId="robots-invalid-summary"
      invalidText={saveRobotsInvalidText}
      buttonId="save-robots-config"
      buttonLabel={saveRobotsLabel}
      buttonDisabled={saveRobotsDisabled}
      onSave={saveRobotsConfig}
    />
  </div>
</section>
