<script>
  import { onDestroy } from 'svelte';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import { deriveAdversaryRunRows } from './monitoring-view-model.js';
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
  export let adversarySimStatus = null;
  export let controllerState = null;
  export let eventsSnapshot = null;
  export let bansSnapshot = null;
  export let monitoringFreshnessSnapshot = null;
  export let lifecycleCopy = '';
  export let noticeText = '';
  export let noticeKind = 'info';
  export let onToggleChange = null;

  let nowMs = Date.now();
  let progressTimer = null;

  const formatTime = (rawTs) => formatUnixSecondsLocal(rawTs, '-');

  function handleToggleChange(event) {
    if (typeof onToggleChange === 'function') {
      void onToggleChange(event);
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
  $: bans = Array.isArray(bansSnapshot?.bans) ? bansSnapshot.bans : [];
  $: freshness = monitoringFreshnessSnapshot && typeof monitoringFreshnessSnapshot === 'object'
    ? monitoringFreshnessSnapshot
    : {};
  $: freshnessStateKey = String(freshness.state || 'stale').trim().toLowerCase() || 'stale';
  $: rawRecentEvents = Array.isArray(events.recent_events) ? events.recent_events : [];
  $: adversaryRunSummary = deriveAdversaryRunRows(rawRecentEvents, bans);
  $: adversaryRunRows = Array.isArray(adversaryRunSummary?.runRows)
    ? adversaryRunSummary.runRows.slice(0, 8)
    : [];
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
      <p id="adversary-sim-lifecycle-copy" class="control-desc text-muted">{lifecycleCopy}</p>
      <div class="dashboard-adversary-sim-progress" aria-hidden="true">
        <div class="dashboard-adversary-sim-progress__fill" style={`width: ${progressWidth};`}></div>
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
    emptyText="No recent adversary simulation runs were observed in the current monitoring window."
    degradedText="Monitoring freshness is degraded or stale. Missing red team run rows may indicate delayed telemetry rather than no simulation activity."
  />
</section>
