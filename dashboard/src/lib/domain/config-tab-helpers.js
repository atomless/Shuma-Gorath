// @ts-check

const EDGE_MODES = new Set(['off', 'advisory', 'authoritative']);
const COMPOSABILITY_MODES = new Set(['off', 'signal', 'enforce', 'both']);
const IP_RANGE_POLICY_MODES = new Set(['off', 'advisory', 'enforce']);

/**
 * @param {unknown} value
 * @param {number} fallback
 */
export const parseInteger = (value, fallback) => {
  const parsed = Number.parseInt(String(value), 10);
  return Number.isInteger(parsed) ? parsed : fallback;
};

/**
 * @param {unknown} value
 * @param {number} fallback
 */
export const parseFloatNumber = (value, fallback) => {
  const parsed = Number.parseFloat(String(value));
  return Number.isFinite(parsed) ? parsed : fallback;
};

/**
 * @param {unknown} value
 */
export const normalizeEdgeMode = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  return EDGE_MODES.has(normalized) ? normalized : 'off';
};

/**
 * @param {unknown} value
 */
export const normalizeIpRangePolicyMode = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  return IP_RANGE_POLICY_MODES.has(normalized) ? normalized : 'off';
};

/**
 * @param {unknown} value
 */
export const isIpRangePolicyMode = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  return IP_RANGE_POLICY_MODES.has(normalized);
};

/**
 * @param {unknown} value
 */
export const normalizeComposabilityMode = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  return COMPOSABILITY_MODES.has(normalized) ? normalized : 'off';
};

/**
 * @param {unknown} mode
 */
export const geoToggleStateFromMode = (mode) => {
  const normalized = normalizeComposabilityMode(mode);
  return {
    scoringEnabled: normalized === 'signal' || normalized === 'both',
    routingEnabled: normalized === 'enforce' || normalized === 'both'
  };
};

/**
 * @param {{ scoringEnabled: boolean, routingEnabled: boolean }} state
 */
export const geoModeFromToggleState = ({ scoringEnabled, routingEnabled }) => {
  if (scoringEnabled && routingEnabled) return 'both';
  if (scoringEnabled) return 'signal';
  if (routingEnabled) return 'enforce';
  return 'off';
};

/**
 * @param {unknown} mode
 */
export const rateEnforcementEnabledFromMode = (mode) => {
  const normalized = normalizeComposabilityMode(mode);
  return normalized === 'enforce' || normalized === 'both';
};

/**
 * @param {{ enforcementEnabled: boolean }} state
 */
export const rateModeFromToggleState = ({ enforcementEnabled }) => {
  if (enforcementEnabled) return 'both';
  return 'signal';
};

/**
 * @param {unknown} values
 */
export const formatCountryCodes = (values) => {
  if (!Array.isArray(values) || values.length === 0) return '';
  return values
    .map((value) => String(value || '').trim().toUpperCase())
    .filter(Boolean)
    .join(',');
};

/**
 * @param {unknown} value
 */
export const normalizeJsonArrayForCompare = (value) => {
  try {
    const parsed = JSON.parse(String(value || '[]'));
    if (!Array.isArray(parsed)) return null;
    return JSON.stringify(parsed);
  } catch (_error) {
    return null;
  }
};

/**
 * @param {unknown} seconds
 * @param {number} fallbackSeconds
 */
export const durationPartsFromSeconds = (seconds, fallbackSeconds) => {
  const source = Number.parseInt(String(seconds), 10);
  const safe = Number.isFinite(source) && source > 0 ? source : fallbackSeconds;
  const days = Math.floor(safe / 86400);
  const remainingAfterDays = safe - (days * 86400);
  const hours = Math.floor(remainingAfterDays / 3600);
  const remainingAfterHours = remainingAfterDays - (hours * 3600);
  const minutes = Math.floor(remainingAfterHours / 60);
  return {
    days,
    hours,
    minutes
  };
};

/**
 * @param {unknown} days
 * @param {unknown} hours
 * @param {unknown} minutes
 */
export const durationSeconds = (days, hours, minutes) => {
  const d = parseInteger(days, 0);
  const h = parseInteger(hours, 0);
  const m = parseInteger(minutes, 0);
  return (d * 86400) + (h * 3600) + (m * 60);
};

/**
 * @param {unknown} value
 * @param {number} min
 * @param {number} max
 */
export const inRange = (value, min, max) => {
  const parsed = Number.parseFloat(String(value));
  return Number.isFinite(parsed) && parsed >= min && parsed <= max;
};

/**
 * @param {unknown} days
 * @param {unknown} hours
 * @param {unknown} minutes
 * @param {{ minSeconds: number, maxSeconds: number }} bounds
 */
export const isDurationTupleValid = (days, hours, minutes, bounds) => {
  if (!inRange(days, 0, 365)) return false;
  if (!inRange(hours, 0, 23)) return false;
  if (!inRange(minutes, 0, 59)) return false;
  const total = durationSeconds(days, hours, minutes);
  return total >= bounds.minSeconds && total <= bounds.maxSeconds;
};

/**
 * @param {unknown} value
 */
export const formatIssueReceived = (value) => {
  if (value === undefined) return '';
  if (value === null) return 'null';
  if (typeof value === 'string') return `"${value}"`;
  try {
    return JSON.stringify(value);
  } catch (_error) {
    return String(value);
  }
};
