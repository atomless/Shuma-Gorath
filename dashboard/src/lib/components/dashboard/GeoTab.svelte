<script>
  import { onMount } from 'svelte';
  import {
    normalizeCountryCodesForCompare,
    parseCountryCodesStrict
  } from '../../domain/config-form-utils.js';
  import { formatCountryCodes, geoModeFromToggleState, geoToggleStateFromMode } from '../../domain/config-tab-helpers.js';
  import ConfigGeoSection from './config/ConfigGeoSection.svelte';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;

  let writable = false;
  let savingGeo = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let geoRiskList = '';
  let geoAllowList = '';
  let geoChallengeList = '';
  let geoMazeList = '';
  let geoBlockList = '';
  let geoScoringEnabled = true;
  let geoRoutingEnabled = true;
  let akamaiGeoSignalEnabled = true;

  let baseline = {
    geo: {
      scoringEnabled: true,
      routingEnabled: true,
      risk: '',
      allow: '',
      challenge: '',
      maze: '',
      block: ''
    },
    akamai: {
      enabled: true
    }
  };

  const readBool = (value) => value === true;

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  const applyConfig = (config = {}) => {
    writable = config.admin_config_write_enabled !== false;
    geoRiskList = formatCountryCodes(config.geo_risk);
    geoAllowList = formatCountryCodes(config.geo_allow);
    geoChallengeList = formatCountryCodes(config.geo_challenge);
    geoMazeList = formatCountryCodes(config.geo_maze);
    geoBlockList = formatCountryCodes(config.geo_block);
    const geoToggleState = geoToggleStateFromMode(config?.defence_modes?.geo);
    geoScoringEnabled = geoToggleState.scoringEnabled;
    geoRoutingEnabled = geoToggleState.routingEnabled;
    akamaiGeoSignalEnabled = config.geo_edge_headers_enabled !== false;

    baseline = {
      geo: {
        scoringEnabled: geoScoringEnabled === true,
        routingEnabled: geoRoutingEnabled === true,
        risk: normalizeCountryCodesForCompare(geoRiskList),
        allow: normalizeCountryCodesForCompare(geoAllowList),
        challenge: normalizeCountryCodesForCompare(geoChallengeList),
        maze: normalizeCountryCodesForCompare(geoMazeList),
        block: normalizeCountryCodesForCompare(geoBlockList)
      },
      akamai: {
        enabled: akamaiGeoSignalEnabled === true
      }
    };
  };

  async function saveGeoConfig() {
    if (saveGeoDisabled || typeof onSaveConfig !== 'function') return;
    savingGeo = true;
    const payload = {};

    if (geoScoringDirty || geoRoutingDirty) {
      payload.defence_modes = {
        geo: geoModeNormalized
      };
    }
    if (geoScoringDirty) {
      payload.geo_risk = parseCountryCodesStrict(geoRiskList);
    }
    if (geoRoutingDirty) {
      payload.geo_allow = parseCountryCodesStrict(geoAllowList);
      payload.geo_challenge = parseCountryCodesStrict(geoChallengeList);
      payload.geo_maze = parseCountryCodesStrict(geoMazeList);
      payload.geo_block = parseCountryCodesStrict(geoBlockList);
    }
    if (akamaiGeoDirty) {
      payload.geo_edge_headers_enabled = akamaiGeoSignalEnabled === true;
    }

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'GEO settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      } else {
        baseline = {
          geo: {
            scoringEnabled: geoScoringEnabled === true,
            routingEnabled: geoRoutingEnabled === true,
            risk: normalizeCountryCodesForCompare(geoRiskList),
            allow: normalizeCountryCodesForCompare(geoAllowList),
            challenge: normalizeCountryCodesForCompare(geoChallengeList),
            maze: normalizeCountryCodesForCompare(geoMazeList),
            block: normalizeCountryCodesForCompare(geoBlockList)
          },
          akamai: {
            enabled: akamaiGeoSignalEnabled === true
          }
        };
      }
    } finally {
      savingGeo = false;
    }
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  $: geoRiskNormalized = normalizeCountryCodesForCompare(geoRiskList);
  $: geoAllowNormalized = normalizeCountryCodesForCompare(geoAllowList);
  $: geoChallengeNormalized = normalizeCountryCodesForCompare(geoChallengeList);
  $: geoMazeNormalized = normalizeCountryCodesForCompare(geoMazeList);
  $: geoBlockNormalized = normalizeCountryCodesForCompare(geoBlockList);
  $: geoModeNormalized = geoModeFromToggleState({
    scoringEnabled: readBool(geoScoringEnabled),
    routingEnabled: readBool(geoRoutingEnabled)
  });

  $: geoRiskListValid = (() => {
    try {
      parseCountryCodesStrict(geoRiskList);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: geoAllowListValid = (() => {
    try {
      parseCountryCodesStrict(geoAllowList);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: geoChallengeListValid = (() => {
    try {
      parseCountryCodesStrict(geoChallengeList);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: geoMazeListValid = (() => {
    try {
      parseCountryCodesStrict(geoMazeList);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: geoBlockListValid = (() => {
    try {
      parseCountryCodesStrict(geoBlockList);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: geoScoringValid = geoRiskListValid;
  $: geoRoutingValid = (
    geoAllowListValid &&
    geoChallengeListValid &&
    geoMazeListValid &&
    geoBlockListValid
  );
  $: geoValid = geoScoringValid && geoRoutingValid;

  $: geoScoringDirty = (
    readBool(geoScoringEnabled) !== baseline.geo.scoringEnabled ||
    geoRiskNormalized !== baseline.geo.risk
  );
  $: geoRoutingDirty = (
    readBool(geoRoutingEnabled) !== baseline.geo.routingEnabled ||
    geoAllowNormalized !== baseline.geo.allow ||
    geoChallengeNormalized !== baseline.geo.challenge ||
    geoMazeNormalized !== baseline.geo.maze ||
    geoBlockNormalized !== baseline.geo.block
  );
  $: akamaiGeoDirty = readBool(akamaiGeoSignalEnabled) !== baseline.akamai.enabled;

  $: hasUnsavedChanges = geoScoringDirty || geoRoutingDirty || akamaiGeoDirty;
  $: saveGeoDisabled = !writable || !hasUnsavedChanges || !geoValid || savingGeo;
  $: saveGeoLabel = savingGeo ? 'Saving...' : 'Save GEO settings';
  $: saveGeoSummary = hasUnsavedChanges
    ? 'GEO has unsaved changes'
    : 'No unsaved changes';
  $: saveGeoInvalidText = geoValid
    ? ''
    : 'Fix invalid GEO country lists before saving.';
  $: warnOnUnload = writable && hasUnsavedChanges;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingGeo) {
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }
</script>

<section
  id="dashboard-panel-geo"
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={hasUnsavedChanges}
  data-dashboard-tab-panel="geo"
  aria-labelledby="dashboard-tab-geo"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="geo" status={tabStatus} />
  <div class="controls-grid controls-grid--config">
    <ConfigGeoSection
      bind:writable
      bind:geoScoringDirty
      bind:geoRoutingDirty
      bind:geoScoringEnabled
      bind:geoRoutingEnabled
      bind:geoRiskList
      bind:geoAllowList
      bind:geoChallengeList
      bind:geoMazeList
      bind:geoBlockList
      {geoRiskListValid}
      {geoAllowListValid}
      {geoChallengeListValid}
      {geoMazeListValid}
      {geoBlockListValid}
    />

    <ConfigPanel writable={writable} dirty={akamaiGeoDirty}>
      <ConfigPanelHeading title="Akamai GEO Signal">
        <label class="toggle-switch" for="geo-akamai-enabled-toggle">
          <input
            type="checkbox"
            id="geo-akamai-enabled-toggle"
            aria-label="Enable Akamai GEO signal ingestion"
            bind:checked={akamaiGeoSignalEnabled}
          >
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">When enabled, Shuma-Gorath accepts trusted edge-provided country headers for GEO scoring and routing decisions. Disable this to force GEO behavior off even when upstream headers are present.</p>
      {#if !akamaiGeoSignalEnabled}
        <p class="message warning">
          Akamai GEO signal is disabled. GEO scoring and routing will not use edge country headers.
        </p>
      {/if}
    </ConfigPanel>

    <SaveChangesBar
      containerId="geo-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="geo-unsaved-summary"
      summaryText={saveGeoSummary}
      summaryClass="text-unsaved-changes"
      invalidId="geo-invalid-summary"
      invalidText={saveGeoInvalidText}
      buttonId="save-geo-config"
      buttonLabel={saveGeoLabel}
      buttonDisabled={saveGeoDisabled}
      onSave={saveGeoConfig}
    />
  </div>
</section>
