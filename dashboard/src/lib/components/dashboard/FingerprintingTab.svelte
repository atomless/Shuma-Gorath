<script>
  import { onMount } from 'svelte';
  import { normalizeEdgeMode } from '../../domain/config-tab-helpers.js';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let cdpSnapshot = null;
  export let onSaveConfig = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  let writable = false;
  let savingFingerprinting = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let akamaiEdgeAvailable = false;
  let akamaiBotSignalEnabled = false;
  let edgeIntegrationMode = 'additive';

  let baseline = {
    akamai: {
      enabled: false,
      mode: 'additive'
    }
  };
  const AKAMAI_EDGE_ADDITIVE_SIGNAL_KEY = 'fp_akamai_edge_additive';
  const FINGERPRINTING_BOTNESS_SIGNAL_KEYS = new Set([
    'js_verification_required',
    'browser_outdated',
    'geo_risk',
    'rate_pressure_medium',
    'rate_pressure_high'
  ]);

  const readBool = (value) => value === true;
  const normalizedModeFromConfig = (value) => {
    const normalized = normalizeEdgeMode(value);
    return normalized === 'off' ? 'additive' : normalized;
  };

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  function applyConfig(config = {}) {
    writable = config.admin_config_write_enabled === true;
    akamaiEdgeAvailable = config.akamai_edge_available === true;
    akamaiBotSignalEnabled = String(config?.provider_backends?.fingerprint_signal || '').toLowerCase() === 'external';
    edgeIntegrationMode = normalizedModeFromConfig(config.edge_integration_mode);

    baseline = {
      akamai: {
        enabled: akamaiBotSignalEnabled,
        mode: edgeIntegrationMode
      }
    };
  }

  async function saveFingerprintingConfig() {
    if (saveFingerprintingDisabled || typeof onSaveConfig !== 'function' || !akamaiEdgeAvailable) return;

    savingFingerprinting = true;
    const payload = {
      edge_integration_mode: akamaiBotSignalEnabled ? normalizeEdgeMode(edgeIntegrationMode) : 'off',
      provider_backends: {
        fingerprint_signal: akamaiBotSignalEnabled ? 'external' : 'internal'
      }
    };

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Fingerprinting settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      } else {
        baseline = {
          akamai: {
            enabled: akamaiBotSignalEnabled === true,
            mode: normalizeEdgeMode(edgeIntegrationMode)
          }
        };
      }
    } finally {
      savingFingerprinting = false;
    }
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  $: edgeModeValid = ['additive', 'authoritative'].includes(normalizeEdgeMode(edgeIntegrationMode));
  $: fingerprintingValid = edgeModeValid;
  $: hasUnsavedChanges = (
    readBool(akamaiBotSignalEnabled) !== baseline.akamai.enabled ||
    normalizeEdgeMode(edgeIntegrationMode) !== baseline.akamai.mode
  );
  $: fingerprintingControlsVisible = akamaiEdgeAvailable === true;

  $: saveFingerprintingDisabled = !writable || !hasUnsavedChanges || !fingerprintingValid || savingFingerprinting;
  $: saveFingerprintingLabel = savingFingerprinting ? 'Saving...' : 'Save fingerprinting settings';
  $: saveFingerprintingSummary = hasUnsavedChanges
    ? 'Fingerprinting has unsaved changes'
    : 'No unsaved changes';
  $: saveFingerprintingInvalidText = fingerprintingValid
    ? ''
    : 'Fix invalid fingerprinting values before saving.';
  $: warnOnUnload = writable && hasUnsavedChanges;
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;

  $: cdpData = cdpSnapshot && typeof cdpSnapshot === 'object' ? cdpSnapshot : {};
  $: cdpStats = cdpData.stats && typeof cdpData.stats === 'object' ? cdpData.stats : {};
  $: fingerprintStats = cdpData.fingerprint_stats && typeof cdpData.fingerprint_stats === 'object'
    ? cdpData.fingerprint_stats
    : {};

  $: signalDefinitions = configSnapshot && typeof configSnapshot.botness_signal_definitions === 'object'
    ? configSnapshot.botness_signal_definitions
    : {};
  $: scoredSignals = Array.isArray(signalDefinitions.scored_signals)
    ? signalDefinitions.scored_signals
    : [];
  $: akamaiEdgeAdditiveSignal = scoredSignals.find(
    (signal) => String(signal?.key || '') === AKAMAI_EDGE_ADDITIVE_SIGNAL_KEY
  ) || null;
  $: fingerprintingBotnessSignals = scoredSignals.filter((signal) => {
    const key = String(signal?.key || '');
    return key !== AKAMAI_EDGE_ADDITIVE_SIGNAL_KEY
      && (FINGERPRINTING_BOTNESS_SIGNAL_KEYS.has(key) || key.startsWith('fp_'));
  });

  $: effectivePosture = (() => {
    if (!akamaiBotSignalEnabled) {
      return 'Akamai bot signals are disabled. Internal passive fingerprint signals remain active.';
    }
    if (normalizeEdgeMode(edgeIntegrationMode) === 'authoritative') {
      return 'Authoritative mode allows high-confidence Akamai outcomes to trigger immediate ban actions.';
    }
    return 'Additive mode contributes Akamai outcomes into internal fingerprint scoring with bounded weight.';
  })();

  $: showAuthoritativeWarning = (
    akamaiBotSignalEnabled &&
    normalizeEdgeMode(edgeIntegrationMode) === 'authoritative'
  );

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingFingerprinting) {
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }
</script>

<section
  id="dashboard-panel-fingerprinting"
  class="admin-group"
  data-dashboard-tab-panel="fingerprinting"
  aria-labelledby="dashboard-tab-fingerprinting"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="fingerprinting" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigPanel writable={writable} dirty={hasUnsavedChanges}>
      <ConfigPanelHeading title="Akamai Bot Signal">
        {#if fingerprintingControlsVisible}
          <label class="toggle-switch" for="fingerprinting-akamai-enabled-toggle">
            <input
              type="checkbox"
              id="fingerprinting-akamai-enabled-toggle"
              aria-label="Enable Akamai bot signals"
              bind:checked={akamaiBotSignalEnabled}
            >
            <span class="toggle-slider"></span>
          </label>
        {/if}
      </ConfigPanelHeading>
      {#if fingerprintingControlsVisible}
        <p class="control-desc text-muted">When calculating bot fingerprinting, Akamai can contribute transport and network-layer telemetry that Shuma-Gorath cannot directly observe at app level. Enable this only when your deployment is hosted on Akamai edge and forwards trusted bot outcomes.</p>
        <div class="admin-controls">
          <div class="input-row" class:input-row--disabled={!akamaiBotSignalEnabled}>
            <label class="control-label control-label--wide" for="fingerprinting-edge-mode-select">Akamai Influence Mode</label>
            <select
              class="input-field input-row-control"
              id="fingerprinting-edge-mode-select"
              aria-label="Akamai influence mode"
              bind:value={edgeIntegrationMode}
              disabled={!akamaiBotSignalEnabled}
            >
              <option value="additive">additive</option>
              <option value="authoritative">authoritative</option>
            </select>
          </div>
          <p class="text-muted">Effective posture: {effectivePosture}</p>
          <div class="info-panel">
            <h4>Current Akamai Edge Contribution</h4>
            <div id="fingerprinting-akamai-signal-list">
              {#if akamaiEdgeAdditiveSignal}
                <div class="info-row">
                  <span class="info-label">{akamaiEdgeAdditiveSignal.label || 'Akamai edge bot signal (additive)'}</span>
                  <span>{akamaiEdgeAdditiveSignal.weight ?? '--'}</span>
                </div>
              {:else}
                <p class="text-muted">No Akamai additive edge signal definition is available.</p>
              {/if}
            </div>
            <p class="text-muted">This scored contribution is used only when Akamai bot signals are enabled and the edge mode is `additive`.</p>
          </div>
        </div>
      {:else}
        <p id="fingerprinting-akamai-unavailable-message" class="control-desc text-muted">Akamai bot-signal controls are available only when Shuma-Gorath is deployed on Akamai edge (`gateway_deployment_profile=edge-fermyon`). Shared-server and other non-edge postures keep this integration hidden.</p>
      {/if}
    </ConfigPanel>

    <ConfigPanel writable={true}>
      <ConfigPanelHeading title="Diagnostics">
        <span class="status-value text-muted">Read-only</span>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">Current runtime counters and active botness scoring signal definitions used for passive fingerprint corroboration.</p>
      <div class="admin-controls">
        <div class="info-panel">
          <h4>Runtime Counters</h4>
          <div class="info-row">
            <span class="info-label text-muted">Total detections</span>
            <span id="fingerprinting-total-detections">{Number(cdpStats.total_detections || 0)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Auto-bans</span>
            <span id="fingerprinting-auto-bans">{Number(cdpStats.auto_bans || 0)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">UA/client-hint mismatch</span>
            <span>{Number(fingerprintStats.ua_client_hint_mismatch || 0)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">UA/transport mismatch</span>
            <span>{Number(fingerprintStats.ua_transport_mismatch || 0)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Temporal transition</span>
            <span>{Number(fingerprintStats.temporal_transition || 0)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Flow violations</span>
            <span>{Number(fingerprintStats.flow_violation || 0)}</span>
          </div>
        </div>

        <div class="info-panel">
          <h4>Botness Scoring Signals</h4>
          <div id="fingerprinting-botness-signal-list">
            {#if fingerprintingBotnessSignals.length === 0}
              <p class="text-muted">No botness scoring signals available in the current scoring definition.</p>
            {:else}
              {#each fingerprintingBotnessSignals as signal}
                <div class="info-row">
                  <span class="info-label">{signal.label || signal.key || '--'}</span>
                  <span>{signal.weight ?? '--'}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    </ConfigPanel>

    {#if showAuthoritativeWarning}
      <p class="message warning">
        Authoritative mode should be enabled only when Akamai signals are trusted and monitored, because high-confidence edge outcomes can trigger direct ban actions.
      </p>
    {/if}

    <SaveChangesBar
      containerId="fingerprinting-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="fingerprinting-unsaved-summary"
      summaryText={saveFingerprintingSummary}
      summaryClass="text-unsaved-changes"
      invalidId="fingerprinting-invalid-summary"
      invalidText={saveFingerprintingInvalidText}
      buttonId="save-fingerprinting-config"
      buttonLabel={saveFingerprintingLabel}
      buttonDisabled={saveFingerprintingDisabled}
      onSave={saveFingerprintingConfig}
    />
  </div>
</section>
