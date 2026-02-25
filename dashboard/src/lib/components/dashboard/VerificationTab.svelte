<script>
  import ConfigChallengeSection from './config/ConfigChallengeSection.svelte';
  import ConfigNetworkSection from './config/ConfigNetworkSection.svelte';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import ConfigWriteModeMessage from './primitives/ConfigWriteModeMessage.svelte';
  import { onMount } from 'svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import {
    formatBrowserRulesTextarea,
    normalizeBrowserRulesForCompare,
    parseBrowserRulesTextarea
  } from '../../domain/config-form-utils.js';
  import { parseFloatNumber, parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import ToggleRow from './primitives/ToggleRow.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;

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

  let browserPolicyEnabled = true;
  let browserBlockRules = '';
  let browserAllowlistRules = '';

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
    browserPolicy: { enabled: true, block: '', allowlist: '' }
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

  function applyConfig(config = {}) {
    hasConfigSnapshot = config && typeof config === 'object' && Object.keys(config).length > 0;
    writable = config.admin_config_write_enabled === true;

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

    browserPolicyEnabled = config.browser_policy_enabled !== false;
    browserBlockRules = formatBrowserRulesTextarea(config.browser_block);
    browserAllowlistRules = formatBrowserRulesTextarea(config.browser_allowlist);

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
      browserPolicy: {
        enabled: browserPolicyEnabled,
        block: normalizeBrowserRulesForCompare(browserBlockRules),
        allowlist: normalizeBrowserRulesForCompare(browserAllowlistRules)
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
    if (includeAll || browserPolicyDirty) {
      patch.browser_policy_enabled = browserPolicyEnabled;
      patch.browser_block = parseBrowserRulesTextarea(browserBlockRules);
      patch.browser_allowlist = parseBrowserRulesTextarea(browserAllowlistRules);
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
        applyConfig(nextConfig);
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

  $: browserBlockNormalized = normalizeBrowserRulesForCompare(browserBlockRules);
  $: browserAllowlistNormalized = normalizeBrowserRulesForCompare(browserAllowlistRules);
  $: browserBlockRulesValid = browserBlockNormalized !== '__invalid__';
  $: browserAllowlistRulesValid = browserAllowlistNormalized !== '__invalid__';
  $: browserPolicyValid = browserBlockRulesValid && browserAllowlistRulesValid;
  $: browserPolicyDirty = (
    readBool(browserPolicyEnabled) !== baseline.browserPolicy.enabled ||
    browserBlockNormalized !== baseline.browserPolicy.block ||
    browserAllowlistNormalized !== baseline.browserPolicy.allowlist
  );

  $: dirtySections = [
    { label: 'JavaScript required', dirty: jsRequiredDirty, valid: true },
    { label: 'Internal CDP probe', dirty: cdpDirty, valid: cdpValid },
    { label: 'Proof of Work', dirty: powDirty, valid: powValid },
    { label: 'Challenge puzzle', dirty: challengePuzzleDirty, valid: challengePuzzleValid },
    { label: 'Not-a-Bot', dirty: notABotDirty, valid: notABotValid },
    { label: 'Browser policy', dirty: browserPolicyDirty, valid: browserPolicyValid }
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
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }

  $: if (deferredConfigApply && !hasUnsavedChanges && !savingAll) {
    deferredConfigApply = false;
    applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
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
  <TabStateMessage tab="verification" status={tabStatus} />
  <ConfigWriteModeMessage
    id="verification-mode-subtitle"
    controlsLabel="Verification controls"
    loading={tabStatus?.loading === true}
    {hasConfigSnapshot}
    {writable}
  />
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
    </ConfigPanel>

    <ConfigChallengeSection bind:writable bind:notABotDirty bind:challengePuzzleDirty bind:notABotEnabled bind:challengePuzzleEnabled bind:notABotScorePassMinFloor bind:notABotScorePassMin bind:notABotScoreFailMaxCap bind:notABotScoreFailMax notABotPassScoreValid={notABotPassScoreValid} notABotFailScoreValid={notABotFailScoreValid} />

    <ConfigNetworkSection
      bind:writable
      showHoneypot={false}
      showBrowserPolicy={true}
      bind:browserPolicyDirty
      bind:browserPolicyEnabled
      bind:browserBlockRules
      bind:browserAllowlistRules
      browserBlockRulesValid={browserBlockRulesValid}
      browserAllowlistRulesValid={browserAllowlistRulesValid}
    />

    <SaveChangesBar containerId="verification-save-all-bar" isHidden={!writable || !hasUnsavedChanges} summaryId="verification-unsaved-summary" summaryText={saveAllSummaryText} summaryClass="text-unsaved-changes" invalidId="verification-invalid-summary" invalidText={saveAllInvalidText} buttonId="save-verification-all" buttonLabel={saveAllConfigLabel} buttonDisabled={saveAllConfigDisabled} onSave={saveAllConfig} />
  </div>
</section>
