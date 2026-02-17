// @ts-check

import * as dashboardCharts from './modules/charts.js';
import * as statusPanel from './modules/status.js';
import * as configControls from './modules/config-controls.js';
import * as adminSessionModule from './modules/admin-session.js';
import * as tabLifecycleModule from './modules/tab-lifecycle.js';
import * as dashboardApiClientModule from './modules/api-client.js';
import * as dashboardStateModule from './modules/dashboard-state.js';
import * as monitoringViewModule from './modules/monitoring-view.js';
import * as tablesViewModule from './modules/tables-view.js';
import * as formatModule from './modules/core/format.js';
import * as domModule from './modules/core/dom.js';
import * as configSchemaModule from './modules/config-schema.js';
import * as configDraftStoreModule from './modules/config-draft-store.js';
import * as configFormUtilsModule from './modules/config-form-utils.js';
import { createRuntimeEffects } from './modules/services/runtime-effects.js';

const {
  parseCountryCodesStrict,
  normalizeCountryCodesForCompare,
  parseListTextarea,
  formatListTextarea,
  normalizeListTextareaForCompare,
  parseHoneypotPathsTextarea,
  formatBrowserRulesTextarea,
  parseBrowserRulesTextarea,
  normalizeBrowserRulesForCompare
} = configFormUtilsModule;
const escapeHtml = formatModule.escapeHtml;

const INTEGER_FIELD_RULES = {
  'ban-duration-days': { min: 0, max: 365, fallback: 0, label: 'Manual ban duration days' },
  'ban-duration-hours': { min: 0, max: 23, fallback: 1, label: 'Manual ban duration hours' },
  'ban-duration-minutes': { min: 0, max: 59, fallback: 0, label: 'Manual ban duration minutes' },
  'robots-crawl-delay': { min: 0, max: 60, fallback: 2, label: 'Crawl delay' },
  'maze-threshold': { min: 5, max: 500, fallback: 50, label: 'Maze threshold' },
  'rate-limit-threshold': { min: 1, max: 1000000, fallback: 80, label: 'Rate limit' },
  'challenge-puzzle-transform-count': { min: 4, max: 8, fallback: 6, label: 'Challenge transform count' },
  'pow-difficulty': { min: 12, max: 20, fallback: 15, label: 'PoW difficulty' },
  'pow-ttl': { min: 30, max: 300, fallback: 90, label: 'PoW seed TTL' },
  'dur-honeypot-days': { min: 0, max: 365, fallback: 1, label: 'Maze Threshold Exceeded days' },
  'dur-honeypot-hours': { min: 0, max: 23, fallback: 0, label: 'Maze Threshold Exceeded hours' },
  'dur-honeypot-minutes': { min: 0, max: 59, fallback: 0, label: 'Maze Threshold Exceeded minutes' },
  'dur-rate-limit-days': { min: 0, max: 365, fallback: 0, label: 'Rate Limit Exceeded days' },
  'dur-rate-limit-hours': { min: 0, max: 23, fallback: 1, label: 'Rate Limit Exceeded hours' },
  'dur-rate-limit-minutes': { min: 0, max: 59, fallback: 0, label: 'Rate Limit Exceeded minutes' },
  'dur-browser-days': { min: 0, max: 365, fallback: 0, label: 'Browser Automation Detected days' },
  'dur-browser-hours': { min: 0, max: 23, fallback: 6, label: 'Browser Automation Detected hours' },
  'dur-browser-minutes': { min: 0, max: 59, fallback: 0, label: 'Browser Automation Detected minutes' },
  'dur-cdp-days': { min: 0, max: 365, fallback: 0, label: 'CDP Automation Detected days' },
  'dur-cdp-hours': { min: 0, max: 23, fallback: 12, label: 'CDP Automation Detected hours' },
  'dur-cdp-minutes': { min: 0, max: 59, fallback: 0, label: 'CDP Automation Detected minutes' },
  'dur-admin-days': { min: 0, max: 365, fallback: 0, label: 'Admin Manual Ban days' },
  'dur-admin-hours': { min: 0, max: 23, fallback: 6, label: 'Admin Manual Ban hours' },
  'dur-admin-minutes': { min: 0, max: 59, fallback: 0, label: 'Admin Manual Ban minutes' },
  'challenge-puzzle-threshold': { min: 1, max: 10, fallback: 3, label: 'Challenge threshold' },
  'maze-threshold-score': { min: 1, max: 10, fallback: 6, label: 'Maze threshold' },
  'weight-js-required': { min: 0, max: 10, fallback: 1, label: 'JS weight' },
  'weight-geo-risk': { min: 0, max: 10, fallback: 2, label: 'GEO weight' },
  'weight-rate-medium': { min: 0, max: 10, fallback: 1, label: 'Rate 50% weight' },
  'weight-rate-high': { min: 0, max: 10, fallback: 2, label: 'Rate 80% weight' }
};

const BAN_DURATION_BOUNDS_SECONDS = { min: 60, max: 31536000 };

const BAN_DURATION_FIELDS = {
  honeypot: {
    label: 'Maze Threshold Exceeded duration',
    fallback: 86400,
    daysId: 'dur-honeypot-days',
    hoursId: 'dur-honeypot-hours',
    minutesId: 'dur-honeypot-minutes'
  },
  rateLimit: {
    label: 'Rate Limit Exceeded duration',
    fallback: 3600,
    daysId: 'dur-rate-limit-days',
    hoursId: 'dur-rate-limit-hours',
    minutesId: 'dur-rate-limit-minutes'
  },
  browser: {
    label: 'Browser Automation Detected duration',
    fallback: 21600,
    daysId: 'dur-browser-days',
    hoursId: 'dur-browser-hours',
    minutesId: 'dur-browser-minutes'
  },
  cdp: {
    label: 'CDP Automation Detected duration',
    fallback: 43200,
    daysId: 'dur-cdp-days',
    hoursId: 'dur-cdp-hours',
    minutesId: 'dur-cdp-minutes'
  },
  admin: {
    label: 'Admin Manual Ban duration',
    fallback: 21600,
    daysId: 'dur-admin-days',
    hoursId: 'dur-admin-hours',
    minutesId: 'dur-admin-minutes'
  }
};

const MANUAL_BAN_DURATION_FIELD = {
  label: 'Manual ban duration',
  fallback: 3600,
  daysId: 'ban-duration-days',
  hoursId: 'ban-duration-hours',
  minutesId: 'ban-duration-minutes'
};

const EDGE_INTEGRATION_MODES = new Set(['off', 'advisory', 'authoritative']);
const ADVANCED_CONFIG_TEMPLATE_PATHS = Object.freeze(
  configSchemaModule.advancedConfigTemplatePaths || []
);

const IPV4_SEGMENT_PATTERN = /^\d{1,3}$/;
const IPV6_INPUT_PATTERN = /^[0-9a-fA-F:.]+$/;
let adminEndpointContext = null;
let adminSessionController = null;
let dashboardTabCoordinator = null;
let dashboardApiClient = null;
let dashboardState = null;
let monitoringView = null;
let tablesView = null;
let configDraftStore = null;
let runtimeEffects = null;
let autoRefreshTimer = null;
let pageVisible = document.visibilityState !== 'hidden';
const domCache = domModule.createCache({ document });
const getById = domCache.byId;
const query = domCache.query;
const queryAll = domCache.queryAll;
const domWriteScheduler = domModule.createWriteScheduler();

const TAB_REFRESH_INTERVAL_MS = Object.freeze({
  monitoring: 30000,
  'ip-bans': 45000,
  status: 60000,
  config: 60000,
  tuning: 60000
});

function runDomWriteBatch(task) {
  return new Promise((resolve) => {
    domWriteScheduler.schedule(() => {
      task();
      resolve();
    });
  });
}

function sanitizeIntegerText(value) {
  return (value || '').replace(/[^\d]/g, '');
}

function sanitizeIpText(value) {
  return (value || '').replace(/[^0-9a-fA-F:.]/g, '');
}

function sanitizeEndpointText(value) {
  return (value || '').replace(/\s+/g, '').trim();
}

function cloneJsonValue(value) {
  if (value === null || value === undefined) return null;
  if (typeof value !== 'object') return value;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (_e) {
    return null;
  }
}

function readValueAtPath(obj, path) {
  const segments = String(path || '').split('.');
  let cursor = obj;
  for (const segment of segments) {
    if (!segment || cursor === null || typeof cursor !== 'object') return undefined;
    if (!Object.prototype.hasOwnProperty.call(cursor, segment)) return undefined;
    cursor = cursor[segment];
  }
  return cursor;
}

function writeValueAtPath(target, path, value) {
  const segments = String(path || '').split('.');
  if (segments.length === 0) return;
  let cursor = target;
  for (let i = 0; i < segments.length; i += 1) {
    const segment = segments[i];
    if (!segment) return;
    const isLeaf = i === segments.length - 1;
    if (isLeaf) {
      cursor[segment] = value;
      return;
    }
    if (!cursor[segment] || typeof cursor[segment] !== 'object' || Array.isArray(cursor[segment])) {
      cursor[segment] = {};
    }
    cursor = cursor[segment];
  }
}

function buildAdvancedConfigTemplate(config) {
  const template = {};
  ADVANCED_CONFIG_TEMPLATE_PATHS.forEach((path) => {
    const rawValue = readValueAtPath(config, path);
    if (rawValue === undefined) return;
    const cloned = cloneJsonValue(rawValue);
    writeValueAtPath(template, path, cloned === null && rawValue !== null ? rawValue : cloned);
  });
  return template;
}

function normalizeJsonObjectForCompare(raw) {
  try {
    const parsed = JSON.parse(String(raw || '{}'));
    if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') return null;
    return JSON.stringify(parsed);
  } catch (_e) {
    return null;
  }
}

function isLoopbackHostname(hostname) {
  const normalized = String(hostname || '').trim().toLowerCase();
  return (
    normalized === 'localhost' ||
    normalized === '127.0.0.1' ||
    normalized === '::1' ||
    normalized === '[::1]'
  );
}

function fieldErrorIdFor(input) {
  const raw = input.id || input.name || 'field';
  return `field-error-${raw.replace(/[^a-zA-Z0-9_-]/g, '-')}`;
}

function getOrCreateFieldErrorElement(input) {
  if (!input || !input.parentElement) return null;
  const id = fieldErrorIdFor(input);
  let errorEl = getById(id);
  if (!errorEl || errorEl.dataset.fieldFor !== input.id) {
    errorEl = document.createElement('div');
    errorEl.id = id;
    errorEl.className = 'field-error';
    errorEl.dataset.fieldFor = input.id || '';
    errorEl.setAttribute('aria-live', 'polite');
    input.insertAdjacentElement('afterend', errorEl);
  }
  if (input.getAttribute('aria-describedby') !== id) {
    input.setAttribute('aria-describedby', id);
  }
  return errorEl;
}

function setFieldError(input, message, showInline = true) {
  if (!input) return;
  input.setCustomValidity(message || '');
  if (!showInline) return;

  const errorEl = getOrCreateFieldErrorElement(input);
  if (!errorEl) return;
  if (message) {
    input.setAttribute('aria-invalid', 'true');
    errorEl.textContent = message;
    errorEl.classList.add('visible');
    return;
  }

  input.removeAttribute('aria-invalid');
  errorEl.textContent = '';
  errorEl.classList.remove('visible');
}

function parseIntegerLoose(id) {
  const input = getById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return null;
  const sanitized = sanitizeIntegerText(input.value);
  if (input.value !== sanitized) input.value = sanitized;
  if (sanitized.length === 0) return null;
  const parsed = Number.parseInt(sanitized, 10);
  if (!Number.isInteger(parsed)) return null;
  return parsed;
}

function durationPartsToSeconds(days, hours, minutes) {
  return (days * 86400) + (hours * 3600) + (minutes * 60);
}

function secondsToDurationParts(totalSeconds, fallbackSeconds) {
  const fallback = Number.parseInt(fallbackSeconds, 10) || 0;
  let seconds = Number.parseInt(totalSeconds, 10);
  if (!Number.isFinite(seconds) || seconds <= 0) seconds = fallback;
  if (seconds < BAN_DURATION_BOUNDS_SECONDS.min) seconds = BAN_DURATION_BOUNDS_SECONDS.min;
  if (seconds > BAN_DURATION_BOUNDS_SECONDS.max) seconds = BAN_DURATION_BOUNDS_SECONDS.max;
  return {
    days: Math.floor(seconds / 86400),
    hours: Math.floor((seconds % 86400) / 3600),
    minutes: Math.floor((seconds % 3600) / 60)
  };
}

function setDurationInputsFromSeconds(group, totalSeconds) {
  if (!group) return;
  const daysInput = getById(group.daysId);
  const hoursInput = getById(group.hoursId);
  const minutesInput = getById(group.minutesId);
  if (!daysInput || !hoursInput || !minutesInput) return;

  const parts = secondsToDurationParts(totalSeconds, group.fallback);
  daysInput.value = String(parts.days);
  hoursInput.value = String(parts.hours);
  minutesInput.value = String(parts.minutes);
}

function setBanDurationInputFromSeconds(durationKey, totalSeconds) {
  const group = BAN_DURATION_FIELDS[durationKey];
  setDurationInputsFromSeconds(group, totalSeconds);
}

function readDurationFromInputs(group, showInline = false) {
  if (!group) return null;

  const daysInput = getById(group.daysId);
  const hoursInput = getById(group.hoursId);
  const minutesInput = getById(group.minutesId);
  if (!daysInput || !hoursInput || !minutesInput) return null;

  const daysValid = validateIntegerFieldById(group.daysId, showInline);
  const hoursValid = validateIntegerFieldById(group.hoursId, showInline);
  const minutesValid = validateIntegerFieldById(group.minutesId, showInline);
  const days = parseIntegerLoose(group.daysId);
  const hours = parseIntegerLoose(group.hoursId);
  const minutes = parseIntegerLoose(group.minutesId);

  if (!daysValid || !hoursValid || !minutesValid || days === null || hours === null || minutes === null) return null;

  const totalSeconds = durationPartsToSeconds(days, hours, minutes);
  if (totalSeconds < BAN_DURATION_BOUNDS_SECONDS.min || totalSeconds > BAN_DURATION_BOUNDS_SECONDS.max) {
    const message = `${group.label} must be between 1 minute and 365 days.`;
    setFieldError(daysInput, message, showInline);
    setFieldError(hoursInput, message, showInline);
    setFieldError(minutesInput, message, showInline);
    return null;
  }

  setFieldError(daysInput, '', showInline);
  setFieldError(hoursInput, '', showInline);
  setFieldError(minutesInput, '', showInline);
  return { days, hours, minutes, totalSeconds };
}

function readBanDurationFromInputs(durationKey, showInline = false) {
  const group = BAN_DURATION_FIELDS[durationKey];
  return readDurationFromInputs(group, showInline);
}

function readBanDurationSeconds(durationKey) {
  const group = BAN_DURATION_FIELDS[durationKey];
  if (!group) return null;
  const result = readDurationFromInputs(group, true);
  if (result) return result.totalSeconds;

  const daysInput = getById(group.daysId);
  const hoursInput = getById(group.hoursId);
  const minutesInput = getById(group.minutesId);
  if (daysInput && !daysInput.checkValidity()) {
    daysInput.reportValidity();
    daysInput.focus();
    return null;
  }
  if (hoursInput && !hoursInput.checkValidity()) {
    hoursInput.reportValidity();
    hoursInput.focus();
    return null;
  }
  if (minutesInput && !minutesInput.checkValidity()) {
    minutesInput.reportValidity();
    minutesInput.focus();
    return null;
  }
  if (daysInput) {
    daysInput.reportValidity();
    daysInput.focus();
  }
  return null;
}

function readManualBanDurationSeconds(showInline = false) {
  const result = readDurationFromInputs(MANUAL_BAN_DURATION_FIELD, showInline);
  if (result) return result.totalSeconds;

  const daysInput = getById(MANUAL_BAN_DURATION_FIELD.daysId);
  const hoursInput = getById(MANUAL_BAN_DURATION_FIELD.hoursId);
  const minutesInput = getById(MANUAL_BAN_DURATION_FIELD.minutesId);
  if (daysInput && !daysInput.checkValidity()) {
    daysInput.reportValidity();
    daysInput.focus();
    return null;
  }
  if (hoursInput && !hoursInput.checkValidity()) {
    hoursInput.reportValidity();
    hoursInput.focus();
    return null;
  }
  if (minutesInput && !minutesInput.checkValidity()) {
    minutesInput.reportValidity();
    minutesInput.focus();
    return null;
  }
  if (daysInput) {
    daysInput.reportValidity();
    daysInput.focus();
  }
  return null;
}

function isValidIpv4(value) {
  const parts = value.split('.');
  if (parts.length !== 4) return false;
  return parts.every(part => {
    if (!IPV4_SEGMENT_PATTERN.test(part)) return false;
    if (part.length > 1 && part.startsWith('0')) return false;
    const num = Number.parseInt(part, 10);
    return num >= 0 && num <= 255;
  });
}

function isValidIpv6(value) {
  if (!IPV6_INPUT_PATTERN.test(value)) return false;
  try {
    new URL(`http://[${value}]/`);
    return true;
  } catch (e) {
    return false;
  }
}

function isValidIpAddress(value) {
  if (!value) return false;
  if (value.includes(':')) return isValidIpv6(value);
  if (value.includes('.')) return isValidIpv4(value);
  return false;
}

function parseEndpointUrl(value) {
  const sanitized = sanitizeEndpointText(value);
  if (!sanitized) return null;
  try {
    const url = new URL(sanitized);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
    if (!url.hostname) return null;
    const pathname = url.pathname === '/' ? '' : url.pathname.replace(/\/+$/, '');
    return `${url.protocol}//${url.host}${pathname}`;
  } catch (e) {
    return null;
  }
}

function resolveAdminApiEndpoint() {
  if (adminEndpointContext) return adminEndpointContext;

  const origin = window.location.origin || `${window.location.protocol}//${window.location.host}`;
  let endpoint = parseEndpointUrl(origin) || origin;

  // Local diagnostics only: allow ?api_endpoint=http://127.0.0.1:3000 override on loopback dashboards.
  if (isLoopbackHostname(window.location.hostname)) {
    const params = new URLSearchParams(window.location.search || '');
    const override = sanitizeEndpointText(params.get('api_endpoint') || '');
    if (override) {
      const parsed = parseEndpointUrl(override);
      if (parsed) {
        try {
          const parsedUrl = new URL(parsed);
          if (isLoopbackHostname(parsedUrl.hostname)) {
            endpoint = parsed;
          }
        } catch (_e) {}
      }
    }
  }

  adminEndpointContext = { endpoint };
  return adminEndpointContext;
}

function validateIntegerFieldById(id, showInline = false) {
  const input = getById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return false;
  const parsed = parseIntegerLoose(id);
  if (parsed === null) {
    setFieldError(input, `${rules.label} is required.`, showInline);
    return false;
  }
  if (parsed < rules.min || parsed > rules.max) {
    setFieldError(input, `${rules.label} must be between ${rules.min} and ${rules.max}.`, showInline);
    return false;
  }
  setFieldError(input, '', showInline);
  return true;
}

function readIntegerFieldValue(id, messageTarget) {
  const input = getById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return null;
  if (!validateIntegerFieldById(id, true)) {
    const parsed = parseIntegerLoose(id);
    const message = parsed === null
      ? `${rules.label} is required.`
      : `${rules.label} must be between ${rules.min} and ${rules.max}.`;
    input.reportValidity();
    input.focus();
    return null;
  }
  const value = parseIntegerLoose(id);
  input.value = String(value);
  setFieldError(input, '', true);
  return value;
}

function validateIpFieldById(id, required, label, showInline = false) {
  const input = getById(id);
  if (!input) return false;
  const sanitized = sanitizeIpText(input.value.trim());
  if (input.value !== sanitized) input.value = sanitized;

  if (!sanitized) {
    if (!required) {
      setFieldError(input, '', showInline);
      return true;
    }
    setFieldError(input, `${label} is required.`, showInline);
    return false;
  }

  if (!isValidIpAddress(sanitized)) {
    setFieldError(input, `${label} must be a valid IPv4 or IPv6 address.`, showInline);
    return false;
  }
  setFieldError(input, '', showInline);
  return true;
}

function readIpFieldValue(id, required, messageTarget, label) {
  const input = getById(id);
  if (!input) return null;
  if (!validateIpFieldById(id, required, label, true)) {
    const sanitized = sanitizeIpText(input.value.trim());
    const message = sanitized.length === 0
      ? `${label} is required.`
      : `${label} must be a valid IPv4 or IPv6 address.`;
    input.reportValidity();
    input.focus();
    return null;
  }
  const sanitized = sanitizeIpText(input.value.trim());
  input.value = sanitized;
  setFieldError(input, '', true);
  return sanitized;
}

function hasValidApiContext() {
  return adminSessionController ? adminSessionController.hasValidApiContext() : false;
}

function refreshMazePreviewLink() {
  const link = getById('preview-maze-link');
  if (!link) return;
  const resolved = resolveAdminApiEndpoint();
  const endpoint = resolved && resolved.endpoint ? resolved.endpoint : '';
  link.href = `${endpoint}/admin/maze/preview`;
}

function redirectToLogin() {
  const next = encodeURIComponent(window.location.pathname + window.location.search);
  window.location.replace(`/dashboard/login.html?next=${next}`);
}

function tabStateElement(tab) {
  return query(`[data-tab-state="${tab}"]`);
}

function setTabStateMessage(tab, kind, message) {
  const stateEl = tabStateElement(tab);
  if (!stateEl) return;
  const normalizedKind = kind === 'error' || kind === 'loading' || kind === 'empty' ? kind : '';
  if (!normalizedKind) {
    stateEl.hidden = true;
    stateEl.textContent = '';
    stateEl.className = 'tab-state';
    return;
  }
  stateEl.hidden = false;
  stateEl.textContent = String(message || '');
  stateEl.className = `tab-state tab-state--${normalizedKind}`;
}

function showTabLoading(tab, message = 'Loading...') {
  if (dashboardState) {
    dashboardState.setTabLoading(tab, true);
    dashboardState.clearTabError(tab);
  }
  setTabStateMessage(tab, 'loading', message);
}

function showTabError(tab, message) {
  if (dashboardState) {
    dashboardState.setTabError(tab, message);
    dashboardState.setTabEmpty(tab, false);
  }
  setTabStateMessage(tab, 'error', message);
}

function showTabEmpty(tab, message) {
  if (dashboardState) {
    dashboardState.setTabEmpty(tab, true);
    dashboardState.clearTabError(tab);
    dashboardState.markTabUpdated(tab);
  }
  setTabStateMessage(tab, 'empty', message);
}

function clearTabStateMessage(tab) {
  if (dashboardState) {
    dashboardState.setTabLoading(tab, false);
    dashboardState.setTabEmpty(tab, false);
    dashboardState.clearTabError(tab);
    dashboardState.markTabUpdated(tab);
  }
  setTabStateMessage(tab, '', '');
}

function validateGeoFieldById(id, showInline = false) {
  const field = getById(id);
  if (!field) return false;
  try {
    parseCountryCodesStrict(field.value);
    setFieldError(field, '', showInline);
    return true;
  } catch (e) {
    setFieldError(field, e.message || 'Invalid country list.', showInline);
    return false;
  }
}

function refreshCoreActionButtonsState() {
  const apiValid = hasValidApiContext();
  refreshMazePreviewLink();
  const logoutBtn = getById('logout-btn');
  if (logoutBtn) {
    logoutBtn.disabled = !apiValid;
  }
  setValidActionButtonState(
    'ban-btn',
    apiValid,
    validateIpFieldById('ban-ip', true, 'Ban IP') && Boolean(readDurationFromInputs(MANUAL_BAN_DURATION_FIELD))
  );
  setValidActionButtonState(
    'unban-btn',
    apiValid,
    validateIpFieldById('unban-ip', true, 'Unban IP')
  );

  if (typeof checkBanDurationsChanged === 'function') {
    checkBanDurationsChanged();
  }
  if (typeof checkMazeConfigChanged === 'function') {
    checkMazeConfigChanged();
  }

  if (typeof checkRobotsConfigChanged === 'function') {
    checkRobotsConfigChanged();
  }
  if (typeof checkGeoConfigChanged === 'function') {
    checkGeoConfigChanged();
  }
  if (typeof checkHoneypotConfigChanged === 'function') {
    checkHoneypotConfigChanged();
  }
  if (typeof checkBrowserPolicyConfigChanged === 'function') {
    checkBrowserPolicyConfigChanged();
  }
  if (typeof checkBypassAllowlistsConfigChanged === 'function') {
    checkBypassAllowlistsConfigChanged();
  }
  if (typeof checkPowConfigChanged === 'function') {
    checkPowConfigChanged();
  }
  if (typeof checkChallengePuzzleConfigChanged === 'function') {
    checkChallengePuzzleConfigChanged();
  }
  if (typeof checkBotnessConfigChanged === 'function') {
    checkBotnessConfigChanged();
  }
  if (typeof checkCdpConfigChanged === 'function') {
    checkCdpConfigChanged();
  }
  if (typeof checkRateLimitConfigChanged === 'function') {
    checkRateLimitConfigChanged();
  }
  if (typeof checkJsRequiredConfigChanged === 'function') {
    checkJsRequiredConfigChanged();
  }
  if (typeof checkAdvancedConfigChanged === 'function') {
    checkAdvancedConfigChanged();
  }
}

function createDashboardTabControllers() {
  function makeController(tab) {
    return {
      init: function initTabController() {},
      mount: function mountTabController() {
        document.body.dataset.activeDashboardTab = tab;
        if (dashboardState) {
          dashboardState.setActiveTab(tab);
        }
        refreshCoreActionButtonsState();
        if (hasValidApiContext()) {
          refreshDashboardForTab(tab, 'tab-mount');
        }
      },
      unmount: function unmountTabController() {},
      refresh: function refreshTabController(context = {}) {
        return refreshDashboardForTab(tab, context.reason || 'manual');
      }
    };
  }

  return {
    monitoring: makeController('monitoring'),
    'ip-bans': makeController('ip-bans'),
    status: makeController('status'),
    config: makeController('config'),
    tuning: makeController('tuning')
  };
}

function getAdminContext(messageTarget) {
  if (!adminSessionController) return null;
  return adminSessionController.getAdminContext(messageTarget);
}

function bindIntegerFieldValidation(id) {
  const input = getById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return;

  const apply = (showInline = false) => {
    const sanitized = sanitizeIntegerText(input.value);
    if (input.value !== sanitized) input.value = sanitized;
    if (!sanitized) {
      setFieldError(input, `${rules.label} is required.`, showInline);
      return;
    }
    const parsed = Number.parseInt(sanitized, 10);
    if (!Number.isInteger(parsed)) {
      setFieldError(input, `${rules.label} must be a whole number.`, showInline);
      return;
    }
    if (parsed < rules.min || parsed > rules.max) {
      setFieldError(input, `${rules.label} must be between ${rules.min} and ${rules.max}.`, showInline);
      return;
    }
    setFieldError(input, '', showInline);
  };

  input.addEventListener('input', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  input.addEventListener('blur', () => {
    if (!input.value) {
      input.value = String(rules.fallback);
    }
    const parsed = parseIntegerLoose(id);
    if (parsed !== null && parsed < rules.min) input.value = String(rules.min);
    if (parsed !== null && parsed > rules.max) input.value = String(rules.max);
    apply(true);
    refreshCoreActionButtonsState();
  });
  apply(false);
}

function bindIpFieldValidation(id, required, label) {
  const input = getById(id);
  if (!input) return;
  const apply = (showInline = false) => {
    validateIpFieldById(id, required, label, showInline);
  };
  input.addEventListener('input', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  input.addEventListener('blur', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  apply(false);
}

function initInputValidation() {
  Object.keys(INTEGER_FIELD_RULES).forEach(bindIntegerFieldValidation);
  bindIpFieldValidation('ban-ip', true, 'Ban IP');
  bindIpFieldValidation('unban-ip', true, 'Unban IP');
  refreshCoreActionButtonsState();
}

function envVar(name) {
  return `<code class="env-var">${name}</code>`;
}

function parseBoolLike(value, fallback = false) {
  if (typeof value === 'boolean') return value;
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') return true;
  if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') return false;
  return fallback;
}

function normalizeEdgeIntegrationMode(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (EDGE_INTEGRATION_MODES.has(normalized)) {
    return normalized;
  }
  return 'off';
}

function adminConfigWriteEnabled(config) {
  return parseBoolLike(config && config.admin_config_write_enabled, false);
}

function updateConfigModeUi(config) {
  const writeEnabled = adminConfigWriteEnabled(config);
  const failModeFromConfig = parseBoolLike(config && config.kv_store_fail_open, true)
    ? 'open'
    : 'closed';
  statusPanel.update({
    testMode: parseBoolLike(config && config.test_mode, false),
    failMode: statusPanel.normalizeFailMode(failModeFromConfig),
    httpsEnforced: parseBoolLike(config && config.https_enforced, false),
    forwardedHeaderTrustConfigured: parseBoolLike(
      config && config.forwarded_header_trust_configured,
      false
    )
  });
  const subtitle = getById('config-mode-subtitle');
  if (subtitle) {
    if (writeEnabled) {
      subtitle.innerHTML =
        `Admin page configuration enabled. Saved changes persist across builds. Set ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} to <strong>false</strong> in deployment env to disable.`;
    } else {
      subtitle.innerHTML =
        `Admin page configuration disabled. Set ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} to <strong>true</strong> to enable.`;
    }
  }

  queryAll('.config-edit-pane').forEach(el => {
    el.classList.toggle('hidden', !writeEnabled);
  });
  statusPanel.render();
}

// Update stat cards
function updateStatCards(analytics, events, bans) {
  getById('total-bans').textContent = analytics.ban_count || 0;
  getById('active-bans').textContent = bans.length || 0;
  getById('total-events').textContent = (events.recent_events || []).length;
  const uniqueIps = typeof events.unique_ips === 'number' ? events.unique_ips : (events.top_ips || []).length;
  getById('unique-ips').textContent = uniqueIps;
  
  // Update test mode banner and toggle
  const testMode = analytics.test_mode === true;
  const banner = getById('test-mode-banner');
  const toggle = getById('test-mode-toggle');
  const status = getById('test-mode-status');
  
  if (testMode) {
    banner.classList.remove('hidden');
    status.textContent = 'ENABLED (LOGGING ONLY)';
    status.style.color = '#d97706';
  } else {
    banner.classList.add('hidden');
    status.textContent = 'DISABLED (BLOCKING ACTIVE)';
    status.style.color = '#10b981';
  }
  toggle.checked = testMode;

  statusPanel.update({
    testMode,
    failMode: statusPanel.normalizeFailMode(analytics.fail_mode)
  });
  statusPanel.render();
}

// Update ban duration fields from config
function updateBanDurations(config) {
  if (config.ban_durations) {
    setBanDurationInputFromSeconds('honeypot', config.ban_durations.honeypot);
    setBanDurationInputFromSeconds('rateLimit', config.ban_durations.rate_limit);
    setBanDurationInputFromSeconds('browser', config.ban_durations.browser);
    setBanDurationInputFromSeconds('cdp', config.ban_durations.cdp);
    setBanDurationInputFromSeconds('admin', config.ban_durations.admin);
    setDraft('banDurations', {
      honeypot: Number.parseInt(config.ban_durations.honeypot, 10) || BAN_DURATION_FIELDS.honeypot.fallback,
      rateLimit: Number.parseInt(config.ban_durations.rate_limit, 10) || BAN_DURATION_FIELDS.rateLimit.fallback,
      browser: Number.parseInt(config.ban_durations.browser, 10) || BAN_DURATION_FIELDS.browser.fallback,
      cdp: Number.parseInt(config.ban_durations.cdp, 10) || BAN_DURATION_FIELDS.cdp.fallback,
      admin: Number.parseInt(config.ban_durations.admin, 10) || BAN_DURATION_FIELDS.admin.fallback
    });
    const btn = getById('save-durations-btn');
    if (btn) {
      btn.dataset.saving = 'false';
      btn.disabled = true;
      btn.textContent = 'Save Durations';
    }
  }
}

// Update maze config controls from loaded config
function updateMazeConfig(config) {
  const statusPatch = {};
  if (config.maze_enabled !== undefined) {
    getById('maze-enabled-toggle').checked = config.maze_enabled;
    statusPatch.mazeEnabled = config.maze_enabled === true;
  }
  if (config.maze_auto_ban !== undefined) {
    getById('maze-auto-ban-toggle').checked = config.maze_auto_ban;
    statusPatch.mazeAutoBan = config.maze_auto_ban === true;
  }
  if (config.maze_auto_ban_threshold !== undefined) {
    getById('maze-threshold').value = config.maze_auto_ban_threshold;
  }
  setDraft('maze', {
    enabled: getById('maze-enabled-toggle').checked,
    autoBan: getById('maze-auto-ban-toggle').checked,
    threshold: parseInt(getById('maze-threshold').value, 10) || 50
  });
  const btn = getById('save-maze-config');
  if (btn) {
    btn.dataset.saving = 'false';
    btn.disabled = true;
    btn.textContent = 'Save Maze Settings';
  }
  statusPanel.update(statusPatch);
  statusPanel.render();
}

function updateGeoConfig(config) {
  const mutable = adminConfigWriteEnabled(config);
  const risk = formatCountryCodes(config.geo_risk);
  const allow = formatCountryCodes(config.geo_allow);
  const challenge = formatCountryCodes(config.geo_challenge);
  const maze = formatCountryCodes(config.geo_maze);
  const block = formatCountryCodes(config.geo_block);

  getById('geo-risk-list').value = risk;
  getById('geo-allow-list').value = allow;
  getById('geo-challenge-list').value = challenge;
  getById('geo-maze-list').value = maze;
  getById('geo-block-list').value = block;

  setDraft('geo', {
    risk: normalizeCountryCodesForCompare(risk),
    allow: normalizeCountryCodesForCompare(allow),
    challenge: normalizeCountryCodesForCompare(challenge),
    maze: normalizeCountryCodesForCompare(maze),
    block: normalizeCountryCodesForCompare(block),
    mutable
  });

  statusPanel.update({
    geoRiskCount: Array.isArray(config.geo_risk) ? config.geo_risk.length : 0,
    geoAllowCount: Array.isArray(config.geo_allow) ? config.geo_allow.length : 0,
    geoChallengeCount: Array.isArray(config.geo_challenge) ? config.geo_challenge.length : 0,
    geoMazeCount: Array.isArray(config.geo_maze) ? config.geo_maze.length : 0,
    geoBlockCount: Array.isArray(config.geo_block) ? config.geo_block.length : 0
  });
  statusPanel.render();

  setGeoConfigEditable(mutable);

  const scoringBtn = getById('save-geo-scoring-config');
  if (scoringBtn) {
    scoringBtn.disabled = true;
    scoringBtn.textContent = 'Save GEO Scoring';
  }

  const routingBtn = getById('save-geo-routing-config');
  if (routingBtn) {
    routingBtn.disabled = true;
    routingBtn.textContent = 'Save GEO Routing';
  }
}

function updateHoneypotConfig(config) {
  const enabledToggle = getById('honeypot-enabled-toggle');
  const field = getById('honeypot-paths');
  if (!field) return;
  if (enabledToggle) {
    enabledToggle.checked = config.honeypot_enabled !== false;
  }
  const formatted = formatListTextarea(config.honeypots);
  field.value = formatted;
  setDraft('honeypot', {
    enabled: enabledToggle ? enabledToggle.checked : true,
    values: normalizeListTextareaForCompare(formatted)
  });
  const btn = getById('save-honeypot-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save Honeypots';
  }
}

function updateBrowserPolicyConfig(config) {
  const blockField = getById('browser-block-rules');
  const whitelistField = getById('browser-whitelist-rules');
  if (!blockField || !whitelistField) return;

  const blockText = formatBrowserRulesTextarea(config.browser_block);
  const whitelistText = formatBrowserRulesTextarea(config.browser_whitelist);
  blockField.value = blockText;
  whitelistField.value = whitelistText;
  setDraft('browserPolicy', {
    block: normalizeBrowserRulesForCompare(blockText),
    whitelist: normalizeBrowserRulesForCompare(whitelistText)
  });
  const btn = getById('save-browser-policy-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save Browser Policy';
  }
}

function updateBypassAllowlistConfig(config) {
  const networkField = getById('network-whitelist');
  const pathField = getById('path-whitelist');
  if (!networkField || !pathField) return;

  const networkText = formatListTextarea(config.whitelist);
  const pathText = formatListTextarea(config.path_whitelist);
  networkField.value = networkText;
  pathField.value = pathText;
  setDraft('bypassAllowlists', {
    network: normalizeListTextareaForCompare(networkText),
    path: normalizeListTextareaForCompare(pathText)
  });
  const btn = getById('save-whitelist-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save Allowlists';
  }
}

const CONFIG_DRAFT_DEFAULTS = Object.freeze({
  robots: { enabled: true, crawlDelay: 2 },
  aiPolicy: { blockTraining: true, blockSearch: false, allowSearch: false },
  cdp: { enabled: true, autoBan: true, threshold: 0.6 },
  edgeMode: { mode: 'off' },
  rateLimit: { value: 80 },
  jsRequired: { enforced: true },
  maze: { enabled: false, autoBan: false, threshold: 50 },
  banDurations: { honeypot: 86400, rateLimit: 3600, browser: 21600, cdp: 43200, admin: 21600 },
  pow: { enabled: true, difficulty: 15, ttl: 90, mutable: true },
  botness: {
    challengeThreshold: 3,
    mazeThreshold: 6,
    weightJsRequired: 1,
    weightGeoRisk: 2,
    weightRateMedium: 1,
    weightRateHigh: 2,
    mutable: true
  },
  geo: { risk: '', allow: '', challenge: '', maze: '', block: '', mutable: false },
  honeypot: { enabled: true, values: '/instaban' },
  browserPolicy: { block: '', whitelist: '' },
  bypassAllowlists: { network: '', path: '' },
  challengePuzzle: { enabled: true, count: 6, mutable: true },
  advancedConfig: { normalized: '{}' }
});

function getDraft(sectionKey) {
  const fallback = CONFIG_DRAFT_DEFAULTS[sectionKey] || null;
  return configDraftStore ? configDraftStore.get(sectionKey, fallback) : cloneJsonValue(fallback);
}

function setDraft(sectionKey, value) {
  if (configDraftStore) {
    configDraftStore.set(sectionKey, value);
  }
}

function isDraftDirty(sectionKey, currentValue) {
  if (!configDraftStore) return true;
  return configDraftStore.isDirty(sectionKey, currentValue);
}

const GEO_SCORING_FIELD_IDS = ['geo-risk-list'];
const GEO_ROUTING_FIELD_IDS = [
  'geo-allow-list',
  'geo-challenge-list',
  'geo-maze-list',
  'geo-block-list'
];
const GEO_FIELD_IDS = [...GEO_SCORING_FIELD_IDS, ...GEO_ROUTING_FIELD_IDS];

function setGeoConfigEditable(editable) {
  GEO_FIELD_IDS.forEach(id => {
    const field = getById(id);
    field.disabled = !editable;
    if (!editable) {
      field.blur();
    }
  });
}

function sanitizeGeoTextareaValue(value) {
  return (value || '')
    .replace(/[^a-zA-Z,]/g, '')
    .toUpperCase();
}

function formatCountryCodes(list) {
  if (!Array.isArray(list) || list.length === 0) return '';
  return list.join(',');
}

function updateRobotsConfig(config) {
  // Update toggles from server config
  if (config.robots_enabled !== undefined) {
    getById('robots-enabled-toggle').checked = config.robots_enabled;
  }
  const aiBlockTraining = config.ai_policy_block_training ?? config.robots_block_ai_training;
  if (aiBlockTraining !== undefined) {
    getById('robots-block-training-toggle').checked = aiBlockTraining;
  }
  const aiBlockSearch = config.ai_policy_block_search ?? config.robots_block_ai_search;
  if (aiBlockSearch !== undefined) {
    getById('robots-block-search-toggle').checked = aiBlockSearch;
  }
  const aiAllowSearch = config.ai_policy_allow_search_engines ?? config.robots_allow_search_engines;
  if (aiAllowSearch !== undefined) {
    // Invert: toggle ON = restrict (allow=false), toggle OFF = allow (allow=true)
    getById('robots-allow-search-toggle').checked = !aiAllowSearch;
  }
  if (config.robots_crawl_delay !== undefined) {
    getById('robots-crawl-delay').value = config.robots_crawl_delay;
  }
  setDraft('robots', {
    enabled: getById('robots-enabled-toggle').checked,
    crawlDelay: parseInt(getById('robots-crawl-delay').value) || 2
  });
  setDraft('aiPolicy', {
    blockTraining: getById('robots-block-training-toggle').checked,
    blockSearch: getById('robots-block-search-toggle').checked,
    allowSearch: getById('robots-allow-search-toggle').checked
  });

  const robotsBtn = getById('save-robots-config');
  robotsBtn.disabled = true;
  robotsBtn.textContent = 'Save robots serving';

  const aiBtn = getById('save-ai-policy-config');
  if (aiBtn) {
    aiBtn.disabled = true;
    aiBtn.textContent = 'Save AI bot policy';
  }
}

// Check if robots config has changed from saved state
function checkRobotsConfigChanged() {
  const apiValid = hasValidApiContext();
  const delayValid = validateIntegerFieldById('robots-crawl-delay');
  const current = {
    enabled: getById('robots-enabled-toggle').checked,
    crawlDelay: parseInt(getById('robots-crawl-delay').value) || 2
  };
  const changed = delayValid && isDraftDirty('robots', current);
  setDirtySaveButtonState('save-robots-config', changed, apiValid, delayValid);
  const btn = getById('save-robots-config');
  if (changed) {
    btn.textContent = 'Save robots serving';
  }
}

function checkAiPolicyConfigChanged() {
  const apiValid = hasValidApiContext();
  const current = {
    blockTraining: getById('robots-block-training-toggle').checked,
    blockSearch: getById('robots-block-search-toggle').checked,
    allowSearch: getById('robots-allow-search-toggle').checked
  };
  const changed = isDraftDirty('aiPolicy', current);
  setDirtySaveButtonState('save-ai-policy-config', changed, apiValid, true);
  const btn = getById('save-ai-policy-config');
  if (changed) {
    btn.textContent = 'Save AI bot policy';
  }
}

function setButtonState(buttonId, apiValid, fieldsValid, changed, requireChange) {
  const btn = getById(buttonId);
  if (!btn) return;
  if (btn.dataset.saving === 'true') return;
  const canSubmit = apiValid && fieldsValid && (!requireChange || changed);
  btn.disabled = !canSubmit;
}

function setDirtySaveButtonState(buttonId, changed, apiValid, fieldsValid = true) {
  setButtonState(buttonId, apiValid, fieldsValid, changed, true);
}

function setValidActionButtonState(buttonId, apiValid, fieldsValid = true) {
  setButtonState(buttonId, apiValid, fieldsValid, true, false);
}

function checkMazeConfigChanged() {
  const currentThreshold = parseIntegerLoose('maze-threshold');
  const fieldsValid = validateIntegerFieldById('maze-threshold');
  const apiValid = hasValidApiContext();
  const changed = fieldsValid && isDraftDirty('maze', {
    enabled: getById('maze-enabled-toggle').checked,
    autoBan: getById('maze-auto-ban-toggle').checked,
    threshold: currentThreshold
  });
  setDirtySaveButtonState('save-maze-config', changed, apiValid, fieldsValid);
}

function checkBanDurationsChanged() {
  const honeypot = readBanDurationFromInputs('honeypot');
  const rateLimit = readBanDurationFromInputs('rateLimit');
  const browser = readBanDurationFromInputs('browser');
  const cdp = readBanDurationFromInputs('cdp');
  const admin = readBanDurationFromInputs('admin');
  const fieldsValid = Boolean(honeypot && rateLimit && browser && cdp && admin);
  const apiValid = hasValidApiContext();
  const current = fieldsValid ? {
    honeypot: honeypot.totalSeconds,
    rateLimit: rateLimit.totalSeconds,
    browser: browser.totalSeconds,
    cdp: cdp.totalSeconds,
    admin: admin.totalSeconds
  } : getDraft('banDurations');
  const changed = fieldsValid && isDraftDirty('banDurations', current);
  setDirtySaveButtonState('save-durations-btn', changed, apiValid, fieldsValid);
}

function validateHoneypotPathsField(showInline = false) {
  const field = getById('honeypot-paths');
  if (!field) return false;
  try {
    parseHoneypotPathsTextarea(field.value);
    setFieldError(field, '', showInline);
    return true;
  } catch (e) {
    setFieldError(field, e.message || 'Invalid honeypot paths.', showInline);
    return false;
  }
}

function checkHoneypotConfigChanged() {
  const apiValid = hasValidApiContext();
  const fieldsValid = validateHoneypotPathsField();
  const currentEnabled = getById('honeypot-enabled-toggle').checked;
  const saved = getDraft('honeypot');
  const current = fieldsValid
    ? normalizeListTextareaForCompare(getById('honeypot-paths').value)
    : saved.values;
  const changed = fieldsValid && (
    currentEnabled !== saved.enabled ||
    current !== saved.values
  );
  setDirtySaveButtonState('save-honeypot-config', changed, apiValid, fieldsValid);
}

function validateBrowserRulesField(id, showInline = false) {
  const field = getById(id);
  if (!field) return false;
  try {
    parseBrowserRulesTextarea(field.value);
    setFieldError(field, '', showInline);
    return true;
  } catch (e) {
    setFieldError(field, e.message || 'Invalid browser rules.', showInline);
    return false;
  }
}

function checkBrowserPolicyConfigChanged() {
  const apiValid = hasValidApiContext();
  const blockValid = validateBrowserRulesField('browser-block-rules');
  const whitelistValid = validateBrowserRulesField('browser-whitelist-rules');
  const fieldsValid = blockValid && whitelistValid;
  const currentBlock = normalizeBrowserRulesForCompare(getById('browser-block-rules').value);
  const currentWhitelist = normalizeBrowserRulesForCompare(getById('browser-whitelist-rules').value);
  const changed = fieldsValid && isDraftDirty('browserPolicy', {
    block: currentBlock,
    whitelist: currentWhitelist
  });
  setDirtySaveButtonState('save-browser-policy-config', changed, apiValid, fieldsValid);
}

function checkBypassAllowlistsConfigChanged() {
  const apiValid = hasValidApiContext();
  const current = {
    network: normalizeListTextareaForCompare(getById('network-whitelist').value),
    path: normalizeListTextareaForCompare(getById('path-whitelist').value)
  };
  const changed = isDraftDirty('bypassAllowlists', current);
  setDirtySaveButtonState('save-whitelist-config', changed, apiValid, true);
}

function checkChallengePuzzleConfigChanged() {
  const apiValid = hasValidApiContext();
  const fieldsValid = validateIntegerFieldById('challenge-puzzle-transform-count');
  const toggle = getById('challenge-puzzle-enabled-toggle');
  const current = parseIntegerLoose('challenge-puzzle-transform-count');
  const saved = getDraft('challengePuzzle');
  const enabledChanged = Boolean(toggle && (toggle.checked !== saved.enabled));
  const countChanged = current !== null && current !== saved.count;
  const changed = fieldsValid && (enabledChanged || countChanged);
  setDirtySaveButtonState('save-challenge-puzzle-config', changed, apiValid, fieldsValid);
}

// Add change listeners for robots serving and AI-policy controls.
['robots-enabled-toggle'].forEach(id => {
  getById(id).addEventListener('change', checkRobotsConfigChanged);
});
getById('robots-crawl-delay').addEventListener('input', checkRobotsConfigChanged);
['robots-block-training-toggle', 'robots-block-search-toggle', 'robots-allow-search-toggle'].forEach(id => {
  getById(id).addEventListener('change', checkAiPolicyConfigChanged);
});
['maze-enabled-toggle', 'maze-auto-ban-toggle'].forEach(id => {
  getById(id).addEventListener('change', checkMazeConfigChanged);
});
['honeypot-paths'].forEach(id => {
  const field = getById(id);
  if (!field) return;
  field.addEventListener('input', () => {
    validateHoneypotPathsField(true);
    checkHoneypotConfigChanged();
    refreshCoreActionButtonsState();
  });
  field.addEventListener('blur', () => {
    validateHoneypotPathsField(true);
    checkHoneypotConfigChanged();
    refreshCoreActionButtonsState();
  });
});
getById('honeypot-enabled-toggle').addEventListener('change', checkHoneypotConfigChanged);
['browser-block-rules', 'browser-whitelist-rules'].forEach((id) => {
  const field = getById(id);
  if (!field) return;
  field.addEventListener('input', () => {
    validateBrowserRulesField(id, true);
    checkBrowserPolicyConfigChanged();
    refreshCoreActionButtonsState();
  });
  field.addEventListener('blur', () => {
    validateBrowserRulesField(id, true);
    checkBrowserPolicyConfigChanged();
    refreshCoreActionButtonsState();
  });
});
['network-whitelist', 'path-whitelist'].forEach((id) => {
  const field = getById(id);
  if (!field) return;
  field.addEventListener('input', () => {
    checkBypassAllowlistsConfigChanged();
    refreshCoreActionButtonsState();
  });
  field.addEventListener('blur', () => {
    checkBypassAllowlistsConfigChanged();
    refreshCoreActionButtonsState();
  });
});
getById('challenge-puzzle-transform-count').addEventListener('input', checkChallengePuzzleConfigChanged);
getById('challenge-puzzle-enabled-toggle').addEventListener('change', checkChallengePuzzleConfigChanged);

// Fetch and update robots.txt preview content
async function refreshRobotsPreview() {
  if (!getAdminContext(getById('admin-msg'))) return;
  const previewContent = getById('robots-preview-content');
  
  try {
    const data = await dashboardApiClient.getRobotsPreview();
    previewContent.textContent = data.content || '# No preview available';
  } catch (e) {
    previewContent.textContent = '# Error loading preview: ' + e.message;
    console.error('Failed to load robots preview:', e);
  }
}

// Toggle robots.txt preview visibility
getById('preview-robots').onclick = async function() {
  const preview = getById('robots-preview');
  const btn = this;
  
  if (preview.classList.contains('hidden')) {
    // Show preview
    btn.textContent = 'Loading...';
    btn.disabled = true;
    await refreshRobotsPreview();
    preview.classList.remove('hidden');
    btn.textContent = 'Hide robots.txt';
    btn.disabled = false;
  } else {
    // Hide preview
    preview.classList.add('hidden');
    btn.textContent = 'Show robots.txt';
  }
};

// Update CDP detection config controls from loaded config
function updateCdpConfig(config) {
  const statusPatch = {};
  if (config.cdp_detection_enabled !== undefined) {
    getById('cdp-enabled-toggle').checked = config.cdp_detection_enabled;
    statusPatch.cdpEnabled = config.cdp_detection_enabled === true;
  }
  if (config.cdp_auto_ban !== undefined) {
    getById('cdp-auto-ban-toggle').checked = config.cdp_auto_ban;
    statusPatch.cdpAutoBan = config.cdp_auto_ban === true;
  }
  if (config.cdp_detection_threshold !== undefined) {
    getById('cdp-threshold-slider').value = config.cdp_detection_threshold;
     getById('cdp-threshold-value').textContent = parseFloat(config.cdp_detection_threshold).toFixed(1);
  }
  // Store saved state for change detection
  setDraft('cdp', {
    enabled: getById('cdp-enabled-toggle').checked,
    autoBan: getById('cdp-auto-ban-toggle').checked,
    threshold: parseFloat(getById('cdp-threshold-slider').value)
  });
  // Reset button state
  const btn = getById('save-cdp-config');
  btn.disabled = true;
  btn.textContent = 'Save CDP Settings';
  statusPanel.update(statusPatch);
  statusPanel.render();
}

function updateEdgeIntegrationModeConfig(config) {
  const mode = normalizeEdgeIntegrationMode(config.edge_integration_mode);
  const select = getById('edge-integration-mode-select');
  if (!select) return;
  select.value = mode;
  setDraft('edgeMode', { mode });

  const btn = getById('save-edge-integration-mode-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save Edge Integration Mode';
  }
}

function updateRateLimitConfig(config) {
  const rateLimit = parseInt(config.rate_limit, 10) || 80;
  const field = getById('rate-limit-threshold');
  field.value = rateLimit;
  setDraft('rateLimit', { value: rateLimit });
  statusPanel.update({ rateLimit });
  statusPanel.render();

  const btn = getById('save-rate-limit-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save Rate Limit';
  }
}

function updateJsRequiredConfig(config) {
  const enforced = parseBoolLike(config.js_required_enforced, true);
  const toggle = getById('js-required-enforced-toggle');
  toggle.checked = enforced;
  setDraft('jsRequired', { enforced });
  statusPanel.update({ jsRequiredEnforced: enforced });
  statusPanel.render();

  const btn = getById('save-js-required-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save JS Required';
  }
}

// Update PoW config controls from loaded config
function updatePowConfig(config) {
  const powEnabled = parseBoolLike(config.pow_enabled, true);
  const difficulty = parseInt(config.pow_difficulty, 10);
  const ttl = parseInt(config.pow_ttl_seconds, 10);

  statusPanel.update({
    powEnabled
  });
  statusPanel.render();

  if (!Number.isNaN(difficulty)) {
    getById('pow-difficulty').value = difficulty;
  }
  if (!Number.isNaN(ttl)) {
    getById('pow-ttl').value = ttl;
  }
  getById('pow-enabled-toggle').checked = powEnabled;

  setDraft('pow', {
    enabled: getById('pow-enabled-toggle').checked,
    difficulty: parseInt(getById('pow-difficulty').value, 10) || 15,
    ttl: parseInt(getById('pow-ttl').value, 10) || 90
  });

  const btn = getById('save-pow-config');
  btn.disabled = true;
  btn.textContent = 'Save PoW Settings';
}

function updateBotnessSignalDefinitions(signalDefinitions) {
  const scoredSignals = (signalDefinitions && Array.isArray(signalDefinitions.scored_signals))
    ? signalDefinitions.scored_signals
    : [];
  const terminalSignals = (signalDefinitions && Array.isArray(signalDefinitions.terminal_signals))
    ? signalDefinitions.terminal_signals
    : [];

  const scoredTarget = getById('botness-signal-list');
  const terminalTarget = getById('botness-terminal-list');

  scoredTarget.innerHTML = scoredSignals.length
    ? scoredSignals.map(signal => `
      <div class="info-row">
        <span class="info-label">${signal.label}</span>
        <span>${signal.weight}</span>
      </div>
    `).join('')
    : '<p class="text-muted">No scored signals</p>';

  terminalTarget.innerHTML = terminalSignals.length
    ? terminalSignals.map(signal => `
      <div class="info-row">
        <span class="info-label">${signal.label}</span>
        <span>${signal.action}</span>
      </div>
    `).join('')
    : '<p class="text-muted">No terminal signals</p>';
}

function updateChallengeConfig(config) {
  const writable = adminConfigWriteEnabled(config);
  const challengeEnabled = config.challenge_puzzle_enabled !== false;
  const challengeTransformCount = parseInt(config.challenge_puzzle_transform_count, 10);
  const challengeThreshold = parseInt(config.challenge_puzzle_risk_threshold, 10);
  const challengeDefault = parseInt(config.challenge_puzzle_risk_threshold_default, 10);
  const mazeThreshold = parseInt(config.botness_maze_threshold, 10);
  const mazeDefault = parseInt(config.botness_maze_threshold_default, 10);
  const weights = config.botness_weights || {};

  if (!Number.isNaN(challengeThreshold)) {
    getById('challenge-puzzle-threshold').value = challengeThreshold;
  }
  if (!Number.isNaN(mazeThreshold)) {
    getById('maze-threshold-score').value = mazeThreshold;
  }
  if (!Number.isNaN(challengeTransformCount)) {
    getById('challenge-puzzle-transform-count').value = challengeTransformCount;
  }
  getById('challenge-puzzle-enabled-toggle').checked = challengeEnabled;
  getById('weight-js-required').value = parseInt(weights.js_required, 10) || 1;
  getById('weight-geo-risk').value = parseInt(weights.geo_risk, 10) || 2;
  getById('weight-rate-medium').value = parseInt(weights.rate_medium, 10) || 1;
  getById('weight-rate-high').value = parseInt(weights.rate_high, 10) || 2;

  getById('botness-config-status').textContent = writable ? 'EDITABLE' : 'READ ONLY';
  getById('challenge-puzzle-default').textContent = Number.isNaN(challengeDefault) ? '--' : challengeDefault;
  getById('maze-threshold-default').textContent = Number.isNaN(mazeDefault) ? '--' : mazeDefault;

  statusPanel.update({
    challengeEnabled,
    challengeThreshold: Number.isNaN(challengeThreshold) ? 3 : challengeThreshold,
    mazeThreshold: Number.isNaN(mazeThreshold) ? 6 : mazeThreshold,
    botnessWeights: {
      js_required: parseInt(weights.js_required, 10) || 0,
      geo_risk: parseInt(weights.geo_risk, 10) || 0,
      rate_medium: parseInt(weights.rate_medium, 10) || 0,
      rate_high: parseInt(weights.rate_high, 10) || 0
    }
  });

  const editableFields = [
    'challenge-puzzle-threshold',
    'maze-threshold-score',
    'weight-js-required',
    'weight-geo-risk',
    'weight-rate-medium',
    'weight-rate-high'
  ];
  editableFields.forEach(id => {
    getById(id).disabled = !writable;
  });

  setDraft('botness', {
    challengeThreshold: parseInt(getById('challenge-puzzle-threshold').value, 10) || 3,
    mazeThreshold: parseInt(getById('maze-threshold-score').value, 10) || 6,
    weightJsRequired: parseInt(getById('weight-js-required').value, 10) || 1,
    weightGeoRisk: parseInt(getById('weight-geo-risk').value, 10) || 2,
    weightRateMedium: parseInt(getById('weight-rate-medium').value, 10) || 1,
    weightRateHigh: parseInt(getById('weight-rate-high').value, 10) || 2
  });

  updateBotnessSignalDefinitions(config.botness_signal_definitions);

  const btn = getById('save-botness-config');
  btn.disabled = true;
  btn.textContent = 'Save Botness Settings';

  setDraft('challengePuzzle', {
    enabled: getById('challenge-puzzle-enabled-toggle').checked,
    count: parseInt(getById('challenge-puzzle-transform-count').value, 10) || 6
  });
  const challengeBtn = getById('save-challenge-puzzle-config');
  if (challengeBtn) {
    challengeBtn.disabled = true;
    challengeBtn.textContent = 'Save Challenge Puzzle';
  }
  const challengeTransformField = getById('challenge-puzzle-transform-count');
  const challengeEnabledToggle = getById('challenge-puzzle-enabled-toggle');
  if (challengeTransformField) {
    challengeTransformField.disabled = !writable;
  }
  if (challengeEnabledToggle) {
    challengeEnabledToggle.disabled = !writable;
  }
  statusPanel.render();
}

function checkPowConfigChanged() {
  const apiValid = hasValidApiContext();
  const powFieldsValid = validateIntegerFieldById('pow-difficulty') && validateIntegerFieldById('pow-ttl');
  const current = {
    enabled: getById('pow-enabled-toggle').checked,
    difficulty: parseInt(getById('pow-difficulty').value, 10) || 15,
    ttl: parseInt(getById('pow-ttl').value, 10) || 90
  };
  const changed = isDraftDirty('pow', current);
  setDirtySaveButtonState('save-pow-config', changed, apiValid, powFieldsValid);
}

getById('pow-enabled-toggle').addEventListener('change', checkPowConfigChanged);
getById('pow-difficulty').addEventListener('input', checkPowConfigChanged);
getById('pow-ttl').addEventListener('input', checkPowConfigChanged);

function checkBotnessConfigChanged() {
  const apiValid = hasValidApiContext();
  const fieldsValid =
    validateIntegerFieldById('challenge-puzzle-threshold') &&
    validateIntegerFieldById('maze-threshold-score') &&
    validateIntegerFieldById('weight-js-required') &&
    validateIntegerFieldById('weight-geo-risk') &&
    validateIntegerFieldById('weight-rate-medium') &&
    validateIntegerFieldById('weight-rate-high');
  const current = {
    challengeThreshold: parseInt(getById('challenge-puzzle-threshold').value, 10) || 3,
    mazeThreshold: parseInt(getById('maze-threshold-score').value, 10) || 6,
    weightJsRequired: parseInt(getById('weight-js-required').value, 10) || 1,
    weightGeoRisk: parseInt(getById('weight-geo-risk').value, 10) || 2,
    weightRateMedium: parseInt(getById('weight-rate-medium').value, 10) || 1,
    weightRateHigh: parseInt(getById('weight-rate-high').value, 10) || 2
  };
  const changed = isDraftDirty('botness', current);
  setDirtySaveButtonState('save-botness-config', changed, apiValid, fieldsValid);
}

[
  'challenge-puzzle-threshold',
  'maze-threshold-score',
  'weight-js-required',
  'weight-geo-risk',
  'weight-rate-medium',
  'weight-rate-high'
].forEach(id => {
  getById(id).addEventListener('input', checkBotnessConfigChanged);
});

function checkGeoConfigChanged() {
  const apiValid = hasValidApiContext();
  const scoringValid = GEO_SCORING_FIELD_IDS.every(validateGeoFieldById);
  const routingValid = GEO_ROUTING_FIELD_IDS.every(validateGeoFieldById);
  const savedGeo = getDraft('geo');
  if (!savedGeo.mutable) {
    const scoringBtn = getById('save-geo-scoring-config');
    if (scoringBtn) scoringBtn.disabled = true;
    const routingBtn = getById('save-geo-routing-config');
    if (routingBtn) routingBtn.disabled = true;
    return;
  }

  const current = {
    risk: normalizeCountryCodesForCompare(getById('geo-risk-list').value),
    allow: normalizeCountryCodesForCompare(getById('geo-allow-list').value),
    challenge: normalizeCountryCodesForCompare(getById('geo-challenge-list').value),
    maze: normalizeCountryCodesForCompare(getById('geo-maze-list').value),
    block: normalizeCountryCodesForCompare(getById('geo-block-list').value)
  };
  const scoringChanged = current.risk !== savedGeo.risk;
  const routingChanged =
    current.allow !== savedGeo.allow ||
    current.challenge !== savedGeo.challenge ||
    current.maze !== savedGeo.maze ||
    current.block !== savedGeo.block;

  setDirtySaveButtonState('save-geo-scoring-config', scoringChanged, apiValid, scoringValid);
  setDirtySaveButtonState('save-geo-routing-config', routingChanged, apiValid, routingValid);
}

GEO_FIELD_IDS.forEach(id => {
  const field = getById(id);
  field.addEventListener('input', () => {
    const sanitized = sanitizeGeoTextareaValue(field.value);
    if (field.value !== sanitized) {
      const cursor = field.selectionStart;
      const delta = field.value.length - sanitized.length;
      field.value = sanitized;
      if (typeof cursor === 'number') {
        const next = Math.max(0, cursor - Math.max(0, delta));
        field.setSelectionRange(next, next);
      }
    }
    validateGeoFieldById(id, true);
    checkGeoConfigChanged();
    refreshCoreActionButtonsState();
  });
  field.addEventListener('blur', () => {
    validateGeoFieldById(id, true);
    checkGeoConfigChanged();
    refreshCoreActionButtonsState();
  });
});

// Check if CDP config has changed from saved state
function checkCdpConfigChanged() {
  const apiValid = hasValidApiContext();
  const current = {
    enabled: getById('cdp-enabled-toggle').checked,
    autoBan: getById('cdp-auto-ban-toggle').checked,
    threshold: parseFloat(getById('cdp-threshold-slider').value)
  };
  const changed = isDraftDirty('cdp', current);
  setDirtySaveButtonState('save-cdp-config', changed, apiValid);
}

function checkEdgeIntegrationModeChanged() {
  const apiValid = hasValidApiContext();
  const select = getById('edge-integration-mode-select');
  if (!select) return;
  const current = normalizeEdgeIntegrationMode(select.value);
  const changed = isDraftDirty('edgeMode', { mode: current });
  setDirtySaveButtonState('save-edge-integration-mode-config', changed, apiValid);
}

function checkRateLimitConfigChanged() {
  const apiValid = hasValidApiContext();
  const valueValid = validateIntegerFieldById('rate-limit-threshold');
  const current = parseIntegerLoose('rate-limit-threshold');
  const changed = current !== null && isDraftDirty('rateLimit', { value: current });
  setDirtySaveButtonState('save-rate-limit-config', changed, apiValid, valueValid);
}

getById('rate-limit-threshold').addEventListener('input', checkRateLimitConfigChanged);

function checkJsRequiredConfigChanged() {
  const apiValid = hasValidApiContext();
  const current = getById('js-required-enforced-toggle').checked;
  const changed = isDraftDirty('jsRequired', { enforced: current });
  setDirtySaveButtonState('save-js-required-config', changed, apiValid);
}

getById('js-required-enforced-toggle').addEventListener('change', checkJsRequiredConfigChanged);

function setAdvancedConfigEditorFromConfig(config, preserveDirty = true) {
  const field = getById('advanced-config-json');
  if (!field) return;
  const previousBaseline = getDraft('advancedConfig').normalized || '{}';
  const template = buildAdvancedConfigTemplate(config || {});
  const formatted = JSON.stringify(template, null, 2);
  const currentNormalized = normalizeJsonObjectForCompare(field.value);
  const hasUnsavedEdits = field.dataset.dirty === 'true';

  setDraft('advancedConfig', { normalized: normalizeJsonObjectForCompare(formatted) || '{}' });

  const shouldReplace =
    !preserveDirty ||
    !hasUnsavedEdits ||
    currentNormalized === previousBaseline ||
    !String(field.value || '').trim();

  if (shouldReplace) {
    field.value = formatted;
  }
  checkAdvancedConfigChanged();
}

function readAdvancedConfigPatch(messageTarget) {
  const field = getById('advanced-config-json');
  if (!field) return null;
  const raw = String(field.value || '').trim();
  const parsedText = raw.length > 0 ? raw : '{}';
  let patch;
  try {
    patch = JSON.parse(parsedText);
  } catch (e) {
    const message = `Advanced config JSON parse error: ${e.message}`;
    setFieldError(field, message, true);
    if (messageTarget) {
      messageTarget.textContent = message;
      messageTarget.className = 'message error';
    }
    return null;
  }
  if (!patch || Array.isArray(patch) || typeof patch !== 'object') {
    const message = 'Advanced config patch must be a JSON object.';
    setFieldError(field, message, true);
    if (messageTarget) {
      messageTarget.textContent = message;
      messageTarget.className = 'message error';
    }
    return null;
  }
  setFieldError(field, '', true);
  return patch;
}

function checkAdvancedConfigChanged() {
  const field = getById('advanced-config-json');
  const btn = getById('save-advanced-config');
  if (!field || !btn) return;
  const apiValid = hasValidApiContext();
  const normalized = normalizeJsonObjectForCompare(field.value);
  const valid = normalized !== null;
  const baseline = getDraft('advancedConfig').normalized || '{}';
  const changed = valid && normalized !== baseline;
  field.dataset.dirty = changed ? 'true' : 'false';
  setFieldError(field, valid ? '' : 'Advanced config patch must be valid JSON object syntax.', true);
  setDirtySaveButtonState('save-advanced-config', changed, apiValid, valid);
}

const advancedConfigField = getById('advanced-config-json');
if (advancedConfigField) {
  advancedConfigField.addEventListener('input', () => {
    checkAdvancedConfigChanged();
    refreshCoreActionButtonsState();
  });
  advancedConfigField.addEventListener('blur', () => {
    checkAdvancedConfigChanged();
    refreshCoreActionButtonsState();
  });
}

// Update threshold display when slider moves
getById('cdp-threshold-slider').addEventListener('input', function() {
  getById('cdp-threshold-value').textContent = parseFloat(this.value).toFixed(1);
  checkCdpConfigChanged();
});

// Add change listeners for CDP config controls
['cdp-enabled-toggle', 'cdp-auto-ban-toggle'].forEach(id => {
  getById(id).addEventListener('change', checkCdpConfigChanged);
});

getById('edge-integration-mode-select').addEventListener('change', checkEdgeIntegrationModeChanged);

function updateLastUpdatedTimestamp() {
  const ts = new Date().toISOString();
  const label = getById('last-updated');
  if (label) label.textContent = `updated: ${ts}`;
}

function isConfigSnapshotEmpty(config) {
  return !config || typeof config !== 'object' || Object.keys(config).length === 0;
}

async function refreshSharedConfig(reason = 'manual') {
  if (!dashboardApiClient) {
    return dashboardState ? dashboardState.getSnapshot('config') : null;
  }
  if (dashboardState && reason === 'auto-refresh' && !dashboardState.isTabStale('config')) {
    return dashboardState.getSnapshot('config');
  }
  const config = await dashboardApiClient.getConfig();
  if (dashboardState) dashboardState.setSnapshot('config', config);
  await runDomWriteBatch(() => {
    statusPanel.update({ configSnapshot: config });
    updateConfigModeUi(config);
    updateBanDurations(config);
    updateRateLimitConfig(config);
    updateJsRequiredConfig(config);
    updateMazeConfig(config);
    updateGeoConfig(config);
    updateHoneypotConfig(config);
    updateBrowserPolicyConfig(config);
    updateBypassAllowlistConfig(config);
    updateRobotsConfig(config);
    updateCdpConfig(config);
    updateEdgeIntegrationModeConfig(config);
    updatePowConfig(config);
    updateChallengeConfig(config);
    setAdvancedConfigEditorFromConfig(config, true);
  });
  return config;
}

async function refreshMonitoringTab(reason = 'manual') {
  if (!dashboardApiClient) return;
  if (reason !== 'auto-refresh') {
    showTabLoading('monitoring', 'Loading monitoring data...');
  }

  getById('total-bans').textContent = '...';
  getById('active-bans').textContent = '...';
  getById('total-events').textContent = '...';
  getById('unique-ips').textContent = '...';
  const cdpTotalDetections = getById('cdp-total-detections');
  const cdpTotalAutoBans = getById('cdp-total-auto-bans');
  if (cdpTotalDetections) cdpTotalDetections.textContent = '...';
  if (cdpTotalAutoBans) cdpTotalAutoBans.textContent = '...';
  const honeypotTotal = getById('honeypot-total-hits');
  const challengeTotal = getById('challenge-failures-total');
  const powTotal = getById('pow-failures-total');
  const rateTotal = getById('rate-violations-total');
  const geoTotal = getById('geo-violations-total');
  const mazeTopOffender = getById('maze-top-offender');
  const mazeTopOffenderLabel = getById('maze-top-offender-label');
  const honeypotTopOffender = getById('honeypot-top-offender');
  const honeypotTopOffenderLabel = getById('honeypot-top-offender-label');
  const challengeTopOffender = getById('challenge-top-offender');
  const challengeTopOffenderLabel = getById('challenge-top-offender-label');
  const powTopOffender = getById('pow-top-offender');
  const powTopOffenderLabel = getById('pow-top-offender-label');
  const rateTopOffender = getById('rate-top-offender');
  const rateTopOffenderLabel = getById('rate-top-offender-label');
  if (honeypotTotal) honeypotTotal.textContent = '...';
  if (challengeTotal) challengeTotal.textContent = '...';
  if (powTotal) powTotal.textContent = '...';
  if (rateTotal) rateTotal.textContent = '...';
  if (geoTotal) geoTotal.textContent = '...';
  if (mazeTopOffender) mazeTopOffender.textContent = '...';
  if (honeypotTopOffender) honeypotTopOffender.textContent = '...';
  if (challengeTopOffender) challengeTopOffender.textContent = '...';
  if (powTopOffender) powTopOffender.textContent = '...';
  if (rateTopOffender) rateTopOffender.textContent = '...';
  if (mazeTopOffenderLabel) mazeTopOffenderLabel.textContent = 'Top Offender';
  if (honeypotTopOffenderLabel) honeypotTopOffenderLabel.textContent = 'Top Offender';
  if (challengeTopOffenderLabel) challengeTopOffenderLabel.textContent = 'Top Offender';
  if (powTopOffenderLabel) powTopOffenderLabel.textContent = 'Top Offender';
  if (rateTopOffenderLabel) rateTopOffenderLabel.textContent = 'Top Offender';

  const [analytics, events, bansData, mazeData, cdpData, cdpEventsData, monitoringData] = await Promise.all([
    dashboardApiClient.getAnalytics(),
    dashboardApiClient.getEvents(24),
    dashboardApiClient.getBans(),
    dashboardApiClient.getMaze(),
    dashboardApiClient.getCdp(),
    dashboardApiClient.getCdpEvents({ hours: 24, limit: 500 }),
    dashboardApiClient.getMonitoring({ hours: 24, limit: 10 })
  ]);

  if (dashboardState) {
    dashboardState.setSnapshot('analytics', analytics);
    dashboardState.setSnapshot('events', events);
    dashboardState.setSnapshot('bans', bansData);
    dashboardState.setSnapshot('maze', mazeData);
    dashboardState.setSnapshot('cdp', cdpData);
    dashboardState.setSnapshot('cdpEvents', cdpEventsData);
    dashboardState.setSnapshot('monitoring', monitoringData);
  }

  await runDomWriteBatch(() => {
    updateStatCards(analytics, events, bansData.bans || []);
    dashboardCharts.updateEventTypesChart(events.event_counts || {});
    dashboardCharts.updateTopIpsChart(events.top_ips || []);
    dashboardCharts.updateTimeSeriesChart();
    if (tablesView) {
      tablesView.updateEventsTable(events.recent_events || []);
      tablesView.updateCdpTotals(cdpData);
      tablesView.updateCdpEventsTable(cdpEventsData.events || []);
    }
    if (monitoringView) {
      monitoringView.updateMazeStats(mazeData);
      monitoringView.updateMonitoringSummary(monitoringData.summary || {});
      monitoringView.updatePrometheusHelper(monitoringData.prometheus || {});
    }
  });

  if (dashboardState && dashboardState.getDerivedState().monitoringEmpty) {
    showTabEmpty('monitoring', 'No operational events yet. Monitoring will populate as traffic arrives.');
  } else {
    clearTabStateMessage('monitoring');
  }
}

async function refreshIpBansTab(reason = 'manual') {
  if (!dashboardApiClient) return;
  if (reason !== 'auto-refresh') {
    showTabLoading('ip-bans', 'Loading ban list...');
  }
  const bansData = await dashboardApiClient.getBans();
  if (dashboardState) dashboardState.setSnapshot('bans', bansData);
  await runDomWriteBatch(() => {
    if (tablesView) {
      tablesView.updateBansTable(bansData.bans || []);
    }
  });
  if (!Array.isArray(bansData.bans) || bansData.bans.length === 0) {
    showTabEmpty('ip-bans', 'No active bans.');
  } else {
    clearTabStateMessage('ip-bans');
  }
}

async function refreshStatusTab(reason = 'manual') {
  if (reason !== 'auto-refresh') {
    showTabLoading('status', 'Loading status signals...');
  }
  const config = await refreshSharedConfig(reason);
  if (isConfigSnapshotEmpty(config)) {
    showTabEmpty('status', 'No status config snapshot available yet.');
  } else {
    clearTabStateMessage('status');
  }
}

async function refreshConfigTab(reason = 'manual') {
  if (reason !== 'auto-refresh') {
    showTabLoading('config', 'Loading config...');
  }
  const config = await refreshSharedConfig(reason);
  if (isConfigSnapshotEmpty(config)) {
    showTabEmpty('config', 'No config snapshot available yet.');
  } else {
    clearTabStateMessage('config');
  }
}

async function refreshTuningTab(reason = 'manual') {
  if (reason !== 'auto-refresh') {
    showTabLoading('tuning', 'Loading tuning values...');
  }
  const config = await refreshSharedConfig(reason);
  if (isConfigSnapshotEmpty(config)) {
    showTabEmpty('tuning', 'No tuning config snapshot available yet.');
  } else {
    clearTabStateMessage('tuning');
  }
}

async function refreshDashboardForTab(tab, reason = 'manual') {
  const activeTab = tabLifecycleModule.normalizeTab(tab);
  try {
    if (activeTab === 'monitoring') {
      await refreshMonitoringTab(reason);
      if (reason !== 'auto-refresh') {
        await refreshSharedConfig(reason);
      }
    } else if (activeTab === 'ip-bans') {
      await refreshIpBansTab(reason);
    } else if (activeTab === 'status') {
      await refreshStatusTab(reason);
    } else if (activeTab === 'config') {
      await refreshConfigTab(reason);
    } else if (activeTab === 'tuning') {
      await refreshTuningTab(reason);
    }
    if (dashboardState) dashboardState.markTabUpdated(activeTab);
    refreshCoreActionButtonsState();
    updateLastUpdatedTimestamp();
  } catch (error) {
    const message = error && error.message ? error.message : 'Refresh failed';
    console.error(`Dashboard refresh error (${activeTab}):`, error);
    showTabError(activeTab, message);
    const msg = getById('admin-msg');
    if (msg) {
      msg.textContent = `Refresh failed: ${message}`;
      msg.className = 'message error';
    }
  }
}

function refreshActiveTab(reason = 'manual') {
  if (dashboardTabCoordinator) {
    return dashboardTabCoordinator.refreshActive({ reason });
  }
  return refreshDashboardForTab('monitoring', reason);
}

// Admin controls - Ban IP
getById('ban-btn').onclick = async function () {
  const msg = getById('admin-msg');
  if (!getAdminContext(msg)) return;
  const ip = readIpFieldValue('ban-ip', true, msg, 'Ban IP');
  if (ip === null) return;
  const duration = readManualBanDurationSeconds(true);
  if (duration === null) return;

  msg.textContent = `Banning ${ip}...`;
  msg.className = 'message info';

  try {
    await dashboardApiClient.banIp(ip, duration);
    msg.textContent = `Banned ${ip} for ${duration}s`;
    msg.className = 'message success';
    getById('ban-ip').value = '';
    if (dashboardState) dashboardState.invalidate('ip-bans');
    runtimeEffects.setTimer(() => refreshActiveTab('ban-save'), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Admin controls - Unban IP
getById('unban-btn').onclick = async function () {
  const msg = getById('admin-msg');
  if (!getAdminContext(msg)) return;
  const ip = readIpFieldValue('unban-ip', true, msg, 'Unban IP');
  if (ip === null) return;

  msg.textContent = `Unbanning ${ip}...`;
  msg.className = 'message info';

  try {
    await dashboardApiClient.unbanIp(ip);
    msg.textContent = `Unbanned ${ip}`;
    msg.className = 'message success';
    getById('unban-ip').value = '';
    if (dashboardState) dashboardState.invalidate('ip-bans');
    runtimeEffects.setTimer(() => refreshActiveTab('unban-save'), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
};

function clearAutoRefreshTimer() {
  if (autoRefreshTimer) {
    runtimeEffects.clearTimer(autoRefreshTimer);
    autoRefreshTimer = null;
  }
}

function scheduleAutoRefresh() {
  clearAutoRefreshTimer();
  if (!hasValidApiContext() || !pageVisible) return;
  const activeTab = dashboardTabCoordinator
    ? dashboardTabCoordinator.getActiveTab()
    : (dashboardState ? dashboardState.getActiveTab() : 'monitoring');
  const interval = TAB_REFRESH_INTERVAL_MS[activeTab] || TAB_REFRESH_INTERVAL_MS.monitoring;
  autoRefreshTimer = runtimeEffects.setTimer(async () => {
    autoRefreshTimer = null;
    if (hasValidApiContext() && pageVisible) {
      await refreshDashboardForTab(activeTab, 'auto-refresh');
    }
    scheduleAutoRefresh();
  }, interval);
}

// Initialize charts and load data on page load
configDraftStore = configDraftStoreModule.create(CONFIG_DRAFT_DEFAULTS);
runtimeEffects = createRuntimeEffects();

dashboardState = dashboardStateModule.create({
  initialTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB
});

adminSessionController = adminSessionModule.create({
  resolveAdminApiEndpoint,
  refreshCoreActionButtonsState,
  redirectToLogin
});
adminSessionController.bindLogoutButton('logout-btn', 'admin-msg');

dashboardApiClient = dashboardApiClientModule.create({
  getAdminContext,
  onUnauthorized: redirectToLogin,
  request: (input, init) => runtimeEffects.request(input, init)
});

monitoringView = monitoringViewModule.create({
  escapeHtml,
  effects: runtimeEffects
});

tablesView = tablesViewModule.create({
  escapeHtml,
  onQuickUnban: async (ip) => {
    const msg = getById('admin-msg');
    if (!getAdminContext(msg)) return;

    msg.textContent = `Unbanning ${ip}...`;
    msg.className = 'message info';

    try {
      await dashboardApiClient.unbanIp(ip);
      msg.textContent = `Unbanned ${ip}`;
      msg.className = 'message success';
      if (dashboardState) dashboardState.invalidate('ip-bans');
      runtimeEffects.setTimer(() => refreshActiveTab('quick-unban'), 500);
    } catch (e) {
      msg.textContent = 'Error: ' + e.message;
      msg.className = 'message error';
    }
  }
});

dashboardTabCoordinator = tabLifecycleModule.createTabLifecycleCoordinator({
  controllers: createDashboardTabControllers(),
  onActiveTabChange: (nextTab) => {
    if (dashboardState) dashboardState.setActiveTab(nextTab);
    scheduleAutoRefresh();
  }
});
dashboardTabCoordinator.init();
initInputValidation();
if (monitoringView) {
  monitoringView.bindPrometheusCopyButtons();
}
dashboardCharts.init({
  getAdminContext,
  apiClient: dashboardApiClient
});
statusPanel.render();
configControls.bind({
  effects: runtimeEffects,
  context: {
    statusPanel,
    apiClient: dashboardApiClient,
    auth: {
      getAdminContext
    },
    callbacks: {
      onConfigSaved: (_patch, result) => {
        if (result && result.config) {
          statusPanel.update({ configSnapshot: result.config });
          setAdvancedConfigEditorFromConfig(result.config, true);
        }
        if (dashboardState) {
          dashboardState.invalidate('securityConfig');
          dashboardState.invalidate('monitoring');
          dashboardState.invalidate('ip-bans');
        }
      }
    },
    readers: {
      readIntegerFieldValue,
      readBanDurationSeconds,
      readAdvancedConfigPatch
    },
    updaters: {
      updateBanDurations,
      updateGeoConfig,
      updateHoneypotConfig,
      updateBrowserPolicyConfig,
      updateBypassAllowlistConfig,
      updateEdgeIntegrationModeConfig,
      refreshRobotsPreview,
      setAdvancedConfigFromConfig: setAdvancedConfigEditorFromConfig
    },
    actions: {
      refreshDashboard: () => refreshActiveTab('config-controls')
    },
    checks: {
      checkMazeConfigChanged,
      checkRobotsConfigChanged,
      checkAiPolicyConfigChanged,
      checkGeoConfigChanged,
      checkHoneypotConfigChanged,
      checkBrowserPolicyConfigChanged,
      checkBypassAllowlistsConfigChanged,
      checkPowConfigChanged,
      checkChallengePuzzleConfigChanged,
      checkBotnessConfigChanged,
      checkCdpConfigChanged,
      checkEdgeIntegrationModeChanged,
      checkRateLimitConfigChanged,
      checkJsRequiredConfigChanged,
      checkAdvancedConfigChanged,
      checkBanDurationsChanged
    },
    draft: {
      get: (sectionKey, fallback) => {
        if (configDraftStore) return configDraftStore.get(sectionKey, fallback);
        return fallback;
      },
      set: (sectionKey, value) => {
        if (configDraftStore) configDraftStore.set(sectionKey, value);
      }
    }
  }
});
adminSessionController.restoreAdminSession().then((authenticated) => {
  const sessionState = adminSessionController.getState();
  if (dashboardState) {
    dashboardState.setSession({
      authenticated: sessionState.authenticated === true,
      csrfToken: sessionState.csrfToken || ''
    });
  }
  if (!authenticated) {
    redirectToLogin();
    return;
  }
  refreshActiveTab('session-restored');
  scheduleAutoRefresh();
});

document.addEventListener('visibilitychange', () => {
  pageVisible = document.visibilityState !== 'hidden';
  if (pageVisible) {
    scheduleAutoRefresh();
  } else {
    clearAutoRefreshTimer();
  }
});
