<script>
  import ConfigChallengeSection from './config/ConfigChallengeSection.svelte';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import NumericInputRow from './primitives/NumericInputRow.svelte';
  import { onMount } from 'svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import { isAdminConfigWritable } from '../../domain/config-runtime.js';
  import { parseFloatNumber, parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import ToggleRow from './primitives/ToggleRow.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let operatorSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  let writable = false;
  let hasConfigSnapshot = false;
  let lastAppliedConfigVersion = -1;
  let deferredConfigApply = false;
  let savingAll = false;
  let warnOnUnload = false;

  let jsRequiredEnforced = true;
  let cdpDetectionEnabled = true;
  let cdpAutoBan = true;
  let cdpDetectionThreshold = 0.6;

  let powEnabled = true;
  let powDifficulty = 15;
  let powTtl = 90;

  let challengePuzzleEnabled = true;
  let notABotEnabled = true;
  let notABotScorePassMin = 7;
  let notABotScoreFailMax = 3;
  let notABotNonceTtl = 120;
  let notABotMarkerTtl = 600;
  let notABotAttemptLimit = 6;
  let notABotAttemptWindow = 300;
  let verifiedIdentityEnabled = false;
  let verifiedIdentityNativeWebBotAuthEnabled = false;
  let verifiedIdentityProviderAssertionsEnabled = true;
  let verifiedIdentityReplayWindowSeconds = 120;
  let verifiedIdentityClockSkewSeconds = 30;
  let verifiedIdentityDirectoryCacheTtlSeconds = 3600;
  let verifiedIdentityDirectoryFreshnessRequirementSeconds = 86400;

  let baseline = {
    jsRequired: { enforced: true },
    cdp: { enabled: true, autoBan: true, threshold: 0.6 },
    pow: { enabled: true, difficulty: 15, ttl: 90 },
    challenge: { enabled: true },
    notABot: {
      enabled: true,
      scorePassMin: 7,
      scoreFailMax: 3,
      nonceTtl: 120,
      markerTtl: 600,
      attemptLimit: 6,
      attemptWindow: 300
    },
    verifiedIdentity: {
      enabled: false,
      nativeWebBotAuthEnabled: false,
      providerAssertionsEnabled: true,
      replayWindowSeconds: 120,
      clockSkewSeconds: 30,
      directoryCacheTtlSeconds: 3600,
      directoryFreshnessRequirementSeconds: 86400
    }
  };

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

  function applyConfig(config = {}, runtime = {}) {
    hasConfigSnapshot = config && typeof config === 'object' && Object.keys(config).length > 0;
    writable = isAdminConfigWritable(runtime);

    jsRequiredEnforced = config.js_required_enforced !== false;
    cdpDetectionEnabled = config.cdp_detection_enabled !== false;
    cdpAutoBan = config.cdp_auto_ban !== false;
    cdpDetectionThreshold = Number(parseFloatNumber(config.cdp_detection_threshold, 0.6).toFixed(1));

    powEnabled = config.pow_enabled !== false;
    powDifficulty = parseInteger(config.pow_difficulty, 15);
    powTtl = parseInteger(config.pow_ttl_seconds, 90);

    challengePuzzleEnabled = config.challenge_puzzle_enabled !== false;
    notABotEnabled = config.not_a_bot_enabled !== false;
    notABotScorePassMin = parseInteger(config.not_a_bot_pass_score, 7);
    notABotScoreFailMax = Math.max(0, Math.min(9, parseInteger(config.not_a_bot_fail_score, 4) - 1));
    notABotNonceTtl = parseInteger(config.not_a_bot_nonce_ttl_seconds, 120);
    notABotMarkerTtl = parseInteger(config.not_a_bot_marker_ttl_seconds, 600);
    notABotAttemptLimit = parseInteger(config.not_a_bot_attempt_limit_per_window, 6);
    notABotAttemptWindow = parseInteger(config.not_a_bot_attempt_window_seconds, 300);
    const verifiedIdentityConfig = config.verified_identity && typeof config.verified_identity === 'object'
      ? config.verified_identity
      : {};
    verifiedIdentityEnabled = verifiedIdentityConfig.enabled === true;
    verifiedIdentityNativeWebBotAuthEnabled = verifiedIdentityConfig.native_web_bot_auth_enabled === true;
    verifiedIdentityProviderAssertionsEnabled = verifiedIdentityConfig.provider_assertions_enabled !== false;
    verifiedIdentityReplayWindowSeconds = parseInteger(
      verifiedIdentityConfig.replay_window_seconds,
      120
    );
    verifiedIdentityClockSkewSeconds = parseInteger(
      verifiedIdentityConfig.clock_skew_seconds,
      30
    );
    verifiedIdentityDirectoryCacheTtlSeconds = parseInteger(
      verifiedIdentityConfig.directory_cache_ttl_seconds,
      3600
    );
    verifiedIdentityDirectoryFreshnessRequirementSeconds = parseInteger(
      verifiedIdentityConfig.directory_freshness_requirement_seconds,
      86400
    );

    baseline = {
      jsRequired: { enforced: jsRequiredEnforced },
      cdp: {
        enabled: cdpDetectionEnabled,
        autoBan: cdpAutoBan,
        threshold: Number(cdpDetectionThreshold)
      },
      pow: {
        enabled: powEnabled,
        difficulty: Number(powDifficulty),
        ttl: Number(powTtl)
      },
      challenge: {
        enabled: challengePuzzleEnabled
      },
      notABot: {
        enabled: notABotEnabled,
        scorePassMin: Number(notABotScorePassMin),
        scoreFailMax: Number(notABotScoreFailMax),
        nonceTtl: Number(notABotNonceTtl),
        markerTtl: Number(notABotMarkerTtl),
        attemptLimit: Number(notABotAttemptLimit),
        attemptWindow: Number(notABotAttemptWindow)
      },
      verifiedIdentity: {
        enabled: verifiedIdentityEnabled,
        nativeWebBotAuthEnabled: verifiedIdentityNativeWebBotAuthEnabled,
        providerAssertionsEnabled: verifiedIdentityProviderAssertionsEnabled,
        replayWindowSeconds: Number(verifiedIdentityReplayWindowSeconds),
        clockSkewSeconds: Number(verifiedIdentityClockSkewSeconds),
        directoryCacheTtlSeconds: Number(verifiedIdentityDirectoryCacheTtlSeconds),
        directoryFreshnessRequirementSeconds: Number(verifiedIdentityDirectoryFreshnessRequirementSeconds)
      }
    };

  }

  const buildConfigPatch = ({ includeAll = false } = {}) => {
    const patch = {};
    if (includeAll || jsRequiredDirty) {
      patch.js_required_enforced = jsRequiredEnforced;
    }
    if (includeAll || cdpDirty) {
      patch.cdp_detection_enabled = cdpDetectionEnabled === true;
      patch.cdp_auto_ban = cdpAutoBan === true;
      patch.cdp_detection_threshold = Number(cdpDetectionThreshold);
    }
    if (includeAll || powDirty) {
      patch.pow_enabled = powEnabled;
      if (includeAll) {
        patch.pow_difficulty = Number(powDifficulty);
        patch.pow_ttl_seconds = Number(powTtl);
      }
    }
    if (includeAll || challengePuzzleDirty) {
      patch.challenge_puzzle_enabled = challengePuzzleEnabled;
    }
    if (includeAll || notABotDirty) {
      patch.not_a_bot_enabled = notABotEnabled;
      patch.not_a_bot_pass_score = Number(notABotScorePassMin);
      patch.not_a_bot_fail_score = Number(notABotScoreFailMax) + 1;
      if (includeAll) {
        patch.not_a_bot_nonce_ttl_seconds = Number(notABotNonceTtl);
        patch.not_a_bot_marker_ttl_seconds = Number(notABotMarkerTtl);
        patch.not_a_bot_attempt_limit_per_window = Number(notABotAttemptLimit);
        patch.not_a_bot_attempt_window_seconds = Number(notABotAttemptWindow);
      }
    }
    if (includeAll || verifiedIdentityDirty) {
      patch.verified_identity = {
        enabled: verifiedIdentityEnabled === true,
        native_web_bot_auth_enabled: verifiedIdentityNativeWebBotAuthEnabled === true,
        provider_assertions_enabled: verifiedIdentityProviderAssertionsEnabled === true,
        replay_window_seconds: Number(verifiedIdentityReplayWindowSeconds),
        clock_skew_seconds: Number(verifiedIdentityClockSkewSeconds),
        directory_cache_ttl_seconds: Number(verifiedIdentityDirectoryCacheTtlSeconds),
        directory_freshness_requirement_seconds: Number(verifiedIdentityDirectoryFreshnessRequirementSeconds)
      };
    }
    return patch;
  };

  async function saveAllConfig() {
    if (saveAllConfigDisabled || typeof onSaveConfig !== 'function') return;

    const patch = buildConfigPatch({ includeAll: false });

    if (Object.keys(patch).length === 0) return;

    savingAll = true;
    try {
      const successMessage = dirtySectionCount === 1
        ? `${dirtySectionLabels[0]} saved`
        : `Saved ${dirtySectionCount} configuration sections`;
      const nextConfig = await onSaveConfig(patch, { successMessage });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    } finally {
      savingAll = false;
    }
  }

  const readBool = (value) => value === true;

  $: jsRequiredDirty = readBool(jsRequiredEnforced) !== baseline.jsRequired.enforced;

  $: cdpThresholdValid = inRange(cdpDetectionThreshold, 0.3, 1.0);
  $: cdpValid = cdpThresholdValid;
  $: cdpDirty = (
    readBool(cdpDetectionEnabled) !== baseline.cdp.enabled ||
    readBool(cdpAutoBan) !== baseline.cdp.autoBan ||
    Number(cdpDetectionThreshold) !== baseline.cdp.threshold
  );

  $: powValid = true;
  $: powDirty = readBool(powEnabled) !== baseline.pow.enabled;

  $: challengePuzzleValid = true;
  $: challengePuzzleDirty = (
    readBool(challengePuzzleEnabled) !== baseline.challenge.enabled
  );

  $: notABotScoreFailMaxCap = Math.max(0, Number(notABotScorePassMin) - 1);
  $: if (Number(notABotScoreFailMax) > notABotScoreFailMaxCap) {
    notABotScoreFailMax = notABotScoreFailMaxCap;
  }
  $: notABotScorePassMinFloor = Math.min(10, Number(notABotScoreFailMax) + 1);

  $: notABotPassScoreValid = (
    inRange(notABotScorePassMin, 1, 10) &&
    Number(notABotScorePassMin) > Number(notABotScoreFailMax)
  );
  $: notABotFailScoreValid = (
    inRange(notABotScoreFailMax, 0, 9) &&
    Number(notABotScoreFailMax) < Number(notABotScorePassMin)
  );
  $: notABotValid = (
    notABotPassScoreValid &&
    notABotFailScoreValid
  );
  $: notABotDirty = (
    readBool(notABotEnabled) !== baseline.notABot.enabled ||
    Number(notABotScorePassMin) !== baseline.notABot.scorePassMin ||
    Number(notABotScoreFailMax) !== baseline.notABot.scoreFailMax
  );

  $: verifiedIdentityVerifierPathValid = (
    readBool(verifiedIdentityEnabled) !== true ||
    readBool(verifiedIdentityNativeWebBotAuthEnabled) === true ||
    readBool(verifiedIdentityProviderAssertionsEnabled) === true
  );
  $: verifiedIdentityReplayWindowValid = inRange(verifiedIdentityReplayWindowSeconds, 30, 3600);
  $: verifiedIdentityClockSkewValid = (
    inRange(verifiedIdentityClockSkewSeconds, 0, 300) &&
    Number(verifiedIdentityClockSkewSeconds) <= Number(verifiedIdentityReplayWindowSeconds)
  );
  $: verifiedIdentityDirectoryCacheTtlValid = inRange(verifiedIdentityDirectoryCacheTtlSeconds, 60, 86400);
  $: verifiedIdentityDirectoryFreshnessRequirementValid = inRange(
    verifiedIdentityDirectoryFreshnessRequirementSeconds,
    60,
    604800
  );
  $: verifiedIdentityValid = (
    verifiedIdentityVerifierPathValid &&
    verifiedIdentityReplayWindowValid &&
    verifiedIdentityClockSkewValid &&
    verifiedIdentityDirectoryCacheTtlValid &&
    verifiedIdentityDirectoryFreshnessRequirementValid
  );
  $: verifiedIdentityDirty = (
    readBool(verifiedIdentityEnabled) !== baseline.verifiedIdentity.enabled ||
    readBool(verifiedIdentityNativeWebBotAuthEnabled) !== baseline.verifiedIdentity.nativeWebBotAuthEnabled ||
    readBool(verifiedIdentityProviderAssertionsEnabled) !== baseline.verifiedIdentity.providerAssertionsEnabled ||
    Number(verifiedIdentityReplayWindowSeconds) !== baseline.verifiedIdentity.replayWindowSeconds ||
    Number(verifiedIdentityClockSkewSeconds) !== baseline.verifiedIdentity.clockSkewSeconds ||
    Number(verifiedIdentityDirectoryCacheTtlSeconds) !== baseline.verifiedIdentity.directoryCacheTtlSeconds ||
    Number(verifiedIdentityDirectoryFreshnessRequirementSeconds) !== baseline.verifiedIdentity.directoryFreshnessRequirementSeconds
  );

  const readOperatorSnapshotVerifiedIdentity = (snapshot) => {
    if (!snapshot || typeof snapshot !== 'object') return {};
    return snapshot.verified_identity && typeof snapshot.verified_identity === 'object'
      ? snapshot.verified_identity
      : {};
  };
  const readCountEntries = (value) => (
    Array.isArray(value)
      ? value.filter((entry) => entry && typeof entry === 'object')
      : []
  );
  const formatAvailability = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'supported') return 'Supported';
    if (normalized === 'not_configured') return 'Not configured';
    return normalized ? normalized.replace(/_/g, ' ') : '-';
  };
  const formatCountEntryLabel = (value) =>
    String(value || '')
      .split('_')
      .filter(Boolean)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ') || '-';

  $: verifiedIdentitySummary = readOperatorSnapshotVerifiedIdentity(operatorSnapshot);
  $: verifiedIdentityTopFailureReasons = readCountEntries(verifiedIdentitySummary.top_failure_reasons);
  $: verifiedIdentityTopSchemes = readCountEntries(verifiedIdentitySummary.top_schemes);
  $: verifiedIdentityTopCategories = readCountEntries(verifiedIdentitySummary.top_categories);
  $: verifiedIdentitySummaryAvailable = Object.keys(verifiedIdentitySummary).length > 0;

  $: dirtySections = [
    { label: 'JavaScript required', dirty: jsRequiredDirty, valid: true },
    { label: 'Internal CDP probe', dirty: cdpDirty, valid: cdpValid },
    { label: 'Proof of Work', dirty: powDirty, valid: powValid },
    { label: 'Challenge puzzle', dirty: challengePuzzleDirty, valid: challengePuzzleValid },
    { label: 'Not-a-Bot', dirty: notABotDirty, valid: notABotValid },
    { label: 'Verified Identity', dirty: verifiedIdentityDirty, valid: verifiedIdentityValid }
  ];
  $: dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
  $: invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
  $: dirtySectionLabels = dirtySectionEntries.map((section) => section.label);
  $: invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
  $: dirtySectionCount = dirtySectionEntries.length;
  $: hasUnsavedChanges = dirtySectionCount > 0;
  $: hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
  $: saveAllConfigDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingAll;
  $: saveAllConfigLabel = savingAll ? 'Saving...' : 'Save all changes';
  $: saveAllSummaryText = hasUnsavedChanges
    ? `${dirtySectionCount} section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
    : 'No unsaved changes';
  $: saveAllInvalidText = hasInvalidUnsavedChanges
    ? `Fix invalid values in: ${invalidDirtySectionLabels.join(', ')}`
    : '';
  $: warnOnUnload = writable && hasUnsavedChanges;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (hasUnsavedChanges && !savingAll) {
        deferredConfigApply = true;
      } else {
        applyConfig(
          configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    }
  }

  $: if (deferredConfigApply && !hasUnsavedChanges && !savingAll) {
    deferredConfigApply = false;
    applyConfig(
      configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
      configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
    );
  }
</script>

<section
  id="dashboard-panel-verification"
  class="admin-group"
  data-dashboard-tab-panel="verification"
  aria-labelledby="dashboard-tab-verification"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="verification" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigPanel writable={writable} dirty={jsRequiredDirty}>
      <ConfigPanelHeading title='<abbr title="JavaScript">JS</abbr> Required'>
        <label class="toggle-switch" for="js-required-enforced-toggle">
          <input type="checkbox" id="js-required-enforced-toggle" aria-label="Enforce JavaScript required" bind:checked={jsRequiredEnforced}>
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">Require non-allowlisted requests to present a valid <code>js_verified</code> cookie. The presence of this cookie is verification that <abbr title="JavaScript">JS</abbr> is enabled. With Shuma-Gorath&rsquo;s <abbr title="Proof of Work">PoW</abbr> requirement also enabled, the cookie is set by the server after <code>/pow/verify</code>; with <abbr title="Proof of Work">PoW</abbr> disabled, it is set directly by the interstitial script. Disable only for non-<abbr title="JavaScript">JS</abbr> clients.</p>
      {#if !jsRequiredEnforced}
        <p class="message warning">
          Disabling JS Required weakens bot defence and bypasses both <abbr title="Proof of Work">PoW</abbr> and the JS Verification Interstitial.
        </p>
      {/if}
    </ConfigPanel>

    <ConfigPanel writable={writable} dirty={cdpDirty}>
      <ConfigPanelHeading title='Browser <abbr title="Chrome DevTools Protocol">CDP</abbr> Automation Probe'>
        <label class="toggle-switch" for="verification-cdp-enabled-toggle">
          <input
            type="checkbox"
            id="verification-cdp-enabled-toggle"
            aria-label="Enable Browser CDP Automation Detection"
            bind:checked={cdpDetectionEnabled}
            disabled={!jsRequiredEnforced}
          >
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">When this is enabled, the JS Verification Interstitial injects a browser <abbr title="Chrome DevTools Protocol">CDP</abbr> automation detection probe that calculates an automation score --the higher the scores the higher the certainty that the visitor is a form of browser automation.</p>
      <div class="admin-controls">
        <ToggleRow
          id="verification-cdp-auto-ban-toggle"
          label="Auto-ban on Strong Detection"
          labelClass="control-label control-label--wide"
          ariaLabel="Enable browser automation auto-ban"
          bind:checked={cdpAutoBan}
          disabled={!jsRequiredEnforced || !cdpDetectionEnabled}
          rowClass={!jsRequiredEnforced || !cdpDetectionEnabled ? 'toggle-row--disabled' : ''}
        />
        <div class="slider-control" class:slider-control--disabled={!jsRequiredEnforced || !cdpDetectionEnabled}>
          <div class="slider-header">
            <label class="control-label control-label--wide" for="verification-cdp-threshold-slider">Detection Threshold</label>
            <span id="verification-cdp-threshold-value" class="slider-badge">{Number(cdpDetectionThreshold).toFixed(1)}</span>
          </div>
          <input
            type="range"
            id="verification-cdp-threshold-slider"
            min="0.3"
            max="1.0"
            step="0.1"
            aria-label="Internal browser CDP detection threshold"
            aria-invalid={cdpThresholdValid ? 'false' : 'true'}
            bind:value={cdpDetectionThreshold}
            disabled={!jsRequiredEnforced || !cdpDetectionEnabled}
          >
          <div class="slider-labels">
            <span>Strict</span>
            <span>Permissive</span>
          </div>
        </div>
      </div>
      {#if !jsRequiredEnforced}
        <p class="message warning">
          JS Required is disabled, so the browser <abbr title="Chrome DevTools Protocol">CDP</abbr> automation probe is inactive and these controls are disabled.
        </p>
      {/if}
    </ConfigPanel>

    <ConfigPanel writable={writable} dirty={powDirty}>
      <ConfigPanelHeading title='Proof-of-Work (<abbr title="Proof of Work">PoW</abbr>)'>
        <label class="toggle-switch" for="pow-enabled-toggle">
          <input type="checkbox" id="pow-enabled-toggle" aria-label="Enable Proof of Work challenge verification" bind:checked={powEnabled}>
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">Similarly injected by the JS Verification Interstitial, <abbr title="Proof of Work">PoW</abbr> is a security mechanism used to help differentiate bots from humans by requiring the requesting client's device to solve a small, moderately complex computational puzzle before being granted access. It will be invisible to human users and incurrs only extremely low energy and request performance costs. <abbr title="Proof of Work">PoW</abbr> depends on <abbr title="JavaScript">JS</abbr> Required being enabled.</p>
      {#if !jsRequiredEnforced}
        <p id="verification-pow-js-required-warning" class="message warning">
          JS Required is disabled, so <abbr title="Proof of Work">PoW</abbr> is inactive. These settings stay saved and apply again when <abbr title="JavaScript">JS</abbr> Required is re-enabled.
        </p>
      {/if}
    </ConfigPanel>

    <ConfigChallengeSection bind:writable bind:notABotDirty bind:challengePuzzleDirty bind:notABotEnabled bind:challengePuzzleEnabled bind:notABotScorePassMinFloor bind:notABotScorePassMin bind:notABotScoreFailMaxCap bind:notABotScoreFailMax notABotPassScoreValid={notABotPassScoreValid} notABotFailScoreValid={notABotFailScoreValid} />

    <ConfigPanel writable={writable} dirty={verifiedIdentityDirty}>
      <ConfigPanelHeading title="Verified Identity">
        <label class="toggle-switch" for="verified-identity-enabled-toggle">
          <input type="checkbox" id="verified-identity-enabled-toggle" aria-label="Enable verified identity" bind:checked={verifiedIdentityEnabled}>
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">
        Configure verified-identity mechanics for native Web Bot Auth and trusted provider assertions. Policy posture and richer category rules stay out of this pane until the dedicated editor lands.
      </p>
      <div class="admin-controls">
        <ToggleRow
          id="verified-identity-native-web-bot-auth-toggle"
          label="Native Web Bot Auth"
          labelClass="control-label control-label--wide"
          ariaLabel="Enable native Web Bot Auth verification"
          bind:checked={verifiedIdentityNativeWebBotAuthEnabled}
          disabled={!verifiedIdentityEnabled}
          rowClass={!verifiedIdentityEnabled ? 'toggle-row--disabled' : ''}
        />
        <ToggleRow
          id="verified-identity-provider-assertions-toggle"
          label="Provider Assertions"
          labelClass="control-label control-label--wide"
          ariaLabel="Enable provider assertion verification"
          bind:checked={verifiedIdentityProviderAssertionsEnabled}
          disabled={!verifiedIdentityEnabled}
          rowClass={!verifiedIdentityEnabled ? 'toggle-row--disabled' : ''}
        />
        <NumericInputRow
          id="verified-identity-replay-window"
          label="Replay Window (seconds)"
          labelClass="control-label control-label--wide"
          min="30"
          max="3600"
          step="1"
          inputmode="numeric"
          ariaLabel="Verified identity replay window seconds"
          ariaInvalid={verifiedIdentityReplayWindowValid ? 'false' : 'true'}
          bind:value={verifiedIdentityReplayWindowSeconds}
          disabled={!verifiedIdentityEnabled}
        />
        <NumericInputRow
          id="verified-identity-clock-skew"
          label="Clock Skew (seconds)"
          labelClass="control-label control-label--wide"
          min="0"
          max="300"
          step="1"
          inputmode="numeric"
          ariaLabel="Verified identity clock skew seconds"
          ariaInvalid={verifiedIdentityClockSkewValid ? 'false' : 'true'}
          bind:value={verifiedIdentityClockSkewSeconds}
          disabled={!verifiedIdentityEnabled}
        />
        <NumericInputRow
          id="verified-identity-directory-cache-ttl"
          label="Directory Cache TTL (seconds)"
          labelClass="control-label control-label--wide"
          min="60"
          max="86400"
          step="1"
          inputmode="numeric"
          ariaLabel="Verified identity directory cache time to live seconds"
          ariaInvalid={verifiedIdentityDirectoryCacheTtlValid ? 'false' : 'true'}
          bind:value={verifiedIdentityDirectoryCacheTtlSeconds}
          disabled={!verifiedIdentityEnabled}
        />
        <NumericInputRow
          id="verified-identity-directory-freshness-requirement"
          label="Directory Freshness Requirement (seconds)"
          labelClass="control-label control-label--wide"
          min="60"
          max="604800"
          step="1"
          inputmode="numeric"
          ariaLabel="Verified identity directory freshness requirement seconds"
          ariaInvalid={verifiedIdentityDirectoryFreshnessRequirementValid ? 'false' : 'true'}
          bind:value={verifiedIdentityDirectoryFreshnessRequirementSeconds}
          disabled={!verifiedIdentityEnabled}
        />
      </div>
      {#if verifiedIdentityEnabled && !verifiedIdentityVerifierPathValid}
        <p id="verified-identity-verifier-warning" class="message warning">
          Verified identity must keep at least one verifier path enabled: Native Web Bot Auth or Provider Assertions.
        </p>
      {/if}
      {#if verifiedIdentityEnabled && !verifiedIdentityClockSkewValid}
        <p id="verified-identity-clock-skew-warning" class="message warning">
          Clock skew must stay between 0 and 300 seconds and must not exceed the replay window.
        </p>
      {/if}
    </ConfigPanel>

    <ConfigPanel writable={true} dirty={false}>
      <ConfigPanelHeading title="Verified Identity Health" />
      <p class="control-desc text-muted">
        Bounded operator snapshot summary for recent verified-identity activity and the main failure or category signals currently observed.
      </p>
      {#if verifiedIdentitySummaryAvailable}
        <div class="status-item">
          <div class="status-rows">
            <div class="info-row">
              <span class="info-label text-muted">Availability:</span>
              <span id="verified-identity-availability" class="status-value">{formatAvailability(verifiedIdentitySummary.availability)}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Attempts:</span>
              <span id="verified-identity-attempts" class="status-value">{verifiedIdentitySummary.attempts}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Verified:</span>
              <span id="verified-identity-verified" class="status-value">{verifiedIdentitySummary.verified}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Failed:</span>
              <span id="verified-identity-failed" class="status-value">{verifiedIdentitySummary.failed}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Unique identities:</span>
              <span id="verified-identity-unique-identities" class="status-value">{verifiedIdentitySummary.unique_verified_identities}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Named policies:</span>
              <span id="verified-identity-named-policy-count" class="status-value">{verifiedIdentitySummary.named_policy_count}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Service profiles:</span>
              <span id="verified-identity-service-profile-count" class="status-value">{verifiedIdentitySummary.service_profile_count}</span>
            </div>
          </div>
        </div>
        <div class="status-item">
          <h3>Top Failure Reasons</h3>
          <ul id="verified-identity-top-failure-reasons" class="metric-list">
            {#if verifiedIdentityTopFailureReasons.length > 0}
              {#each verifiedIdentityTopFailureReasons as entry}
                <li><strong>{formatCountEntryLabel(entry.label)}:</strong> {entry.count}</li>
              {/each}
            {:else}
              <li>No recent verification failures.</li>
            {/if}
          </ul>
        </div>
        <div class="status-item">
          <h3>Top Schemes</h3>
          <ul id="verified-identity-top-schemes" class="metric-list">
            {#if verifiedIdentityTopSchemes.length > 0}
              {#each verifiedIdentityTopSchemes as entry}
                <li><strong>{formatCountEntryLabel(entry.label)}:</strong> {entry.count}</li>
              {/each}
            {:else}
              <li>No recent verified-identity scheme activity.</li>
            {/if}
          </ul>
        </div>
        <div class="status-item">
          <h3>Top Categories</h3>
          <ul id="verified-identity-top-categories" class="metric-list">
            {#if verifiedIdentityTopCategories.length > 0}
              {#each verifiedIdentityTopCategories as entry}
                <li><strong>{formatCountEntryLabel(entry.label)}:</strong> {entry.count}</li>
              {/each}
            {:else}
              <li>No recent verified-identity category activity.</li>
            {/if}
          </ul>
        </div>
      {:else}
        <p id="verified-identity-summary-empty" class="message info">
          Verified-identity summary is not materialized yet.
        </p>
      {/if}
    </ConfigPanel>

    <SaveChangesBar containerId="verification-save-all-bar" isHidden={!writable || !hasUnsavedChanges} summaryId="verification-unsaved-summary" summaryText={saveAllSummaryText} summaryClass="text-unsaved-changes" invalidId="verification-invalid-summary" invalidText={saveAllInvalidText} buttonId="save-verification-all" buttonLabel={saveAllConfigLabel} buttonDisabled={saveAllConfigDisabled} onSave={saveAllConfig} />
  </div>
</section>
