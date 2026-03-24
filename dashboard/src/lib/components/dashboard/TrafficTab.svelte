<script>
  import { browser } from '$app/environment';
  import { onDestroy, onMount } from 'svelte';
  import {
    deriveEnforcedMonitoringChartRows,
    deriveMonitoringEventDisplay,
    deriveRecentEventFilterOptions,
    filterRecentEvents
  } from './monitoring-view-model.js';
  import {
    buildTimeSeries,
    hoursForRange,
    shouldFetchRange
  } from '../../domain/monitoring-normalizers.js';
  import {
    buildMonitoringCountYAxis,
    buildMonitoringTimeSeriesXAxis,
    resolveMonitoringChartTheme
  } from '../../domain/monitoring-chart-presets.js';
  import { deriveFreshnessSummary } from '../../domain/telemetry-freshness.js';
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
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import SectionBlock from './primitives/SectionBlock.svelte';
  import OverviewStats from './monitoring/OverviewStats.svelte';
  import PrimaryCharts from './monitoring/PrimaryCharts.svelte';
  import RecentEventsTable from './monitoring/RecentEventsTable.svelte';

  const EVENT_ROW_RENDER_LIMIT = 100;
  const RANGE_EVENTS_FETCH_LIMIT = 5000;
  const RANGE_EVENTS_REQUEST_TIMEOUT_MS = 10000;
  const RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS = 180000;
  const CHART_RESIZE_REDRAW_DEBOUNCE_MS = 180;
  const TIME_RANGES = Object.freeze(['hour', 'day', 'week', 'month']);
  const RANGE_CACHEABLE_WINDOWS = Object.freeze(['week', 'month']);

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
  export let analyticsSnapshot = null;
  export let eventsSnapshot = null;
  export let bansSnapshot = null;
  export let monitoringFreshnessSnapshot = null;
  export let onFetchEventsRange = null;
  export let autoRefreshEnabled = false;

  let eventTypesCanvas = null;
  let topIpsCanvas = null;
  let timeSeriesCanvas = null;
  let eventTypesChart = null;
  let topIpsChart = null;
  let timeSeriesChart = null;

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
  let eventTypesReadout = EMPTY_HALF_DOUGHNUT_READOUT;
  let rawRecentEvents = [];
  let filteredRecentEvents = [];
  let resizeRedrawTimer = null;
  let chartRefreshNonce = 0;
  let wasActive = false;
  let detachColorSchemeListener = () => {};

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

  const eventBadgeClass = (eventType) => {
    const normalized = normalizeLowerTrimmed(eventType).replace(/[^a-z_]/g, '');
    return normalized ? `badge ${normalized}` : 'badge';
  };

  const formatTimestamp = (value) => formatUnixSecondsLocal(value, '-');
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
  $: banSnapshotStatus = String(
    bansSnapshot?.status || analytics?.ban_store_status || 'available'
  ).trim().toLowerCase() || 'available';
  $: banSnapshotUnavailableMessage = banSnapshotStatus === 'unavailable'
    ? String(bansSnapshot?.message || analytics?.ban_store_message || '').trim()
    : '';
  $: freshnessStateKey = String(monitoringFreshnessSnapshot?.state || '').trim().toLowerCase();
  $: freshnessSummary = deriveFreshnessSummary(monitoringFreshnessSnapshot, {
    formatTimestamp
  });

  $: rawRecentEvents = Array.isArray(events.recent_events)
    ? events.recent_events.slice(0, RANGE_EVENTS_FETCH_LIMIT)
    : [];
  $: eventFilterOptions = deriveRecentEventFilterOptions(rawRecentEvents);
  $: filteredRecentEvents = filterRecentEvents(rawRecentEvents.slice(0, EVENT_ROW_RENDER_LIMIT), eventFilters);
  $: recentEvents = filteredRecentEvents.map((event) => deriveMonitoringEventDisplay(event));

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

  $: monitoringEventEmptyState = (() => {
    if (tabStatus?.error) {
      return {
        kind: 'error',
        message: `Traffic refresh error: ${String(tabStatus.error)}`
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
    eventTypesChart = null;
    topIpsChart = null;
    timeSeriesChart = null;
  });
</script>

<section
  id="dashboard-panel-traffic"
  class="admin-group dashboard-tab-panel"
  data-dashboard-tab-panel="traffic"
  aria-labelledby="dashboard-tab-traffic"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'false'}
  tabindex="-1"
>
  <TabStateMessage tab="traffic" status={tabStatus} />

  <div class="control-group panel-soft pad-md" data-traffic-intro>
    <h3>Traffic Visibility</h3>
    <p class="control-desc text-muted">
      Use this tab for the bounded live traffic picture: event volume, enforced-event charts,
      recent request outcomes, and a light telemetry-health summary.
    </p>
    <p class="control-desc text-muted">
      Monitoring owns loop accountability. Deep subsystem and furniture diagnostics stay in
      <a href="#diagnostics">Diagnostics</a>.
    </p>
  </div>

  <section class="section" data-traffic-section="telemetry-health">
    <SectionBlock
      title="Traffic Telemetry Health"
      description="Freshness and read-path truth for the traffic picture shown below."
      rootClass="section"
    >
      <div class="control-group panel-soft pad-sm">
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Freshness:</span>
            <span class="status-value">
              <strong>{freshnessSummary.stateLabel}</strong> | lag: {freshnessSummary.lagText} | last event: {freshnessSummary.lastEventText}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Read path:</span>
            <span class="status-value">
              transport: <code>{freshnessSummary.transportCode}</code> | slow consumer: <code>{freshnessSummary.slowConsumerState}</code> | overflow: <code>{freshnessSummary.overflow}</code>
            </span>
          </div>
          {#if freshnessSummary.partialDataWarning}
            <div class="info-row">
              <span class="info-label text-muted">Note:</span>
              <span class="status-value">{freshnessSummary.partialDataWarning}</span>
            </div>
          {/if}
        </div>
      </div>
    </SectionBlock>
  </section>

  <section class="section" data-traffic-section="traffic-overview">
    <h2>Traffic Overview</h2>
    <p class="text-muted">
      Inspect the bounded traffic summary and enforced-event charts for traffic reaching Shuma and
      the host.
    </p>
    <OverviewStats
      loading={tabStatus?.loading === true}
      {totalBans}
      {activeBans}
      {eventCount}
      {uniqueIps}
    />
    {#if banSnapshotUnavailableMessage}
      <p id="traffic-ban-state-unavailable" class="message warning">
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

  <section class="section" data-traffic-section="recent-events">
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
</section>
