// @ts-check

import { durationSeconds } from './date-time.js';
import { normalizeLowerTrimmed } from './strings.js';

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
 * @param {unknown} value
 * @param {Set<string>} allowedValues
 */
export const isNormalizedInSet = (value, allowedValues) => {
  const normalized = normalizeLowerTrimmed(value);
  return allowedValues.has(normalized);
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
