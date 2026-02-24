// @ts-check

import { durationPartsFromSeconds, durationSeconds } from './core/date-time.js';
import { parseFloatNumber, parseInteger } from './core/math.js';
import { formatUnknownForDisplay, normalizeLowerTrimmed } from './core/strings.js';
import { inRange, isDurationTupleValid, isNormalizedInSet } from './core/validation.js';

const EDGE_MODES = new Set(['off', 'advisory', 'authoritative']);
const COMPOSABILITY_MODES = new Set(['off', 'signal', 'enforce', 'both']);
const IP_RANGE_POLICY_MODES = new Set(['off', 'advisory', 'enforce']);

/**
 * @param {unknown} value
 */
export const normalizeEdgeMode = (value) => {
  const normalized = normalizeLowerTrimmed(value);
  return isNormalizedInSet(normalized, EDGE_MODES) ? normalized : 'off';
};

/**
 * @param {unknown} value
 */
export const normalizeIpRangePolicyMode = (value) => {
  const normalized = normalizeLowerTrimmed(value);
  return isNormalizedInSet(normalized, IP_RANGE_POLICY_MODES) ? normalized : 'off';
};

/**
 * @param {unknown} value
 */
export const isIpRangePolicyMode = (value) => isNormalizedInSet(value, IP_RANGE_POLICY_MODES);

/**
 * @param {unknown} value
 */
export const normalizeComposabilityMode = (value) => {
  const normalized = normalizeLowerTrimmed(value);
  return isNormalizedInSet(normalized, COMPOSABILITY_MODES) ? normalized : 'off';
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

export const formatIssueReceived = formatUnknownForDisplay;

export {
  durationPartsFromSeconds,
  durationSeconds,
  inRange,
  isDurationTupleValid,
  parseFloatNumber,
  parseInteger
};
