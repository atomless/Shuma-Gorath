import { deriveDashboardRequestBudgets } from './dashboard-request-budgets.js';

export function createDashboardRefreshRuntime(options = {}) {
  const MONITORING_CACHE_KEY = 'shuma_dashboard_cache_monitoring_v2';
  const IP_BANS_CACHE_KEY = 'shuma_dashboard_cache_ip_bans_v1';
  const DEFAULT_CACHE_TTL_MS = 60000;
  const MONITORING_CACHE_MAX_RECENT_EVENTS = 25;
  const MONITORING_CACHE_MAX_RECENT_SIM_RUNS = 12;
  const MONITORING_CACHE_MAX_CDP_EVENTS = 50;
  const MONITORING_CACHE_MAX_BANS = 100;
  const IP_BANS_CACHE_MAX_SUGGESTIONS = 50;
  const MONITORING_DELTA_LIMIT = 120;
  const MONITORING_BOOTSTRAP_DELTA_LIMIT = 40;
  const IP_BANS_DELTA_LIMIT = 120;
  const MONITORING_FULL_RECENT_EVENTS_LIMIT = 200;
  const IP_RANGE_SUGGESTIONS_HOURS = 24;
  const IP_RANGE_SUGGESTIONS_LIMIT = 20;
  const LIVE_HOURS_WINDOW = 24;
  const DELTA_RECENT_EVENTS_LIMIT = 200;
  const STREAMABLE_TABS = Object.freeze(new Set(['monitoring', 'ip-bans']));
  // Keep dashboard updates single-writer via refresh polling to avoid
  // poll+SSE races and hidden background update churn.
  const ENABLE_REALTIME_STREAMS = false;
  const DEFAULT_FRESHNESS_SNAPSHOT = Object.freeze({
    state: 'stale',
    lag_ms: null,
    last_event_ts: null,
    slow_consumer_lag_state: 'normal',
    overflow: 'none',
    transport: 'polling',
    query_budget_requests_per_second_per_client: 1,
    refreshed_at: ''
  });
  const cursorState = {
    monitoring: '',
    ipBans: ''
  };
  const streamState = {
    monitoring: null,
    ipBans: null
  };
  const baselineState = {
    monitoring: false,
    ipBans: false
  };
  let activeRealtimeTab = '';
  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (value) => String(value || '');
  const getApiClient =
    typeof options.getApiClient === 'function' ? options.getApiClient : () => null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;
  const deriveMonitoringAnalytics =
    typeof options.deriveMonitoringAnalytics === 'function'
      ? options.deriveMonitoringAnalytics
      : () => ({ ban_count: 0, shadow_mode: false, fail_mode: 'open' });
  const storage = options.storage && typeof options.storage === 'object'
    ? options.storage
    : (typeof window !== 'undefined' && window.localStorage ? window.localStorage : null);
  const cacheTtlMs = (() => {
    const numeric = Number(options.cacheTtlMs);
    if (!Number.isFinite(numeric) || numeric <= 0) return DEFAULT_CACHE_TTL_MS;
    return Math.max(1000, Math.floor(numeric));
  })();

  const isConfigSnapshotEmpty = (config) =>
    !config || typeof config !== 'object' || Object.keys(config).length === 0;
  const hasRequiredSharedConfigRuntimeTruth = (runtime) => {
    if (!runtime || typeof runtime !== 'object') return false;
    if (!Object.prototype.hasOwnProperty.call(runtime, 'admin_config_write_enabled')) return false;
    return typeof runtime.runtime_environment === 'string' && runtime.runtime_environment.trim().length > 0;
  };
  const isConfigRuntimeSnapshotEmpty = (runtime) =>
    !runtime ||
    typeof runtime !== 'object' ||
    Object.keys(runtime).length === 0 ||
    !hasRequiredSharedConfigRuntimeTruth(runtime);
  const hasConfigEnvelope = (configEnvelope) => {
    const source = configEnvelope && typeof configEnvelope === 'object' ? configEnvelope : {};
    return !isConfigSnapshotEmpty(source.config) && !isConfigRuntimeSnapshotEmpty(source.runtime);
  };
  const toArray = (value) => (Array.isArray(value) ? value : []);
  const toTabCursorKey = (tab) => (tab === 'ip-bans' ? 'ipBans' : 'monitoring');
  const freshnessSnapshotKey = (tab) =>
    (tab === 'ip-bans' ? 'ipBansFreshness' : 'monitoringFreshness');
  const parseNonNegativeOrNull = (value) => {
    if (value === null || value === undefined || value === '') return null;
    const numeric = Number(value);
    return Number.isFinite(numeric) && numeric >= 0 ? numeric : null;
  };

  const normalizeFreshnessSnapshot = (value = {}, fallbackTransport = 'polling') => {
    const source = value && typeof value === 'object' ? value : {};
    const lagValue = parseNonNegativeOrNull(source.lag_ms);
    const lastEventValue = Number(source.last_event_ts);
    const queryBudgetValue = Number(source.query_budget_requests_per_second_per_client);
    return {
      state: String(source.state || DEFAULT_FRESHNESS_SNAPSHOT.state),
      lag_ms: lagValue,
      last_event_ts: Number.isFinite(lastEventValue) && lastEventValue > 0 ? lastEventValue : null,
      slow_consumer_lag_state: String(
        source.slow_consumer_lag_state || DEFAULT_FRESHNESS_SNAPSHOT.slow_consumer_lag_state
      ),
      overflow: String(source.overflow || DEFAULT_FRESHNESS_SNAPSHOT.overflow),
      transport: String(source.transport || fallbackTransport || DEFAULT_FRESHNESS_SNAPSHOT.transport),
      query_budget_requests_per_second_per_client:
        Number.isFinite(queryBudgetValue) && queryBudgetValue > 0
          ? queryBudgetValue
          : DEFAULT_FRESHNESS_SNAPSHOT.query_budget_requests_per_second_per_client,
      refreshed_at: new Date().toISOString()
    };
  };

  const updateFreshnessSnapshot = (tab, payload = {}, fallbackTransport = 'polling') => {
    const key = freshnessSnapshotKey(tab);
    const sourcePayload = payload && typeof payload === 'object' ? payload : {};
    const next = normalizeFreshnessSnapshot(sourcePayload, fallbackTransport);
    applySnapshots({ [key]: next });
    return next;
  };

  const applyConfigEnvelope = (configEnvelope = null) => {
    const source = configEnvelope && typeof configEnvelope === 'object' ? configEnvelope : {};
    const config = source.config && typeof source.config === 'object' ? source.config : {};
    const runtime = source.runtime && typeof source.runtime === 'object' ? source.runtime : {};
    applySnapshots({
      config,
      configRuntime: runtime
    });
    return { config, runtime };
  };

  const eventCursorKey = (event = {}) => {
    const source = event && typeof event === 'object' ? event : {};
    if (typeof source.cursor === 'string' && source.cursor.trim()) {
      return source.cursor.trim();
    }
    const ts = Number(source.ts || 0);
    const eventName = String(source.event || '');
    const ip = String(source.ip || '');
    const reason = String(source.reason || '');
    const outcome = String(source.outcome || '');
    return `${ts}|${eventName}|${ip}|${reason}|${outcome}`;
  };

  const mergeRecentEvents = (existing = [], incoming = []) => {
    const mergedByKey = new Map();
    toArray(existing).forEach((event) => {
      mergedByKey.set(eventCursorKey(event), event);
    });
    toArray(incoming).forEach((event) => {
      mergedByKey.set(eventCursorKey(event), event);
    });
    return Array.from(mergedByKey.values())
      .sort((left, right) => Number(right?.ts || 0) - Number(left?.ts || 0))
      .slice(0, DELTA_RECENT_EVENTS_LIMIT);
  };

  const closeStream = (tab) => {
    const key = toTabCursorKey(tab);
    const stream = streamState[key];
    if (!stream || typeof stream.close !== 'function') return;
    try {
      stream.close();
    } catch (_error) {}
    streamState[key] = null;
  };

  const closeAllStreams = () => {
    closeStream('monitoring');
    closeStream('ip-bans');
  };

  function compactBansSnapshot(bansData = {}) {
    const source = bansData && typeof bansData === 'object' ? bansData : {};
    return {
      ...source,
      bans: toArray(source.bans).slice(0, MONITORING_CACHE_MAX_BANS)
    };
  }

  function compactIpRangeSuggestionsSnapshot(suggestionsData = {}) {
    const source = suggestionsData && typeof suggestionsData === 'object' ? suggestionsData : {};
    return {
      ...source,
      summary: source.summary && typeof source.summary === 'object' ? source.summary : {},
      suggestions: toArray(source.suggestions).slice(0, IP_BANS_CACHE_MAX_SUGGESTIONS)
    };
  }

  function compactMonitoringSnapshot(monitoringData = {}) {
    const source = monitoringData && typeof monitoringData === 'object' ? monitoringData : {};
    const details = source.details && typeof source.details === 'object' ? source.details : {};
    const events = details.events && typeof details.events === 'object' ? details.events : {};
    const cdpEvents = details.cdp_events && typeof details.cdp_events === 'object' ? details.cdp_events : {};
    const bans = details.bans && typeof details.bans === 'object' ? details.bans : {};
    return {
      ...source,
      details: {
        ...details,
        events: {
          ...events,
          recent_events: toArray(events.recent_events).slice(0, MONITORING_CACHE_MAX_RECENT_EVENTS),
          recent_sim_runs: toArray(events.recent_sim_runs).slice(0, MONITORING_CACHE_MAX_RECENT_SIM_RUNS)
        },
        bans: compactBansSnapshot(bans),
        cdp_events: {
          ...cdpEvents,
          events: toArray(cdpEvents.events).slice(0, MONITORING_CACHE_MAX_CDP_EVENTS),
          limit: Math.min(
            MONITORING_CACHE_MAX_CDP_EVENTS,
            Number.isFinite(Number(cdpEvents.limit)) && Number(cdpEvents.limit) > 0
              ? Math.floor(Number(cdpEvents.limit))
              : MONITORING_CACHE_MAX_CDP_EVENTS
          )
        }
      }
    };
  }

  function buildMonitoringSnapshots(monitoringData = {}, configSnapshot = {}, configRuntimeSnapshot = {}) {
    const monitoring = monitoringData && typeof monitoringData === 'object' ? monitoringData : {};
    const monitoringDetails =
      monitoring && typeof monitoring.details === 'object' ? monitoring.details : {};
    const analyticsResponse = monitoringDetails.analytics || {};
    const events = monitoringDetails.events || {};
    const bansData = monitoringDetails.bans || { bans: [] };
    const mazeData = monitoringDetails.maze || {};
    const cdpData = monitoringDetails.cdp || {};
    const cdpEventsData = monitoringDetails.cdp_events || { events: [] };
    const analytics = deriveMonitoringAnalytics(configSnapshot, configRuntimeSnapshot, analyticsResponse);
    if (
      !Number.isFinite(Number(analytics.ban_count)) &&
      String(bansData.status || 'available') !== 'unavailable' &&
      Array.isArray(bansData.bans)
    ) {
      analytics.ban_count = bansData.bans.length;
    }
    return {
      monitoring,
      analytics,
      events,
      bans: bansData,
      maze: mazeData,
      cdp: cdpData,
      cdpEvents: cdpEventsData
    };
  }

  function shouldReadFromCache(reason = 'manual') {
    return !(
      reason === 'auto-refresh' ||
      reason === 'manual-refresh' ||
      reason === 'click' ||
      reason === 'keyboard' ||
      reason === 'hashchange' ||
      reason === 'config-save' ||
      reason === 'ban-save' ||
      reason === 'unban-save'
    );
  }

  function shouldReuseExistingSharedConfig(reason = 'manual') {
    return reason === 'auto-refresh';
  }

  function readCache(cacheKey) {
    if (!storage) return null;
    try {
      const raw = storage.getItem(cacheKey);
      if (!raw) return null;
      const parsed = JSON.parse(raw);
      const cachedAt = Number(parsed?.cachedAt || 0);
      if (!Number.isFinite(cachedAt) || cachedAt <= 0 || (Date.now() - cachedAt) > cacheTtlMs) {
        storage.removeItem(cacheKey);
        return null;
      }
      const payload = parsed && typeof parsed.payload === 'object' ? parsed.payload : null;
      return payload && typeof payload === 'object' ? payload : null;
    } catch (_error) {
      return null;
    }
  }

  function writeCache(cacheKey, payload) {
    if (!storage || !payload || typeof payload !== 'object') return;
    try {
      storage.setItem(cacheKey, JSON.stringify({
        cachedAt: Date.now(),
        payload
      }));
    } catch (_error) {}
  }

  function clearCache(cacheKey) {
    if (!storage) return;
    try {
      storage.removeItem(cacheKey);
    } catch (_error) {}
  }

  function clearAllCaches() {
    clearCache(MONITORING_CACHE_KEY);
    clearCache(IP_BANS_CACHE_KEY);
    closeAllStreams();
    cursorState.monitoring = '';
    cursorState.ipBans = '';
    baselineState.monitoring = false;
    baselineState.ipBans = false;
    activeRealtimeTab = '';
    applySnapshots({
      monitoringFreshness: normalizeFreshnessSnapshot({}, 'polling'),
      ipBansFreshness: normalizeFreshnessSnapshot({}, 'polling')
    });
  }

  function toRequestOptions(runtimeOptions = {}, telemetry = {}) {
    const next = {};
    if (runtimeOptions && runtimeOptions.signal) {
      next.signal = runtimeOptions.signal;
    }
    const source = telemetry && typeof telemetry.source === 'string' && telemetry.source.trim()
      ? telemetry.source.trim()
      : 'tab-refresh';
    const reason = telemetry && typeof telemetry.reason === 'string'
      ? telemetry.reason
      : '';
    const tab = telemetry && typeof telemetry.tab === 'string'
      ? telemetry.tab
      : '';
    next.telemetry = {
      source,
      reason,
      tab
    };
    return next;
  }

  function applySnapshots(updates = {}) {
    const dashboardState = getStateStore();
    if (!dashboardState || !updates || typeof updates !== 'object') return;
    if (typeof dashboardState.setSnapshots === 'function') {
      dashboardState.setSnapshots(updates);
      return;
    }
    Object.entries(updates).forEach(([key, value]) => {
      dashboardState.setSnapshot(key, value);
    });
  }

  function showTabLoading(tab, message = 'Loading...') {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.clearTabError(tab);
    dashboardState.setTabEmpty(tab, false, '');
    dashboardState.setTabLoading(tab, true, message);
  }

  function showTabError(tab, message) {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.setTabEmpty(tab, false, '');
    dashboardState.setTabError(tab, message);
  }

  function errorMessage(error, fallback = 'Refresh failed') {
    const message = error && typeof error === 'object' ? error.message : '';
    const trimmed = String(message || '').trim();
    return trimmed || fallback;
  }

  function showTabEmpty(tab, message) {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.clearTabError(tab);
    dashboardState.setTabLoading(tab, false, '');
    dashboardState.setTabEmpty(tab, true, message);
  }

  function clearTabStateMessage(tab) {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.setTabLoading(tab, false, '');
    dashboardState.setTabEmpty(tab, false, '');
    dashboardState.clearTabError(tab);
  }

  function shouldUseCursorDelta(reason = 'manual') {
    return !(
      reason === 'config-save' ||
      reason === 'ban-save' ||
      reason === 'unban-save'
    );
  }

  function shouldForceFullMonitoringSnapshot(reason = 'manual') {
    return false;
  }

  function syncCursorFromDelta(tab, delta = {}) {
    const key = toTabCursorKey(tab);
    const nextCursor = typeof delta.next_cursor === 'string' ? delta.next_cursor : '';
    const windowEndCursor =
      typeof delta.window_end_cursor === 'string' ? delta.window_end_cursor : '';
    if (nextCursor.trim()) {
      cursorState[key] = nextCursor;
      return;
    }
    if (windowEndCursor.trim()) {
      cursorState[key] = windowEndCursor;
    }
  }

  async function seedCursorToWindowEnd(tab, requestOptions = {}) {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return '';
    const isIpBans = tab === 'ip-bans';
    const key = toTabCursorKey(tab);
    const delta = isIpBans
      ? await dashboardApiClient.getIpBansDelta(
          { hours: LIVE_HOURS_WINDOW, limit: 1 },
          requestOptions
        )
      : await dashboardApiClient.getMonitoringDelta(
          { hours: LIVE_HOURS_WINDOW, limit: 1 },
          requestOptions
        );
    const windowEndCursor =
      typeof delta.window_end_cursor === 'string' ? delta.window_end_cursor.trim() : '';
    const nextCursor =
      typeof delta.next_cursor === 'string' ? delta.next_cursor.trim() : '';
    if (windowEndCursor) {
      cursorState[key] = windowEndCursor;
    } else if (nextCursor) {
      cursorState[key] = nextCursor;
    }
    return cursorState[key] || '';
  }

  function seedCursorToWindowEndDeferred(tab, requestOptions = {}) {
    void seedCursorToWindowEnd(tab, requestOptions).catch(() => {});
  }

  function applyMonitoringDeltaSnapshots(delta = {}, transport = 'cursor_delta_poll') {
    const dashboardState = getStateStore();
    const snapshots = dashboardState ? dashboardState.getState().snapshots || {} : {};
    const priorEvents = snapshots.events && typeof snapshots.events === 'object'
      ? snapshots.events
      : {};
    const priorMonitoring = snapshots.monitoring && typeof snapshots.monitoring === 'object'
      ? snapshots.monitoring
      : {};
    const priorMonitoringDetails = priorMonitoring.details && typeof priorMonitoring.details === 'object'
      ? priorMonitoring.details
      : {};
    const incomingEvents = toArray(delta.events);
    const mergedRecentEvents = mergeRecentEvents(priorEvents.recent_events, incomingEvents);
    const hasRecentSimRuns = Object.prototype.hasOwnProperty.call(delta || {}, 'recent_sim_runs');
    const nextEvents = {
      ...priorEvents,
      recent_events: mergedRecentEvents,
      recent_sim_runs: hasRecentSimRuns
        ? toArray(delta.recent_sim_runs).slice(0, MONITORING_CACHE_MAX_RECENT_SIM_RUNS)
        : toArray(priorEvents.recent_sim_runs).slice(0, MONITORING_CACHE_MAX_RECENT_SIM_RUNS)
    };
    const nextMonitoring = {
      ...priorMonitoring,
      details: {
        ...priorMonitoringDetails,
        events: nextEvents
      }
    };
    applySnapshots({
      events: nextEvents,
      monitoring: nextMonitoring
    });
    const freshnessPayload = delta && typeof delta.freshness === 'object'
      ? delta.freshness
      : {};
    updateFreshnessSnapshot(
      'monitoring',
      freshnessPayload,
      transport
    );
    const compactMonitoring = compactMonitoringSnapshot(nextMonitoring);
    writeCache(MONITORING_CACHE_KEY, { monitoring: compactMonitoring });
  }

  function applyIpBansDeltaSnapshots(delta = {}, transport = 'cursor_delta_poll') {
    const dashboardState = getStateStore();
    const snapshots = dashboardState ? dashboardState.getState().snapshots || {} : {};
    const priorBans = snapshots.bans && typeof snapshots.bans === 'object' ? snapshots.bans : {};
    const nextBans = {
      ...priorBans,
      status: String(delta.active_bans_status || priorBans.status || 'available'),
      message: String(delta.active_bans_message || priorBans.message || ''),
      bans: toArray(delta.active_bans)
    };
    applySnapshots({ bans: nextBans });
    const freshnessPayload = delta && typeof delta.freshness === 'object'
      ? delta.freshness
      : {};
    updateFreshnessSnapshot(
      'ip-bans',
      freshnessPayload,
      transport
    );
    const existingCache = readCache(IP_BANS_CACHE_KEY) || {};
    writeCache(IP_BANS_CACHE_KEY, {
      ...existingCache,
      bans: compactBansSnapshot(nextBans)
    });
  }

  function startRealtimeStream(tab) {
    const normalized = normalizeTab(tab);
    if (!STREAMABLE_TABS.has(normalized)) return;
    if (typeof EventSource !== 'function') return;
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return;
    const key = toTabCursorKey(normalized);
    if (streamState[key]) return;
    const streamUrlFactory = normalized === 'monitoring'
      ? dashboardApiClient.getMonitoringStreamUrl
      : dashboardApiClient.getIpBansStreamUrl;
    if (typeof streamUrlFactory !== 'function') return;
    let streamUrl = '';
    try {
      streamUrl = streamUrlFactory({
        hours: LIVE_HOURS_WINDOW,
        limit: normalized === 'monitoring' ? MONITORING_DELTA_LIMIT : IP_BANS_DELTA_LIMIT,
        after_cursor: cursorState[key]
      });
    } catch (_error) {
      return;
    }
    if (!streamUrl) return;
    const source = new EventSource(streamUrl, { withCredentials: true });
    source.onmessage = (event) => {
      try {
        const payload = JSON.parse(String(event.data || '{}'));
        syncCursorFromDelta(normalized, payload);
        if (normalized === 'monitoring') {
          applyMonitoringDeltaSnapshots(payload, 'sse');
        } else {
          applyIpBansDeltaSnapshots(payload, 'sse');
        }
        const dashboardState = getStateStore();
        if (dashboardState) {
          dashboardState.markTabUpdated(normalized);
        }
      } catch (_error) {}
    };
    source.onerror = () => {
      if (streamState[key] === source) {
        try {
          source.close();
        } catch (_error) {}
        streamState[key] = null;
      }
    };
    streamState[key] = source;
  }

  async function refreshSharedConfig(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    const dashboardState = getStateStore();
    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: 'config',
      reason,
      source: 'shared-config-refresh'
    });
    const existingConfig = dashboardState ? dashboardState.getSnapshot('config') : null;
    const existingRuntime = dashboardState ? dashboardState.getSnapshot('configRuntime') : null;

    if (!dashboardApiClient) {
      return {
        config: existingConfig && typeof existingConfig === 'object' ? existingConfig : {},
        runtime: existingRuntime && typeof existingRuntime === 'object' ? existingRuntime : {}
      };
    }
    if (
      shouldReuseExistingSharedConfig(reason) &&
      !isConfigSnapshotEmpty(existingConfig) &&
      !isConfigRuntimeSnapshotEmpty(existingRuntime)
    ) {
      return {
        config: existingConfig,
        runtime: existingRuntime
      };
    }

    const configEnvelope = await dashboardApiClient.getConfig(requestOptions);
    return applyConfigEnvelope(configEnvelope);
  }

  async function refreshMonitoringTab(reason = 'manual', runtimeOptions = {}, surfaceTab = 'game-loop') {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return;

    const isAutoRefresh = reason === 'auto-refresh';
    const loadingMessage =
      surfaceTab === 'traffic'
        ? 'Loading traffic visibility...'
        : (surfaceTab === 'diagnostics'
          ? 'Loading diagnostics...'
          : 'Loading game loop data...');
    const emptyMessage =
      surfaceTab === 'traffic'
        ? 'No traffic events are visible yet. Traffic will populate as telemetry arrives.'
        : 'No operational events yet. Game Loop will populate as traffic arrives.';
    if (!isAutoRefresh) {
      showTabLoading(surfaceTab, loadingMessage);
    }

    const dashboardState = getStateStore();
    if (shouldReadFromCache(reason)) {
      const cachedMonitoring = readCache(MONITORING_CACHE_KEY);
      if (cachedMonitoring) {
        const configSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
        const configRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
        const monitoringData =
          cachedMonitoring && typeof cachedMonitoring.monitoring === 'object'
            ? cachedMonitoring.monitoring
            : (cachedMonitoring && typeof cachedMonitoring === 'object'
              ? cachedMonitoring
              : {});
        applySnapshots(buildMonitoringSnapshots(monitoringData, configSnapshot, configRuntimeSnapshot));
        baselineState.monitoring = true;
        if (dashboardState && dashboardState.getDerivedState().monitoringEmpty) {
          showTabEmpty(surfaceTab, emptyMessage);
        } else {
          clearTabStateMessage(surfaceTab);
        }
        return;
      }
    }

    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: surfaceTab,
      reason,
      source: 'tab-refresh'
    });
    const currentConfigRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
    const requestBudgets = deriveDashboardRequestBudgets(currentConfigRuntimeSnapshot);
    const monitoringRequestOptions = {
      ...requestOptions,
      timeoutMs: requestBudgets.monitoringRequestTimeoutMs
    };
    const monitoringDeltaRequestOptions = {
      ...requestOptions,
      timeoutMs: requestBudgets.monitoringDeltaTimeoutMs
    };
    const applyMonitoringSnapshot = (monitoringData = {}, { writeCursor = false } = {}) => {
      const configSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
      const configRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
      const monitoringSnapshots = buildMonitoringSnapshots(
        monitoringData,
        configSnapshot,
        configRuntimeSnapshot
      );
      const compactMonitoring = compactMonitoringSnapshot(monitoringData);
      const compactBans = compactBansSnapshot(monitoringSnapshots.bans);
      applySnapshots(monitoringSnapshots);
      baselineState.monitoring = true;
      writeCache(MONITORING_CACHE_KEY, { monitoring: compactMonitoring });
      const existingIpBansCache = readCache(IP_BANS_CACHE_KEY) || {};
      writeCache(IP_BANS_CACHE_KEY, { ...existingIpBansCache, bans: compactBans });
      updateFreshnessSnapshot(
        'monitoring',
        monitoringData.freshness || {},
        'snapshot_poll'
      );
      if (writeCursor) {
        const windowEndCursor =
          monitoringData && typeof monitoringData.window_end_cursor === 'string'
            ? monitoringData.window_end_cursor.trim()
            : '';
        if (windowEndCursor) {
          cursorState.monitoring = windowEndCursor;
        }
      }
    };
    const showMonitoringStateMessage = () => {
      if (dashboardState && dashboardState.getDerivedState().monitoringEmpty) {
        showTabEmpty(surfaceTab, emptyMessage);
      } else {
        clearTabStateMessage(surfaceTab);
      }
    };
    const handledBootstrapResult = (promise) =>
      promise
        .then((value) => ({ ok: true, value }))
        .catch((error) => ({ ok: false, error }));
    const queueDetailedMonitoringHydration = (monitoringPromise) => {
      if (!monitoringPromise || typeof monitoringPromise.then !== 'function') return;
      void monitoringPromise
        .then((result) => {
          if (!result || result.ok !== true) {
            showTabError(surfaceTab, errorMessage(result?.error, 'Failed to load monitoring data.'));
            return null;
          }
          const monitoringData = result.value;
          applyMonitoringSnapshot(monitoringData, { writeCursor: !cursorState.monitoring.trim() });
          showMonitoringStateMessage();
          if (requestBudgets.autoHydrateFullMonitoring !== true) {
            return null;
          }
          return fetchFullMonitoring();
        })
        .catch(() => {});
    };
    const fetchFullMonitoring = async () => {
      const monitoringData = await dashboardApiClient.getMonitoring(
        { hours: 24, limit: MONITORING_FULL_RECENT_EVENTS_LIMIT },
        monitoringRequestOptions
      );
      applyMonitoringSnapshot(monitoringData, { writeCursor: false });
      const shouldSeedCursor =
        !shouldForceFullMonitoringSnapshot(reason) &&
        typeof dashboardApiClient.getMonitoringDelta === 'function' &&
        !cursorState.monitoring.trim();
      if (shouldSeedCursor) {
        seedCursorToWindowEndDeferred('monitoring', monitoringDeltaRequestOptions);
      }
    };

    const canUseBootstrap =
      !baselineState.monitoring &&
      typeof dashboardApiClient.getMonitoringBootstrap === 'function';
    const canUseDeltaBootstrap =
      !baselineState.monitoring &&
      typeof dashboardApiClient.getMonitoringDelta === 'function' &&
      shouldUseCursorDelta(reason);
    if (canUseDeltaBootstrap) {
      const bootstrapPromise = canUseBootstrap
        ? handledBootstrapResult(dashboardApiClient.getMonitoringBootstrap(
          { hours: 24, limit: MONITORING_FULL_RECENT_EVENTS_LIMIT },
          monitoringRequestOptions
        ))
        : null;
      try {
        const delta = await dashboardApiClient.getMonitoringDelta(
          {
            hours: LIVE_HOURS_WINDOW,
            limit: MONITORING_BOOTSTRAP_DELTA_LIMIT,
            after_cursor: ''
          },
          monitoringDeltaRequestOptions
        );
        syncCursorFromDelta('monitoring', delta);
        applyMonitoringDeltaSnapshots(delta, 'cursor_delta_bootstrap');
        baselineState.monitoring = true;
        showMonitoringStateMessage();
        if (bootstrapPromise) {
          queueDetailedMonitoringHydration(bootstrapPromise);
        } else if (requestBudgets.autoHydrateFullMonitoring === true) {
          void fetchFullMonitoring().catch(() => {});
        }
        return;
      } catch (_error) {
        if (bootstrapPromise) {
          const bootstrapResult = await bootstrapPromise;
          if (bootstrapResult && bootstrapResult.ok === true) {
            const monitoringData = bootstrapResult.value;
            applyMonitoringSnapshot(monitoringData, { writeCursor: true });
            showMonitoringStateMessage();
            if (requestBudgets.autoHydrateFullMonitoring === true) {
              void fetchFullMonitoring().catch(() => {});
            }
            return;
          }
          showTabError(
            surfaceTab,
            errorMessage(bootstrapResult?.error, 'Failed to load monitoring data.')
          );
          return;
        }
      }
    }
    if (canUseBootstrap) {
      try {
        const monitoringData = await dashboardApiClient.getMonitoringBootstrap(
          { hours: 24, limit: MONITORING_FULL_RECENT_EVENTS_LIMIT },
          monitoringRequestOptions
        );
        applyMonitoringSnapshot(monitoringData, { writeCursor: true });
        showMonitoringStateMessage();
        if (requestBudgets.autoHydrateFullMonitoring === true) {
          void fetchFullMonitoring().catch(() => {});
        }
        return;
      } catch (_error) {}
    }

    const canUseDelta =
      !shouldForceFullMonitoringSnapshot(reason) &&
      baselineState.monitoring &&
      shouldUseCursorDelta(reason) &&
      typeof dashboardApiClient.getMonitoringDelta === 'function';
    if (canUseDelta) {
      try {
        if (!cursorState.monitoring.trim()) {
          await seedCursorToWindowEnd('monitoring', monitoringDeltaRequestOptions);
        }
        const delta = await dashboardApiClient.getMonitoringDelta(
          {
            hours: LIVE_HOURS_WINDOW,
            limit: MONITORING_DELTA_LIMIT,
            after_cursor: cursorState.monitoring
          },
          monitoringDeltaRequestOptions
        );
        syncCursorFromDelta('monitoring', delta);
        applyMonitoringDeltaSnapshots(delta, 'cursor_delta_poll');
        if (delta.overflow === 'limit_exceeded') {
          await fetchFullMonitoring();
        }
      } catch (_error) {
        await fetchFullMonitoring();
      }
    } else {
      await fetchFullMonitoring();
    }

    showMonitoringStateMessage();
  }

  const refreshTrafficTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshMonitoringTab(reason, runtimeOptions, 'traffic');

  const refreshDiagnosticsTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshMonitoringTab(reason, runtimeOptions, 'diagnostics');

  async function refreshIpBansTab(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return;
    const includeConfigRefresh = reason !== 'auto-refresh';
    if (reason !== 'auto-refresh') {
      showTabLoading('ip-bans', 'Loading ban list...');
    }
    const dashboardState = getStateStore();
    if (shouldReadFromCache(reason)) {
      const cachedIpBans = readCache(IP_BANS_CACHE_KEY);
      if (cachedIpBans) {
        applySnapshots(cachedIpBans);
        baselineState.ipBans = true;
        clearTabStateMessage('ip-bans');
        return;
      }
    }

    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: 'ip-bans',
      reason,
      source: 'tab-refresh'
    });
    const readIpRangeSuggestions = async () => {
      try {
        return await dashboardApiClient.getIpRangeSuggestions(
          { hours: IP_RANGE_SUGGESTIONS_HOURS, limit: IP_RANGE_SUGGESTIONS_LIMIT },
          requestOptions
        );
      } catch (_error) {
        return null;
      }
    };
    const readSharedConfigSupport = async () => {
      if (!includeConfigRefresh) return null;
      try {
        return await refreshSharedConfig(reason, runtimeOptions);
      } catch (_error) {
        return null;
      }
    };
    const applyIpRangeSuggestionsSupport = (ipRangeSuggestions) => {
      if (!ipRangeSuggestions || typeof ipRangeSuggestions !== 'object') return;
      const compactSuggestions = compactIpRangeSuggestionsSnapshot(ipRangeSuggestions);
      applySnapshots({ ipRangeSuggestions });
      const existingCache = readCache(IP_BANS_CACHE_KEY) || {};
      writeCache(IP_BANS_CACHE_KEY, {
        ...existingCache,
        ipRangeSuggestions: compactSuggestions
      });
    };
    const fetchFullIpBans = async () => {
      const [bansData, ipRangeSuggestions, configEnvelope] = await Promise.all([
        dashboardApiClient.getBans(requestOptions),
        readIpRangeSuggestions(),
        readSharedConfigSupport()
      ]);
      const compactBans = compactBansSnapshot(bansData);
      applySnapshots({ bans: bansData });
      baselineState.ipBans = true;
      applyIpRangeSuggestionsSupport(ipRangeSuggestions);
      if (hasConfigEnvelope(configEnvelope)) {
        applyConfigEnvelope(configEnvelope);
      }
      const existingCache = readCache(IP_BANS_CACHE_KEY) || {};
      writeCache(IP_BANS_CACHE_KEY, {
        ...existingCache,
        bans: compactBans
      });
      try {
        await seedCursorToWindowEnd('ip-bans', requestOptions);
      } catch (_error) {}
    };

    const canUseDelta =
      baselineState.ipBans &&
      shouldUseCursorDelta(reason) &&
      typeof dashboardApiClient.getIpBansDelta === 'function';
    if (canUseDelta) {
      try {
        if (!cursorState.ipBans.trim()) {
          await seedCursorToWindowEnd('ip-bans', requestOptions);
        }
        const delta = await dashboardApiClient.getIpBansDelta(
          {
            hours: LIVE_HOURS_WINDOW,
            limit: IP_BANS_DELTA_LIMIT,
            after_cursor: cursorState.ipBans
          },
          requestOptions
        );
        syncCursorFromDelta('ip-bans', delta);
        applyIpBansDeltaSnapshots(delta, 'cursor_delta_poll');
        const ipRangeSuggestions = await readIpRangeSuggestions();
        applyIpRangeSuggestionsSupport(ipRangeSuggestions);
        if (delta.overflow === 'limit_exceeded') {
          await fetchFullIpBans();
        } else if (includeConfigRefresh) {
          const configEnvelope = await readSharedConfigSupport();
          if (hasConfigEnvelope(configEnvelope)) {
            applyConfigEnvelope(configEnvelope);
          }
        }
      } catch (_error) {
        await fetchFullIpBans();
      }
    } else {
      await fetchFullIpBans();
    }

    if (reason === 'ban-save' || reason === 'unban-save') {
      clearCache(MONITORING_CACHE_KEY);
    }

    clearTabStateMessage('ip-bans');
  }

  async function refreshConfigBackedTab(
    tab,
    reason = 'manual',
    loadingMessage,
    emptyMessage,
    runtimeOptions = {}
  ) {
    if (reason !== 'auto-refresh') {
      showTabLoading(tab, loadingMessage);
    }
    const configEnvelope = await refreshSharedConfig(reason, runtimeOptions);
    if (isConfigSnapshotEmpty(configEnvelope.config)) {
      showTabEmpty(tab, emptyMessage);
    } else {
      clearTabStateMessage(tab);
    }
  }

  async function refreshStatusTab(reason = 'manual', runtimeOptions = {}) {
    await refreshConfigBackedTab(
      'status',
      reason,
      'Loading status signals...',
      'No status config snapshot available yet.',
      runtimeOptions
    );

    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient || typeof dashboardApiClient.getMonitoring !== 'function') return;

    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: 'status',
      reason,
      source: 'status-operational-refresh'
    });
    const dashboardState = getStateStore();
    const configRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
    const requestBudgets = deriveDashboardRequestBudgets(configRuntimeSnapshot);

    try {
      const monitoringData = await dashboardApiClient.getMonitoring(
        { hours: 24, limit: 1 },
        {
          ...requestOptions,
          timeoutMs: requestBudgets.monitoringRequestTimeoutMs
        }
      );
      const currentConfigSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
      const currentConfigRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
      const monitoringSnapshots = buildMonitoringSnapshots(
        monitoringData,
        currentConfigSnapshot,
        currentConfigRuntimeSnapshot
      );
      applySnapshots(monitoringSnapshots);
      updateFreshnessSnapshot(
        'monitoring',
        monitoringData && typeof monitoringData === 'object' ? monitoringData.freshness || {} : {},
        'snapshot_poll'
      );
    } catch (_error) {}

    if (typeof dashboardApiClient.getIpBansDelta !== 'function') return;

    try {
      const ipBansDelta = await dashboardApiClient.getIpBansDelta(
        {
          hours: LIVE_HOURS_WINDOW,
          limit: 1,
          after_cursor: ''
        },
        {
          ...requestOptions,
          timeoutMs: requestBudgets.monitoringDeltaTimeoutMs
        }
      );
      updateFreshnessSnapshot(
        'ip-bans',
        ipBansDelta && typeof ipBansDelta === 'object' ? ipBansDelta.freshness || {} : {},
        'cursor_delta_poll'
      );
    } catch (_error) {}
  }

  const refreshRedTeamTab = async (reason = 'manual', runtimeOptions = {}) => {
    if (reason === 'auto-refresh') {
      const dashboardState = getStateStore();
      const existingConfig = dashboardState ? dashboardState.getSnapshot('config') : null;
      const existingRuntime = dashboardState ? dashboardState.getSnapshot('configRuntime') : null;
      if (isConfigSnapshotEmpty(existingConfig) || isConfigRuntimeSnapshotEmpty(existingRuntime)) {
        await Promise.all([
          refreshMonitoringTab(reason, runtimeOptions),
          refreshSharedConfig(reason, runtimeOptions)
        ]);
        return;
      }
      await refreshMonitoringTab(reason, runtimeOptions);
      return;
    }
    await Promise.all([
      refreshMonitoringTab(reason, runtimeOptions),
      refreshSharedConfig(reason, runtimeOptions)
    ]);
  };

  async function refreshVerificationTab(reason = 'manual', runtimeOptions = {}) {
    if (reason !== 'auto-refresh') {
      showTabLoading('verification', 'Loading verification controls...');
    }

    const configEnvelope = await refreshSharedConfig(reason, runtimeOptions);
    if (isConfigSnapshotEmpty(configEnvelope.config)) {
      applySnapshots({ operatorSnapshot: null });
      showTabEmpty('verification', 'No verification config snapshot available yet.');
      return;
    }

    const dashboardApiClient = getApiClient();
    const dashboardState = getStateStore();
    const configRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
    const requestBudgets = deriveDashboardRequestBudgets(configRuntimeSnapshot);
    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: 'verification',
      reason,
      source: 'verification-operator-snapshot-refresh'
    });

    if (dashboardApiClient && typeof dashboardApiClient.getOperatorSnapshot === 'function') {
      try {
        const operatorSnapshot = await dashboardApiClient.getOperatorSnapshot({
          ...requestOptions,
          timeoutMs: requestBudgets.monitoringRequestTimeoutMs
        });
        applySnapshots({ operatorSnapshot });
      } catch (_error) {
        applySnapshots({ operatorSnapshot: null });
      }
    }

    clearTabStateMessage('verification');
  }

  async function refreshMonitoringAccountabilityData(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) {
      applySnapshots({ operatorSnapshot: null, oversightHistory: null, oversightAgentStatus: null });
      return;
    }

    const dashboardState = getStateStore();
    const configRuntimeSnapshot = dashboardState ? dashboardState.getSnapshot('configRuntime') : {};
    const requestBudgets = deriveDashboardRequestBudgets(configRuntimeSnapshot);
    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: 'game-loop',
      reason,
      source: 'game-loop-accountability-refresh'
    });
    const accountabilityRequestOptions = {
      ...requestOptions,
      timeoutMs: requestBudgets.monitoringRequestTimeoutMs
    };

    const readOptional = async (reader) => {
      if (typeof reader !== 'function') return null;
      try {
        return await reader(accountabilityRequestOptions);
      } catch (_error) {
        return null;
      }
    };

    const [operatorSnapshot, oversightHistory, oversightAgentStatus] = await Promise.all([
      readOptional(dashboardApiClient.getOperatorSnapshot),
      readOptional(dashboardApiClient.getOversightHistory),
      readOptional(dashboardApiClient.getOversightAgentStatus)
    ]);

    applySnapshots({
      operatorSnapshot,
      oversightHistory,
      oversightAgentStatus
    });
  }

  const refreshTrapsTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'traps',
      reason,
      'Loading trap controls...',
      'No trap config snapshot available yet.',
      runtimeOptions
    );

  const refreshAdvancedTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'advanced',
      reason,
      'Loading advanced controls...',
      'No advanced config snapshot available yet.',
      runtimeOptions
    );

  const refreshRateLimitingTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'rate-limiting',
      reason,
      'Loading rate limiting controls...',
      'No rate limiting config snapshot available yet.',
      runtimeOptions
    );

  const refreshGeoTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'geo',
      reason,
      'Loading GEO controls...',
      'No GEO config snapshot available yet.',
      runtimeOptions
    );

  async function refreshFingerprintingTab(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return;

    if (reason !== 'auto-refresh') {
      showTabLoading('fingerprinting', 'Loading fingerprinting controls...');
    }

    const requestOptions = toRequestOptions(runtimeOptions, {
      tab: 'fingerprinting',
      reason,
      source: 'tab-refresh'
    });
    const [configEnvelope, cdp] = await Promise.all([
      refreshSharedConfig(reason, runtimeOptions),
      dashboardApiClient.getCdp(requestOptions)
    ]);

    applySnapshots({ cdp });
    if (isConfigSnapshotEmpty(configEnvelope?.config)) {
      showTabEmpty('fingerprinting', 'No fingerprinting config snapshot available yet.');
    } else {
      clearTabStateMessage('fingerprinting');
    }
  }

  const refreshPolicyTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'policy',
      reason,
      'Loading policy controls...',
      'No policy config snapshot available yet.',
      runtimeOptions
    );

  const refreshTuningTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'tuning',
      reason,
      'Loading tuning values...',
      'No tuning config snapshot available yet.',
      runtimeOptions
    );

  const TAB_REFRESH_HANDLERS = Object.freeze({
    traffic: refreshTrafficTab,
    'game-loop': async (reason = 'manual', runtimeOptions = {}) => {
      if (reason !== 'auto-refresh') {
        showTabLoading('game-loop', 'Loading closed-loop accountability...');
      }
      await Promise.all([
        refreshSharedConfig(reason, runtimeOptions),
        refreshMonitoringAccountabilityData(reason, runtimeOptions)
      ]);
      clearTabStateMessage('game-loop');
    },
    diagnostics: async (reason = 'manual', runtimeOptions = {}) => {
      if (reason === 'auto-refresh') {
        await refreshDiagnosticsTab(reason, runtimeOptions);
        return;
      }
      await Promise.all([
        refreshDiagnosticsTab(reason, runtimeOptions),
        refreshSharedConfig(reason, runtimeOptions)
      ]);
    },
    'ip-bans': refreshIpBansTab,
    status: refreshStatusTab,
    'red-team': refreshRedTeamTab,
    verification: refreshVerificationTab,
    traps: refreshTrapsTab,
    advanced: refreshAdvancedTab,
    'rate-limiting': refreshRateLimitingTab,
    geo: refreshGeoTab,
    fingerprinting: refreshFingerprintingTab,
    policy: refreshPolicyTab,
    tuning: refreshTuningTab
  });

  async function refreshDashboardForTab(tab, reason = 'manual', runtimeOptions = {}) {
    const activeTab = normalizeTab(tab);
    if (activeRealtimeTab && activeRealtimeTab !== activeTab) {
      closeStream(activeRealtimeTab);
      activeRealtimeTab = '';
    }
    if (!STREAMABLE_TABS.has(activeTab)) {
      closeAllStreams();
      activeRealtimeTab = '';
    }
    try {
      const handler = TAB_REFRESH_HANDLERS[activeTab] || TAB_REFRESH_HANDLERS['game-loop'];
      await handler(reason, runtimeOptions);
      const dashboardState = getStateStore();
      if (dashboardState) {
        dashboardState.markTabUpdated(activeTab);
      }
      if (ENABLE_REALTIME_STREAMS && STREAMABLE_TABS.has(activeTab)) {
        activeRealtimeTab = activeTab;
        startRealtimeStream(activeTab);
      } else {
        closeAllStreams();
        activeRealtimeTab = '';
      }
    } catch (error) {
      if (error && error.name === 'AbortError') {
        return;
      }
      const message = error && error.message ? error.message : 'Refresh failed';
      console.error(`Dashboard refresh error (${activeTab}):`, error);
      showTabError(activeTab, message);
    }
  }

  function refreshActiveTab(reason = 'manual') {
    const dashboardState = getStateStore();
    const activeTab = dashboardState ? dashboardState.getActiveTab() : 'game-loop';
    return refreshDashboardForTab(activeTab, reason);
  }

  return {
    clearAllCaches,
    refreshSharedConfig,
    refreshMonitoringTab,
    refreshDiagnosticsTab,
    refreshIpBansTab,
    refreshStatusTab,
    refreshRedTeamTab,
    refreshVerificationTab,
    refreshTrapsTab,
    refreshAdvancedTab,
    refreshRateLimitingTab,
    refreshGeoTab,
    refreshFingerprintingTab,
    refreshPolicyTab,
    refreshTuningTab,
    refreshDashboardForTab,
    refreshActiveTab
  };
}
