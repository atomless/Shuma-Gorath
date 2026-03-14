<script>
  import { onMount } from 'svelte';
  import {
    formatListTextarea,
    normalizeHoneypotPathsForCompare,
    parseHoneypotPathsTextarea
  } from '../../domain/config-form-utils.js';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import ConfigMazeSection from './config/ConfigMazeSection.svelte';
  import ConfigNetworkSection from './config/ConfigNetworkSection.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  let writable = false;
  let savingTraps = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let honeypotEnabled = true;
  let honeypotPaths = '';
  let honeypotInvalidMessage = '';

  let mazeEnabled = true;
  let tarpitEnabled = true;
  let mazeAutoBan = true;
  let mazeThreshold = 50;

  let baseline = {
    honeypot: {
      enabled: true,
      values: ''
    },
    maze: {
      enabled: true,
      autoBan: true,
      threshold: 50
    },
    tarpit: {
      enabled: true
    }
  };

  const readBool = (value) => value === true;

  const applyConfig = (config = {}) => {
    writable = config.admin_config_write_enabled !== false;
    honeypotEnabled = config.honeypot_enabled !== false;
    honeypotPaths = formatListTextarea(config.honeypots);
    mazeEnabled = config.maze_enabled !== false;
    tarpitEnabled = config.tarpit_enabled !== false;
    mazeAutoBan = config.maze_auto_ban !== false;
    mazeThreshold = parseInteger(config.maze_auto_ban_threshold, 50);

    baseline = {
      honeypot: {
        enabled: honeypotEnabled === true,
        values: normalizeHoneypotPathsForCompare(honeypotPaths)
      },
      maze: {
        enabled: mazeEnabled === true,
        autoBan: mazeAutoBan === true,
        threshold: Number(mazeThreshold)
      },
      tarpit: {
        enabled: tarpitEnabled === true
      }
    };
  };

  async function saveTrapsConfig() {
    if (saveTrapsDisabled || typeof onSaveConfig !== 'function') return;
    savingTraps = true;

    const payload = {};
    if (honeypotDirty) {
      payload.honeypot_enabled = honeypotEnabled;
      payload.honeypots = parseHoneypotPathsTextarea(honeypotPaths);
    }
    if (mazeDirty) {
      payload.maze_enabled = mazeEnabled;
      payload.maze_auto_ban = mazeAutoBan;
      payload.maze_auto_ban_threshold = Number(mazeThreshold);
    }
    if (tarpitDirty) {
      payload.tarpit_enabled = tarpitEnabled;
    }

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Trap settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      } else {
        baseline = {
          honeypot: {
            enabled: honeypotEnabled === true,
            values: honeypotNormalized
          },
          maze: {
            enabled: mazeEnabled === true,
            autoBan: mazeAutoBan === true,
            threshold: Number(mazeThreshold)
          },
          tarpit: {
            enabled: tarpitEnabled === true
          }
        };
      }
    } finally {
      savingTraps = false;
    }
  }

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  $: honeypotNormalized = normalizeHoneypotPathsForCompare(honeypotPaths);
  $: honeypotValid = (() => {
    try {
      parseHoneypotPathsTextarea(honeypotPaths);
      honeypotInvalidMessage = '';
      return true;
    } catch (error) {
      honeypotInvalidMessage = error && error.message
        ? String(error.message)
        : 'Invalid honeypot path list.';
      return false;
    }
  })();
  $: honeypotDirty = (
    readBool(honeypotEnabled) !== baseline.honeypot.enabled ||
    honeypotNormalized !== baseline.honeypot.values
  );

  $: mazeThresholdValid = inRange(mazeThreshold, 5, 500);
  $: mazeValid = mazeThresholdValid;
  $: mazeDirty = (
    readBool(mazeEnabled) !== baseline.maze.enabled ||
    readBool(mazeAutoBan) !== baseline.maze.autoBan ||
    Number(mazeThreshold) !== baseline.maze.threshold
  );

  $: tarpitValid = true;
  $: tarpitDirty = readBool(tarpitEnabled) !== baseline.tarpit.enabled;

  $: dirtySections = [
    { label: 'Maze', dirty: mazeDirty, valid: mazeValid },
    { label: 'Tarpit', dirty: tarpitDirty, valid: tarpitValid },
    { label: 'Honeypot paths', dirty: honeypotDirty, valid: honeypotValid }
  ];
  $: dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
  $: invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
  $: invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
  $: dirtySectionCount = dirtySectionEntries.length;
  $: hasUnsavedChanges = dirtySectionCount > 0;
  $: hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
  $: saveTrapsDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingTraps;
  $: saveTrapsLabel = savingTraps ? 'Saving...' : 'Save trap settings';
  $: saveTrapsSummary = hasUnsavedChanges
    ? `${dirtySectionCount} section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
    : 'No unsaved changes';
  $: saveTrapsInvalidText = hasInvalidUnsavedChanges
    ? `Fix invalid values in: ${invalidDirtySectionLabels.join(', ')}`
    : '';
  $: warnOnUnload = writable && hasUnsavedChanges;
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingTraps) {
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }
</script>

<section
  id="dashboard-panel-traps"
  class="admin-group"
  data-dashboard-tab-panel="traps"
  aria-labelledby="dashboard-tab-traps"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="traps" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigMazeSection
      bind:writable
      bind:mazeDirty
      bind:tarpitDirty
      bind:mazeEnabled
      bind:mazeAutoBan
      bind:mazeThreshold
      mazeThresholdValid={mazeThresholdValid}
      bind:tarpitEnabled
    />

    <ConfigNetworkSection
      bind:writable
      showHoneypot={true}
      showBrowserPolicy={false}
      bind:honeypotDirty
      bind:honeypotEnabled
      bind:honeypotPaths
      honeypotPathsValid={honeypotValid}
      honeypotInvalidMessage={honeypotInvalidMessage}
    />

    <SaveChangesBar
      containerId="traps-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="traps-unsaved-summary"
      summaryText={saveTrapsSummary}
      summaryClass="text-unsaved-changes"
      invalidId="traps-invalid-summary"
      invalidText={saveTrapsInvalidText}
      buttonId="save-traps-config"
      buttonLabel={saveTrapsLabel}
      buttonDisabled={saveTrapsDisabled}
      onSave={saveTrapsConfig}
    />
  </div>
</section>
