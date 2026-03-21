<script>
  import { onDestroy } from 'svelte';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import { deriveAdversaryRunRowsFromSummaries } from './monitoring-view-model.js';
  import AdversaryRunPanel from './monitoring/AdversaryRunPanel.svelte';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import {
    deriveAdversarySimProgressState,
    normalizeAdversarySimStatus
  } from '../../runtime/dashboard-adversary-sim.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let toggleEnabled = false;
  export let toggleDisabled = false;
  export let toggleDisabledReason = '';
  export let laneValue = 'synthetic_traffic';
  export let laneDisabled = false;
  export let laneDisabledReason = '';
  export let adversarySimStatus = null;
  export let controllerState = null;
  export let eventsSnapshot = null;
  export let bansSnapshot = null;
  export let monitoringFreshnessSnapshot = null;
  export let lifecycleCopy = '';
  export let noticeText = '';
  export let noticeKind = 'info';
  export let onToggleChange = null;
  export let onLaneChange = null;

  let nowMs = Date.now();
  let progressTimer = null;
  const laneLabels = {
    synthetic_traffic: 'Synthetic Traffic',
    scrapling_traffic: 'Scrapling Traffic',
    bot_red_team: 'Bot Red Team'
  };

  const formatTime = (rawTs) => formatUnixSecondsLocal(rawTs, '-');
  const lanePropertyForKey = (laneKey) => {
    if (laneKey === 'scrapling_traffic') return 'scraplingTraffic';
    if (laneKey === 'bot_red_team') return 'botRedTeam';
    return 'syntheticTraffic';
  };
  const formatLaneLabel = (laneKey, fallback = '-') => laneLabels[laneKey] || fallback;
  const formatStatusCount = (value) => {
    const entries = value && typeof value === 'object'
      ? Object.entries(value)
      : [];
    if (entries.length === 0) return '-';
    return entries
      .map(([code, count]) => `${code}: ${count}`)
      .join(', ');
  };
  const formatFailureClasses = (value) => {
    const failureClasses = value && typeof value === 'object' ? value : {};
    return ['cancelled', 'timeout', 'transport', 'http']
      .map((key) => `${key}: ${Number(failureClasses[key]?.count || 0)}`)
      .join(', ');
  };
  const formatOptionalTime = (rawTs) => rawTs > 0 ? formatTime(rawTs) : '-';

  function handleToggleChange(event) {
    if (typeof onToggleChange === 'function') {
      void onToggleChange(event);
    }
  }

  function handleLaneChange(event) {
    if (typeof onLaneChange === 'function') {
      void onLaneChange(event);
    }
  }

  function syncProgressTimer(shouldTick) {
    if (typeof window === 'undefined') return;
    if (shouldTick) {
      nowMs = Date.now();
      if (progressTimer !== null) return;
      progressTimer = window.setInterval(() => {
        nowMs = Date.now();
      }, 250);
      return;
    }
    if (progressTimer !== null) {
      clearInterval(progressTimer);
      progressTimer = null;
    }
  }

  $: progressState = deriveAdversarySimProgressState({
    status: adversarySimStatus,
    controllerState,
    nowMs
  });
  $: normalizedStatus = normalizeAdversarySimStatus(adversarySimStatus);
  $: events = eventsSnapshot && typeof eventsSnapshot === 'object' ? eventsSnapshot : {};
  $: bans = Array.isArray(bansSnapshot?.bans) ? bansSnapshot.bans : [];
  $: freshness = monitoringFreshnessSnapshot && typeof monitoringFreshnessSnapshot === 'object'
    ? monitoringFreshnessSnapshot
    : {};
  $: freshnessStateKey = String(freshness.state || 'stale').trim().toLowerCase() || 'stale';
  $: rawRecentSimRuns = Array.isArray(events.recent_sim_runs) ? events.recent_sim_runs : [];
  $: adversaryRunSummary = deriveAdversaryRunRowsFromSummaries(rawRecentSimRuns, bans);
  $: adversaryRunRows = Array.isArray(adversaryRunSummary?.runRows)
    ? adversaryRunSummary.runRows.slice(0, 8)
    : [];
  $: desiredLaneLabel = formatLaneLabel(normalizedStatus.desiredLane, 'Synthetic Traffic');
  $: activeLaneLabel = formatLaneLabel(normalizedStatus.activeLane, 'Not running');
  $: diagnosticsLaneKey = normalizedStatus.activeLane || normalizedStatus.desiredLane;
  $: diagnosticsLaneLabel = formatLaneLabel(diagnosticsLaneKey, 'Synthetic Traffic');
  $: diagnosticsLaneProperty = lanePropertyForKey(diagnosticsLaneKey);
  $: diagnosticsLaneState = normalizedStatus.laneDiagnostics.lanes[diagnosticsLaneProperty];
  $: laneDivergence =
    normalizedStatus.activeLane &&
    normalizedStatus.activeLane !== normalizedStatus.desiredLane;
  $: failureClassesText = formatFailureClasses(
    normalizedStatus.laneDiagnostics.requestFailureClasses
  );
  $: progressWidth = `${Number(progressState.progressPercent || 0).toFixed(3)}%`;
  $: shouldTickProgress = progressState.active === true && (managed ? isActive : true);
  $: syncProgressTimer(shouldTickProgress);

  onDestroy(() => {
    syncProgressTimer(false);
  });
</script>

<section
  id="dashboard-panel-red-team"
  class="admin-group"
  data-dashboard-tab-panel="red-team"
  aria-labelledby="dashboard-tab-red-team"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
  <TabStateMessage tab="red-team" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigPanel writable={true} dirty={false}>
      <ConfigPanelHeading title="Adversary Simulation">
        <label class="toggle-switch" for="global-adversary-sim-toggle" title={toggleDisabledReason}>
          <input
            id="global-adversary-sim-toggle"
            type="checkbox"
            aria-label="Enable adversary simulation"
            checked={toggleEnabled}
            disabled={toggleDisabled}
            title={toggleDisabledReason}
            on:change={handleToggleChange}
          >
          <span class="toggle-slider"></span>
        </label>
      </ConfigPanelHeading>
      <p class="control-desc text-muted">
        Start or stop adversary simulation and review the current lifecycle separately from the rest of the dashboard controls.
      </p>
      <div class="admin-controls">
        <div class="input-row" class:input-row--disabled={laneDisabled} aria-disabled={laneDisabled}>
          <label class="control-label control-label--wide" for="adversary-sim-lane-select">Lane</label>
          <select
            id="adversary-sim-lane-select"
            class="input-field"
            aria-label="Select adversary simulation lane"
            value={laneValue}
            disabled={laneDisabled}
            title={laneDisabledReason}
            on:change={handleLaneChange}
          >
            <option value="synthetic_traffic">Synthetic Traffic</option>
            <option value="scrapling_traffic">Scrapling Traffic</option>
            <option value="bot_red_team" disabled>Bot Red Team (coming soon)</option>
          </select>
        </div>
      </div>
      <p id="adversary-sim-lifecycle-copy" class="control-desc text-muted">{lifecycleCopy}</p>
      <div class="dashboard-adversary-sim-progress" aria-hidden="true">
        <div class="dashboard-adversary-sim-progress__fill" style={`width: ${progressWidth};`}></div>
      </div>
      <div class="status-item">
        <h3>Lane State</h3>
        <p class="control-desc text-muted">
          Desired lane records operator intent. Active lane shows the beat-boundary lane executing right now.
        </p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Desired lane:</span>
            <span id="adversary-sim-lane-state-desired" class="status-value">{desiredLaneLabel}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Active lane:</span>
            <span id="adversary-sim-lane-state-active" class="status-value">{activeLaneLabel}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Switch sequence:</span>
            <span class="status-value">{normalizedStatus.laneSwitchSeq}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last switch at:</span>
            <span class="status-value">{formatOptionalTime(normalizedStatus.lastLaneSwitchAt)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last switch reason:</span>
            <span class="status-value">{normalizedStatus.lastLaneSwitchReason || '-'}</span>
          </div>
          {#if laneDivergence}
            <div class="info-row">
              <span class="info-label text-muted">Handoff:</span>
              <span class="status-value">Awaiting beat-boundary reconciliation.</span>
            </div>
          {/if}
        </div>
      </div>
      <div class="status-item">
        <h3>Lane Diagnostics</h3>
        <p class="control-desc text-muted">
          Diagnostics follow the active lane while running, and the desired lane while the simulator is off.
        </p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Diagnostics lane:</span>
            <span class="status-value">{diagnosticsLaneLabel}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Beat attempts:</span>
            <span class="status-value">{diagnosticsLaneState.beatAttempts}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Beat successes:</span>
            <span class="status-value">{diagnosticsLaneState.beatSuccesses}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Beat failures:</span>
            <span class="status-value">{diagnosticsLaneState.beatFailures}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Generated requests:</span>
            <span class="status-value">{diagnosticsLaneState.generatedRequests}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Blocked requests:</span>
            <span class="status-value">{diagnosticsLaneState.blockedRequests}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Offsite requests:</span>
            <span class="status-value">{diagnosticsLaneState.offsiteRequests}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Response bytes:</span>
            <span class="status-value">{diagnosticsLaneState.responseBytes}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Response status counts:</span>
            <span class="status-value">{formatStatusCount(diagnosticsLaneState.responseStatusCount)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last generated at:</span>
            <span class="status-value">{formatOptionalTime(diagnosticsLaneState.lastGeneratedAt)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last error:</span>
            <span class="status-value">{diagnosticsLaneState.lastError || '-'}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Failure classes:</span>
            <span class="status-value">{failureClassesText}</span>
          </div>
        </div>
      </div>
    </ConfigPanel>
  </div>

  <AdversaryRunPanel
    loading={tabStatus?.loading === true}
    runRows={adversaryRunRows}
    activeBanCount={adversaryRunSummary?.activeBanCount || bans.length}
    {freshnessStateKey}
    {formatTime}
    title="Recent Red Team Runs"
    description="Recent adversary simulation runs linked to monitoring and IP ban outcomes."
    summaryLabel="Active bans linked to recent runs"
    emptyText="No recent adversary simulation runs are currently retained in the compact run history."
    degradedText="Monitoring freshness is degraded or stale. Missing red team run rows may indicate delayed telemetry rather than no simulation activity."
  />
</section>
