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
    deriveDefenseTrendRows,
    deriveEnforcedMonitoringChartRows,
    deriveMonitoringEventDisplay,
    deriveRecentEventFilterOptions,
    filterRecentEvents,
    deriveIpRangeMonitoringViewModel,
    deriveMazeStatsViewModel,
    deriveMonitoringSummaryViewModel,
    deriveTarpitViewModel,
    derivePrometheusHelperViewModel
  } from './monitoring-view-model.js';
  import {
    buildTimeSeries,
    hoursForRange,
    normalizeDimensionRows,
    normalizePairRows,
    normalizeReasonRows,
    normalizeTopCountries,
    normalizeTopPaths,
    normalizeTrendRows,
    normalizeTrendSeries,
    shouldFetchRange
  } from '../../domain/monitoring-normalizers.js';
  import {
    buildMonitoringCountYAxis,
    buildMonitoringTimeSeriesXAxis,
    resolveMonitoringChartTheme
  } from '../../domain/monitoring-chart-presets.js';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import { arraysEqualShallow } from '../../domain/core/format.js';
  import {
    buildHalfDoughnutSeries,
    EMPTY_HALF_DOUGHNUT_READOUT,
    HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN,
    buildHalfDoughnutOptions,
    syncHalfDoughnutReadout
  } from '../../domain/half-doughnut-chart.js';
  import { normalizeLowerTrimmed } from '../../domain/core/strings.js';
  import { formatIpRangeReasonLabel } from '../../domain/ip-range-policy.js';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import OverviewStats from './monitoring/OverviewStats.svelte';
  import PrimaryCharts from './monitoring/PrimaryCharts.svelte';
  import DiagnosticsSection from './monitoring/DiagnosticsSection.svelte';
  import DefenseTrendBlocks from './monitoring/DefenseTrendBlocks.svelte';
  import RecentEventsTable from './monitoring/RecentEventsTable.svelte';
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

  const EVENT_ROW_RENDER_LIMIT = 100;
  const RAW_FEED_MAX_LINES = 200;
  const CDP_ROW_RENDER_LIMIT = 500;
  const RANGE_EVENTS_FETCH_LIMIT = 5000;
  const RANGE_EVENTS_REQUEST_TIMEOUT_MS = 10000;
  const RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS = 180000;
  const CHART_RESIZE_REDRAW_DEBOUNCE_MS = 180;
  const POW_OUTCOME_LABELS = Object.freeze({
    success: 'Success',
    failure: 'Failure'
  });
  const TIME_RANGES = Object.freeze(['hour', 'day', 'week', 'month']);
  const RANGE_CACHEABLE_WINDOWS = Object.freeze(['week', 'month']);

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
  export let analyticsSnapshot = null;
  export let eventsSnapshot = null;
  export let bansSnapshot = null;
  export let mazeSnapshot = null;
  export let cdpSnapshot = null;
  export let cdpEventsSnapshot = null;
  export let monitoringSnapshot = null;
  export let monitoringFreshnessSnapshot = null;
  export let ipBansFreshnessSnapshot = null;
  export let configSnapshot = null;
  export let onFetchEventsRange = null;
  export let autoRefreshEnabled = false;

  let eventTypesCanvas = null;
  let topIpsCanvas = null;
  let timeSeriesCanvas = null;
  let challengeTrendCanvas = null;
  let powTrendCanvas = null;
  let eventTypesChart = null;
  let topIpsChart = null;
  let timeSeriesChart = null;
  let challengeTrendChart = null;
  let powTrendChart = null;

  let selectedTimeRange = 'hour';
  let eventFilters = {
    origin: 'all',
    mode: 'all',
    scenario: 'all',
    lane: 'all',
    defense: 'all',
    outcome: 'all'
  };
  let rangeEventsByWindow = {
    week: { recent_events: [], fetchedAtMs: 0, loading: false },
    month: { recent_events: [], fetchedAtMs: 0, loading: false }
  };
  let rangeEventsAbortController = null;
  let lastRequestedRange = '';
  let lastRangeTabUpdateAnchor = '';
  let rawRecentEvents = [];
  let filteredRecentEvents = [];
  let rawTelemetryFeed = [];
  let defenseTrendRows = [];
  let eventTypesReadout = EMPTY_HALF_DOUGHNUT_READOUT;

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

  const eventBadgeClass = (eventType) => {
    const normalized = normalizeLowerTrimmed(eventType).replace(/[^a-z_]/g, '');
    return normalized ? `badge ${normalized}` : 'badge';
  };

  const formatTime = (rawTs) => formatUnixSecondsLocal(rawTs, '-');
  const toNonNegativeIntOrNull = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric < 0) return null;
    return Math.floor(numeric);
  };
  const normalizeEventCounts = (value) =>
    value && typeof value === 'object' && !Array.isArray(value) ? value : {};
  const sumEventCounts = (eventCounts = {}) =>
    Object.values(eventCounts).reduce((total, value) => {
      const next = toNonNegativeIntOrNull(value);
      return next === null ? total : total + next;
    }, 0);
  const getEventCountByName = (eventCounts = {}, eventName = '') => {
    const target = normalizeLowerTrimmed(eventName);
    if (!target) return null;
    const direct = toNonNegativeIntOrNull(eventCounts[eventName]);
    if (direct !== null) return direct;
    const matchedKey = Object.keys(eventCounts).find(
      (key) => normalizeLowerTrimmed(key) === target
    );
    if (!matchedKey) return null;
    return toNonNegativeIntOrNull(eventCounts[matchedKey]);
  };

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

  const updateDoughnutChart = (
    chart,
    canvas,
    counts,
    refreshNonce = 0,
    onReadoutChange = null
  ) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    const chartTheme = resolveMonitoringChartTheme();
    const palette = chartTheme.palette;
    const { labels, values: data } = buildHalfDoughnutSeries(counts);
    const colors = data.map((_, index) => palette[index % palette.length]);

    if (!chart) {
      const nextChart = stampChartRefresh(new chartCtor(ctx, {
        type: 'doughnut',
        plugins: [HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN],
        data: {
          labels,
          datasets: [{
            data,
            backgroundColor: colors,
            borderColor: 'rgba(0, 0, 0, 0)',
            borderWidth: 0,
            hoverBorderWidth: 0
          }]
        },
        options: buildHalfDoughnutOptions({
          legendColor: chartTheme.legendColor,
          maintainAspectRatio: false,
          onReadoutChange
        })
      }), refreshNonce);
      syncHalfDoughnutReadout(nextChart, onReadoutChange);
      return nextChart;
    }

    const needsRefresh = chartNeedsRefresh(chart, refreshNonce);
    const hasSameSeries = sameSeries(chart, labels, data);
    const hasSameColors = sameColorSeries(chart.data.datasets?.[0]?.backgroundColor, colors);
    if (!needsRefresh && hasSameSeries && hasSameColors) {
      return chart;
    }
    resizeChartIfNeeded(chart, needsRefresh);
    chart.data.labels = labels;
    chart.data.datasets[0].data = data;
    chart.data.datasets[0].backgroundColor = colors;
    chart.data.datasets[0].borderColor = 'rgba(0, 0, 0, 0)';
    chart.data.datasets[0].borderWidth = 0;
    chart.data.datasets[0].hoverBorderWidth = 0;
    const halfDoughnutOptions = buildHalfDoughnutOptions({
      legendColor: chartTheme.legendColor,
      maintainAspectRatio: false,
      onReadoutChange
    });
    chart.options.rotation = halfDoughnutOptions.rotation;
    chart.options.circumference = halfDoughnutOptions.circumference;
    chart.options.cutout = halfDoughnutOptions.cutout;
    chart.options.aspectRatio = halfDoughnutOptions.aspectRatio;
    chart.options.maintainAspectRatio = halfDoughnutOptions.maintainAspectRatio;
    chart.options.onHover = halfDoughnutOptions.onHover;
    if (chart.options?.plugins?.tooltip) {
      chart.options.plugins.tooltip.enabled = false;
    }
    if (chart.options?.plugins?.legend) {
      chart.options.plugins.legend.position = halfDoughnutOptions.plugins.legend.position;
    }
    if (chart.options?.plugins?.legend?.labels) {
      chart.options.plugins.legend.labels.color = chartTheme.legendColor;
    }
    chart.update('none');
    const nextChart = stampChartRefresh(chart, refreshNonce);
    syncHalfDoughnutReadout(nextChart, onReadoutChange);
    return nextChart;
  };

  const updateTopIpsChart = (chart, canvas, topIps, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    const chartTheme = resolveMonitoringChartTheme();
    const palette = chartTheme.palette;
    const pairs = Array.isArray(topIps) ? topIps : [];
    const labels = pairs.map(([ip]) => String(ip || '-'));
    const data = pairs.map(([, count]) => Number(count || 0));
    const colors = data.map((_, index) => palette[index % palette.length]);

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
        type: 'bar',
        data: {
          labels,
          datasets: [{
            label: 'Events',
            data,
            backgroundColor: colors,
            borderColor: 'rgba(0, 0, 0, 0)',
            borderWidth: 0
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          aspectRatio: 2.2,
          scales: {
            y: buildMonitoringCountYAxis(data)
          },
          plugins: { legend: { display: false } }
        }
      }), refreshNonce);
    }

    const needsRefresh = chartNeedsRefresh(chart, refreshNonce);
    const hasSameSeries = sameSeries(chart, labels, data);
    const hasSameColors = sameColorSeries(chart.data.datasets?.[0]?.backgroundColor, colors);
    if (!needsRefresh && hasSameSeries && hasSameColors) {
      return chart;
    }
    resizeChartIfNeeded(chart, needsRefresh);
    chart.data.labels = labels;
    chart.data.datasets[0].data = data;
    chart.data.datasets[0].backgroundColor = colors;
    chart.data.datasets[0].borderColor = 'rgba(0, 0, 0, 0)';
    chart.data.datasets[0].borderWidth = 0;
    if (chart.options?.scales) {
      chart.options.scales.y = buildMonitoringCountYAxis(data);
    }
    chart.update('none');
    return stampChartRefresh(chart, refreshNonce);
  };

  const updateTimeSeriesChart = (chart, canvas, series, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    const chartTheme = resolveMonitoringChartTheme();
    const fillColor = chartTheme.timeSeriesFill.events;

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
        type: 'line',
        data: {
          labels: series.labels,
          datasets: [{
            label: 'Events',
            data: series.data,
            fill: true,
            tension: 0.4,
            borderWidth: 0,
            pointRadius: 0,
            pointHoverRadius: 0,
            borderColor: 'rgba(0, 0, 0, 0)',
            backgroundColor: fillColor
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          scales: {
            x: buildMonitoringTimeSeriesXAxis(),
            y: buildMonitoringCountYAxis(series.data)
          },
          plugins: { legend: { display: false } }
        }
      }), refreshNonce);
    }

    const needsRefresh = chartNeedsRefresh(chart, refreshNonce);
    const hasSameSeries = sameSeries(chart, series.labels, series.data);
    const hasSameColor = sameColorSeries(chart.data.datasets?.[0]?.backgroundColor, fillColor);
    if (!needsRefresh && hasSameSeries && hasSameColor) {
      return chart;
    }
    resizeChartIfNeeded(chart, needsRefresh);
    chart.data.labels = series.labels;
    chart.data.datasets[0].data = series.data;
    chart.data.datasets[0].backgroundColor = fillColor;
    if (chart.options?.scales) {
      chart.options.scales.y = buildMonitoringCountYAxis(series.data);
    }
    chart.update('none');
    return stampChartRefresh(chart, refreshNonce);
  };

  function selectTimeRange(range) {
    if (!TIME_RANGES.includes(range)) return;
    if (selectedTimeRange === range) return;
    selectedTimeRange = range;
    if (!shouldFetchRange(range)) return;
    lastRequestedRange = '';
  }

  function onEventFilterChange(key, value) {
    const normalizedKey = String(key || '').trim();
    if (!normalizedKey || !Object.prototype.hasOwnProperty.call(eventFilters, normalizedKey)) return;
    eventFilters = {
      ...eventFilters,
      [normalizedKey]: String(value || 'all').trim() || 'all'
    };
  }

  function abortRangeEventsFetch() {
    if (!rangeEventsAbortController) return;
    rangeEventsAbortController.abort();
    rangeEventsAbortController = null;
  }

  function normalizeRangeWindowKey(range) {
    const value = String(range || '').trim();
    return RANGE_CACHEABLE_WINDOWS.includes(value) ? value : '';
  }

  function readRangeWindowState(range) {
    const key = normalizeRangeWindowKey(range);
    if (!key) return { recent_events: [], fetchedAtMs: 0, loading: false };
    const snapshot = rangeEventsByWindow[key] || {};
    return {
      recent_events: Array.isArray(snapshot.recent_events) ? snapshot.recent_events : [],
      fetchedAtMs: Number(snapshot.fetchedAtMs || 0),
      loading: snapshot.loading === true
    };
  }

  function writeRangeWindowState(range, nextState = {}) {
    const key = normalizeRangeWindowKey(range);
    if (!key) return;
    const previous = readRangeWindowState(key);
    rangeEventsByWindow = {
      ...rangeEventsByWindow,
      [key]: {
        ...previous,
        ...nextState,
        recent_events: Array.isArray(nextState.recent_events)
          ? nextState.recent_events
          : previous.recent_events,
        fetchedAtMs: Number.isFinite(Number(nextState.fetchedAtMs))
          ? Number(nextState.fetchedAtMs)
          : previous.fetchedAtMs,
        loading: nextState.loading === true
      }
    };
  }

  async function fetchRangeEvents(range) {
    if (!browser || !shouldFetchRange(range)) return;
    const targetRange = normalizeRangeWindowKey(range);
    if (!targetRange) return;
    const hours = hoursForRange(range);
    if (!Number.isFinite(hours)) return;
    if (typeof onFetchEventsRange !== 'function') {
      writeRangeWindowState(targetRange, {
        recent_events: [],
        fetchedAtMs: Date.now(),
        loading: false
      });
      return;
    }
    abortRangeEventsFetch();
    const abortController = new AbortController();
    writeRangeWindowState(targetRange, { loading: true });
    const timeoutId = setTimeout(() => {
      abortController.abort();
    }, RANGE_EVENTS_REQUEST_TIMEOUT_MS);
    rangeEventsAbortController = abortController;
    try {
      const payload = await onFetchEventsRange(hours, {
        signal: abortController.signal
      });
      if (rangeEventsAbortController !== abortController) return;
      writeRangeWindowState(targetRange, {
        recent_events: Array.isArray(payload?.recent_events)
          ? payload.recent_events.slice(0, RANGE_EVENTS_FETCH_LIMIT)
          : [],
        fetchedAtMs: Date.now(),
        loading: false
      });
    } catch (error) {
      if (error && error.name === 'AbortError') return;
      if (rangeEventsAbortController !== abortController) return;
      writeRangeWindowState(targetRange, { loading: false });
    } finally {
      clearTimeout(timeoutId);
      if (rangeEventsAbortController === abortController) {
        rangeEventsAbortController = null;
        writeRangeWindowState(targetRange, { loading: false });
      }
    }
  }

  $: analytics = analyticsSnapshot && typeof analyticsSnapshot === 'object' ? analyticsSnapshot : {};
  $: events = eventsSnapshot && typeof eventsSnapshot === 'object' ? eventsSnapshot : {};
  $: bans = Array.isArray(bansSnapshot?.bans) ? bansSnapshot.bans : [];
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
  $: banSnapshotStatus = String(
    bansSnapshot?.status || analytics?.ban_store_status || 'available'
  ).trim().toLowerCase() || 'available';
  $: banSnapshotUnavailableMessage = banSnapshotStatus === 'unavailable'
    ? String(bansSnapshot?.message || analytics?.ban_store_message || '').trim()
    : '';
  $: freshnessStateKey = String(monitoringFreshnessSnapshot?.state || '').trim().toLowerCase();

  $: rawRecentEvents = Array.isArray(events.recent_events)
    ? events.recent_events.slice(0, RAW_FEED_MAX_LINES)
    : [];
  $: rawTelemetryFeed = buildRawTelemetryFeed(rawRecentEvents);
  $: defenseTrendRows = deriveDefenseTrendRows(rawRecentEvents);
  $: eventFilterOptions = deriveRecentEventFilterOptions(rawRecentEvents);
  $: filteredRecentEvents = filterRecentEvents(rawRecentEvents.slice(0, EVENT_ROW_RENDER_LIMIT), eventFilters);
  $: recentEvents = filteredRecentEvents.map((event) => deriveMonitoringEventDisplay(event));
  $: recentCdpEvents = Array.isArray(cdpEventsData.events)
    ? cdpEventsData.events.slice(0, CDP_ROW_RENDER_LIMIT)
    : [];

  $: eventCounts = normalizeEventCounts(events.event_counts);
  $: eventWindowTotal = toNonNegativeIntOrNull(events?.recent_events_window?.total_events_in_window);
  $: eventCount = eventWindowTotal !== null
    ? eventWindowTotal
    : (() => {
      const summed = sumEventCounts(eventCounts);
      return summed > 0 ? summed : rawRecentEvents.length;
    })();
  $: totalBans = (() => {
    const byEventType = getEventCountByName(eventCounts, 'Ban');
    if (byEventType !== null) return byEventType;
    const analyticsBanCount = toNonNegativeIntOrNull(analytics.ban_count);
    if (analyticsBanCount !== null) return analyticsBanCount;
    if (banSnapshotStatus === 'unavailable') return null;
    return bans.length;
  })();
  $: activeBans = banSnapshotStatus === 'unavailable' ? null : bans.length;
  $: uniqueIps = Number.isFinite(Number(events.unique_ips))
    ? Number(events.unique_ips)
    : (Array.isArray(events.top_ips) ? events.top_ips.length : 0);

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

  $: monitoringEventEmptyState = (() => {
    if (tabStatus?.error) {
      return {
        kind: 'error',
        message: `Monitoring refresh error: ${String(tabStatus.error)}`
      };
    }
    if (rawRecentEvents.length === 0 && (freshnessStateKey === 'degraded' || freshnessStateKey === 'stale')) {
      return {
        kind: 'degraded',
        message: 'No events loaded while freshness is degraded/stale. Data may be delayed.'
      };
    }
    if (rawRecentEvents.length > 0 && recentEvents.length === 0) {
      return {
        kind: 'filtered-empty',
        message: 'No events match the current filter combination.'
      };
    }
    return {
      kind: 'empty',
      message: 'No recent events'
    };
  })();

  $: challengeTrendSeries = normalizeTrendSeries(monitoringSummary.challenge.trend);
  $: powTrendSeries = normalizeTrendSeries(monitoringSummary.pow.trend);

  $: defaultRangeEvents = Array.isArray(events.recent_events)
    ? events.recent_events.slice(0, RANGE_EVENTS_FETCH_LIMIT)
    : [];
  $: selectedRangeWindowState = shouldFetchRange(selectedTimeRange)
    ? readRangeWindowState(selectedTimeRange)
    : { recent_events: [], fetchedAtMs: 0, loading: false };
  $: selectedRangeEvents = shouldFetchRange(selectedTimeRange)
    ? (
      selectedRangeWindowState.recent_events.length > 0 || selectedRangeWindowState.fetchedAtMs > 0
        ? selectedRangeWindowState.recent_events
        : defaultRangeEvents
    )
    : defaultRangeEvents;
  $: enforcedRecentChartRows = deriveEnforcedMonitoringChartRows(defaultRangeEvents, { topIpLimit: 10 });
  $: enforcedSelectedRangeEvents =
    deriveEnforcedMonitoringChartRows(selectedRangeEvents, { topIpLimit: 10 }).events;
  $: timeSeries = buildTimeSeries(enforcedSelectedRangeEvents, selectedTimeRange, {
    maxEvents: RANGE_EVENTS_FETCH_LIMIT
  });

  $: if (browser && !isActive) {
    abortRangeEventsFetch();
  }

  $: if (browser && !autoRefreshEnabled) {
    lastRangeTabUpdateAnchor = '';
  }

  $: if (browser && isActive && autoRefreshEnabled && shouldFetchRange(selectedTimeRange)) {
    const currentUpdatedAt = String(tabStatus?.updatedAt || '');
    if (currentUpdatedAt && currentUpdatedAt !== lastRangeTabUpdateAnchor) {
      lastRangeTabUpdateAnchor = currentUpdatedAt;
      const selectedFetchedAtMs = Number(selectedRangeWindowState.fetchedAtMs || 0);
      const isRangeFetchInFlight = selectedRangeWindowState.loading === true;
      if (
        !isRangeFetchInFlight &&
        (Date.now() - selectedFetchedAtMs) >= RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS
      ) {
        lastRequestedRange = '';
      }
    }
  }

  $: if (browser && isActive && shouldFetchRange(selectedTimeRange) && lastRequestedRange !== selectedTimeRange) {
    const selectedFetchedAtMs = Number(selectedRangeWindowState.fetchedAtMs || 0);
    if (selectedFetchedAtMs > 0 && (Date.now() - selectedFetchedAtMs) < RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS) {
      lastRequestedRange = selectedTimeRange;
    } else {
      lastRequestedRange = selectedTimeRange;
      void fetchRangeEvents(selectedTimeRange);
    }
  }

  $: if (browser) {
    const nextActive = isActive === true;
    if (nextActive && !wasActive) {
      requestChartRefresh();
    }
    wasActive = nextActive;
  }

  $: if (browser && eventTypesCanvas) {
    eventTypesChart = updateDoughnutChart(
      eventTypesChart,
      eventTypesCanvas,
      enforcedRecentChartRows.eventCounts,
      chartRefreshNonce,
      (nextReadout) => {
        eventTypesReadout = nextReadout;
      }
    );
  }

  $: if (browser && topIpsCanvas) {
    topIpsChart = updateTopIpsChart(
      topIpsChart,
      topIpsCanvas,
      enforcedRecentChartRows.topIps,
      chartRefreshNonce
    );
  }

  $: if (browser && timeSeriesCanvas) {
    timeSeriesChart = updateTimeSeriesChart(timeSeriesChart, timeSeriesCanvas, timeSeries, chartRefreshNonce);
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
    abortRangeEventsFetch();
    if (eventTypesChart && typeof eventTypesChart.destroy === 'function') {
      eventTypesChart.destroy();
    }
    if (topIpsChart && typeof topIpsChart.destroy === 'function') {
      topIpsChart.destroy();
    }
    if (timeSeriesChart && typeof timeSeriesChart.destroy === 'function') {
      timeSeriesChart.destroy();
    }
    if (challengeTrendChart && typeof challengeTrendChart.destroy === 'function') {
      challengeTrendChart.destroy();
    }
    if (powTrendChart && typeof powTrendChart.destroy === 'function') {
      powTrendChart.destroy();
    }
    eventTypesChart = null;
    topIpsChart = null;
    timeSeriesChart = null;
    challengeTrendChart = null;
    powTrendChart = null;
  });
</script>

<section
  id="dashboard-panel-diagnostics"
  class="admin-group dashboard-tab-panel"
  data-dashboard-tab-panel="diagnostics"
  aria-labelledby="dashboard-tab-diagnostics"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'false'}
  tabindex="-1"
>
  <TabStateMessage tab="diagnostics" status={tabStatus} />

  <div
    class="control-group panel-soft pad-md"
    data-diagnostics-intro
    data-diagnostics-section="deep-inspection-intro"
  >
    <h3>Diagnostics</h3>
    <p class="control-desc text-muted">
      Use this tab for deep inspection of subsystem telemetry, external-traffic traces, and
      freshness or transport detail.
    </p>
    <p class="control-desc text-muted">
      Monitoring now owns the loop-accountability story for the live stance. Diagnostics keeps
      the contributor-style investigation surface.
    </p>
  </div>

  <section class="section" data-diagnostics-section="traffic-overview">
    <h2>Traffic Overview</h2>
    <p class="text-muted">
      Inspect the bounded external-traffic summary and enforced-event charts that still power
      deep diagnostics while Monitoring is rebuilt.
    </p>
    <OverviewStats
      loading={tabStatus?.loading === true}
      {totalBans}
      {activeBans}
      {eventCount}
      {uniqueIps}
    />
    {#if banSnapshotUnavailableMessage}
      <p id="diagnostics-ban-state-unavailable" class="message warning">
        {banSnapshotUnavailableMessage}
      </p>
    {/if}

    <PrimaryCharts
      {selectedTimeRange}
      {eventTypesReadout}
      onSelectTimeRange={selectTimeRange}
      bind:eventTypesCanvas
      bind:topIpsCanvas
      bind:timeSeriesCanvas
    />
  </section>

  <section class="section" data-diagnostics-section="defense-breakdown">
    <h2>Defense Breakdown</h2>
    <p class="text-muted">
      Review per-defense trend blocks and execution-mode splits without turning Monitoring back
      into a subsystem-by-subsystem dashboard.
    </p>
    <DefenseTrendBlocks
      loading={tabStatus?.loading === true}
      trendRows={defenseTrendRows}
    />
  </section>

  <section class="section" data-diagnostics-section="recent-external-traffic">
    <h2>Recent External Traffic</h2>
    <p class="text-muted">
      Filter recent external-traffic events directly when you need to inspect concrete request and
      defense outcomes.
    </p>
    <RecentEventsTable
      {recentEvents}
      filterOptions={eventFilterOptions}
      filters={eventFilters}
      onFilterChange={onEventFilterChange}
      emptyState={monitoringEventEmptyState}
      {formatTime}
      {eventBadgeClass}
    />
  </section>

  <section class="section" data-diagnostics-section="defense-specific-diagnostics">
    <h2>Defense-Specific Diagnostics</h2>
    <p class="text-muted">
      Dive into per-defense sections when you need subsystem detail rather than the higher-level
      loop-accountability narrative.
    </p>
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
  </section>

  <section class="section" data-diagnostics-section="telemetry-diagnostics">
    <h2>Telemetry Diagnostics</h2>
    <p class="text-muted">
      Freshness, transport-path notes, and bounded raw-feed diagnostics stay here rather than
      leaking back into Monitoring.
    </p>
    <DiagnosticsSection
      monitoringFreshnessSnapshot={monitoringFreshnessSnapshot}
      ipBansFreshnessSnapshot={ipBansFreshnessSnapshot}
      rawTelemetryFeed={rawTelemetryFeed}
      rawFeedMaxLines={RAW_FEED_MAX_LINES}
    />
  </section>

  <section class="section" data-diagnostics-section="external-monitoring">
    <h2>External Monitoring</h2>
    <p class="text-muted">
      Use the bounded helper examples here when you need to export or mirror diagnostics into
      external monitoring systems.
    </p>
    <ExternalMonitoringSection
      {prometheusHelper}
      {copyButtonLabel}
      copyCurlButtonLabel={copyCurlButtonLabel}
      onCopyJs={(text) => copyToClipboard(text, 'js')}
      onCopyCurl={(text) => copyToClipboard(text, 'curl')}
    />
  </section>
</section>
