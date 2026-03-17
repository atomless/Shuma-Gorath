<script>
  import {
    buildFeatureStatusItems,
    deriveStatusSnapshot
  } from '../../domain/status.js';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import { deriveFreshnessSummary } from '../../domain/telemetry-freshness.js';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let runtimeTelemetry = null;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let monitoringSnapshot = null;
  export let monitoringFreshnessSnapshot = null;
  export let ipBansFreshnessSnapshot = null;

  const CONNECTION_STATE_LABELS = Object.freeze({
    connected: 'Connected',
    degraded: 'Degraded',
    disconnected: 'Disconnected'
  });
  const RETENTION_STATE_LABELS = Object.freeze({
    healthy: 'Healthy',
    degraded: 'Degraded',
    stalled: 'Stalled',
    unknown: 'Unknown'
  });
  const HEARTBEAT_FAILURE_CLASS_LABELS = Object.freeze({
    cancelled: 'Cancelled',
    timeout: 'Timeout',
    transport: 'Transport',
    http: 'HTTP'
  });

  const formatMetricMs = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return '-';
    return `${numeric.toFixed(2)} ms`;
  };

  const formatIsoTimestamp = (value) => {
    const raw = String(value || '').trim();
    if (!raw) return '-';
    const parsed = Date.parse(raw);
    if (!Number.isFinite(parsed)) return raw;
    return new Date(parsed).toLocaleString();
  };

  const formatUnixTimestamp = (value) => formatUnixSecondsLocal(value, '-');
  const formatRetentionLagHours = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric < 0) return 'n/a';
    return `${numeric.toFixed(1)}h`;
  };
  const formatConnectionState = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return CONNECTION_STATE_LABELS[normalized] || CONNECTION_STATE_LABELS.disconnected;
  };
  const formatHeartbeatFailureClass = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    if (!normalized) return '-';
    return HEARTBEAT_FAILURE_CLASS_LABELS[normalized] || normalized.toUpperCase();
  };
  const formatRetentionState = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return RETENTION_STATE_LABELS[normalized] || RETENTION_STATE_LABELS.unknown;
  };
  const formatHeartbeatBreadcrumbs = (entries = []) => {
    if (!Array.isArray(entries) || entries.length === 0) return '-';
    return entries
      .slice(-3)
      .map((entry) => {
        const source = entry && typeof entry === 'object' ? entry : {};
        const eventType = String(source.eventType || '').trim() || 'event';
        const reason = String(source.reason || '').trim();
        return reason ? `${eventType} (${reason})` : eventType;
      })
      .join(' -> ');
  };

  $: statusSnapshot = deriveStatusSnapshot(configSnapshot || {}, configRuntimeSnapshot || {});
  $: featureStatusItems = buildFeatureStatusItems(statusSnapshot);
  $: refresh = runtimeTelemetry && runtimeTelemetry.refresh ? runtimeTelemetry.refresh : {};
  $: polling = runtimeTelemetry && runtimeTelemetry.polling ? runtimeTelemetry.polling : {};
  $: connection = runtimeTelemetry && runtimeTelemetry.connection ? runtimeTelemetry.connection : {};
  $: heartbeat = runtimeTelemetry && runtimeTelemetry.heartbeat ? runtimeTelemetry.heartbeat : {};
  $: monitoringFreshness = deriveFreshnessSummary(monitoringFreshnessSnapshot, {
    formatTimestamp: formatUnixTimestamp
  });
  $: ipBansFreshness = deriveFreshnessSummary(ipBansFreshnessSnapshot, {
    formatTimestamp: formatUnixTimestamp
  });
  $: retentionHealth = monitoringSnapshot && monitoringSnapshot.retention_health
    ? monitoringSnapshot.retention_health
    : {};
</script>

<section
  id="dashboard-panel-status"
  class="admin-group admin-group--status"
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
      <div class="status-item">
        <h3>Dashboard Connectivity</h3>
        <p class="control-desc text-muted">
          Global dashboard connection state is owned by the admin-session heartbeat. Other request failures stay local
          and must not flip the global connection state.
        </p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Status:</span>
            <span id="status-connection-state" class="status-value">{formatConnectionState(connection.state)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last heartbeat success:</span>
            <span id="status-connection-last-success" class="status-value">{formatIsoTimestamp(connection.lastSuccessAt)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last heartbeat failure:</span>
            <span id="status-connection-last-failure" class="status-value">{formatIsoTimestamp(connection.lastFailureAt)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Consecutive failures:</span>
            <span id="status-connection-failure-budget" class="status-value">
              {connection.consecutiveFailures || 0} / {connection.disconnectThreshold || 0}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last heartbeat failure class:</span>
            <span id="status-connection-last-failure-class" class="status-value">
              {formatHeartbeatFailureClass(heartbeat.lastFailureClass)}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Ignored cancelled requests:</span>
            <span id="status-connection-ignored-cancelled" class="status-value">
              {heartbeat.ignoredCancelledCount || 0}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Ignored non-heartbeat failures:</span>
            <span id="status-connection-ignored-non-heartbeat" class="status-value">
              {heartbeat.ignoredNonHeartbeatFailureCount || 0}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Recent heartbeat events:</span>
            <span id="status-connection-breadcrumbs" class="status-value">
              {formatHeartbeatBreadcrumbs(heartbeat.breadcrumbs)}
            </span>
          </div>
        </div>
      </div>

      <div class="status-item">
        <h3>Telemetry Delivery Health</h3>
        <p class="control-desc text-muted">
          Monitoring and IP-ban views should stay current without presenting partial or misleading recent data.
        </p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Monitoring feed status:</span>
            <span id="status-monitoring-freshness-state" class="status-value">{monitoringFreshness.stateLabel}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Monitoring lag / last event:</span>
            <span id="status-monitoring-freshness-lag" class="status-value">
              {monitoringFreshness.lagText} / {monitoringFreshness.lastEventText}
            </span>
          </div>
          {#if monitoringFreshness.partialDataWarning}
            <div class="info-row">
              <span class="info-label text-muted">Monitoring note:</span>
              <span id="status-monitoring-freshness-warning" class="status-value">{monitoringFreshness.partialDataWarning}</span>
            </div>
          {/if}
          <div class="info-row">
            <span class="info-label text-muted">IP bans feed status:</span>
            <span id="status-ip-bans-freshness-state" class="status-value">{ipBansFreshness.stateLabel}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">IP bans lag / last event:</span>
            <span id="status-ip-bans-freshness-lag" class="status-value">
              {ipBansFreshness.lagText} / {ipBansFreshness.lastEventText}
            </span>
          </div>
          {#if ipBansFreshness.partialDataWarning}
            <div class="info-row">
              <span class="info-label text-muted">IP bans note:</span>
              <span id="status-ip-bans-freshness-warning" class="status-value">{ipBansFreshness.partialDataWarning}</span>
            </div>
          {/if}
        </div>
      </div>

      <div class="status-item">
        <h3>Retention Health</h3>
        <p class="control-desc text-muted">
          Retention health should stay ahead of expiration so telemetry remains bounded without dropping operationally
          useful history too early or too late.
        </p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Status:</span>
            <span id="status-retention-health-state" class="status-value">{formatRetentionState(retentionHealth.state)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Retention window:</span>
            <span id="status-retention-window" class="status-value">
              {Number.isFinite(Number(retentionHealth.retention_hours)) && Number(retentionHealth.retention_hours) > 0
                ? `${Math.floor(Number(retentionHealth.retention_hours))}h`
                : 'n/a'}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Purge lag:</span>
            <span id="status-retention-purge-lag" class="status-value">{formatRetentionLagHours(retentionHealth.purge_lag_hours)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Pending expired buckets:</span>
            <span id="status-retention-pending-expired" class="status-value">{retentionHealth.pending_expired_buckets || 0}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Oldest retained event:</span>
            <span id="status-retention-oldest-retained" class="status-value">{formatUnixTimestamp(retentionHealth.oldest_retained_ts)}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last purge success:</span>
            <span id="status-retention-last-success" class="status-value">{formatUnixTimestamp(retentionHealth.last_purge_success_ts)}</span>
          </div>
          {#if retentionHealth.last_purge_error}
            <div class="info-row">
              <span class="info-label text-muted">Last purge error:</span>
              <span id="status-retention-last-error" class="status-value"><code>{retentionHealth.last_purge_error}</code></span>
            </div>
          {/if}
        </div>
      </div>

      <div class="status-item">
        <h3>Runtime Performance Telemetry</h3>
        <p class="control-desc text-muted">
          Operator thresholds for auto-refresh tabs (<code>monitoring</code>, <code>ip-bans</code>, and
          <code>red-team</code>): keep rolling p95 fetch latency under <strong>500 ms</strong>, rolling p95 render
          timing under <strong>16 ms</strong>, and investigate sustained polling skip/resume churn.
        </p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Fetch latency (last / rolling):</span>
            <span id="runtime-fetch-latency-last" class="status-value">
              {formatMetricMs(refresh.fetchLatencyMs?.last)}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Fetch latency detail:</span>
            <span id="runtime-fetch-latency-avg" class="status-value">
              avg {formatMetricMs(refresh.fetchLatencyMs?.avg)} | p95 {formatMetricMs(refresh.fetchLatencyMs?.p95)} |
              window {refresh.fetchLatencyMs?.samples || 0}/{refresh.fetchLatencyMs?.windowSize || 0} |
              total {refresh.fetchLatencyMs?.totalSamples || 0}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Render timing (last / rolling):</span>
            <span id="runtime-render-timing-last" class="status-value">
              {formatMetricMs(refresh.renderTimingMs?.last)}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Render timing detail:</span>
            <span id="runtime-render-timing-avg" class="status-value">
              avg {formatMetricMs(refresh.renderTimingMs?.avg)} | p95 {formatMetricMs(refresh.renderTimingMs?.p95)} |
              window {refresh.renderTimingMs?.samples || 0}/{refresh.renderTimingMs?.windowSize || 0} |
              total {refresh.renderTimingMs?.totalSamples || 0}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Polling skip / resume:</span>
            <span id="runtime-polling-skip-count" class="status-value">
              {polling.skips || 0} / {polling.resumes || 0}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last skip reason:</span>
            <span id="runtime-polling-last-skip-reason" class="status-value">{polling.lastSkipReason || '-'}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Last resume:</span>
            <span id="runtime-polling-last-resume-at" class="status-value">{formatIsoTimestamp(polling.lastResumeAt)}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>
