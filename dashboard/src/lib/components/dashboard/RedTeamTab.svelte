<script>
  import { onDestroy } from 'svelte';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import {
    formatAdversarySimLaneLabel,
    formatRepresentativenessStatusLabel
  } from '../../domain/adversary-sim.js';
  import { deriveAdversaryRunRowsFromSummaries } from './monitoring-view-model.js';
  import AdversaryRunPanel from './monitoring/AdversaryRunPanel.svelte';
  import ConfigPanel from './primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from './primitives/ConfigPanelHeading.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import { deriveAdversarySimProgressState } from '../../runtime/dashboard-adversary-sim.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let toggleEnabled = false;
  export let toggleDisabled = false;
  export let toggleDisabledReason = '';
  export let laneValue = 'scrapling_traffic';
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

  const formatTime = (rawTs) => formatUnixSecondsLocal(rawTs, '-');
  const toStringArray = (value) => Array.isArray(value)
    ? value.map((entry) => String(entry || '').trim()).filter(Boolean)
    : [];

  function selectLaneRepresentativeness(readiness, lane) {
    const source = readiness && typeof readiness === 'object' ? readiness : {};
    const laneStatuses = source.laneStatuses && typeof source.laneStatuses === 'object'
      ? source.laneStatuses
      : (source.lane_statuses && typeof source.lane_statuses === 'object' ? source.lane_statuses : {});
    const normalizedLane = String(lane || '').trim().toLowerCase();
    if (!normalizedLane) return {};
    return laneStatuses[normalizedLane] || laneStatuses[normalizedLane.replace(/_([a-z])/g, (_, char) => char.toUpperCase())] || {};
  }

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
  $: events = eventsSnapshot && typeof eventsSnapshot === 'object' ? eventsSnapshot : {};
  $: banSnapshotStatus = String(bansSnapshot?.status || 'available').trim().toLowerCase() || 'available';
  $: banSnapshotUnavailableMessage = banSnapshotStatus === 'unavailable'
    ? String(bansSnapshot?.message || '').trim()
    : '';
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
  $: representativenessReadiness = adversarySimStatus && typeof adversarySimStatus === 'object'
    ? (adversarySimStatus.representativenessReadiness || adversarySimStatus.representativeness_readiness || {})
    : {};
  $: selectedLaneRepresentativeness = selectLaneRepresentativeness(representativenessReadiness, laneValue);
  $: selectedLaneRepresentativenessLabel = formatRepresentativenessStatusLabel(
    selectedLaneRepresentativeness?.status || '',
    ''
  );
  $: selectedLaneRepresentativenessSummary = String(
    selectedLaneRepresentativeness?.summary || ''
  ).trim();
  $: selectedLaneRepresentativenessBlockers = toStringArray(
    selectedLaneRepresentativeness?.blockers
  );
  $: selectedLaneRepresentativenessCopyParts = [];
  $: if (selectedLaneRepresentativenessLabel) {
    selectedLaneRepresentativenessCopyParts = [`${selectedLaneRepresentativenessLabel}.`];
    if (selectedLaneRepresentativenessSummary) {
      selectedLaneRepresentativenessCopyParts.push(selectedLaneRepresentativenessSummary);
    }
    if (selectedLaneRepresentativenessBlockers.length > 0) {
      selectedLaneRepresentativenessCopyParts.push(
        `Missing: ${selectedLaneRepresentativenessBlockers.join('; ')}`
      );
    }
  } else if (selectedLaneRepresentativenessSummary) {
    selectedLaneRepresentativenessCopyParts = [selectedLaneRepresentativenessSummary];
  } else {
    selectedLaneRepresentativenessCopyParts = [];
  }
  $: selectedLaneRepresentativenessCopy = selectedLaneRepresentativenessCopyParts.join(' ');
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
            <option value="bot_red_team">Agentic Traffic</option>
            <option value="parallel_mixed_traffic">Scrapling + Agentic</option>
          </select>
        </div>
      </div>
      <p id="adversary-sim-lifecycle-copy" class="control-desc text-muted">{lifecycleCopy}</p>
      {#if selectedLaneRepresentativenessCopy}
        <p id="adversary-sim-representativeness-copy" class="control-desc text-muted">
          Realism readiness for {formatAdversarySimLaneLabel(laneValue)}: {selectedLaneRepresentativenessCopy}
        </p>
      {/if}
      <div class="dashboard-adversary-sim-progress" aria-hidden="true">
        <div class="dashboard-adversary-sim-progress__fill" style={`width: ${progressWidth};`}></div>
      </div>
    </ConfigPanel>
  </div>

  {#if banSnapshotUnavailableMessage}
    <p id="red-team-ban-state-unavailable" class="message warning">
      {banSnapshotUnavailableMessage}
    </p>
  {/if}

  <AdversaryRunPanel
    loading={tabStatus?.loading === true}
    runRows={adversaryRunRows}
    activeBanCount={banSnapshotStatus === 'unavailable'
      ? 'Unavailable'
      : (adversaryRunSummary?.activeBanCount || bans.length)}
    {freshnessStateKey}
    {formatTime}
    title="Recent Red Team Runs"
    description="Recent adversary simulation runs linked to monitoring and IP ban outcomes."
    summaryLabel="Active bans linked to recent runs"
    emptyText="No recent adversary simulation runs are currently retained in the compact run history."
    degradedText="Monitoring freshness is degraded or stale. Missing red team run rows may indicate delayed telemetry rather than no simulation activity."
  />
</section>
