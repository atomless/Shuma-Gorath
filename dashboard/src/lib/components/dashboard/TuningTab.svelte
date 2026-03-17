<script>
  import { onMount } from 'svelte';
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
  import ConfigDurationsSection from './config/ConfigDurationsSection.svelte';
  import ConfigNetworkSection from './config/ConfigNetworkSection.svelte';
  import NumericInputRow from './primitives/NumericInputRow.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TextareaField from './primitives/TextareaField.svelte';
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

  const MAX_DURATION_SECONDS = 31536000;
  const MIN_DURATION_SECONDS = 60;
  const DURATION_VALIDATION_BOUNDS = Object.freeze({
    minSeconds: MIN_DURATION_SECONDS,
    maxSeconds: MAX_DURATION_SECONDS
  });

  let notABotThreshold = 2;
  let challengeThreshold = 3;
  let mazeThreshold = 6;
  let weightJsRequired = 1;
  let weightGeoRisk = 2;
  let weightRateMedium = 1;
  let weightRateHigh = 2;

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

  let savingTuning = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let baseline = {
    botness: {
      notABotThreshold: 2,
      challengeThreshold: 3,
      mazeThreshold: 6,
      weightJsRequired: 1,
      weightGeoRisk: 2,
      weightRateMedium: 1,
      weightRateHigh: 2
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
  let lastSaveInvalidLabel = '';

  const toBotnessBaseline = (config = {}) => {
    const weights = config && typeof config.botness_weights === 'object'
      ? config.botness_weights
      : {};
    return {
      notABotThreshold: parseInteger(config.not_a_bot_risk_threshold, 2),
      challengeThreshold: parseInteger(config.challenge_puzzle_risk_threshold, 3),
      mazeThreshold: parseInteger(config.botness_maze_threshold, 6),
      weightJsRequired: parseInteger(weights.js_required, 1),
      weightGeoRisk: parseInteger(weights.geo_risk, 2),
      weightRateMedium: parseInteger(weights.rate_medium, 1),
      weightRateHigh: parseInteger(weights.rate_high, 2)
    };
  };

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

  function applyConfig(config = {}) {
    const botness = toBotnessBaseline(config);
    const durations = toDurationBaseline(config);
    baseline = {
      botness,
      durations,
      browserPolicy: {
        enabled: config.browser_policy_enabled !== false,
        block: normalizeBrowserRulesForCompare(formatBrowserRulesTextarea(config.browser_block))
      },
      pathAllowlist: {
        enabled: config.path_allowlist_enabled !== false,
        entries: normalizeListTextareaForCompare(formatListTextarea(config.path_allowlist))
      }
    };

    notABotThreshold = botness.notABotThreshold;
    challengeThreshold = botness.challengeThreshold;
    mazeThreshold = botness.mazeThreshold;
    weightJsRequired = botness.weightJsRequired;
    weightGeoRisk = botness.weightGeoRisk;
    weightRateMedium = botness.weightRateMedium;
    weightRateHigh = botness.weightRateHigh;

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
  }

  async function saveTuningConfig() {
    if (!tuningValid || !hasUnsavedChanges || !writable || typeof onSaveConfig !== 'function') return;
    savingTuning = true;
    const payload = {};

    if (botnessDirty) {
      payload.not_a_bot_risk_threshold = Number(notABotThreshold);
      payload.challenge_puzzle_risk_threshold = Number(challengeThreshold);
      payload.botness_maze_threshold = Number(mazeThreshold);
      payload.botness_weights = {
        js_required: Number(weightJsRequired),
        geo_risk: Number(weightGeoRisk),
        rate_medium: Number(weightRateMedium),
        rate_high: Number(weightRateHigh)
      };
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
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Tuning settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      } else {
        baseline = {
          botness: {
            notABotThreshold: Number(notABotThreshold),
            challengeThreshold: Number(challengeThreshold),
            mazeThreshold: Number(mazeThreshold),
            weightJsRequired: Number(weightJsRequired),
            weightGeoRisk: Number(weightGeoRisk),
            weightRateMedium: Number(weightRateMedium),
            weightRateHigh: Number(weightRateHigh)
          },
          durations: {
            honeypot: honeypotDurationSeconds,
            rateLimit: rateDurationSeconds,
            cdp: cdpDurationSeconds,
            admin: adminDurationSeconds
          },
          browserPolicy: {
            enabled: browserPolicyEnabled,
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
      savingTuning = false;
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

  $: writable = isAdminConfigWritable(configRuntimeSnapshot);
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;
  $: notABotThresholdValid = (
    inRange(notABotThreshold, 1, 10) &&
    (Number(challengeThreshold) <= 1 || Number(notABotThreshold) < Number(challengeThreshold))
  );
  $: challengeThresholdValid = (
    inRange(challengeThreshold, 1, 10) &&
    (Number(challengeThreshold) <= 1 || Number(notABotThreshold) < Number(challengeThreshold))
  );
  $: mazeThresholdValid = inRange(mazeThreshold, 1, 10);
  $: weightJsRequiredValid = inRange(weightJsRequired, 0, 10);
  $: weightGeoRiskValid = inRange(weightGeoRisk, 0, 10);
  $: weightRateMediumValid = inRange(weightRateMedium, 0, 10);
  $: weightRateHighValid = inRange(weightRateHigh, 0, 10);
  $: botnessValid = (
    notABotThresholdValid &&
    challengeThresholdValid &&
    mazeThresholdValid &&
    weightJsRequiredValid &&
    weightGeoRiskValid &&
    weightRateMediumValid &&
    weightRateHighValid
  );
  $: botnessDirty = (
    Number(notABotThreshold) !== baseline.botness.notABotThreshold ||
    Number(challengeThreshold) !== baseline.botness.challengeThreshold ||
    Number(mazeThreshold) !== baseline.botness.mazeThreshold ||
    Number(weightJsRequired) !== baseline.botness.weightJsRequired ||
    Number(weightGeoRisk) !== baseline.botness.weightGeoRisk ||
    Number(weightRateMedium) !== baseline.botness.weightRateMedium ||
    Number(weightRateHigh) !== baseline.botness.weightRateHigh
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
    { label: 'Botness scoring', dirty: botnessDirty, valid: botnessValid },
    { label: 'Ban durations', dirty: durationsDirty, valid: durationsValid },
    { label: 'Browser policy', dirty: browserPolicyDirty, valid: browserPolicyValid },
    { label: 'Path allowlist', dirty: pathAllowlistDirty, valid: true }
  ];
  $: dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
  $: invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
  $: invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
  $: dirtySectionCount = dirtySectionEntries.length;
  $: hasUnsavedChanges = dirtySectionCount > 0;
  $: tuningValid = botnessValid && durationsValid;
  $: hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
  $: if (hasInvalidUnsavedChanges) {
    lastSaveInvalidLabel = invalidDirtySectionLabels.join(', ');
  } else if (!hasUnsavedChanges) {
    lastSaveInvalidLabel = '';
  }
  $: saveAllTuningDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingTuning;
  $: saveAllTuningLabel = savingTuning ? 'Saving...' : 'Save all changes';
  $: saveAllTuningSummary = hasUnsavedChanges
    ? `${dirtySectionCount} section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
    : 'No unsaved changes';
  $: saveAllTuningInvalidText = hasInvalidUnsavedChanges
    ? `Fix invalid values in: ${lastSaveInvalidLabel}`
    : '';
  $: warnOnUnload = writable && hasUnsavedChanges;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingTuning) {
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }
</script>

<section
  id="dashboard-panel-tuning"
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={hasUnsavedChanges}
  data-dashboard-tab-panel="tuning"
  aria-labelledby="dashboard-tab-tuning"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="tuning" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <div class="control-group panel-soft pad-md">
      <h3>Botness Scoring</h3>
      <p class="control-desc text-muted">Weighted signals form a unified score. Moderate scores get the challenge; higher scores route to maze.</p>
      <div class="admin-controls">
        <NumericInputRow id="not-a-bot-threshold-score" label="Not-a-Bot (score)" min="1" max="10" step="1" inputmode="numeric" ariaLabel="Not-a-Bot risk threshold" ariaInvalid={notABotThresholdValid ? 'false' : 'true'} bind:value={notABotThreshold} disabled={!writable} />
        <NumericInputRow id="challenge-puzzle-threshold" label="Challenge (score)" min="1" max="10" step="1" inputmode="numeric" ariaLabel="Challenge risk threshold" ariaInvalid={challengeThresholdValid ? 'false' : 'true'} bind:value={challengeThreshold} disabled={!writable} />
        <NumericInputRow id="maze-threshold-score" label="Maze (score)" min="1" max="10" step="1" inputmode="numeric" ariaLabel="Maze risk threshold" ariaInvalid={mazeThresholdValid ? 'false' : 'true'} bind:value={mazeThreshold} disabled={!writable} />
        <NumericInputRow id="weight-js-required" label='Weight: <abbr title="JavaScript">JS</abbr> (points)' min="0" max="10" step="1" inputmode="numeric" ariaLabel="Weight for JavaScript verification required" ariaInvalid={weightJsRequiredValid ? 'false' : 'true'} bind:value={weightJsRequired} disabled={!writable} />
        <NumericInputRow id="weight-geo-risk" label="Weight: Geo (points)" min="0" max="10" step="1" inputmode="numeric" ariaLabel="Weight for high-risk geography" ariaInvalid={weightGeoRiskValid ? 'false' : 'true'} bind:value={weightGeoRisk} disabled={!writable} />
        <NumericInputRow id="weight-rate-medium" label="Weight: Rate 50% (points)" min="0" max="10" step="1" inputmode="numeric" ariaLabel="Weight for medium rate pressure" ariaInvalid={weightRateMediumValid ? 'false' : 'true'} bind:value={weightRateMedium} disabled={!writable} />
        <NumericInputRow id="weight-rate-high" label="Weight: Rate 80% (points)" min="0" max="10" step="1" inputmode="numeric" ariaLabel="Weight for high rate pressure" ariaInvalid={weightRateHighValid ? 'false' : 'true'} bind:value={weightRateHigh} disabled={!writable} />
      </div>
    </div>

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

    <div class="control-group panel-soft pad-md config-edit-pane" class:config-edit-pane--dirty={pathAllowlistDirty}>
      <div class="panel-heading-with-control">
        <h3>Path Allowlist</h3>
        <label class="toggle-switch" for="path-allowlist-enabled-toggle">
          <input
            type="checkbox"
            id="path-allowlist-enabled-toggle"
            aria-label="Enable path allowlist bypass"
            bind:checked={pathAllowlistEnabled}
            disabled={!writable}
          >
          <span class="toggle-slider"></span>
        </label>
      </div>
      <p class="control-desc text-muted">
        Use this list to bypass bot defenses for trusted machine paths such as payment webhooks and partner callbacks.
      </p>
      <p class="control-desc text-muted">
        Rules: each entry must be a path, not a full URL. Exact paths must match exactly (example: <code>/webhook/stripe</code>). Prefix rules must end with <code>*</code> (example: <code>/api/integrations/*</code>).
      </p>
      <p class="control-desc text-muted">
        You must not enter hostnames, query strings, or fragments (for example, do not enter <code>https://example.com/hook</code>, <code>/hook?token=1</code>, or <code>/hook#frag</code>).
      </p>
      {#if !pathAllowlistEnabled}
        <p class="message warning">
          Path allowlist bypass is disabled. Existing entries are preserved and will be applied again when you re-enable this toggle.
        </p>
      {/if}
      <TextareaField
        id="path-allowlist"
        label="Path Allowlist"
        rows="5"
        ariaLabel="Path allowlist"
        spellcheck={false}
        disabled={!writable || !pathAllowlistEnabled}
        bind:value={pathAllowlist}
      />
    </div>

    <SaveChangesBar
      containerId="tuning-save-all-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="tuning-unsaved-summary"
      summaryText={saveAllTuningSummary}
      summaryClass="text-muted"
      invalidId="tuning-invalid-summary"
      invalidText={saveAllTuningInvalidText}
      buttonId="save-tuning-all"
      buttonLabel={saveAllTuningLabel}
      buttonDisabled={saveAllTuningDisabled}
      onSave={saveTuningConfig}
    />
  </div>
</section>
