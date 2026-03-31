import { derived, get, writable } from 'svelte/store';
import {
  DASHBOARD_TABS,
  DEFAULT_TAB,
  createInitialState,
  reduceState,
  normalizeTab
} from '../domain/dashboard-state.js';
import { REQUEST_FAILURE_CLASSES } from '../domain/core/request-failure.js';

export { DASHBOARD_TABS, DEFAULT_TAB, normalizeTab };

export const TAB_REFRESH_INTERVAL_MS = Object.freeze({
  traffic: 30000,
  'game-loop': 30000,
  diagnostics: 30000,
  'ip-bans': 45000,
  status: 60000,
  'red-team': 60000,
  verification: 60000,
  traps: 60000,
  advanced: 60000,
  'rate-limiting': 60000,
  geo: 60000,
  policy: 60000,
  tuning: 60000
});
export const RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE = 20;
export const CONNECTION_DISCONNECT_THRESHOLD = 3;
export const REQUEST_DIAGNOSTIC_MAX_ENTRIES = 200;
export const HEARTBEAT_BREADCRUMB_MAX_ENTRIES = 100;
const RUNTIME_TELEMETRY_TABS = Object.freeze(new Set(['traffic', 'diagnostics', 'ip-bans']));
const CONNECTION_STATES = Object.freeze({
  connected: 'connected',
  degraded: 'degraded',
  disconnected: 'disconnected'
});

const clampMetric = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric < 0) return 0;
  return Number(numeric.toFixed(2));
};
const normalizeWindowSize = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE;
  return Math.max(1, Math.floor(numeric));
};
const calculateP95 = (values = []) => {
  if (!Array.isArray(values) || values.length === 0) return 0;
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil(sorted.length * 0.95) - 1)
  );
  return clampMetric(sorted[index]);
};
const trimWindow = (values = [], limit = RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE) => {
  if (!Array.isArray(values)) return [];
  if (values.length <= limit) return values;
  return values.slice(values.length - limit);
};
const trimEntries = (entries = [], limit = REQUEST_DIAGNOSTIC_MAX_ENTRIES) => {
  if (!Array.isArray(entries)) return [];
  if (entries.length <= limit) return entries;
  return entries.slice(entries.length - limit);
};
const toStatusCode = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return 0;
  return Math.floor(numeric);
};
const normalizeFailureClass = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === REQUEST_FAILURE_CLASSES.cancelled) return REQUEST_FAILURE_CLASSES.cancelled;
  if (normalized === REQUEST_FAILURE_CLASSES.timeout) return REQUEST_FAILURE_CLASSES.timeout;
  if (normalized === REQUEST_FAILURE_CLASSES.http) return REQUEST_FAILURE_CLASSES.http;
  return REQUEST_FAILURE_CLASSES.transport;
};
const normalizeConnectionState = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === CONNECTION_STATES.connected) return CONNECTION_STATES.connected;
  if (normalized === CONNECTION_STATES.degraded) return CONNECTION_STATES.degraded;
  return CONNECTION_STATES.disconnected;
};
const heartbeatReasonFromFailureClass = (failureClass) => {
  if (failureClass === REQUEST_FAILURE_CLASSES.timeout) return 'heartbeat_timeout';
  if (failureClass === REQUEST_FAILURE_CLASSES.http) return 'heartbeat_http_error';
  if (failureClass === REQUEST_FAILURE_CLASSES.cancelled) return 'heartbeat_cancelled_ignored';
  return 'heartbeat_transport_error';
};
const createMetricState = () => ({
  last: 0,
  avg: 0,
  p95: 0,
  max: 0,
  samples: 0,
  totalSamples: 0,
  windowSize: RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE,
  window: []
});

function createRuntimeTelemetryState() {
  return {
    refresh: {
      lastTab: DEFAULT_TAB,
      lastReason: 'init',
      updatedAt: '',
      fetchLatencyMs: createMetricState(),
      renderTimingMs: createMetricState()
    },
    connection: {
      state: CONNECTION_STATES.disconnected,
      lastTransitionAt: '',
      lastSuccessAt: '',
      lastFailureAt: '',
      lastError: '',
      consecutiveFailures: 0,
      disconnectThreshold: CONNECTION_DISCONNECT_THRESHOLD,
      lastTransitionReason: 'boot_disconnected'
    },
    heartbeat: {
      lastHeartbeatSuccessAt: '',
      lastHeartbeatFailureAt: '',
      consecutiveFailures: 0,
      lastFailureClass: '',
      lastFailureError: '',
      lastTransitionReason: 'boot_disconnected',
      ignoredCancelledCount: 0,
      ignoredNonHeartbeatFailureCount: 0,
      breadcrumbs: [],
      maxBreadcrumbs: HEARTBEAT_BREADCRUMB_MAX_ENTRIES
    },
    requests: {
      maxEntries: REQUEST_DIAGNOSTIC_MAX_ENTRIES,
      recent: [],
      total: 0,
      successCount: 0,
      failureCount: 0,
      cancelledCount: 0,
      timeoutCount: 0,
      transportCount: 0,
      httpCount: 0
    },
    polling: {
      skips: 0,
      resumes: 0,
      lastSkipReason: '',
      lastSkipAt: '',
      lastResumeReason: '',
      lastResumeAt: '',
      activeTab: DEFAULT_TAB,
      intervalMs: 0
    }
  };
}

const updateMetric = (metric = {}, rawValue) => {
  const value = clampMetric(rawValue);
  const windowSize = normalizeWindowSize(metric.windowSize);
  const currentWindow = Array.isArray(metric.window) ? metric.window : [];
  const nextWindow = trimWindow([...currentWindow, value], windowSize);
  const samples = nextWindow.length;
  const sum = nextWindow.reduce((accumulator, entry) => accumulator + Number(entry || 0), 0);
  const max = nextWindow.reduce((accumulator, entry) => Math.max(accumulator, Number(entry || 0)), 0);
  const totalSamples = Number(metric.totalSamples || 0) + 1;

  return {
    last: value,
    avg: samples > 0 ? clampMetric(sum / samples) : 0,
    p95: calculateP95(nextWindow),
    max: clampMetric(max),
    samples,
    totalSamples,
    windowSize,
    window: nextWindow
  };
};

const appendHeartbeatBreadcrumb = (heartbeat = {}, eventType = '', details = {}) => {
  const maxBreadcrumbs = Number(heartbeat.maxBreadcrumbs || HEARTBEAT_BREADCRUMB_MAX_ENTRIES);
  const breadcrumb = {
    eventType: String(eventType || ''),
    at: new Date().toISOString(),
    requestId: String(details.requestId || ''),
    path: String(details.path || ''),
    method: String(details.method || ''),
    statusCode: toStatusCode(details.statusCode),
    failureClass: String(details.failureClass || ''),
    reason: String(details.reason || '')
  };
  return trimEntries([...(Array.isArray(heartbeat.breadcrumbs) ? heartbeat.breadcrumbs : []), breadcrumb], maxBreadcrumbs);
};

const appendControllerResetBreadcrumb = (heartbeat = {}, reason = 'heartbeat_controller_reset') =>
  trimEntries(
    [
      ...(Array.isArray(heartbeat.breadcrumbs) ? heartbeat.breadcrumbs : []),
      {
        eventType: 'controller_reset',
        at: new Date().toISOString(),
        requestId: '',
        path: '/shuma/admin/session',
        method: 'GET',
        statusCode: 0,
        failureClass: '',
        reason: String(reason || 'heartbeat_controller_reset')
      }
    ],
    Number(heartbeat.maxBreadcrumbs || HEARTBEAT_BREADCRUMB_MAX_ENTRIES)
  );

export function createDashboardStore(options = {}) {
  const initialTab = normalizeTab(options.initialTab || DEFAULT_TAB);
  const internal = writable(createInitialState(initialTab));
  const runtimeTelemetryStore = writable(createRuntimeTelemetryState());

  const dispatch = (event = {}) => {
    let next = null;
    internal.update((state) => {
      next = reduceState(state, event);
      return next;
    });
    return next;
  };

  const getState = () => get(internal);

  const setActiveTab = (tab) => dispatch({ type: 'set-active-tab', tab: normalizeTab(tab) });
  const setSession = (session = {}) => dispatch({ type: 'set-session', session });
  const setSnapshot = (key, value) => dispatch({ type: 'set-snapshot', key, value });
  const setSnapshots = (updates = {}) => dispatch({ type: 'set-snapshots', snapshots: updates });
  const setTabLoading = (tab, loading, message = undefined) => {
    const event = { type: 'set-tab-loading', tab, loading };
    if (message !== undefined) {
      event.message = message;
    }
    return dispatch(event);
  };
  const setTabError = (tab, message) => dispatch({ type: 'set-tab-error', tab, message });
  const clearTabError = (tab) => dispatch({ type: 'clear-tab-error', tab });
  const setTabEmpty = (tab, empty, message = 'No data.') =>
    dispatch({ type: 'set-tab-empty', tab, empty, message });
  const markTabUpdated = (tab) => dispatch({ type: 'mark-tab-updated', tab });
  const invalidate = (scope = 'all') => dispatch({ type: 'invalidate', scope });
  const getActiveTab = () => normalizeTab(getState().activeTab);
  const getSession = () => {
    const current = getState().session || {};
    return {
      authenticated: current.authenticated === true,
      csrfToken: String(current.csrfToken || ''),
      runtimeEnvironment: String(current.runtimeEnvironment || '')
    };
  };
  const getSnapshot = (key) => {
    const current = getState();
    if (!current || !current.snapshots || !Object.prototype.hasOwnProperty.call(current.snapshots, key)) {
      return null;
    }
    return current.snapshots[key];
  };
  const getSnapshotVersion = (key) => {
    const current = getState();
    if (
      !current ||
      !current.snapshotVersions ||
      !Object.prototype.hasOwnProperty.call(current.snapshotVersions, key)
    ) {
      return 0;
    }
    return Number(current.snapshotVersions[key] || 0);
  };
  const getSnapshotVersions = () => {
    const current = getState();
    const versions = current && current.snapshotVersions && typeof current.snapshotVersions === 'object'
      ? current.snapshotVersions
      : {};
    return { ...versions };
  };
  const isTabStale = (tab) => {
    const key = normalizeTab(tab);
    const current = getState();
    return Boolean(current?.stale?.[key]);
  };
  const getDerivedState = () => {
    const current = getState();
    const events = current?.snapshots?.events || {};
    const bans = current?.snapshots?.bans || {};
    const maze = current?.snapshots?.maze || {};
    const monitoringEmpty =
      (Array.isArray(events.recent_events) ? events.recent_events.length : 0) === 0 &&
      (Array.isArray(bans.bans) ? bans.bans.length : 0) === 0 &&
      Number(maze.total_hits || 0) === 0;
    return {
      monitoringEmpty,
      hasConfigSnapshot: Boolean(current?.snapshots?.config),
      activeTab: getActiveTab()
    };
  };

  const reset = (tab = DEFAULT_TAB) => {
    internal.set(createInitialState(normalizeTab(tab)));
    resetRuntimeTelemetry();
  };

  const tabStatus = (tab) => derived(internal, ($state) => {
    const key = normalizeTab(tab);
    const value = $state.tabStatus[key] || {};
    return {
      loading: value.loading === true,
      error: String(value.error || ''),
      message: String(value.message || ''),
      empty: value.empty === true,
      updatedAt: String(value.updatedAt || ''),
      stale: $state.stale[key] === true
    };
  });

  const session = derived(internal, ($state) => ({
    authenticated: $state.session.authenticated === true,
    csrfToken: String($state.session.csrfToken || ''),
    runtimeEnvironment: String($state.session.runtimeEnvironment || '')
  }));

  const activeTab = derived(internal, ($state) => normalizeTab($state.activeTab));

  const selectRefreshInterval = (tab) => {
    const key = normalizeTab(tab);
    return TAB_REFRESH_INTERVAL_MS[key] || TAB_REFRESH_INTERVAL_MS['game-loop'];
  };

  const getRuntimeTelemetry = () => get(runtimeTelemetryStore);
  const resetRuntimeTelemetry = () => {
    runtimeTelemetryStore.set(createRuntimeTelemetryState());
  };

  const setPollingContext = (tab, intervalMs) => {
    runtimeTelemetryStore.update((telemetry) => ({
      ...telemetry,
      polling: {
        ...telemetry.polling,
        activeTab: normalizeTab(tab),
        intervalMs: Number.isFinite(Number(intervalMs)) ? Number(intervalMs) : telemetry.polling.intervalMs
      }
    }));
  };

  const recordRefreshMetrics = (metrics = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const tab = normalizeTab(metrics.tab);
      if (!RUNTIME_TELEMETRY_TABS.has(tab)) {
        return telemetry;
      }
      const fetchLatencyMs = updateMetric(
        telemetry.refresh.fetchLatencyMs,
        metrics.fetchLatencyMs
      );
      const renderTimingMs = updateMetric(
        telemetry.refresh.renderTimingMs,
        metrics.renderTimingMs
      );
      return {
        ...telemetry,
        refresh: {
          ...telemetry.refresh,
          lastTab: tab,
          lastReason: String(metrics.reason || 'manual'),
          updatedAt: new Date().toISOString(),
          fetchLatencyMs,
          renderTimingMs
        }
      };
    });
  };

  const recordPollingSkip = (reason = 'unspecified', tab = DEFAULT_TAB, intervalMs = 0) => {
    runtimeTelemetryStore.update((telemetry) => ({
      ...telemetry,
      polling: {
        ...telemetry.polling,
        skips: Number(telemetry.polling.skips || 0) + 1,
        lastSkipReason: String(reason || 'unspecified'),
        lastSkipAt: new Date().toISOString(),
        activeTab: normalizeTab(tab),
        intervalMs: Number.isFinite(Number(intervalMs)) ? Number(intervalMs) : telemetry.polling.intervalMs
      }
    }));
  };

  const recordPollingResume = (reason = 'resume', tab = DEFAULT_TAB, intervalMs = 0) => {
    runtimeTelemetryStore.update((telemetry) => ({
      ...telemetry,
      polling: {
        ...telemetry.polling,
        resumes: Number(telemetry.polling.resumes || 0) + 1,
        lastResumeReason: String(reason || 'resume'),
        lastResumeAt: new Date().toISOString(),
        activeTab: normalizeTab(tab),
        intervalMs: Number.isFinite(Number(intervalMs)) ? Number(intervalMs) : telemetry.polling.intervalMs
      }
    }));
  };

  const recordRequestTelemetry = (event = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const nowIso = new Date().toISOString();
      const requests =
        telemetry && telemetry.requests && typeof telemetry.requests === 'object'
          ? telemetry.requests
          : {};
      const heartbeat =
        telemetry && telemetry.heartbeat && typeof telemetry.heartbeat === 'object'
          ? telemetry.heartbeat
          : {};
      const source = String(event.source || '');
      const outcome = String(event.outcome || '').trim().toLowerCase() === 'success' ? 'success' : 'failure';
      const failureClass = outcome === 'failure'
        ? normalizeFailureClass(event.failureClass)
        : '';
      const statusCode = toStatusCode(event.statusCode);
      const entry = {
        requestId: String(event.requestId || ''),
        path: String(event.path || ''),
        method: String(event.method || 'GET').toUpperCase(),
        tab: String(event.tab || ''),
        reason: String(event.reason || ''),
        source,
        startedAt: String(event.startedAt || ''),
        durationMs: clampMetric(event.durationMs),
        outcome,
        failureClass,
        statusCode,
        aborted: event.aborted === true,
        recordedAt: nowIso
      };
      const maxEntries = Math.max(1, Math.floor(Number(requests.maxEntries || REQUEST_DIAGNOSTIC_MAX_ENTRIES)));
      const nextRecent = trimEntries([...(Array.isArray(requests.recent) ? requests.recent : []), entry], maxEntries);
      const nextRequests = {
        ...requests,
        maxEntries,
        recent: nextRecent,
        total: Number(requests.total || 0) + 1,
        successCount: Number(requests.successCount || 0) + (outcome === 'success' ? 1 : 0),
        failureCount: Number(requests.failureCount || 0) + (outcome === 'failure' ? 1 : 0),
        cancelledCount:
          Number(requests.cancelledCount || 0) + (failureClass === REQUEST_FAILURE_CLASSES.cancelled ? 1 : 0),
        timeoutCount:
          Number(requests.timeoutCount || 0) + (failureClass === REQUEST_FAILURE_CLASSES.timeout ? 1 : 0),
        transportCount:
          Number(requests.transportCount || 0) + (failureClass === REQUEST_FAILURE_CLASSES.transport ? 1 : 0),
        httpCount:
          Number(requests.httpCount || 0) + (failureClass === REQUEST_FAILURE_CLASSES.http ? 1 : 0)
      };
      const shouldIncrementIgnoredCounters = outcome === 'failure' && source !== 'heartbeat';
      const nextHeartbeat = shouldIncrementIgnoredCounters
        ? {
            ...heartbeat,
            ignoredNonHeartbeatFailureCount:
              Number(heartbeat.ignoredNonHeartbeatFailureCount || 0) + 1,
            ignoredCancelledCount:
              Number(heartbeat.ignoredCancelledCount || 0) +
              (failureClass === REQUEST_FAILURE_CLASSES.cancelled ? 1 : 0)
          }
        : heartbeat;
      return {
        ...telemetry,
        requests: nextRequests,
        heartbeat: nextHeartbeat
      };
    });
  };

  const recordHeartbeatAttemptStarted = (event = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const heartbeat =
        telemetry && telemetry.heartbeat && typeof telemetry.heartbeat === 'object'
          ? telemetry.heartbeat
          : {};
      return {
        ...telemetry,
        heartbeat: {
          ...heartbeat,
          breadcrumbs: appendHeartbeatBreadcrumb(heartbeat, 'attempt_started', {
            requestId: event.requestId,
            path: event.path,
            method: event.method,
            reason: event.reason
          })
        }
      };
    });
  };

  const recordHeartbeatSuccess = (event = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const nowIso = new Date().toISOString();
      const currentConnection =
        telemetry && telemetry.connection && typeof telemetry.connection === 'object'
          ? telemetry.connection
          : {};
      const heartbeat =
        telemetry && telemetry.heartbeat && typeof telemetry.heartbeat === 'object'
          ? telemetry.heartbeat
          : {};
      const priorState = normalizeConnectionState(currentConnection.state);
      const transitionReason = String(event.transitionReason || 'heartbeat_ok');
      return {
        ...telemetry,
        connection: {
          ...currentConnection,
          state: CONNECTION_STATES.connected,
          lastTransitionAt: priorState === CONNECTION_STATES.connected
            ? String(currentConnection.lastTransitionAt || '')
            : nowIso,
          lastSuccessAt: nowIso,
          lastError: '',
          consecutiveFailures: 0,
          disconnectThreshold: Math.max(
            1,
            Math.floor(Number(currentConnection.disconnectThreshold || CONNECTION_DISCONNECT_THRESHOLD))
          ),
          lastTransitionReason: transitionReason
        },
        heartbeat: {
          ...heartbeat,
          lastHeartbeatSuccessAt: nowIso,
          consecutiveFailures: 0,
          lastFailureClass: '',
          lastFailureError: '',
          lastTransitionReason: transitionReason,
          breadcrumbs: appendHeartbeatBreadcrumb(heartbeat, 'attempt_succeeded', {
            requestId: event.requestId,
            path: event.path,
            method: event.method,
            statusCode: event.statusCode,
            reason: transitionReason
          })
        }
      };
    });
  };

  const recordHeartbeatFailure = (event = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const nowIso = new Date().toISOString();
      const currentConnection =
        telemetry && telemetry.connection && typeof telemetry.connection === 'object'
          ? telemetry.connection
          : {};
      const heartbeat =
        telemetry && telemetry.heartbeat && typeof telemetry.heartbeat === 'object'
          ? telemetry.heartbeat
          : {};
      const disconnectThreshold = Math.max(
        1,
        Math.floor(Number(currentConnection.disconnectThreshold || CONNECTION_DISCONNECT_THRESHOLD))
      );
      const failureClass = normalizeFailureClass(event.failureClass);
      const failureError = String(event.error || '');
      const reason = String(event.transitionReason || heartbeatReasonFromFailureClass(failureClass));
      const nextHeartbeats = appendHeartbeatBreadcrumb(heartbeat, 'attempt_failed', {
        requestId: event.requestId,
        path: event.path,
        method: event.method,
        statusCode: event.statusCode,
        failureClass,
        reason
      });
      if (failureClass === REQUEST_FAILURE_CLASSES.cancelled) {
        const retryBreadcrumbs = trimEntries(
          [
            ...nextHeartbeats,
            {
              eventType: 'retry_scheduled',
              at: nowIso,
              requestId: String(event.requestId || ''),
              path: String(event.path || ''),
              method: String(event.method || ''),
              statusCode: 0,
              failureClass: REQUEST_FAILURE_CLASSES.cancelled,
              reason
            }
          ],
          Number(heartbeat.maxBreadcrumbs || HEARTBEAT_BREADCRUMB_MAX_ENTRIES)
        );
        return {
          ...telemetry,
          heartbeat: {
            ...heartbeat,
            ignoredCancelledCount: Number(heartbeat.ignoredCancelledCount || 0) + 1,
            lastFailureClass: REQUEST_FAILURE_CLASSES.cancelled,
            lastFailureError: failureError,
            lastTransitionReason: reason,
            breadcrumbs: retryBreadcrumbs
          }
        };
      }
      const nextConsecutiveFailures = Number(currentConnection.consecutiveFailures || 0) + 1;
      const nextState = nextConsecutiveFailures >= disconnectThreshold
        ? CONNECTION_STATES.disconnected
        : CONNECTION_STATES.degraded;
      const priorState = normalizeConnectionState(currentConnection.state);
      return {
        ...telemetry,
        connection: {
          ...currentConnection,
          state: nextState,
          lastTransitionAt: priorState === nextState
            ? String(currentConnection.lastTransitionAt || '')
            : nowIso,
          lastFailureAt: nowIso,
          lastError: failureError,
          consecutiveFailures: nextConsecutiveFailures,
          disconnectThreshold,
          lastTransitionReason: reason
        },
        heartbeat: {
          ...heartbeat,
          lastHeartbeatFailureAt: nowIso,
          consecutiveFailures: nextConsecutiveFailures,
          lastFailureClass: failureClass,
          lastFailureError: failureError,
          lastTransitionReason: reason,
          breadcrumbs: nextHeartbeats
        }
      };
    });
  };

  const recordHeartbeatControllerReset = (event = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const nowIso = new Date().toISOString();
      const currentConnection =
        telemetry && telemetry.connection && typeof telemetry.connection === 'object'
          ? telemetry.connection
          : {};
      const heartbeat =
        telemetry && telemetry.heartbeat && typeof telemetry.heartbeat === 'object'
          ? telemetry.heartbeat
          : {};
      const reason = String(event.reason || 'heartbeat_controller_reset');
      const priorState = normalizeConnectionState(currentConnection.state);
      return {
        ...telemetry,
        connection: {
          ...currentConnection,
          state: CONNECTION_STATES.disconnected,
          lastTransitionAt:
            priorState === CONNECTION_STATES.disconnected
              ? String(currentConnection.lastTransitionAt || '')
              : nowIso,
          lastError: '',
          consecutiveFailures: 0,
          disconnectThreshold: Math.max(
            1,
            Math.floor(Number(currentConnection.disconnectThreshold || CONNECTION_DISCONNECT_THRESHOLD))
          ),
          lastTransitionReason: reason
        },
        heartbeat: {
          ...heartbeat,
          consecutiveFailures: 0,
          lastFailureClass: '',
          lastFailureError: '',
          lastTransitionReason: reason,
          breadcrumbs: appendControllerResetBreadcrumb(heartbeat, reason)
        }
      };
    });
  };

  return {
    subscribe: internal.subscribe,
    getState,
    dispatch,
    reset,
    setActiveTab,
    getActiveTab,
    setSession,
    getSession,
    setSnapshot,
    setSnapshots,
    getSnapshot,
    getSnapshotVersion,
    getSnapshotVersions,
    setTabLoading,
    setTabError,
    clearTabError,
    setTabEmpty,
    markTabUpdated,
    invalidate,
    isTabStale,
    getDerivedState,
    tabStatus,
    session,
    activeTab,
    selectRefreshInterval,
    runtimeTelemetryStore: {
      subscribe: runtimeTelemetryStore.subscribe
    },
    getRuntimeTelemetry,
    resetRuntimeTelemetry,
    setPollingContext,
    recordRefreshMetrics,
    recordPollingSkip,
    recordPollingResume,
    recordRequestTelemetry,
    recordHeartbeatAttemptStarted,
    recordHeartbeatSuccess,
    recordHeartbeatFailure,
    recordHeartbeatControllerReset
  };
}
