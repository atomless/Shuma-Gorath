// @ts-check

import { formatUnixSecondsLocal } from './core/date-time.js';

export const FRESHNESS_STATE_LABELS = Object.freeze({
  fresh: 'Fresh',
  degraded: 'Degraded',
  stale: 'Stale',
  unknown: 'Unknown'
});

const TRANSPORT_LABELS = Object.freeze({
  polling: 'Polling',
  snapshot_poll: 'Snapshot Poll',
  cursor_delta_poll: 'Incremental Poll',
  sse: 'Live Stream'
});

const normalizeLowerTrimmed = (value) => String(value || '').trim().toLowerCase();

export const normalizeFreshnessStateKey = (value) => {
  const normalized = normalizeLowerTrimmed(value);
  if (normalized === 'fresh' || normalized === 'degraded' || normalized === 'stale') {
    return normalized;
  }
  return 'unknown';
};

export const normalizeFreshnessTransport = (value) => {
  const normalized = normalizeLowerTrimmed(value);
  return normalized || 'polling';
};

export const deriveFreshnessSummary = (snapshot = {}, options = {}) => {
  const source = snapshot && typeof snapshot === 'object' ? snapshot : {};
  const formatTimestamp = typeof options.formatTimestamp === 'function'
    ? options.formatTimestamp
    : (value) => formatUnixSecondsLocal(value, '-');
  const stateKey = normalizeFreshnessStateKey(source.state);
  const lagMs = Number(source.lag_ms);
  const lastEventTs = Number(source.last_event_ts);
  const transportCode = normalizeFreshnessTransport(source.transport);
  const slowConsumerState = normalizeLowerTrimmed(source.slow_consumer_lag_state) || 'normal';
  const overflow = normalizeLowerTrimmed(source.overflow) || 'none';
  const lagText = Number.isFinite(lagMs) && lagMs >= 0 ? `${Math.round(lagMs)} ms` : 'n/a';
  const lastEventText = Number.isFinite(lastEventTs) && lastEventTs > 0
    ? formatTimestamp(lastEventTs)
    : 'n/a';
  const transportLabel = TRANSPORT_LABELS[transportCode] || transportCode || TRANSPORT_LABELS.polling;
  const partialDataWarning = overflow === 'limit_exceeded'
    ? 'Recent view may be partial because the bounded window reached its event limit.'
    : slowConsumerState === 'lagged'
      ? 'Recent view may lag behind live activity.'
      : '';

  return {
    stateKey,
    stateLabel: FRESHNESS_STATE_LABELS[stateKey] || FRESHNESS_STATE_LABELS.unknown,
    lagMs,
    lagText,
    lastEventTs,
    lastEventText,
    transportCode,
    transportLabel,
    slowConsumerState,
    overflow,
    partialDataWarning
  };
};
