<script>
  import { browser } from '$app/environment';
  import { onDestroy, onMount } from 'svelte';
  import {
    CHALLENGE_REASON_LABELS,
    IP_RANGE_ACTION_LABELS,
    IP_RANGE_FALLBACK_LABELS,
    IP_RANGE_SOURCE_LABELS,
    NOT_A_BOT_OUTCOME_LABELS,
    NOT_A_BOT_LATENCY_LABELS,
    POW_REASON_LABELS,
    RATE_OUTCOME_LABELS,
    deriveIpRangeMonitoringViewModel,
    deriveMazeStatsViewModel,
    deriveMonitoringSummaryViewModel,
    deriveTarpitViewModel,
    derivePrometheusHelperViewModel
  } from './monitoring-view-model.js';
  import {
    normalizeDimensionRows,
    normalizePairRows,
    normalizeReasonRows,
    normalizeTopCountries,
    normalizeTopPaths,
    normalizeTrendRows,
    normalizeTrendSeries
  } from '../../domain/monitoring-normalizers.js';
  import {
    buildMonitoringCountYAxis,
    buildMonitoringTimeSeriesXAxis,
    resolveMonitoringChartTheme
  } from '../../domain/monitoring-chart-presets.js';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import { arraysEqualShallow } from '../../domain/core/format.js';
  import { formatIpRangeReasonLabel } from '../../domain/ip-range-policy.js';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import DiagnosticsSection from './monitoring/DiagnosticsSection.svelte';
  import CdpSection from './monitoring/CdpSection.svelte';
  import MazeSection from './monitoring/MazeSection.svelte';
  import TarpitSection from './monitoring/TarpitSection.svelte';
  import HoneypotSection from './monitoring/HoneypotSection.svelte';
  import ChallengeSection from './monitoring/ChallengeSection.svelte';
  import PowSection from './monitoring/PowSection.svelte';
  import RateSection from './monitoring/RateSection.svelte';
  import GeoSection from './monitoring/GeoSection.svelte';
  import IpRangeSection from './monitoring/IpRangeSection.svelte';
  import ExternalMonitoringSection from './monitoring/ExternalMonitoringSection.svelte';

  const RAW_FEED_MAX_LINES = 200;
  const CDP_ROW_RENDER_LIMIT = 500;
  const CHART_RESIZE_REDRAW_DEBOUNCE_MS = 180;
  const POW_OUTCOME_LABELS = Object.freeze({
    success: 'Success',
    failure: 'Failure'
  });

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
  export let eventsSnapshot = null;
  export let mazeSnapshot = null;
  export let cdpSnapshot = null;
  export let cdpEventsSnapshot = null;
  export let monitoringSnapshot = null;
  export let monitoringFreshnessSnapshot = null;
  export let ipBansFreshnessSnapshot = null;
  export let configSnapshot = null;

  let challengeTrendCanvas = null;
  let powTrendCanvas = null;
  let challengeTrendChart = null;
  let powTrendChart = null;

  let rawRecentEvents = [];
  let rawTelemetryFeed = [];

  let copyButtonLabel = 'Copy JavaScript Example';
  let copyCurlButtonLabel = 'Copy Curl Example';
  let copyButtonTimer = null;
  let copyCurlButtonTimer = null;
  let resizeRedrawTimer = null;
  let chartRefreshNonce = 0;
  let wasActive = false;
  let detachColorSchemeListener = () => {};

  const defaultMonitoringSummary = deriveMonitoringSummaryViewModel({});
  const defaultMazeStats = deriveMazeStatsViewModel({});
  const defaultTarpitSummary = deriveTarpitViewModel({});
  const defaultPrometheusHelper = derivePrometheusHelperViewModel({}, '');
  const clearTimer = (timerId) => {
    if (timerId === null) return null;
    clearTimeout(timerId);
    return null;
  };

  const sameSeries = (chart, nextLabels, nextData) => {
    if (!chart || !chart.data || !Array.isArray(chart.data.datasets) || chart.data.datasets.length === 0) {
      return false;
    }
    const currentLabels = Array.isArray(chart.data.labels) ? chart.data.labels : [];
    const currentData = Array.isArray(chart.data.datasets[0].data) ? chart.data.datasets[0].data : [];
    return arraysEqualShallow(currentLabels, nextLabels) && arraysEqualShallow(currentData, nextData);
  };

  const sameColorSeries = (currentColors, nextColors) => {
    const current = Array.isArray(currentColors)
      ? currentColors.map((color) => String(color || ''))
      : [String(currentColors || '')];
    const next = Array.isArray(nextColors)
      ? nextColors.map((color) => String(color || ''))
      : [String(nextColors || '')];
    return arraysEqualShallow(current, next);
  };

  const scheduleCopyLabelReset = (kind) => {
    if (kind === 'js') {
      copyButtonTimer = clearTimer(copyButtonTimer);
      copyButtonTimer = setTimeout(() => {
        copyButtonLabel = 'Copy JavaScript Example';
      }, 1200);
      return;
    }
    copyCurlButtonTimer = clearTimer(copyCurlButtonTimer);
    copyCurlButtonTimer = setTimeout(() => {
      copyCurlButtonLabel = 'Copy Curl Example';
    }, 1200);
  };

  const copyToClipboard = async (text, kind) => {
    if (!browser) return;
    const value = String(text || '');
    try {
      await navigator.clipboard.writeText(value);
      if (kind === 'js') {
        copyButtonLabel = 'Copied';
      } else {
        copyCurlButtonLabel = 'Copied';
      }
    } catch (_error) {
      if (kind === 'js') {
        copyButtonLabel = 'Copy Failed';
      } else {
        copyCurlButtonLabel = 'Copy Failed';
      }
    }
    scheduleCopyLabelReset(kind);
  };

  const formatTime = (rawTs) => formatUnixSecondsLocal(rawTs, '-');

  const rawFeedKey = (event = {}) => {
    const source = event && typeof event === 'object' ? event : {};
    const cursor = String(source.cursor || source.event_cursor || source.event_id || '').trim();
    if (cursor) return cursor;
    const operationId = String(source.operation_id || '').trim();
    if (operationId) return operationId;
    const ts = Number(source.ts || 0);
    const eventName = String(source.event || '').trim();
    const ip = String(source.ip || '').trim();
    const reason = String(source.reason || '').trim();
    const outcome = String(source.outcome || '').trim();
    const admin = String(source.admin || '').trim();
    const simRunId = String(source.sim_run_id || '').trim();
    const simProfile = String(source.sim_profile || '').trim();
    const simLane = String(source.sim_lane || '').trim();
    return [
      ts,
      eventName,
      ip,
      reason,
      outcome,
      admin,
      simRunId,
      simProfile,
      simLane
    ].join('|');
  };

  const rawFeedPayload = (event = {}) => {
    const source = event && typeof event === 'object' ? event : {};
    const payload = {};
    Object.keys(source)
      .sort()
      .forEach((key) => {
        if (source[key] === undefined) return;
        payload[key] = source[key];
      });
    return payload;
  };

  const buildRawFeedLine = (event = {}) => {
    const ts = formatTime(event?.ts);
    return `[${ts}] ${JSON.stringify(rawFeedPayload(event))}`;
  };

  const buildRawTelemetryFeed = (events = []) =>
    (Array.isArray(events) ? events : [])
      .slice(0, RAW_FEED_MAX_LINES)
      .map((event, index) => ({
        key: `${rawFeedKey(event)}|${index}`,
        line: buildRawFeedLine(event)
      }));

  const readCdpField = (text, key) => {
    const match = new RegExp(`${key}=([^\\s]+)`, 'i').exec(String(text || ''));
    return match ? match[1] : '-';
  };

  const getChartConstructor = () => {
    if (!browser || !window || typeof window.Chart !== 'function') return null;
    return window.Chart;
  };

  const chartNeedsRefresh = (chart, refreshNonce) =>
    Number(chart?.__shumaRefreshNonce || 0) !== Number(refreshNonce || 0);

  const stampChartRefresh = (chart, refreshNonce) => {
    if (chart && typeof chart === 'object') {
      chart.__shumaRefreshNonce = Number(refreshNonce || 0);
    }
    return chart;
  };

  const resizeChartIfNeeded = (chart, needsRefresh) => {
    if (!needsRefresh) return;
    if (chart && typeof chart.resize === 'function') {
      chart.resize();
    }
  };

  const requestChartRefresh = () => {
    chartRefreshNonce += 1;
  };

  const scheduleChartRefreshAfterResize = () => {
    resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    resizeRedrawTimer = setTimeout(() => {
      resizeRedrawTimer = null;
      if (!isActive) return;
      requestChartRefresh();
    }, CHART_RESIZE_REDRAW_DEBOUNCE_MS);
  };

  const attachColorSchemeChangeListener = () => {
    if (!browser || !window || typeof window.matchMedia !== 'function') {
      return () => {};
    }
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => {
      if (!isActive) return;
      requestChartRefresh();
    };
    if (typeof mediaQuery.addEventListener === 'function') {
      mediaQuery.addEventListener('change', onChange);
      return () => {
        mediaQuery.removeEventListener('change', onChange);
      };
    }
    if (typeof mediaQuery.addListener === 'function') {
      mediaQuery.addListener(onChange);
      return () => {
        mediaQuery.removeListener(onChange);
      };
    }
    return () => {};
  };

  const updateTrendChart = (chart, canvas, title, fillKey, trendSeries, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    const chartTheme = resolveMonitoringChartTheme();
    const color = chartTheme.timeSeriesFill[fillKey] || chartTheme.timeSeriesFill.events;

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
        type: 'line',
        data: {
          labels: trendSeries.labels,
          datasets: [{
            label: title,
            data: trendSeries.data,
            backgroundColor: color,
            fill: true,
            tension: 0.35,
            pointRadius: 0,
            pointHoverRadius: 0,
            borderWidth: 0
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          plugins: { legend: { display: false } },
          scales: {
            x: buildMonitoringTimeSeriesXAxis(),
            y: buildMonitoringCountYAxis(trendSeries.data)
          }
        }
      }), refreshNonce);
    }

    const needsRefresh = chartNeedsRefresh(chart, refreshNonce);
    const hasSameSeries = sameSeries(chart, trendSeries.labels, trendSeries.data);
    const hasSameColor = sameColorSeries(chart.data.datasets?.[0]?.backgroundColor, color);
    if (!needsRefresh && hasSameSeries && hasSameColor) {
      return chart;
    }
    resizeChartIfNeeded(chart, needsRefresh);
    chart.data.labels = trendSeries.labels;
    chart.data.datasets[0].data = trendSeries.data;
    chart.data.datasets[0].backgroundColor = color;
    if (chart.options?.scales) {
      chart.options.scales.y = buildMonitoringCountYAxis(trendSeries.data);
    }
    chart.update('none');
    return stampChartRefresh(chart, refreshNonce);
  };

  $: events = eventsSnapshot && typeof eventsSnapshot === 'object' ? eventsSnapshot : {};
  $: maze = mazeSnapshot && typeof mazeSnapshot === 'object' ? mazeSnapshot : {};
  $: cdp = cdpSnapshot && typeof cdpSnapshot === 'object' ? cdpSnapshot : {};
  $: cdpEventsData = cdpEventsSnapshot && typeof cdpEventsSnapshot === 'object'
    ? cdpEventsSnapshot
    : {};
  $: config = configSnapshot && typeof configSnapshot === 'object'
    ? configSnapshot
    : {};
  $: monitoring = monitoringSnapshot && typeof monitoringSnapshot === 'object'
    ? monitoringSnapshot
    : {};

  $: rawRecentEvents = Array.isArray(events.recent_events)
    ? events.recent_events.slice(0, RAW_FEED_MAX_LINES)
    : [];
  $: rawTelemetryFeed = buildRawTelemetryFeed(rawRecentEvents);
  $: recentCdpEvents = Array.isArray(cdpEventsData.events)
    ? cdpEventsData.events.slice(0, CDP_ROW_RENDER_LIMIT)
    : [];

  $: cdpDetections = Number(cdp?.stats?.total_detections || 0);
  $: cdpAutoBans = Number(cdp?.stats?.auto_bans || 0);
  $: cdpFingerprintUaClientHintMismatch = Number(cdp?.fingerprint_stats?.ua_client_hint_mismatch || 0);
  $: cdpFingerprintUaTransportMismatch = Number(cdp?.fingerprint_stats?.ua_transport_mismatch || 0);
  $: cdpFingerprintTemporalTransitions = Number(cdp?.fingerprint_stats?.temporal_transition || 0);
  $: cdpFingerprintFlowViolations = Number(cdp?.fingerprint_stats?.flow_violation || 0);

  $: mazeStats = deriveMazeStatsViewModel(maze || {}) || defaultMazeStats;
  $: tarpitSummary = deriveTarpitViewModel(monitoring?.details?.tarpit || {}) || defaultTarpitSummary;
  $: monitoringSummary =
    deriveMonitoringSummaryViewModel(monitoring.summary || {}) || defaultMonitoringSummary;
  $: prometheusHelper = derivePrometheusHelperViewModel(
    monitoring.prometheus || {},
    browser && window?.location?.origin ? window.location.origin : ''
  ) || defaultPrometheusHelper;

  $: honeypotTopPaths = normalizeTopPaths(monitoringSummary.honeypot.topPaths);
  $: challengeReasonRows = normalizeReasonRows(
    monitoringSummary.challenge.reasons,
    CHALLENGE_REASON_LABELS
  );
  $: notABotOutcomeRows = normalizeReasonRows(
    monitoringSummary.notABot.outcomes,
    NOT_A_BOT_OUTCOME_LABELS
  );
  $: notABotLatencyRows = normalizeReasonRows(
    monitoringSummary.notABot.latencyBuckets,
    NOT_A_BOT_LATENCY_LABELS
  );
  $: powReasonRows = normalizeReasonRows(monitoringSummary.pow.reasons, POW_REASON_LABELS);
  $: powOutcomeRows = normalizePairRows(monitoringSummary.pow.outcomes, POW_OUTCOME_LABELS);
  $: rateOutcomeRows = normalizePairRows(monitoringSummary.rate.outcomes, RATE_OUTCOME_LABELS);
  $: geoTopCountries = normalizeTopCountries(monitoringSummary.geo.topCountries);
  $: ipRangeSummary = deriveIpRangeMonitoringViewModel(rawRecentEvents, config);
  $: ipRangeReasonRows = normalizeDimensionRows(
    ipRangeSummary.reasons,
    (key) => formatIpRangeReasonLabel(key)
  );
  $: ipRangeSourceRows = normalizeDimensionRows(ipRangeSummary.sources, IP_RANGE_SOURCE_LABELS);
  $: ipRangeActionRows = normalizeDimensionRows(ipRangeSummary.actions, IP_RANGE_ACTION_LABELS);
  $: ipRangeDetectionRows = normalizeDimensionRows(
    ipRangeSummary.detections,
    (value) => value
  );
  $: ipRangeSourceIdRows = normalizeDimensionRows(ipRangeSummary.sourceIds, (value) => value);
  $: ipRangeFallbackRows = normalizeDimensionRows(
    ipRangeSummary.fallbacks,
    IP_RANGE_FALLBACK_LABELS
  );
  $: ipRangeTrendRows = normalizeTrendRows(ipRangeSummary.trend);

  $: challengeTrendSeries = normalizeTrendSeries(monitoringSummary.challenge.trend);
  $: powTrendSeries = normalizeTrendSeries(monitoringSummary.pow.trend);

  $: if (browser) {
    const nextActive = isActive === true;
    if (nextActive && !wasActive) {
      requestChartRefresh();
    }
    wasActive = nextActive;
  }

  $: if (browser && challengeTrendCanvas) {
    challengeTrendChart = updateTrendChart(
      challengeTrendChart,
      challengeTrendCanvas,
      'Puzzle Outcomes',
      'challenge',
      challengeTrendSeries,
      chartRefreshNonce
    );
  }

  $: if (browser && powTrendCanvas) {
    powTrendChart = updateTrendChart(
      powTrendChart,
      powTrendCanvas,
      'Proof of Work Failures',
      'pow',
      powTrendSeries,
      chartRefreshNonce
    );
  }

  onMount(() => {
    if (!browser || !window) return undefined;
    const onResize = () => {
      scheduleChartRefreshAfterResize();
    };
    window.addEventListener('resize', onResize, { passive: true });
    detachColorSchemeListener = attachColorSchemeChangeListener();
    return () => {
      window.removeEventListener('resize', onResize);
      if (typeof detachColorSchemeListener === 'function') {
        detachColorSchemeListener();
        detachColorSchemeListener = () => {};
      }
      resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    };
  });

  onDestroy(() => {
    copyButtonTimer = clearTimer(copyButtonTimer);
    copyCurlButtonTimer = clearTimer(copyCurlButtonTimer);
    resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    if (typeof detachColorSchemeListener === 'function') {
      detachColorSchemeListener();
      detachColorSchemeListener = () => {};
    }
    if (challengeTrendChart && typeof challengeTrendChart.destroy === 'function') {
      challengeTrendChart.destroy();
    }
    if (powTrendChart && typeof powTrendChart.destroy === 'function') {
      powTrendChart.destroy();
    }
    challengeTrendChart = null;
    powTrendChart = null;
  });
</script>

<section
  id="dashboard-panel-diagnostics"
  class="dashboard-tab-panel"
  data-dashboard-tab-panel="diagnostics"
  aria-labelledby="dashboard-tab-diagnostics"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'false'}
  tabindex="-1"
>
  <TabStateMessage tab="diagnostics" status={tabStatus} />

  <CdpSection
    loading={tabStatus?.loading === true}
    {cdpDetections}
    {cdpAutoBans}
    {cdpFingerprintUaClientHintMismatch}
    {cdpFingerprintUaTransportMismatch}
    {cdpFingerprintTemporalTransitions}
    {cdpFingerprintFlowViolations}
    {recentCdpEvents}
    {formatTime}
    {readCdpField}
  />

  <MazeSection
    loading={tabStatus?.loading === true}
    {mazeStats}
  />

  <TarpitSection
    loading={tabStatus?.loading === true}
    {tarpitSummary}
  />

  <HoneypotSection
    loading={tabStatus?.loading === true}
    honeypot={monitoringSummary.honeypot}
    topPaths={honeypotTopPaths}
  />

  <ChallengeSection
    loading={tabStatus?.loading === true}
    challengeSummary={monitoringSummary.challenge}
    notABotSummary={monitoringSummary.notABot}
    {challengeReasonRows}
    {notABotOutcomeRows}
    {notABotLatencyRows}
    bind:challengeTrendCanvas
  />

  <PowSection
    loading={tabStatus?.loading === true}
    powSummary={monitoringSummary.pow}
    {powReasonRows}
    {powOutcomeRows}
    bind:powTrendCanvas
  />

  <RateSection
    loading={tabStatus?.loading === true}
    rateSummary={monitoringSummary.rate}
    {rateOutcomeRows}
  />

  <GeoSection
    loading={tabStatus?.loading === true}
    geoSummary={monitoringSummary.geo}
    {geoTopCountries}
  />

  <IpRangeSection
    loading={tabStatus?.loading === true}
    summary={ipRangeSummary}
    reasonRows={ipRangeReasonRows}
    sourceRows={ipRangeSourceRows}
    actionRows={ipRangeActionRows}
    detectionRows={ipRangeDetectionRows}
    sourceIdRows={ipRangeSourceIdRows}
    fallbackRows={ipRangeFallbackRows}
    trendRows={ipRangeTrendRows}
  />

  <DiagnosticsSection
    data-diagnostics-section="telemetry-diagnostics"
    monitoringFreshnessSnapshot={monitoringFreshnessSnapshot}
    ipBansFreshnessSnapshot={ipBansFreshnessSnapshot}
    rawTelemetryFeed={rawTelemetryFeed}
    rawFeedMaxLines={RAW_FEED_MAX_LINES}
  />

  <ExternalMonitoringSection
    data-diagnostics-section="external-monitoring"
    description="Use the bounded helper examples here when you need to export or mirror diagnostics into external monitoring systems."
    {prometheusHelper}
    {copyButtonLabel}
    copyCurlButtonLabel={copyCurlButtonLabel}
    onCopyJs={(text) => copyToClipboard(text, 'js')}
    onCopyCurl={(text) => copyToClipboard(text, 'curl')}
  />
</section>
