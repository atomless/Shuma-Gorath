<script>
  import { onMount } from 'svelte';
  import { normalizeEdgeMode } from '../../domain/config-tab-helpers.js';
  import { parseFloatNumber } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import ToggleRow from './primitives/ToggleRow.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let cdpSnapshot = null;
  export let onSaveConfig = null;

  let writable = false;
  let savingFingerprinting = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let cdpEnabled = true;
  let cdpAutoBan = true;
  let cdpThreshold = 0.6;
  let edgeIntegrationMode = 'off';
  let fingerprintProviderBackend = 'internal';

  let baseline = {
    cdp: {
      enabled: true,
      autoBan: true,
      threshold: 0.6
    },
    edgeMode: {
      mode: 'off'
    },
    providerBackend: {
      fingerprintSignal: 'internal'
    }
  };

  const readBool = (value) => value === true;
  const toProviderBackend = (value) => (String(value || '').toLowerCase() === 'external' ? 'external' : 'internal');

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  function applyConfig(config = {}) {
    writable = config.admin_config_write_enabled === true;

    cdpEnabled = config.cdp_detection_enabled !== false;
    cdpAutoBan = config.cdp_auto_ban !== false;
    cdpThreshold = Number(parseFloatNumber(config.cdp_detection_threshold, 0.6).toFixed(1));
    edgeIntegrationMode = normalizeEdgeMode(config.edge_integration_mode);
    fingerprintProviderBackend = toProviderBackend(config?.provider_backends?.fingerprint_signal);

    baseline = {
      cdp: {
        enabled: cdpEnabled,
        autoBan: cdpAutoBan,
        threshold: Number(cdpThreshold)
      },
      edgeMode: {
        mode: edgeIntegrationMode
      },
      providerBackend: {
        fingerprintSignal: fingerprintProviderBackend
      }
    };
  }

  async function saveFingerprintingConfig() {
    if (saveFingerprintingDisabled || typeof onSaveConfig !== 'function') return;

    savingFingerprinting = true;
    const payload = {
      cdp_detection_enabled: cdpEnabled === true,
      cdp_auto_ban: cdpAutoBan === true,
      cdp_detection_threshold: Number(cdpThreshold),
      edge_integration_mode: normalizeEdgeMode(edgeIntegrationMode),
      provider_backends: {
        fingerprint_signal: toProviderBackend(fingerprintProviderBackend)
      }
    };

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Fingerprinting settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      } else {
        baseline = {
          cdp: {
            enabled: cdpEnabled === true,
            autoBan: cdpAutoBan === true,
            threshold: Number(cdpThreshold)
          },
          edgeMode: {
            mode: normalizeEdgeMode(edgeIntegrationMode)
          },
          providerBackend: {
            fingerprintSignal: toProviderBackend(fingerprintProviderBackend)
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

  $: cdpValid = inRange(cdpThreshold, 0.3, 1.0);
  $: edgeModeValid = ['off', 'advisory', 'authoritative'].includes(normalizeEdgeMode(edgeIntegrationMode));
  $: fingerprintProviderBackendValid = ['internal', 'external'].includes(toProviderBackend(fingerprintProviderBackend));
  $: fingerprintingValid = cdpValid && edgeModeValid && fingerprintProviderBackendValid;

  $: cdpDirty = (
    readBool(cdpEnabled) !== baseline.cdp.enabled ||
    readBool(cdpAutoBan) !== baseline.cdp.autoBan ||
    Number(cdpThreshold) !== baseline.cdp.threshold
  );
  $: edgeModeDirty = normalizeEdgeMode(edgeIntegrationMode) !== baseline.edgeMode.mode;
  $: providerBackendDirty = toProviderBackend(fingerprintProviderBackend) !== baseline.providerBackend.fingerprintSignal;
  $: hasUnsavedChanges = cdpDirty || edgeModeDirty || providerBackendDirty;

  $: saveFingerprintingDisabled = !writable || !hasUnsavedChanges || !fingerprintingValid || savingFingerprinting;
  $: saveFingerprintingLabel = savingFingerprinting ? 'Saving...' : 'Save fingerprinting settings';
  $: saveFingerprintingSummary = hasUnsavedChanges
    ? 'Fingerprinting has unsaved changes'
    : 'No unsaved changes';
  $: saveFingerprintingInvalidText = fingerprintingValid
    ? ''
    : 'Fix invalid fingerprinting values before saving.';
  $: warnOnUnload = writable && hasUnsavedChanges;

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
  $: fingerprintSignals = scoredSignals.filter((signal) => String(signal?.key || '').startsWith('fp_'));

  $: effectivePosture = (() => {
    if (!cdpEnabled) return 'Detection is disabled.';
    if (!cdpAutoBan) return 'Detection and logging only. Auto-ban is disabled.';
    if (toProviderBackend(fingerprintProviderBackend) === 'external' && normalizeEdgeMode(edgeIntegrationMode) === 'authoritative') {
      return `Strong external edge outcomes can trigger immediate bans at threshold ${Number(cdpThreshold).toFixed(1)}.`;
    }
    return `Detection and auto-ban enabled at threshold ${Number(cdpThreshold).toFixed(1)}.`;
  })();

  $: showStrictComboWarning = (
    cdpEnabled &&
    cdpAutoBan &&
    toProviderBackend(fingerprintProviderBackend) === 'external' &&
    normalizeEdgeMode(edgeIntegrationMode) === 'authoritative' &&
    Number(cdpThreshold) <= 0.5
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
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={hasUnsavedChanges}
  data-dashboard-tab-panel="fingerprinting"
  aria-labelledby="dashboard-tab-fingerprinting"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="fingerprinting" status={tabStatus} />
  <div class="controls-grid controls-grid--config">
    <ConfigPanel writable={writable} dirty={edgeModeDirty || providerBackendDirty}>
      <ConfigPanelHeading title="Edge and Provider">
        <span class="status-value text-muted">External integration</span>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">When calculating bot fingerprinting, edge hosting platforms can use transport/network-layer telemetry (for example TLS/client fingerprinting, network reputation, global bot intel). By enabling edge integration Shuma-Gorath can thereby take into account and optionally act on signals that it cannot otherwise directly observe. Supported external fingerprint providers currently: ["Akamai"].</p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label control-label--wide" for="fingerprinting-provider-backend-select">Fingerprint Provider Backend</label>
          <select
            class="input-field input-row-control"
            id="fingerprinting-provider-backend-select"
            aria-label="Fingerprint provider backend"
            bind:value={fingerprintProviderBackend}
          >
            <option value="internal">internal</option>
            <option value="external">external (Akamai)</option>
          </select>
        </div>
        <div class="input-row">
          <label class="control-label control-label--wide" for="fingerprinting-edge-mode-select">Edge Integration Mode</label>
          <select
            class="input-field input-row-control"
            id="fingerprinting-edge-mode-select"
            aria-label="Edge integration mode"
            bind:value={edgeIntegrationMode}
          >
            <option value="off">off</option>
            <option value="advisory">advisory</option>
            <option value="authoritative">authoritative</option>
          </select>
        </div>
        <p class="text-muted">Effective posture: {effectivePosture}</p>
      </div>
    </ConfigPanel>

    <ConfigPanel writable={writable} dirty={cdpDirty}>
      <ConfigPanelHeading title='<abbr title="Chrome DevTools Protocol">CDP</abbr> Detection'>
        <label class="toggle-switch" for="fingerprinting-cdp-enabled-toggle">
          <input
            type="checkbox"
            id="fingerprinting-cdp-enabled-toggle"
            aria-label="Enable Chrome DevTools Protocol detection"
            bind:checked={cdpEnabled}
          >
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">Detect browser automation using <abbr title="Chrome DevTools Protocol">CDP</abbr> signals, then optionally auto-ban based on confidence threshold.</p>
      <div class="admin-controls">
        <ToggleRow
          id="fingerprinting-cdp-auto-ban-toggle"
          label="Auto-ban on Detection"
          labelClass="control-label control-label--wide"
          ariaLabel="Enable Chrome DevTools Protocol auto-ban"
          bind:checked={cdpAutoBan}
        />
        <div class="slider-control">
          <div class="slider-header">
            <label class="control-label control-label--wide" for="fingerprinting-cdp-threshold-slider">Detection Threshold</label>
            <span id="fingerprinting-cdp-threshold-value" class="slider-badge">{Number(cdpThreshold).toFixed(1)}</span>
          </div>
          <input
            type="range"
            id="fingerprinting-cdp-threshold-slider"
            min="0.3"
            max="1.0"
            step="0.1"
            aria-label="Chrome DevTools Protocol detection threshold"
            aria-invalid={cdpValid ? 'false' : 'true'}
            bind:value={cdpThreshold}
          >
          <div class="slider-labels">
            <span>Strict</span>
            <span>Permissive</span>
          </div>
        </div>
      </div>
    </ConfigPanel>

    <ConfigPanel writable={true}>
      <ConfigPanelHeading title="Diagnostics">
        <span class="status-value text-muted">Read-only</span>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">Current runtime counters and active fingerprint signal definitions.</p>
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
          <h4>Scored Fingerprint Signals</h4>
          {#if fingerprintSignals.length === 0}
            <p class="text-muted">No fingerprint signals available in the current scoring definition.</p>
          {:else}
            {#each fingerprintSignals as signal}
              <div class="info-row">
                <span class="info-label">{signal.label || signal.key || '--'}</span>
                <span>{signal.weight ?? '--'}</span>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </ConfigPanel>

    {#if showStrictComboWarning}
      <p class="message warning">
        This combination is strict: external provider + authoritative edge mode + low threshold can increase false positives.
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
