<script>
  import { onMount } from 'svelte';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import NumericInputRow from './primitives/NumericInputRow.svelte';
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

  let notABotThreshold = 2;
  let challengeThreshold = 3;
  let mazeThreshold = 6;
  let weightJsRequired = 1;
  let weightGeoRisk = 2;
  let weightRateMedium = 1;
  let weightRateHigh = 2;

  let savingTuning = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;
  let lastSaveInvalidLabel = '';

  let baseline = {
    botness: {
      notABotThreshold: 2,
      challengeThreshold: 3,
      mazeThreshold: 6,
      weightJsRequired: 1,
      weightGeoRisk: 2,
      weightRateMedium: 1,
      weightRateHigh: 2
    }
  };

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

  function applyConfig(config = {}) {
    const botness = toBotnessBaseline(config);
    baseline = { botness };

    notABotThreshold = botness.notABotThreshold;
    challengeThreshold = botness.challengeThreshold;
    mazeThreshold = botness.mazeThreshold;
    weightJsRequired = botness.weightJsRequired;
    weightGeoRisk = botness.weightGeoRisk;
    weightRateMedium = botness.weightRateMedium;
    weightRateHigh = botness.weightRateHigh;
  }

  async function saveTuningConfig() {
    if (!botnessValid || !hasUnsavedChanges || !writable || typeof onSaveConfig !== 'function') return;
    savingTuning = true;
    const payload = {
      not_a_bot_risk_threshold: Number(notABotThreshold),
      challenge_puzzle_risk_threshold: Number(challengeThreshold),
      botness_maze_threshold: Number(mazeThreshold),
      botness_weights: {
        js_required: Number(weightJsRequired),
        geo_risk: Number(weightGeoRisk),
        rate_medium: Number(weightRateMedium),
        rate_high: Number(weightRateHigh)
      }
    };

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

  $: dirtySections = [
    { label: 'Botness scoring', dirty: botnessDirty, valid: botnessValid }
  ];
  $: dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
  $: invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
  $: invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
  $: hasUnsavedChanges = dirtySectionEntries.length > 0;
  $: hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
  $: if (hasInvalidUnsavedChanges) {
    lastSaveInvalidLabel = invalidDirtySectionLabels.join(', ');
  } else if (!hasUnsavedChanges) {
    lastSaveInvalidLabel = '';
  }
  $: saveAllTuningDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingTuning;
  $: saveAllTuningLabel = savingTuning ? 'Saving...' : 'Save tuning changes';
  $: saveAllTuningSummary = hasUnsavedChanges
    ? 'Botness scoring has unsaved changes'
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
