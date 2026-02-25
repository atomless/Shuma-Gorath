<script>
  import { onMount } from 'svelte';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange } from '../../domain/core/validation.js';
  import NumericInputRow from './primitives/NumericInputRow.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;

  let notABotThreshold = 2;
  let challengeThreshold = 3;
  let mazeThreshold = 6;
  let weightJsRequired = 1;
  let weightGeoRisk = 2;
  let weightRateMedium = 1;
  let weightRateHigh = 2;
  let savingBotness = false;
  let warnOnUnload = false;

  let baseline = {
    notABotThreshold: 2,
    challengeThreshold: 3,
    mazeThreshold: 6,
    weightJsRequired: 1,
    weightGeoRisk: 2,
    weightRateMedium: 1,
    weightRateHigh: 2
  };
  let lastAppliedConfigVersion = -1;

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
    const next = toBotnessBaseline(config);
    baseline = next;
    notABotThreshold = next.notABotThreshold;
    challengeThreshold = next.challengeThreshold;
    mazeThreshold = next.mazeThreshold;
    weightJsRequired = next.weightJsRequired;
    weightGeoRisk = next.weightGeoRisk;
    weightRateMedium = next.weightRateMedium;
    weightRateHigh = next.weightRateHigh;
  }

  async function saveBotness() {
    if (!botnessValid || !botnessDirty || !writable || typeof onSaveConfig !== 'function') return;
    savingBotness = true;
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
      await onSaveConfig(payload, { successMessage: 'Botness scoring saved' });
      baseline = {
        notABotThreshold: Number(notABotThreshold),
        challengeThreshold: Number(challengeThreshold),
        mazeThreshold: Number(mazeThreshold),
        weightJsRequired: Number(weightJsRequired),
        weightGeoRisk: Number(weightGeoRisk),
        weightRateMedium: Number(weightRateMedium),
        weightRateHigh: Number(weightRateHigh)
      };
    } finally {
      savingBotness = false;
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

  $: writable = configSnapshot && configSnapshot.admin_config_write_enabled === true;
  $: notABotDefault = parseInteger(configSnapshot?.not_a_bot_risk_threshold_default, 2);
  $: challengeDefault = parseInteger(configSnapshot?.challenge_puzzle_risk_threshold_default, 3);
  $: mazeDefault = parseInteger(configSnapshot?.botness_maze_threshold_default, 6);
  $: signalDefinitions = configSnapshot && typeof configSnapshot.botness_signal_definitions === 'object'
    ? configSnapshot.botness_signal_definitions
    : {};
  $: scoredSignals = Array.isArray(signalDefinitions.scored_signals)
    ? signalDefinitions.scored_signals
    : [];
  $: terminalSignals = Array.isArray(signalDefinitions.terminal_signals)
    ? signalDefinitions.terminal_signals
    : [];

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
    Number(notABotThreshold) !== baseline.notABotThreshold ||
    Number(challengeThreshold) !== baseline.challengeThreshold ||
    Number(mazeThreshold) !== baseline.mazeThreshold ||
    Number(weightJsRequired) !== baseline.weightJsRequired ||
    Number(weightGeoRisk) !== baseline.weightGeoRisk ||
    Number(weightRateMedium) !== baseline.weightRateMedium ||
    Number(weightRateHigh) !== baseline.weightRateHigh
  );
  $: saveBotnessDisabled = !writable || !botnessDirty || !botnessValid || savingBotness;
  $: saveAllTuningDisabled = saveBotnessDisabled;
  $: saveAllTuningLabel = savingBotness ? 'Saving...' : 'Save all changes';
  $: saveAllTuningSummary = botnessDirty
    ? '1 section with unsaved changes'
    : 'No unsaved changes';
  $: saveAllTuningInvalidText = botnessDirty && !botnessValid
    ? 'Fix invalid tuning values before saving.'
    : '';
  $: warnOnUnload = writable && botnessDirty;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
    }
  }
</script>

<section
  id="dashboard-panel-tuning"
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={botnessDirty}
  data-dashboard-tab-panel="tuning"
  aria-labelledby="dashboard-tab-tuning"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="tuning" status={tabStatus} />
  <div class="controls-grid controls-grid--tuning">
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
        <div class="info-panel">
          <h4>Status</h4>
          <div class="info-row">
            <span class="info-label text-muted">Config:</span>
            <span id="botness-config-status" class="status-value">{writable ? 'EDITABLE' : 'READ ONLY'}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Default Not-a-Bot:</span>
            <span id="not-a-bot-default">{notABotDefault}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Default Challenge:</span>
            <span id="challenge-puzzle-default">{challengeDefault}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Default Maze:</span>
            <span id="maze-threshold-default">{mazeDefault}</span>
          </div>
        </div>
        <div class="info-panel">
          <h4>Scored Signals</h4>
          <div id="botness-signal-list">
            {#if scoredSignals.length === 0}
              <p class="text-muted">No scored signals</p>
            {:else}
              {#each scoredSignals as signal}
                <div class="info-row">
                  <span class="info-label">{signal.label || '--'}</span>
                  <span>{signal.weight ?? '--'}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
        <div class="info-panel">
          <h4>Terminal Signals</h4>
          <div id="botness-terminal-list">
            {#if terminalSignals.length === 0}
              <p class="text-muted">No terminal signals</p>
            {:else}
              {#each terminalSignals as signal}
                <div class="info-row">
                  <span class="info-label">{signal.label || '--'}</span>
                  <span>{signal.action ?? '--'}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    </div>
    <SaveChangesBar containerId="tuning-save-all-bar" isHidden={!writable || !botnessDirty} summaryId="tuning-unsaved-summary" summaryText={saveAllTuningSummary} summaryClass="text-muted" invalidId="tuning-invalid-summary" invalidText={saveAllTuningInvalidText} buttonId="save-tuning-all" buttonLabel={saveAllTuningLabel} buttonDisabled={saveAllTuningDisabled} onSave={saveBotness} />
  </div>
</section>
