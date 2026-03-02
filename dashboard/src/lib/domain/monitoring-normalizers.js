// @ts-check

import { toBoundedNonNegativeInteger } from './core/math.js';
import { sanitizeDisplayText } from './core/strings.js';

const DEFAULT_SAFE_COUNT_MAX = 1_000_000_000;

const toSafeCount = (value, max = DEFAULT_SAFE_COUNT_MAX) =>
  toBoundedNonNegativeInteger(value, max);

const toSafeText = (value, fallback = '-') => sanitizeDisplayText(value, fallback);

const resolveMetricLabel = (key, labelMap = null) => {
  const normalizedKey = String(key || '');
  if (labelMap && Object.prototype.hasOwnProperty.call(labelMap, normalizedKey)) {
    return String(labelMap[normalizedKey] || normalizedKey);
  }
  return normalizedKey
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase());
};

/**
 * @param {unknown} range
 */
export const shouldFetchRange = (range) => range === 'week' || range === 'month';

/**
 * @param {unknown} range
 */
export const hoursForRange = (range) => {
  if (range === 'hour') return 1;
  if (range === 'day') return 24;
  if (range === 'week') return 168;
  return 720;
};

/**
 * @param {unknown} range
 * @param {number} nowMs
 */
export const cutoffForRange = (range, nowMs) => {
  if (range === 'hour') return nowMs - (60 * 60 * 1000);
  if (range === 'day') return nowMs - (24 * 60 * 60 * 1000);
  if (range === 'week') return nowMs - (7 * 24 * 60 * 60 * 1000);
  return nowMs - (30 * 24 * 60 * 60 * 1000);
};

/**
 * @param {unknown} range
 */
export const bucketSizeForRange = (range) =>
  range === 'hour' ? 300000 : range === 'day' ? 3600000 : 86400000;

/**
 * @param {unknown} range
 * @param {number} epochMs
 */
export const formatBucketLabel = (range, epochMs) => {
  const date = new Date(epochMs);
  const timeOnly = date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
  if (range === 'hour') {
    return timeOnly;
  }
  if (range === 'day') {
    return timeOnly;
  }
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
};

/**
 * @param {unknown} rows
 * @param {Record<string, string>} labels
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizeReasonRows = (rows, labels, options = {}) => {
  const limit = Number(options.limit || 10);
  const maxCount = Number(options.maxCount || DEFAULT_SAFE_COUNT_MAX);
  if (!Array.isArray(rows)) return [];
  return rows.slice(0, limit).map(([key, value]) => ({
    key: toSafeText(key),
    label: toSafeText(resolveMetricLabel(key, labels)),
    count: toSafeCount(value, maxCount)
  }));
};

/**
 * @param {unknown} rows
 * @param {Record<string, string>} labels
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizePairRows = (rows, labels, options = {}) =>
  normalizeReasonRows(rows, labels, options);

/**
 * @param {unknown} rows
 * @param {Record<string, string> | ((value: string) => string)} mapOrFormatter
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizeDimensionRows = (rows, mapOrFormatter, options = {}) => {
  const limit = Number(options.limit || 10);
  const maxCount = Number(options.maxCount || DEFAULT_SAFE_COUNT_MAX);
  if (!Array.isArray(rows)) return [];
  return rows.slice(0, limit).map(([key, value]) => {
    const safeKey = toSafeText(key);
    const label = typeof mapOrFormatter === 'function'
      ? mapOrFormatter(safeKey)
      : resolveMetricLabel(safeKey, mapOrFormatter);
    return {
      key: safeKey,
      label: toSafeText(label, safeKey),
      count: toSafeCount(value, maxCount)
    };
  });
};

/**
 * @param {{ labels?: unknown[], data?: unknown[] } | null | undefined} trend
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizeTrendRows = (trend = {}, options = {}) => {
  const limit = Number(options.limit || 10);
  const maxCount = Number(options.maxCount || DEFAULT_SAFE_COUNT_MAX);
  const labels = Array.isArray(trend?.labels) ? trend.labels : [];
  const data = Array.isArray(trend?.data) ? trend.data : [];
  const count = Math.min(labels.length, data.length);
  const start = Math.max(0, count - limit);
  const rows = [];
  for (let index = start; index < count; index += 1) {
    rows.push({
      label: toSafeText(labels[index], '-'),
      count: toSafeCount(data[index], maxCount)
    });
  }
  return rows;
};

/**
 * @param {unknown} paths
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizeTopPaths = (paths, options = {}) => {
  const limit = Number(options.limit || 10);
  const maxCount = Number(options.maxCount || DEFAULT_SAFE_COUNT_MAX);
  if (!Array.isArray(paths)) return [];
  return paths.slice(0, limit).map((entry) => ({
    path: toSafeText(entry.path, '-'),
    count: toSafeCount(entry.count, maxCount)
  }));
};

/**
 * @param {unknown} rows
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizeTopCountries = (rows, options = {}) => {
  const limit = Number(options.limit || 10);
  const maxCount = Number(options.maxCount || DEFAULT_SAFE_COUNT_MAX);
  if (!Array.isArray(rows)) return [];
  return rows.slice(0, limit).map((entry) => ({
    country: toSafeText(entry.country, '-'),
    count: toSafeCount(entry.count, maxCount)
  }));
};

/**
 * @param {{ labels?: unknown[], data?: unknown[] } | null | undefined} series
 * @param {{ limit?: number, maxCount?: number }} options
 */
export const normalizeTrendSeries = (series, options = {}) => {
  const limit = Number(options.limit || 720);
  const maxCount = Number(options.maxCount || DEFAULT_SAFE_COUNT_MAX);
  const labels = Array.isArray(series?.labels) ? series.labels : [];
  const data = Array.isArray(series?.data) ? series.data : [];
  const pointCount = Math.min(labels.length, data.length);
  const start = Math.max(0, pointCount - limit);
  const nextLabels = [];
  const nextData = [];
  for (let index = start; index < pointCount; index += 1) {
    nextLabels.push(toSafeText(labels[index], '-'));
    nextData.push(toSafeCount(data[index], maxCount));
  }
  return {
    labels: nextLabels,
    data: nextData
  };
};

/**
 * @param {unknown} events
 * @param {unknown} range
 * @param {{ nowMs?: number, maxEvents?: number }} options
 */
export const buildTimeSeries = (events, range, options = {}) => {
  const nowMs = Number(options.nowMs || Date.now());
  const maxEvents = Number(options.maxEvents || 5000);
  const cutoffTime = cutoffForRange(range, nowMs);
  const bucketSize = bucketSizeForRange(range);
  const normalized = Array.isArray(events) ? events : [];
  const filteredEvents = normalized.filter((entry) => (Number(entry?.ts || 0) * 1000) >= cutoffTime);
  const boundedEvents = filteredEvents.length > maxEvents
    ? filteredEvents.slice(0, maxEvents)
    : filteredEvents;
  const buckets = {};
  for (let time = cutoffTime; time <= nowMs; time += bucketSize) {
    const bucketKey = Math.floor(time / bucketSize) * bucketSize;
    buckets[bucketKey] = 0;
  }
  boundedEvents.forEach((entry) => {
    const eventTime = Number(entry?.ts || 0) * 1000;
    if (!Number.isFinite(eventTime) || eventTime <= 0) return;
    const bucketKey = Math.floor(eventTime / bucketSize) * bucketSize;
    buckets[bucketKey] = Number(buckets[bucketKey] || 0) + 1;
  });
  const sortedBuckets = Object.keys(buckets)
    .map((key) => Number.parseInt(key, 10))
    .sort((left, right) => left - right);
  return {
    labels: sortedBuckets.map((epochMs) => formatBucketLabel(range, epochMs)),
    data: sortedBuckets.map((epochMs) => Number(buckets[epochMs] || 0))
  };
};
