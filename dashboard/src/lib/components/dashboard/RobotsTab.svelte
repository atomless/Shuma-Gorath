<script>
  import { onDestroy, onMount } from 'svelte';
  import {
    formatBrowserRulesTextarea,
    formatListTextarea,
    normalizeListTextareaForCompare,
    parseListTextarea,
    normalizeBrowserRulesForCompare,
    parseBrowserRulesTextarea
  } from '../../domain/config-form-utils.js';
  import { durationPartsFromSeconds, durationSeconds } from '../../domain/core/date-time.js';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange, isDurationTupleValid } from '../../domain/core/validation.js';
  import { isAdminConfigWritable } from '../../domain/config-runtime.js';
  import ConfigDurationsSection from './config/ConfigDurationsSection.svelte';
  import ConfigNetworkSection from './config/ConfigNetworkSection.svelte';
  import ConfigPathAllowlistSection from './config/ConfigPathAllowlistSection.svelte';
  import ConfigRobotsSection from './config/ConfigRobotsSection.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let onFetchRobotsPreview = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  const ROBOTS_PREVIEW_AUTO_REFRESH_DEBOUNCE_MS = 120;
  const MAX_DURATION_SECONDS = 31536000;
  const MIN_DURATION_SECONDS = 60;
  const DURATION_VALIDATION_BOUNDS = Object.freeze({
    minSeconds: MIN_DURATION_SECONDS,
    maxSeconds: MAX_DURATION_SECONDS
  });

  let writable = false;
  let savingPolicy = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;
  let lastSaveInvalidLabel = '';

  let robotsEnabled = true;
  let robotsCrawlDelay = 2;
  let robotsBlockTraining = true;
  let robotsBlockSearch = false;
  let restrictSearchEngines = false;

  let durHoneypotDays = 1;
  let durHoneypotHours = 0;
  let durHoneypotMinutes = 0;
  let durRateLimitDays = 0;
  let durRateLimitHours = 1;
  let durRateLimitMinutes = 0;
  let durCdpDays = 0;
  let durCdpHours = 12;
  let durCdpMinutes = 0;
  let durAdminDays = 0;
  let durAdminHours = 6;
  let durAdminMinutes = 0;

  let browserPolicyEnabled = true;
  let browserBlockRules = '';
  let pathAllowlistEnabled = true;
  let pathAllowlist = '';

  let robotsPreviewOpen = false;
  let robotsPreviewLoading = false;
  let robotsPreviewContent = '';
  let robotsPreviewPatchKey = '';
  let robotsPreviewRequestId = 0;
  let robotsPreviewAutoRefreshTimer = null;

  let baseline = {
    robotsPolicy: {
      enabled: true,
      crawlDelay: 2,
      blockTraining: true,
      blockSearch: false,
      restrictSearchEngines: false
    },
    durations: {
      honeypot: 86400,
      rateLimit: 3600,
      cdp: 43200,
      admin: 21600
    },
    browserPolicy: {
      enabled: true,
      block: ''
    },
    pathAllowlist: {
      enabled: true,
      entries: ''
    }
  };

  const readBool = (value) => value === true;

  const toDurationBaseline = (config = {}) => {
    const durations = config && typeof config.ban_durations === 'object'
      ? config.ban_durations
      : {};
    return {
      honeypot: parseInteger(durations.honeypot, 86400),
      rateLimit: parseInteger(durations.rate_limit, 3600),
      cdp: parseInteger(durations.cdp, 43200),
      admin: parseInteger(durations.admin, 21600)
    };
  };

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
    robotsEnabled = config.robots_enabled !== false;
    robotsCrawlDelay = parseInteger(config.robots_crawl_delay, 2);
    robotsBlockTraining = config.ai_policy_block_training !== false;
    robotsBlockSearch = config.ai_policy_block_search === true;
    const aiAllowSearchEngines = config.ai_policy_allow_search_engines;
    restrictSearchEngines = aiAllowSearchEngines === undefined ? false : aiAllowSearchEngines !== true;

    const durations = toDurationBaseline(config);
    const honeypotParts = durationPartsFromSeconds(durations.honeypot, 86400);
    durHoneypotDays = honeypotParts.days;
    durHoneypotHours = honeypotParts.hours;
    durHoneypotMinutes = honeypotParts.minutes;

    const rateLimitParts = durationPartsFromSeconds(durations.rateLimit, 3600);
    durRateLimitDays = rateLimitParts.days;
    durRateLimitHours = rateLimitParts.hours;
    durRateLimitMinutes = rateLimitParts.minutes;

    const cdpParts = durationPartsFromSeconds(durations.cdp, 43200);
    durCdpDays = cdpParts.days;
    durCdpHours = cdpParts.hours;
    durCdpMinutes = cdpParts.minutes;

    const adminParts = durationPartsFromSeconds(durations.admin, 21600);
    durAdminDays = adminParts.days;
    durAdminHours = adminParts.hours;
    durAdminMinutes = adminParts.minutes;

    browserPolicyEnabled = config.browser_policy_enabled !== false;
    browserBlockRules = formatBrowserRulesTextarea(config.browser_block);
    pathAllowlistEnabled = config.path_allowlist_enabled !== false;
    pathAllowlist = formatListTextarea(config.path_allowlist);

    baseline = {
      robotsPolicy: {
        enabled: robotsEnabled === true,
        crawlDelay: Number(robotsCrawlDelay),
        blockTraining: robotsBlockTraining === true,
        blockSearch: robotsBlockSearch === true,
        restrictSearchEngines: restrictSearchEngines === true
      },
      durations,
      browserPolicy: {
        enabled: browserPolicyEnabled === true,
        block: normalizeBrowserRulesForCompare(formatBrowserRulesTextarea(config.browser_block))
      },
      pathAllowlist: {
        enabled: pathAllowlistEnabled === true,
        entries: normalizeListTextareaForCompare(formatListTextarea(config.path_allowlist))
      }
    };
  };

  async function savePolicyConfig() {
    if (savePolicyDisabled || typeof onSaveConfig !== 'function') return;
    savingPolicy = true;
    const payload = {};

    if (robotsDirty || aiPolicyDirty) {
      payload.robots_enabled = robotsEnabled === true;
      payload.robots_crawl_delay = Number(robotsCrawlDelay);
      payload.ai_policy_block_training = robotsBlockTraining === true;
      payload.ai_policy_block_search = robotsBlockSearch === true;
      payload.ai_policy_allow_search_engines = !restrictSearchEngines;
    }
    if (durationsDirty) {
      payload.ban_durations = {
        honeypot: honeypotDurationSeconds,
        rate_limit: rateDurationSeconds,
        cdp: cdpDurationSeconds,
        admin: adminDurationSeconds
      };
    }
    if (browserPolicyDirty) {
      payload.browser_policy_enabled = browserPolicyEnabled;
      payload.browser_block = parseBrowserRulesTextarea(browserBlockRules);
    }
    if (pathAllowlistDirty) {
      payload.path_allowlist_enabled = pathAllowlistEnabled === true;
      payload.path_allowlist = parseListTextarea(pathAllowlist);
    }

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Policy settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      } else {
        baseline = {
          robotsPolicy: {
            enabled: robotsEnabled === true,
            crawlDelay: Number(robotsCrawlDelay),
            blockTraining: robotsBlockTraining === true,
            blockSearch: robotsBlockSearch === true,
            restrictSearchEngines: restrictSearchEngines === true
          },
          durations: {
            honeypot: honeypotDurationSeconds,
            rateLimit: rateDurationSeconds,
            cdp: cdpDurationSeconds,
            admin: adminDurationSeconds
          },
          browserPolicy: {
            enabled: browserPolicyEnabled === true,
            block: normalizeBrowserRulesForCompare(browserBlockRules)
          },
          pathAllowlist: {
            enabled: pathAllowlistEnabled === true,
            entries: normalizeListTextareaForCompare(pathAllowlist)
          }
        };
      }
      lastSaveInvalidLabel = '';
    } finally {
      savingPolicy = false;
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

  $: writable = isAdminConfigWritable(configRuntimeSnapshot);
  $: robotsCrawlDelayValid = inRange(robotsCrawlDelay, 0, 60);
  $: robotsValid = robotsCrawlDelayValid;
  $: robotsDirty = (
    readBool(robotsEnabled) !== baseline.robotsPolicy.enabled ||
    Number(robotsCrawlDelay) !== baseline.robotsPolicy.crawlDelay
  );
  $: aiPolicyDirty = (
    readBool(robotsBlockTraining) !== baseline.robotsPolicy.blockTraining ||
    readBool(robotsBlockSearch) !== baseline.robotsPolicy.blockSearch ||
    readBool(restrictSearchEngines) !== baseline.robotsPolicy.restrictSearchEngines
  );

  $: honeypotDurationSeconds = durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes);
  $: rateDurationSeconds = durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes);
  $: cdpDurationSeconds = durationSeconds(durCdpDays, durCdpHours, durCdpMinutes);
  $: adminDurationSeconds = durationSeconds(durAdminDays, durAdminHours, durAdminMinutes);

  $: durHoneypotValid = isDurationTupleValid(
    durHoneypotDays,
    durHoneypotHours,
    durHoneypotMinutes,
    DURATION_VALIDATION_BOUNDS
  );
  $: durRateLimitValid = isDurationTupleValid(
    durRateLimitDays,
    durRateLimitHours,
    durRateLimitMinutes,
    DURATION_VALIDATION_BOUNDS
  );
  $: durCdpValid = isDurationTupleValid(
    durCdpDays,
    durCdpHours,
    durCdpMinutes,
    DURATION_VALIDATION_BOUNDS
  );
  $: durAdminValid = isDurationTupleValid(
    durAdminDays,
    durAdminHours,
    durAdminMinutes,
    DURATION_VALIDATION_BOUNDS
  );
  $: durationsValid = (
    durHoneypotValid &&
    durRateLimitValid &&
    durCdpValid &&
    durAdminValid
  );
  $: durationsDirty = (
    honeypotDurationSeconds !== baseline.durations.honeypot ||
    rateDurationSeconds !== baseline.durations.rateLimit ||
    cdpDurationSeconds !== baseline.durations.cdp ||
    adminDurationSeconds !== baseline.durations.admin
  );

  $: browserBlockNormalized = normalizeBrowserRulesForCompare(browserBlockRules);
  $: browserBlockRulesValid = browserBlockNormalized !== '__invalid__';
  $: browserPolicyValid = browserBlockRulesValid;
  $: browserPolicyDirty = (
    (browserPolicyEnabled === true) !== baseline.browserPolicy.enabled ||
    browserBlockNormalized !== baseline.browserPolicy.block
  );

  $: pathAllowlistNormalized = normalizeListTextareaForCompare(pathAllowlist);
  $: pathAllowlistDirty = (
    (pathAllowlistEnabled === true) !== baseline.pathAllowlist.enabled ||
    pathAllowlistNormalized !== baseline.pathAllowlist.entries
  );

  $: dirtySections = [
    { label: 'Robots and AI policy', dirty: robotsDirty || aiPolicyDirty, valid: robotsValid },
    { label: 'Ban durations', dirty: durationsDirty, valid: durationsValid },
    { label: 'Browser policy', dirty: browserPolicyDirty, valid: browserPolicyValid },
    { label: 'Path allowlist', dirty: pathAllowlistDirty, valid: true }
  ];
  $: dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
  $: invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
  $: invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
  $: dirtySectionCount = dirtySectionEntries.length;
  $: hasUnsavedChanges = dirtySectionCount > 0;
  $: hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
  $: savePolicyDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingPolicy;
  $: savePolicyLabel = savingPolicy ? 'Saving...' : 'Save policy changes';
  $: savePolicySummary = hasUnsavedChanges
    ? `${dirtySectionCount} policy section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
    : 'No unsaved changes';
  $: savePolicyInvalidText = hasInvalidUnsavedChanges
    ? `Fix invalid values in: ${lastSaveInvalidLabel}`
    : '';
  $: warnOnUnload = writable && hasUnsavedChanges;
  $: robotsPreviewPatchKey = JSON.stringify(buildRobotsPreviewPatch());

  $: if (hasInvalidUnsavedChanges) {
    lastSaveInvalidLabel = invalidDirtySectionLabels.join(', ');
  } else if (!hasUnsavedChanges) {
    lastSaveInvalidLabel = '';
  }

  $: if (robotsPreviewOpen) {
    void robotsPreviewPatchKey;
    scheduleRobotsPreviewRefresh();
  }

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingPolicy) {
        applyConfig(
          configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    }
  }
</script>

<section
  id="dashboard-panel-policy"
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={hasUnsavedChanges}
  data-dashboard-tab-panel="policy"
  aria-labelledby="dashboard-tab-policy"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="policy" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
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

    <ConfigDurationsSection
      bind:writable
      durationsDirty={durationsDirty}
      bind:durHoneypotDays
      bind:durHoneypotHours
      bind:durHoneypotMinutes
      bind:durRateLimitDays
      bind:durRateLimitHours
      bind:durRateLimitMinutes
      bind:durCdpDays
      bind:durCdpHours
      bind:durCdpMinutes
      bind:durAdminDays
      bind:durAdminHours
      bind:durAdminMinutes
      durHoneypotValid={durHoneypotValid}
      durRateLimitValid={durRateLimitValid}
      durCdpValid={durCdpValid}
      durAdminValid={durAdminValid}
    />

    <ConfigNetworkSection
      bind:writable
      showHoneypot={false}
      showBrowserPolicy={true}
      bind:browserPolicyDirty
      bind:browserPolicyEnabled
      bind:browserBlockRules
      browserBlockRulesValid={browserBlockRulesValid}
    />

    <ConfigPathAllowlistSection
      bind:writable
      bind:pathAllowlistDirty
      bind:pathAllowlistEnabled
      bind:pathAllowlist
    />

    <SaveChangesBar
      containerId="policy-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="policy-unsaved-summary"
      summaryText={savePolicySummary}
      summaryClass="text-unsaved-changes"
      invalidId="policy-invalid-summary"
      invalidText={savePolicyInvalidText}
      buttonId="save-policy-config"
      buttonLabel={savePolicyLabel}
      buttonDisabled={savePolicyDisabled}
      onSave={savePolicyConfig}
    />
  </div>
</section>
