<script>
  import { deriveFreshnessSummary } from '../../../domain/telemetry-freshness.js';
  import { formatUnixSecondsLocal } from '../../../domain/core/date-time.js';
  import DisclosureSection from '../primitives/DisclosureSection.svelte';
  import RawTelemetryFeed from './RawTelemetryFeed.svelte';

  export let monitoringFreshnessSnapshot = null;
  export let ipBansFreshnessSnapshot = null;
  export let rawTelemetryFeed = [];
  export let rawFeedMaxLines = 200;

  const formatTimestamp = (value) => formatUnixSecondsLocal(value, '-');

  $: monitoringFreshness = deriveFreshnessSummary(monitoringFreshnessSnapshot, {
    formatTimestamp
  });
  $: ipBansFreshness = deriveFreshnessSummary(ipBansFreshnessSnapshot, {
    formatTimestamp
  });
</script>

<DisclosureSection
  {...$$restProps}
  title="Telemetry Diagnostics"
  description="Low-level freshness, transport, overflow, and raw telemetry diagnostics for contributor debugging."
  rootClass="section panel panel-border"
>
  <div class="control-group panel-soft pad-sm">
    <h3 class="caps-label">Monitoring Feed</h3>
    <div class="status-rows">
      <div class="info-row">
        <span class="info-label text-muted">Freshness:</span>
        <span id="monitoring-freshness-state" class="status-value">
          <strong>{monitoringFreshness.stateLabel}</strong> | lag: {monitoringFreshness.lagText} | last event: {monitoringFreshness.lastEventText}
        </span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Read path:</span>
        <span id="monitoring-freshness-meta" class="status-value">
          transport: <code>{monitoringFreshness.transportCode}</code> | slow consumer: <code>{monitoringFreshness.slowConsumerState}</code> | overflow: <code>{monitoringFreshness.overflow}</code>
        </span>
      </div>
      {#if monitoringFreshness.partialDataWarning}
        <div class="info-row">
          <span class="info-label text-muted">Note:</span>
          <span id="monitoring-freshness-warning" class="status-value">{monitoringFreshness.partialDataWarning}</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="control-group panel-soft pad-sm">
    <h3 class="caps-label">IP Bans Feed</h3>
    <div class="status-rows">
      <div class="info-row">
        <span class="info-label text-muted">Freshness:</span>
        <span id="ip-bans-freshness-state" class="status-value">
          <strong>{ipBansFreshness.stateLabel}</strong> | lag: {ipBansFreshness.lagText} | last event: {ipBansFreshness.lastEventText}
        </span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Read path:</span>
        <span id="ip-bans-freshness-meta" class="status-value">
          transport: <code>{ipBansFreshness.transportCode}</code> | slow consumer: <code>{ipBansFreshness.slowConsumerState}</code> | overflow: <code>{ipBansFreshness.overflow}</code>
        </span>
      </div>
      {#if ipBansFreshness.partialDataWarning}
        <div class="info-row">
          <span class="info-label text-muted">Note:</span>
          <span id="ip-bans-freshness-warning" class="status-value">{ipBansFreshness.partialDataWarning}</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="control-group panel-soft pad-sm">
    <h3 class="caps-label">Raw Telemetry Feed</h3>
    <RawTelemetryFeed
      lines={rawTelemetryFeed}
      maxLines={rawFeedMaxLines}
      wrapped={false}
    />
  </div>
</DisclosureSection>
