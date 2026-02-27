<script>
  import {
    buildFeatureStatusItems,
    deriveStatusSnapshot
  } from '../../domain/status.js';
  import MetricStatCard from './primitives/MetricStatCard.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let runtimeTelemetry = null;
  export let tabStatus = null;
  export let configSnapshot = null;

  const formatMetricMs = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return '-';
    return `${numeric.toFixed(2)} ms`;
  };

  const formatTimestamp = (value) => {
    const raw = String(value || '').trim();
    if (!raw) return '-';
    return raw;
  };

  $: statusSnapshot = deriveStatusSnapshot(configSnapshot || {});
  $: featureStatusItems = buildFeatureStatusItems(statusSnapshot);
  $: refresh = runtimeTelemetry && runtimeTelemetry.refresh ? runtimeTelemetry.refresh : {};
  $: polling = runtimeTelemetry && runtimeTelemetry.polling ? runtimeTelemetry.polling : {};
</script>

<section
  id="dashboard-panel-status"
  class="admin-group"
  data-dashboard-tab-panel="status"
  aria-labelledby="dashboard-tab-status"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
  <TabStateMessage tab="status" status={tabStatus} />
  <div class="controls-grid controls-grid--status">
    <div class="control-group panel-soft pad-md">
      <div id="status-items">
        {#each featureStatusItems as item}
          <div class="status-item">
            <h3>{@html item.title}</h3>
            <p class="control-desc text-muted">{@html item.description}</p>
            <div class="status-rows">
              <div class="info-row">
                <span class="info-label text-muted">Status:</span>
                <span class="status-value">{item.status}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
    <div class="control-group panel-soft pad-md">
      <h3>Runtime Performance Telemetry</h3>
      <p class="control-desc text-muted">
        Operator thresholds for auto-refresh tabs (<code>monitoring</code> and <code>ip-bans</code>): keep rolling
        p95 fetch latency under <strong>500 ms</strong>, rolling p95 render timing under <strong>16 ms</strong>,
        and investigate sustained polling skip/resume churn.
      </p>
      <div class="stats-cards stats-cards--compact">
        <MetricStatCard
          title="Fetch Latency (Last / Rolling)"
          valueId="runtime-fetch-latency-last"
          value={formatMetricMs(refresh.fetchLatencyMs?.last)}
          small={true}
        >
          <p id="runtime-fetch-latency-avg" class="text-muted">
            avg: {formatMetricMs(refresh.fetchLatencyMs?.avg)} | p95: {formatMetricMs(refresh.fetchLatencyMs?.p95)}
            (window: {refresh.fetchLatencyMs?.samples || 0}/{refresh.fetchLatencyMs?.windowSize || 0}, total: {refresh.fetchLatencyMs?.totalSamples || 0})
          </p>
        </MetricStatCard>
        <MetricStatCard
          title="Render Timing (Last / Rolling)"
          valueId="runtime-render-timing-last"
          value={formatMetricMs(refresh.renderTimingMs?.last)}
          small={true}
        >
          <p id="runtime-render-timing-avg" class="text-muted">
            avg: {formatMetricMs(refresh.renderTimingMs?.avg)} | p95: {formatMetricMs(refresh.renderTimingMs?.p95)}
            (window: {refresh.renderTimingMs?.samples || 0}/{refresh.renderTimingMs?.windowSize || 0}, total: {refresh.renderTimingMs?.totalSamples || 0})
          </p>
        </MetricStatCard>
        <MetricStatCard
          title="Polling Skip / Resume"
          valueId="runtime-polling-skip-count"
          value={polling.skips || 0}
          small={true}
        >
          <p id="runtime-polling-resume-count" class="text-muted">resumes: {polling.resumes || 0}</p>
          <p id="runtime-polling-last-skip-reason" class="text-muted">
            last skip: {polling.lastSkipReason || '-'}
          </p>
          <p id="runtime-polling-last-resume-at" class="text-muted">
            last resume: {formatTimestamp(polling.lastResumeAt)}
          </p>
        </MetricStatCard>
      </div>
    </div>
  </div>
</section>
