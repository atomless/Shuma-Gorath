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
  import { isAdminConfigWritable } from '../../domain/config-runtime.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  let writable = false;
  let savingTraps = false;
  let warnOnUnload = false;
  let hasConfigSnapshot = false;
  let lastAppliedConfigVersion = -1;
  let pendingConfigVersion = -1;

  let honeypotEnabled = true;
  let honeypotPaths = '';
  let honeypotInvalidMessage = '';

  let mazeEnabled = true;
  let tarpitEnabled = true;
  let mazeAutoBan = true;
  let mazeThreshold = 50;
  let honeypotNormalized = '';
  let honeypotValid = true;
  let honeypotDirty = false;
  let mazeThresholdValid = true;
  let mazeValid = true;
  let mazeDirty = false;
  let tarpitValid = true;
  let tarpitDirty = false;
  let dirtySections = [];
  let dirtySectionEntries = [];
  let invalidDirtySectionEntries = [];
  let invalidDirtySectionLabels = [];
  let dirtySectionCount = 0;
  let hasUnsavedChanges = false;
  let hasInvalidUnsavedChanges = false;
  let saveTrapsDisabled = true;
  let saveTrapsLabel = 'Save trap settings';
  let saveTrapsSummary = 'No unsaved changes';
  let saveTrapsInvalidText = '';

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

  const applyConfig = (config = {}, runtime = {}) => {
    writable = isAdminConfigWritable(runtime);
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

  const applyTrapFieldChange = (field, value) => {
    switch (field) {
      case 'honeypotEnabled':
        honeypotEnabled = value === true;
        break;
      case 'honeypotPaths':
        honeypotPaths = String(value || '');
        break;
      case 'mazeEnabled':
        mazeEnabled = value === true;
        break;
      case 'mazeAutoBan':
        mazeAutoBan = value === true;
        break;
      case 'mazeThreshold':
        mazeThreshold = value;
        break;
      case 'tarpitEnabled':
        tarpitEnabled = value === true;
        break;
      default:
        break;
    }
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
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
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

  $: {
    honeypotNormalized = normalizeHoneypotPathsForCompare(honeypotPaths);
    try {
      parseHoneypotPathsTextarea(honeypotPaths);
      honeypotInvalidMessage = '';
      honeypotValid = true;
    } catch (error) {
      honeypotInvalidMessage = error && error.message
        ? String(error.message)
        : 'Invalid honeypot path list.';
      honeypotValid = false;
    }

    honeypotDirty = (
      readBool(honeypotEnabled) !== baseline.honeypot.enabled ||
      honeypotNormalized !== baseline.honeypot.values
    );

    mazeThresholdValid = inRange(mazeThreshold, 5, 500);
    mazeValid = mazeThresholdValid;
    mazeDirty = (
      readBool(mazeEnabled) !== baseline.maze.enabled ||
      readBool(mazeAutoBan) !== baseline.maze.autoBan ||
      Number(mazeThreshold) !== baseline.maze.threshold
    );

    tarpitValid = true;
    tarpitDirty = readBool(tarpitEnabled) !== baseline.tarpit.enabled;

    dirtySections = [
      { label: 'Maze', dirty: mazeDirty, valid: mazeValid },
      { label: 'Tarpit', dirty: tarpitDirty, valid: tarpitValid },
      { label: 'Honeypot paths', dirty: honeypotDirty, valid: honeypotValid }
    ];
    dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
    invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
    invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
    dirtySectionCount = dirtySectionEntries.length;
    hasUnsavedChanges = dirtySectionCount > 0;
    hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
    saveTrapsDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingTraps;
    saveTrapsLabel = savingTraps ? 'Saving...' : 'Save trap settings';
    saveTrapsSummary = hasUnsavedChanges
      ? `${dirtySectionCount} section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
      : 'No unsaved changes';
    saveTrapsInvalidText = hasInvalidUnsavedChanges
      ? `Fix invalid values in: ${invalidDirtySectionLabels.join(', ')}`
      : '';
    warnOnUnload = writable && hasUnsavedChanges;
  }
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion && nextVersion !== pendingConfigVersion) {
      pendingConfigVersion = nextVersion;
    }
  }

  $: if (pendingConfigVersion !== -1 && hasConfigSnapshot && !hasUnsavedChanges && !savingTraps) {
    applyConfig(
      configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
      configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
    );
    lastAppliedConfigVersion = pendingConfigVersion;
    pendingConfigVersion = -1;
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
      writable={writable}
      mazeDirty={mazeDirty}
      tarpitDirty={tarpitDirty}
      mazeEnabled={mazeEnabled}
      mazeAutoBan={mazeAutoBan}
      mazeThreshold={mazeThreshold}
      mazeThresholdValid={mazeThresholdValid}
      tarpitEnabled={tarpitEnabled}
      onFieldChange={applyTrapFieldChange}
    />

    <ConfigNetworkSection
      writable={writable}
      showHoneypot={true}
      showBrowserPolicy={false}
      honeypotDirty={honeypotDirty}
      honeypotEnabled={honeypotEnabled}
      honeypotPaths={honeypotPaths}
      honeypotPathsValid={honeypotValid}
      honeypotInvalidMessage={honeypotInvalidMessage}
      onFieldChange={applyTrapFieldChange}
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
