<script>
  import { onMount } from 'svelte';
  import {
    normalizeCountryCodesForCompare,
    parseCountryCodesStrict
  } from '../../domain/config-form-utils.js';
  import {
    formatCountryCodes,
    geoModeFromToggleState,
    geoToggleStateFromMode
  } from '../../domain/config-tab-helpers.js';
  import {
    isAdminConfigWritable,
    isAkamaiEdgeAvailable
  } from '../../domain/config-runtime.js';
  import ConfigGeoSection from './config/ConfigGeoSection.svelte';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

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
  let savingGeo = false;
  let warnOnUnload = false;
  let hasConfigSnapshot = false;
  let lastAppliedConfigVersion = -1;
  let pendingConfigVersion = -1;
  let akamaiEdgeAvailable = false;

  let geoRiskList = '';
  let geoAllowList = '';
  let geoChallengeList = '';
  let geoMazeList = '';
  let geoBlockList = '';
  let geoScoringEnabled = true;
  let geoRoutingEnabled = true;
  let geoEdgeHeaderSignalEnabled = false;

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
    edgeHeaderSignal: {
      enabled: false
    }
  };

  const readBool = (value) => value === true;

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  const applyConfig = (config = {}, runtime = {}) => {
    writable = isAdminConfigWritable(runtime);
    akamaiEdgeAvailable = isAkamaiEdgeAvailable(runtime);
    geoRiskList = formatCountryCodes(config.geo_risk);
    geoAllowList = formatCountryCodes(config.geo_allow);
    geoChallengeList = formatCountryCodes(config.geo_challenge);
    geoMazeList = formatCountryCodes(config.geo_maze);
    geoBlockList = formatCountryCodes(config.geo_block);
    const geoToggleState = geoToggleStateFromMode(config?.defence_modes?.geo);
    geoScoringEnabled = geoToggleState.scoringEnabled;
    geoRoutingEnabled = geoToggleState.routingEnabled;
    geoEdgeHeaderSignalEnabled = config.geo_edge_headers_enabled === true;

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
      edgeHeaderSignal: {
        enabled: geoEdgeHeaderSignalEnabled === true
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
    if (geoEdgeControlsVisible && geoEdgeHeaderDirty) {
      payload.geo_edge_headers_enabled = geoEdgeHeaderSignalEnabled === true;
    }

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'GEO settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
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
          edgeHeaderSignal: {
            enabled: geoEdgeHeaderSignalEnabled === true
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
  $: geoEdgeControlsVisible = akamaiEdgeAvailable === true;
  $: geoEdgeHeaderDirty = geoEdgeControlsVisible &&
    readBool(geoEdgeHeaderSignalEnabled) !== baseline.edgeHeaderSignal.enabled;

  $: hasUnsavedChanges = geoScoringDirty || geoRoutingDirty || geoEdgeHeaderDirty;
  $: saveGeoDisabled = !writable || !hasUnsavedChanges || !geoValid || savingGeo;
  $: saveGeoLabel = savingGeo ? 'Saving...' : 'Save GEO settings';
  $: saveGeoSummary = hasUnsavedChanges
    ? 'GEO has unsaved changes'
    : 'No unsaved changes';
  $: saveGeoInvalidText = geoValid
    ? ''
    : 'Fix invalid GEO country lists before saving.';
  $: warnOnUnload = writable && hasUnsavedChanges;
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion && nextVersion !== pendingConfigVersion) {
      pendingConfigVersion = nextVersion;
    }
  }

  $: if (pendingConfigVersion !== -1 && hasConfigSnapshot && !hasUnsavedChanges && !savingGeo) {
    applyConfig(
      configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
      configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
    );
    lastAppliedConfigVersion = pendingConfigVersion;
    pendingConfigVersion = -1;
  }
</script>

<section
  id="dashboard-panel-geo"
  class="admin-group"
  data-dashboard-tab-panel="geo"
  aria-labelledby="dashboard-tab-geo"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="geo" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigPanel writable={writable} dirty={geoEdgeHeaderDirty}>
      <ConfigPanelHeading title="Trusted GEO Edge Header Signal">
        {#if geoEdgeControlsVisible}
          <label class="toggle-switch" for="geo-edge-header-enabled-toggle">
            <input
              type="checkbox"
              id="geo-edge-header-enabled-toggle"
              aria-label="Enable trusted GEO edge header signal ingestion"
              bind:checked={geoEdgeHeaderSignalEnabled}
            >
            <span class="toggle-slider"></span>
          </label>
        {/if}
      </ConfigPanelHeading>
      {#if geoEdgeControlsVisible}
        <p class="control-desc text-muted">When enabled, Shuma-Gorath accepts a trusted edge-provided country header for GEO scoring and routing decisions. The current runtime expects the upstream edge layer to map provider-native GEO data into <code>X-Geo-Country</code>; this is not yet a direct Akamai EdgeScape parser.</p>
      {:else}
        <p id="geo-edge-unavailable-message" class="control-desc text-muted">Trusted GEO edge-header controls are available only when Shuma-Gorath is deployed on Akamai edge (`gateway_deployment_profile=edge-fermyon`). Shared-server and other non-edge postures keep this integration hidden.</p>
      {/if}
    </ConfigPanel>

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
