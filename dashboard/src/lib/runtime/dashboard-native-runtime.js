// @ts-check

import * as dashboardApiClientModule from '../domain/api-client.js';
import {
  classifyRequestFailure,
  REQUEST_FAILURE_CLASSES
} from '../domain/core/request-failure.js';
import * as adminEndpointModule from '../domain/services/admin-endpoint.js';
import {
  acquireChartRuntime,
  releaseChartRuntime
} from '../domain/services/chart-runtime-adapter.js';
import { createDashboardRefreshRuntime } from './dashboard-runtime-refresh.js';
import {
  normalizeDashboardBasePath,
  resolveDashboardBasePathFromLocation
} from './dashboard-paths.js';

const DASHBOARD_TABS = Object.freeze(['monitoring', 'ip-bans', 'red-team', 'tuning', 'verification', 'traps', 'rate-limiting', 'geo', 'fingerprinting', 'policy', 'status', 'advanced', 'diagnostics']);
const CONNECTION_HEARTBEAT_PATH = '/admin/session';
const CONNECTION_HEARTBEAT_METHOD = 'GET';
const CONNECTION_HEARTBEAT_INTERVAL_MS = 1000;
const CONNECTION_HEARTBEAT_TIMEOUT_MS = 2500;

const DASHBOARD_STATE_REQUIRED_METHODS = Object.freeze([
  'getState',
  'setActiveTab',
  'getActiveTab',
  'setSession',
  'getSession',
  'setSnapshot',
  'setSnapshots',
  'getSnapshot',
  'getSnapshotVersion',
  'getSnapshotVersions',
  'setTabLoading',
  'setTabError',
  'clearTabError',
  'setTabEmpty',
  'markTabUpdated',
  'invalidate',
  'isTabStale',
  'getDerivedState',
  'recordHeartbeatControllerReset'
]);

const isObject = (value) => value && typeof value === 'object';

function hasDashboardStateContract(candidate) {
  if (!isObject(candidate)) return false;
  return DASHBOARD_STATE_REQUIRED_METHODS.every(
    (methodName) => typeof candidate[methodName] === 'function'
  );
}

function resolveDashboardStateStore(options = {}) {
  const providedStore = options.store;
  if (hasDashboardStateContract(providedStore)) {
    return providedStore;
  }
  throw new Error('mountDashboardApp requires an injected dashboard store contract.');
}

function normalizeTab(value) {
  const normalized = String(value || '').trim().toLowerCase();
  return DASHBOARD_TABS.includes(normalized) ? normalized : 'monitoring';
}

function parseBoolLike(value, fallback = false) {
  if (typeof value === 'boolean') return value;
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') {
    return true;
  }
  if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') {
    return false;
  }
  return fallback;
}

function deriveMonitoringAnalytics(configSnapshot = {}, configRuntimeSnapshot = {}, analyticsSnapshot = {}) {
  const config = isObject(configSnapshot) ? configSnapshot : {};
  const runtime = isObject(configRuntimeSnapshot) ? configRuntimeSnapshot : {};
  const analytics = isObject(analyticsSnapshot) ? analyticsSnapshot : {};
  const rawBanCount = analytics.ban_count;
  const banCount = rawBanCount === null || rawBanCount === undefined || rawBanCount === ''
    ? null
    : Number(rawBanCount);
  return {
    ban_count: Number.isFinite(banCount) && banCount >= 0 ? banCount : null,
    ban_store_status: String(analytics.ban_store_status || 'available'),
    ban_store_message: String(analytics.ban_store_message || ''),
    shadow_mode: parseBoolLike(config.shadow_mode, analytics.shadow_mode === true),
    fail_mode: parseBoolLike(runtime.kv_store_fail_open, true) ? 'open' : 'closed'
  };
}

function normalizeRuntimeMountOptions(options = {}) {
  const source = options || {};
  const locationLike = typeof window !== 'undefined' ? window.location : null;
  const basePath = normalizeDashboardBasePath(
    source.basePath || resolveDashboardBasePathFromLocation(locationLike)
  );
  return {
    basePath,
    initialTab: normalizeTab(source.initialTab || 'monitoring')
  };
}

let runtimeMounted = false;
let runtimeMountOptions = normalizeRuntimeMountOptions({});
let resolveAdminApiEndpoint = () => ({ endpoint: '' });
let dashboardState = null;
let dashboardApiClient = null;
let dashboardRefreshRuntime = null;
let connectionHeartbeatTimer = null;
let connectionHeartbeatInFlight = null;
let connectionHeartbeatSequence = 0;

const sessionState = {
  authenticated: false,
  csrfToken: '',
  expiresAt: 0,
  runtimeEnvironment: ''
};

function normalizeRuntimeEnvironment(value = '') {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === 'runtime-dev' || normalized === 'runtime-prod') {
    return normalized;
  }
  return '';
}

function hasRuntimeEnvironment() {
  return sessionState.runtimeEnvironment === 'runtime-dev' || sessionState.runtimeEnvironment === 'runtime-prod';
}

function setSessionState(authenticated, csrfToken = '', expiresAt = 0, runtimeEnvironment = '') {
  const parsedExpiry = Number(expiresAt);
  sessionState.authenticated = authenticated === true;
  sessionState.csrfToken = sessionState.authenticated ? String(csrfToken || '') : '';
  sessionState.expiresAt =
    sessionState.authenticated && Number.isFinite(parsedExpiry) && parsedExpiry > 0
      ? Math.floor(parsedExpiry)
      : 0;
  sessionState.runtimeEnvironment = sessionState.authenticated
    ? normalizeRuntimeEnvironment(runtimeEnvironment)
    : '';
  if (dashboardState) {
    dashboardState.setSession({
      authenticated: sessionState.authenticated,
      csrfToken: sessionState.csrfToken,
      runtimeEnvironment: sessionState.runtimeEnvironment
    });
    if (sessionState.authenticated !== true) {
      resetHeartbeatConnectionState('session_cleared');
    }
  }
  syncConnectionHeartbeatLoop(sessionState.authenticated ? 'session-authenticated' : 'session-cleared');
}

function resetHeartbeatConnectionState(reason = 'heartbeat_controller_reset') {
  if (!dashboardState || typeof dashboardState.recordHeartbeatControllerReset !== 'function') return;
  dashboardState.recordHeartbeatControllerReset({
    reason
  });
}

function clearConnectionHeartbeatTimer() {
  if (!connectionHeartbeatTimer) return;
  clearTimeout(connectionHeartbeatTimer);
  connectionHeartbeatTimer = null;
}

function shouldRunConnectionHeartbeat() {
  if (!runtimeMounted) return false;
  if (!dashboardState) return false;
  if (sessionState.authenticated !== true) return false;
  if (!hasRuntimeEnvironment()) return false;
  return resolveEndpoint().trim().length > 0;
}

function scheduleConnectionHeartbeat(delayMs = CONNECTION_HEARTBEAT_INTERVAL_MS) {
  clearConnectionHeartbeatTimer();
  if (!shouldRunConnectionHeartbeat()) return;
  const numericDelay = Number(delayMs);
  const waitMs = Number.isFinite(numericDelay) && numericDelay >= 0
    ? Math.floor(numericDelay)
    : CONNECTION_HEARTBEAT_INTERVAL_MS;
  connectionHeartbeatTimer = setTimeout(() => {
    connectionHeartbeatTimer = null;
    void runConnectionHeartbeat('interval');
  }, waitMs);
}

function stopConnectionHeartbeat() {
  clearConnectionHeartbeatTimer();
  if (!connectionHeartbeatInFlight) return;
  const inFlightController = connectionHeartbeatInFlight.controller;
  if (inFlightController && typeof inFlightController.abort === 'function') {
    inFlightController.abort();
  }
  connectionHeartbeatInFlight = null;
}

function syncConnectionHeartbeatLoop(_reason = 'sync') {
  if (shouldRunConnectionHeartbeat()) {
    scheduleConnectionHeartbeat(0);
    return;
  }
  stopConnectionHeartbeat();
}

async function runConnectionHeartbeat(reason = 'manual') {
  if (!shouldRunConnectionHeartbeat()) return;
  if (connectionHeartbeatInFlight && connectionHeartbeatInFlight.promise) {
    return connectionHeartbeatInFlight.promise;
  }
  const endpoint = resolveEndpoint();
  if (!endpoint) return;
  const requestId = `hb-${Date.now().toString(16)}-${(connectionHeartbeatSequence += 1).toString(16)}`;
  const path = CONNECTION_HEARTBEAT_PATH;
  const method = CONNECTION_HEARTBEAT_METHOD;
  const startedAtMs = Date.now();
  const startedAtIso = new Date(startedAtMs).toISOString();
  const timeoutMs = CONNECTION_HEARTBEAT_TIMEOUT_MS;
  let didTimeout = false;
  let timeoutId = null;
  const controller = typeof AbortController === 'function' ? new AbortController() : null;
  if (dashboardState && typeof dashboardState.recordHeartbeatAttemptStarted === 'function') {
    dashboardState.recordHeartbeatAttemptStarted({
      requestId,
      path,
      method,
      reason
    });
  }
  const heartbeatPromise = (async () => {
    try {
      if (controller) {
        timeoutId = setTimeout(() => {
          didTimeout = true;
          if (!controller.signal.aborted) {
            controller.abort();
          }
        }, timeoutMs);
      }
      const response = await fetch(`${endpoint}${path}`, {
        method,
        credentials: 'same-origin',
        signal: controller ? controller.signal : undefined
      });
      if (!response.ok) {
        const httpError = new Error(`Heartbeat request failed with status ${response.status}`);
        httpError.name = 'DashboardHeartbeatHttpError';
        /** @type {number} */
        httpError.status = Number(response.status || 0);
        throw httpError;
      }
      if (dashboardState && typeof dashboardState.recordRequestTelemetry === 'function') {
        dashboardState.recordRequestTelemetry({
          requestId,
          path,
          method,
          tab: 'status',
          reason,
          source: 'heartbeat',
          startedAt: startedAtIso,
          durationMs: Math.max(0, Date.now() - startedAtMs),
          outcome: 'success',
          statusCode: Number(response.status || 0),
          aborted: false
        });
      }
      if (dashboardState && typeof dashboardState.recordHeartbeatSuccess === 'function') {
        dashboardState.recordHeartbeatSuccess({
          requestId,
          path,
          method,
          statusCode: Number(response.status || 0),
          transitionReason: 'heartbeat_ok'
        });
      }
    } catch (error) {
      const statusCode = Number(error && typeof error === 'object' ? error.status || 0 : 0);
      const failureClass = classifyRequestFailure(error, { didTimeout, statusCode });
      const errorMessage = didTimeout
        ? `Request timed out after ${timeoutMs}ms`
        : String(error && typeof error === 'object' ? error.message || 'Heartbeat request failed' : 'Heartbeat request failed');
      if (dashboardState && typeof dashboardState.recordRequestTelemetry === 'function') {
        dashboardState.recordRequestTelemetry({
          requestId,
          path,
          method,
          tab: 'status',
          reason,
          source: 'heartbeat',
          startedAt: startedAtIso,
          durationMs: Math.max(0, Date.now() - startedAtMs),
          outcome: 'failure',
          failureClass,
          statusCode,
          aborted: failureClass === REQUEST_FAILURE_CLASSES.cancelled || didTimeout === true,
          errorMessage
        });
      }
      if (dashboardState && typeof dashboardState.recordHeartbeatFailure === 'function') {
        dashboardState.recordHeartbeatFailure({
          requestId,
          path,
          method,
          statusCode,
          failureClass,
          error: errorMessage
        });
      }
    } finally {
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }
      connectionHeartbeatInFlight = null;
      scheduleConnectionHeartbeat(CONNECTION_HEARTBEAT_INTERVAL_MS);
    }
  })();
  connectionHeartbeatInFlight = {
    promise: heartbeatPromise,
    controller
  };
  return heartbeatPromise;
}

function resolveEndpoint() {
  const resolved = resolveAdminApiEndpoint();
  if (!resolved || typeof resolved.endpoint !== 'string') return '';
  return resolved.endpoint;
}

function getAdminContext() {
  if (!sessionState.authenticated) return null;
  const endpoint = resolveEndpoint();
  if (!endpoint) return null;
  return {
    endpoint,
    apikey: '',
    sessionAuth: true,
    csrfToken: sessionState.csrfToken
  };
}

function hasRuntimeReadyApiClient() {
  return Boolean(runtimeMounted && dashboardApiClient);
}

function requireApiClient() {
  if (!hasRuntimeReadyApiClient()) {
    throw new Error('Dashboard runtime API client is unavailable.');
  }
  return dashboardApiClient;
}

async function restoreSessionFromServer() {
  const endpoint = resolveEndpoint();
  if (!endpoint) {
    setSessionState(false, '');
    return false;
  }

  try {
    const response = await fetch(`${endpoint}/admin/session`, {
      method: 'GET',
      credentials: 'same-origin'
    });
    if (!response.ok) {
      setSessionState(false, '');
      return false;
    }
    const payload = await response.json().catch(() => ({}));
    const authenticated = payload && payload.authenticated === true;
    const csrfToken = authenticated ? String(payload.csrf_token || '') : '';
    const expiresAtRaw = payload && payload.expires_at !== undefined ? payload.expires_at : 0;
    const runtimeEnvironmentRaw = payload && payload.runtime_environment !== undefined
      ? payload.runtime_environment
      : '';
    setSessionState(authenticated, csrfToken, expiresAtRaw, runtimeEnvironmentRaw);
    return authenticated;
  } catch (_error) {
    setSessionState(false, '');
    return false;
  }
}

function applyConfigEnvelopeSnapshots(nextConfig = null, nextRuntime = null) {
  if (!dashboardState) return;
  if (isObject(nextConfig)) {
    dashboardState.setSnapshot('config', nextConfig);
  }
  if (isObject(nextRuntime)) {
    dashboardState.setSnapshot('configRuntime', nextRuntime);
  }
}

function invalidateAfterConfigSave(nextConfig = null, nextRuntime = null) {
  if (!dashboardState) return;
  applyConfigEnvelopeSnapshots(nextConfig, nextRuntime);
  dashboardState.invalidate('securityConfig');
  dashboardState.invalidate('monitoring');
  dashboardState.invalidate('ip-bans');
}

function applyAdversarySimStatusSnapshot(status = null) {
  if (!dashboardState || !isObject(status)) return;
  const existingConfig = dashboardState.getSnapshot('config');
  const configBase = isObject(existingConfig) ? existingConfig : {};
  const existingRuntime = dashboardState.getSnapshot('configRuntime');
  const runtimeBase = isObject(existingRuntime) ? existingRuntime : {};
  if (Object.keys(configBase).length === 0 && Object.keys(runtimeBase).length === 0) return;
  const nextConfig = { ...configBase };
  const nextRuntime = { ...runtimeBase };
  let changed = false;
  if (
    (typeof nextRuntime.runtime_environment !== 'string' || !nextRuntime.runtime_environment.trim()) &&
    typeof status.runtime_environment === 'string' &&
    status.runtime_environment.trim()
  ) {
    nextRuntime.runtime_environment = status.runtime_environment.trim();
    changed = true;
  }
  if (
    nextRuntime.adversary_sim_available === undefined &&
    typeof status.adversary_sim_available === 'boolean'
  ) {
    nextRuntime.adversary_sim_available = status.adversary_sim_available;
    changed = true;
  }
  const durationSeconds = Number(status.duration_seconds);
  if (
    nextConfig.adversary_sim_duration_seconds === undefined &&
    Number.isFinite(durationSeconds) &&
    durationSeconds > 0
  ) {
    nextConfig.adversary_sim_duration_seconds = Math.floor(durationSeconds);
    changed = true;
  }
  if (changed) {
    applyConfigEnvelopeSnapshots(nextConfig, nextRuntime);
  }
}

function invalidateAfterBanMutation() {
  if (!dashboardState) return;
  dashboardState.invalidate('ip-bans');
  dashboardState.invalidate('monitoring');
}

export async function mountDashboardApp(options = {}) {
  if (runtimeMounted) return;

  runtimeMountOptions = normalizeRuntimeMountOptions(options);
  dashboardState = resolveDashboardStateStore(options);
  resetHeartbeatConnectionState('runtime_mount_boot');

  resolveAdminApiEndpoint = adminEndpointModule.createAdminEndpointResolver({ window });

  await acquireChartRuntime({
    window
  });

  dashboardApiClient = dashboardApiClientModule.create({
    getAdminContext,
    onUnauthorized: () => {
      if (dashboardRefreshRuntime && typeof dashboardRefreshRuntime.clearAllCaches === 'function') {
        dashboardRefreshRuntime.clearAllCaches();
      }
      setSessionState(false, '');
    },
    onRequestTelemetry: (event) => {
      if (!dashboardState || typeof dashboardState.recordRequestTelemetry !== 'function') return;
      dashboardState.recordRequestTelemetry(event);
    },
  });

  dashboardRefreshRuntime = createDashboardRefreshRuntime({
    normalizeTab,
    getApiClient: () => dashboardApiClient,
    getStateStore: () => dashboardState,
    deriveMonitoringAnalytics
  });

  dashboardState.setActiveTab(runtimeMountOptions.initialTab);
  runtimeMounted = true;
  syncConnectionHeartbeatLoop('runtime-mounted');
}

export function getDashboardActiveTab() {
  if (!dashboardState) return 'monitoring';
  return normalizeTab(dashboardState.getActiveTab());
}

export function setDashboardActiveTab(tab, _reason = 'external') {
  const normalized = normalizeTab(tab);
  if (dashboardState) {
    dashboardState.setActiveTab(normalized);
  }
  return normalized;
}

export async function refreshDashboardTab(tab, reason = 'manual', options = {}) {
  if (!dashboardRefreshRuntime || typeof dashboardRefreshRuntime.refreshDashboardForTab !== 'function') return;
  return dashboardRefreshRuntime.refreshDashboardForTab(tab, reason, options || {});
}

export async function restoreDashboardSession() {
  const restored = await restoreSessionFromServer();
  syncConnectionHeartbeatLoop(restored ? 'session-restored' : 'session-restore-failed');
  return restored;
}

export function getDashboardSessionState() {
  return {
    authenticated: sessionState.authenticated === true,
    csrfToken: String(sessionState.csrfToken || ''),
    expiresAt: Number(sessionState.expiresAt || 0),
    runtimeEnvironment: String(sessionState.runtimeEnvironment || '')
  };
}

export async function logoutDashboardSession() {
  if (dashboardRefreshRuntime && typeof dashboardRefreshRuntime.clearAllCaches === 'function') {
    dashboardRefreshRuntime.clearAllCaches();
  }
  const endpoint = resolveEndpoint();
  if (endpoint) {
    const headers = new Headers();
    if (sessionState.csrfToken) {
      headers.set('X-Shuma-CSRF', sessionState.csrfToken);
    }
    try {
      await fetch(`${endpoint}/admin/logout`, {
        method: 'POST',
        headers,
        credentials: 'same-origin'
      });
    } catch (_error) {}
  }
  setSessionState(false, '');
  syncConnectionHeartbeatLoop('logout');
}

export async function updateDashboardConfig(patch, requestOptions = {}) {
  const apiClient = requireApiClient();
  const response = await apiClient.updateConfig(patch || {}, requestOptions || {});
  const nextConfig =
    response && typeof response === 'object' && response.config && typeof response.config === 'object'
      ? response.config
      : {};
  const nextRuntime =
    response && typeof response === 'object' && response.runtime && typeof response.runtime === 'object'
      ? response.runtime
      : {};
  invalidateAfterConfigSave(nextConfig, nextRuntime);
  return nextConfig;
}

export async function validateDashboardConfigPatch(patch, requestOptions = {}) {
  const apiClient = requireApiClient();
  return apiClient.validateConfigPatch(patch || {}, requestOptions || {});
}

export async function getDashboardAdversarySimStatus(requestOptions = {}) {
  const apiClient = requireApiClient();
  const status = await apiClient.getAdversarySimStatus(requestOptions || {});
  applyAdversarySimStatusSnapshot(status);
  return status;
}

export async function controlDashboardAdversarySim(enabled, requestOptions = {}) {
  const apiClient = requireApiClient();
  const response = await apiClient.controlAdversarySim(enabled === true, requestOptions || {});
  const status = isObject(response.status) ? response.status : {};
  const nextConfig = isObject(response.config) ? response.config : null;
  const nextRuntime = isObject(response.runtime) ? response.runtime : null;
  if (nextConfig || nextRuntime) {
    invalidateAfterConfigSave(nextConfig, nextRuntime);
  }
  applyAdversarySimStatusSnapshot(status);
  return {
    requested_enabled: response.requested_enabled === true,
    status,
    config: nextConfig,
    runtime: nextRuntime
  };
}

export async function banDashboardIp(ip, duration, reason = 'manual_ban', requestOptions = {}) {
  const apiClient = requireApiClient();
  const response = await apiClient.banIp(ip, duration, reason, requestOptions || {});
  invalidateAfterBanMutation();
  return response;
}

export async function unbanDashboardIp(ip, requestOptions = {}) {
  const apiClient = requireApiClient();
  const response = await apiClient.unbanIp(ip, requestOptions || {});
  invalidateAfterBanMutation();
  return response;
}

export async function getDashboardRobotsPreview(patch = null, requestOptions = {}) {
  const apiClient = requireApiClient();
  return apiClient.getRobotsPreview(patch, requestOptions || {});
}

export async function getDashboardEvents(hours = 24, options = {}) {
  const apiClient = requireApiClient();
  return apiClient.getEvents(hours, options || {});
}

export function unmountDashboardApp() {
  if (!runtimeMounted) return;
  runtimeMounted = false;
  stopConnectionHeartbeat();
  resetHeartbeatConnectionState('runtime_unmounted');
  runtimeMountOptions = normalizeRuntimeMountOptions({});
  dashboardApiClient = null;
  dashboardState = null;
  dashboardRefreshRuntime = null;
  resolveAdminApiEndpoint = () => ({ endpoint: '' });
  setSessionState(false, '');
  releaseChartRuntime({ window });
}
