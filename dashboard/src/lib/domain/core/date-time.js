// @ts-check

import { parseInteger } from './math.js';

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
 * Normalize a configured duration to the UI granularity and then expose exact day/hour/minute parts.
 * Unlike `durationPartsFromSeconds`, a real `0` value stays `0` instead of falling back.
 *
 * @param {unknown} seconds
 * @param {number} fallbackSeconds
 */
export const durationPartsFromConfiguredSeconds = (seconds, fallbackSeconds) => {
  const safe = normalizeDurationSecondsToMinuteUiGranularity(seconds, fallbackSeconds);
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
 * Normalize a duration to the same day/hour/minute granularity the dashboard form can represent.
 *
 * @param {unknown} seconds
 * @param {number} fallbackSeconds
 */
export const normalizeDurationSecondsToMinuteUiGranularity = (seconds, fallbackSeconds) => {
  const source = Number.parseInt(String(seconds), 10);
  if (Number.isFinite(source) && source >= 0) {
    return Math.floor(source / 60) * 60;
  }
  const fallback = Number.parseInt(String(fallbackSeconds), 10);
  const safeFallback = Number.isFinite(fallback) && fallback > 0 ? fallback : 0;
  return Math.floor(safeFallback / 60) * 60;
};

/**
 * @param {unknown} epochSeconds
 * @param {string} fallback
 */
export const formatUnixSecondsLocal = (epochSeconds, fallback = '-') => {
  const parsed = Number(epochSeconds || 0);
  if (!Number.isFinite(parsed) || parsed <= 0) return fallback;
  return new Date(parsed * 1000).toLocaleString();
};
