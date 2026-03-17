<script>
  import { onMount } from 'svelte';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import {
    rateEnforcementEnabledFromMode,
    rateModeFromToggleState
  } from '../../domain/config-tab-helpers.js';
  import {
    isAdminConfigWritable,
    isAkamaiEdgeAvailable
  } from '../../domain/config-runtime.js';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import NumericInputRow from './primitives/NumericInputRow.svelte';
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
  let savingRateLimiting = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;
  let akamaiEdgeAvailable = false;

  let rateLimitingEnabled = true;
  let rateLimitThreshold = 80;
  let externalRateBackendEnabled = false;

  let baseline = {
    rate: {
      enabled: true,
      value: 80
    },
    externalBackend: {
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
    rateLimitThreshold = parseInteger(config.rate_limit, 80);
    rateLimitingEnabled = rateEnforcementEnabledFromMode(config?.defence_modes?.rate ?? 'both');
    externalRateBackendEnabled =
      String(config?.provider_backends?.rate_limiter || '').toLowerCase() === 'external';

    baseline = {
      rate: {
        enabled: rateLimitingEnabled === true,
        value: Number(rateLimitThreshold)
      },
      externalBackend: {
        enabled: externalRateBackendEnabled === true
      }
    };
  };

  async function saveRateLimitingConfig() {
    if (saveRateLimitingDisabled || typeof onSaveConfig !== 'function') return;
    savingRateLimiting = true;
    const payload = {};

    if (rateLimitDirty) {
      payload.rate_limit = Number(rateLimitThreshold);
      payload.defence_modes = {
        rate: rateModeNormalized
      };
    }
    if (externalRateBackendDirty) {
      payload.provider_backends = {
        rate_limiter: externalRateBackendEnabled ? 'external' : 'internal'
      };
    }

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Rate limiting settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      } else {
        baseline = {
          rate: {
            enabled: rateLimitingEnabled === true,
            value: Number(rateLimitThreshold)
          },
          externalBackend: {
            enabled: externalRateBackendEnabled === true
          }
        };
      }
    } finally {
      savingRateLimiting = false;
    }
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  $: rateModeNormalized = rateModeFromToggleState({
    enforcementEnabled: readBool(rateLimitingEnabled)
  });
  $: rateLimitValid = inRange(rateLimitThreshold, 1, 1000000);
  $: rateLimitDirty = (
    readBool(rateLimitingEnabled) !== baseline.rate.enabled ||
    Number(rateLimitThreshold) !== baseline.rate.value
  );
  $: externalRateBackendDirty =
    readBool(externalRateBackendEnabled) !== baseline.externalBackend.enabled;
  $: rateEdgeControlsVisible = akamaiEdgeAvailable === true;
  $: hasUnsavedChanges = rateLimitDirty || externalRateBackendDirty;
  $: rateLimitingValid = rateLimitValid;
  $: saveRateLimitingDisabled =
    !writable || !hasUnsavedChanges || !rateLimitingValid || savingRateLimiting;
  $: saveRateLimitingLabel = savingRateLimiting ? 'Saving...' : 'Save rate limiting settings';
  $: saveRateLimitingSummary = hasUnsavedChanges
    ? 'Rate limiting has unsaved changes'
    : 'No unsaved changes';
  $: saveRateLimitingInvalidText = rateLimitingValid
    ? ''
    : 'Requests per minute must be between 1 and 1,000,000.';
  $: warnOnUnload = writable && hasUnsavedChanges;
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingRateLimiting) {
        applyConfig(
          configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    }
  }
</script>

<section
  id="dashboard-panel-rate-limiting"
  class="admin-group"
  data-dashboard-tab-panel="rate-limiting"
  aria-labelledby="dashboard-tab-rate-limiting"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="rate-limiting" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigPanel writable={writable} dirty={externalRateBackendDirty}>
      <ConfigPanelHeading title="External Rate Limiter Backend">
        {#if rateEdgeControlsVisible}
          <label class="toggle-switch" for="rate-external-backend-enabled-toggle">
            <input
              type="checkbox"
              id="rate-external-backend-enabled-toggle"
              aria-label="Enable external rate limiter backend"
              bind:checked={externalRateBackendEnabled}
            >
            <span class="toggle-slider"></span>
          </label>
        {/if}
      </ConfigPanelHeading>
      {#if rateEdgeControlsVisible}
        <p class="control-desc text-muted">Enable this when the deployment provides a trusted distributed rate-limiter backend path. Today this switches Shuma-Gorath between its internal limiter and the external provider backend on <code>provider_backends.rate_limiter</code>; it is not yet direct Akamai rate-signal ingestion.</p>
      {:else}
        <p id="rate-edge-unavailable-message" class="control-desc text-muted">External rate-backend controls are available only when Shuma-Gorath is deployed on Akamai edge (`gateway_deployment_profile=edge-fermyon`). Shared-server and other non-edge postures keep this integration hidden.</p>
      {/if}
    </ConfigPanel>

    <ConfigPanel writable={writable} dirty={rateLimitDirty}>
      <ConfigPanelHeading title="Rate Limiting">
        <label class="toggle-switch" for="rate-limiting-enabled-toggle">
          <input
            type="checkbox"
            id="rate-limiting-enabled-toggle"
            aria-label="Enable rate limiting"
            bind:checked={rateLimitingEnabled}
          >
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">Define the allowed requests per minute per <abbr title="Internet Protocol">IP</abbr> bucket (<abbr title="Internet Protocol Version 4">IPv4</abbr> /24, <abbr title="Internet Protocol Version 6">IPv6</abbr> /64), not a single host <abbr title="Internet Protocol">IP</abbr>. Default budget is <code>80</code> requests; lower values are more strict but can affect legitimate burst traffic and innocent visitors when the budget of their <abbr title="Internet Protocol">IP</abbr> bucket is exhausted by a malicious bot.</p>
      <div class="admin-controls">
        <NumericInputRow
          id="rate-limit-threshold"
          label='Requests Per Minute (per <abbr title="Internet Protocol">IP</abbr> bucket)'
          labelClass="control-label control-label--wide"
          min="1"
          max="1000000"
          step="1"
          inputmode="numeric"
          ariaLabel="Rate limit requests per minute"
          ariaInvalid={rateLimitValid ? 'false' : 'true'}
          bind:value={rateLimitThreshold}
        />
      </div>
      {#if !rateLimitingEnabled}
        <p class="message warning">
          Rate limiting is strongly advised. Disable only if upstream already enforces it or for temporary
          testing. Scoring still stays active.
        </p>
      {/if}
    </ConfigPanel>

    <SaveChangesBar
      containerId="rate-limiting-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="rate-limiting-unsaved-summary"
      summaryText={saveRateLimitingSummary}
      summaryClass="text-unsaved-changes"
      invalidId="rate-limiting-invalid-summary"
      invalidText={saveRateLimitingInvalidText}
      buttonId="save-rate-limiting-config"
      buttonLabel={saveRateLimitingLabel}
      buttonDisabled={saveRateLimitingDisabled}
      onSave={saveRateLimitingConfig}
    />
  </div>
</section>
