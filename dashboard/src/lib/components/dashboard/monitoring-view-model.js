import { formatCompactNumber } from '../../domain/core/format.js';
import {
  classifyIpRangeFallback,
  formatIpRangeReasonLabel,
  isIpRangeReason,
  parseIpRangeOutcome
} from '../../domain/ip-range-policy.js';

const CHALLENGE_REASON_LABELS = Object.freeze({
  incorrect: 'Incorrect',
  expired_replay: 'Expired/Replay',
  sequence_violation: 'Sequence Violation',
  invalid_output: 'Invalid Output',
  forbidden: 'Forbidden'
});

const POW_REASON_LABELS = Object.freeze({
  invalid_proof: 'Invalid Proof',
  missing_seed_nonce: 'Missing Seed/Number Used Once',
  sequence_violation: 'Sequence Violation',
  expired_replay: 'Expired/Replay',
  binding_timing_mismatch: 'Binding/Timing Mismatch'
});

const RATE_OUTCOME_LABELS = Object.freeze({
  limited: 'Limited',
  banned: 'Banned',
  fallback_allow: 'Fallback Allow',
  fallback_deny: 'Fallback Deny'
});

const NOT_A_BOT_OUTCOME_LABELS = Object.freeze({
  pass: 'Pass',
  escalate: 'Escalate',
  fail: 'Fail',
  replay: 'Replay'
});

const NOT_A_BOT_LATENCY_LABELS = Object.freeze({
  lt_1s: '<1s',
  '1_3s': '1-3s',
  '3_10s': '3-10s',
  '10s_plus': '10s+'
});

const IP_RANGE_SOURCE_LABELS = Object.freeze({
  custom: 'Custom Rule',
  managed: 'Managed Set',
  unknown: 'Unknown'
});

const IP_RANGE_ACTION_LABELS = Object.freeze({
  forbidden_403: '403 Forbidden',
  custom_message: 'Custom Message',
  drop_connection: 'Drop Connection',
  redirect_308: '308 Redirect',
  rate_limit: 'Rate Limit',
  honeypot: 'Honeypot',
  maze: 'Maze',
  tarpit: 'Tarpit',
  unknown: 'Unknown'
});

const IP_RANGE_FALLBACK_LABELS = Object.freeze({
  none: 'Direct',
  challenge: 'Fallback Puzzle',
  maze: 'Fallback Maze',
  block: 'Fallback Block',
  block_missing_redirect: 'Block (Missing Redirect URL)'
});

const toNonNegativeNumber = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric < 0) return 0;
  return numeric;
};

const normalizeOffenderBucketLabel = (rawLabel) => {
  const label = String(rawLabel || '').trim();
  if (!label) return 'untrusted/unknown';
  if (label.toLowerCase() === 'unknown') return 'untrusted/unknown';
  if (/^h\d+$/i.test(label)) return 'untrusted/unknown';
  return label;
};

const formatUnitLabel = (count, singular, plural) => (count === 1 ? singular : plural);

const deriveTopOffenderViewModel = (rawLabel, rawCount, singularUnit, pluralUnit) => {
  const label = String(rawLabel || '').trim();
  const count = Number(rawCount || 0);
  if (!label || !Number.isFinite(count) || count <= 0) {
    return {
      value: 'None',
      label: 'Top Offender'
    };
  }
  const normalizedLabel = normalizeOffenderBucketLabel(label);
  const unit = formatUnitLabel(count, singularUnit, pluralUnit);
  return {
    value: normalizedLabel,
    label: `Top Offender (${formatCompactNumber(count, '0')} ${unit})`
  };
};

const formatTrendTimestamp = (ts) => {
  if (!Number.isFinite(ts)) return '-';
  return new Date(ts * 1000).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric'
  });
};

const sortCountEntries = (source) =>
  Object.entries(source && typeof source === 'object' ? source : {})
    .sort((left, right) => Number(right[1] || 0) - Number(left[1] || 0));

const sortCountEntryPairs = (source, limit = 10) =>
  Object.entries(source && typeof source === 'object' ? source : {})
    .sort((left, right) =>
      Number(right[1] || 0) - Number(left[1] || 0) || String(left[0] || '').localeCompare(String(right[0] || ''))
    )
    .slice(0, Math.max(0, Number(limit || 0)));

const deriveTrendSeries = (trend = []) => {
  const points = Array.isArray(trend) ? trend : [];
  return {
    labels: points.map((point) => formatTrendTimestamp(Number(point.ts || 0))),
    data: points.map((point) => Number(point.total || 0))
  };
};

const normalizeToken = (value) => String(value || '').trim().toLowerCase();

const EVENT_ORIGIN_LABELS = Object.freeze({
  sim: 'Simulation',
  manual: 'Manual',
  other: 'Other'
});

const EXECUTION_MODE_LABELS = Object.freeze({
  enforced: 'Enforced',
  shadow: 'Shadow'
});

const SHADOW_ACTION_LABELS = Object.freeze({
  not_a_bot: 'Would Not-a-Bot',
  challenge: 'Would Challenge',
  js_challenge: 'Would JS Challenge',
  maze: 'Would Maze',
  block: 'Would Block',
  tarpit: 'Would Tarpit',
  redirect: 'Would Redirect',
  drop_connection: 'Would Drop Connection'
});

const DEFENSE_LABELS = Object.freeze({
  challenge: 'Puzzle',
  not_a_bot: 'Not-a-Bot',
  pow: 'Proof of Work',
  rate_limit: 'Rate Limiting',
  geo: 'GEO',
  maze: 'Maze',
  tarpit: 'Tarpit',
  honeypot: 'Honeypot',
  cdp: 'CDP',
  fingerprint: 'Fingerprint',
  ban_path: 'Ban Path',
  event_stream: 'Event Stream',
  other: 'Other'
});

const OUTCOME_BUCKET_LABELS = Object.freeze({
  pass: 'Pass',
  fail: 'Fail',
  escalate: 'Escalate',
  unknown: 'Unknown'
});

const EVENT_DISPLAY_LABELS = Object.freeze({
  challenge: 'Puzzle',
  not_a_bot: 'Not-a-Bot',
  pow: 'Proof of Work',
  rate_limit: 'Rate Limit'
});

const OUTCOME_CODE_LABELS = Object.freeze({
  served: 'Served',
  required: 'Required',
  fallback_maze: 'Fallback Maze',
  fallback_block: 'Fallback Block'
});

const OUTCOME_BUCKET_SUPPORTED_DEFENSES = new Set([
  'challenge',
  'not_a_bot',
  'pow',
  'rate_limit',
  'geo'
]);

const classifyEventOrigin = (event) => {
  const row = event && typeof event === 'object' ? event : {};
  if (row.is_simulation === true || String(row.sim_run_id || '').trim().length > 0) {
    return 'sim';
  }
  if (String(row.admin || '').trim().length > 0) {
    return 'manual';
  }
  return 'other';
};

const classifyExecutionMode = (event) => {
  const mode = normalizeToken(event?.execution_mode);
  if (mode === 'shadow') return 'shadow';
  if (event?.enforcement_applied === false) return 'shadow';
  return 'enforced';
};

export const deriveEnforcedMonitoringChartRows = (events = [], options = {}) => {
  const source = Array.isArray(events) ? events : [];
  const topIpLimit = Math.max(1, Number(options.topIpLimit || 10));
  const enforcedEvents = [];
  const eventCounts = {};
  const ipCounts = {};

  for (const event of source) {
    if (classifyExecutionMode(event) === 'shadow') continue;
    enforcedEvents.push(event);

    const eventName = String(event?.event || '').trim();
    if (eventName) {
      eventCounts[eventName] = Number(eventCounts[eventName] || 0) + 1;
    }

    const ip = String(event?.ip || '').trim();
    if (ip) {
      ipCounts[ip] = Number(ipCounts[ip] || 0) + 1;
    }
  }

  return {
    events: enforcedEvents,
    eventCounts,
    topIps: sortCountEntryPairs(ipCounts, topIpLimit)
  };
};

const classifyShadowAction = (event) => normalizeToken(event?.intended_action);

const classificationText = (event = {}) => {
  const eventType = normalizeToken(event?.event);
  const reason = normalizeToken(event?.reason);
  const outcome = normalizeToken(event?.outcome);
  const outcomeCode = normalizeToken(event?.outcome_code);
  return `${eventType} ${reason} ${outcomeCode} ${outcome}`;
};

const classifyChallengeDisplayLabel = (event = {}) => {
  const combined = classificationText(event);
  if (combined.includes('not_a_bot') || combined.includes('not-a-bot')) {
    return 'Not-a-Bot';
  }
  if (combined.includes('js_verification') || combined.includes('js_challenge')) {
    return 'JS Challenge';
  }
  if (combined.includes('maze')) {
    return 'Maze';
  }
  const eventType = normalizeToken(event?.event);
  if (EVENT_DISPLAY_LABELS[eventType]) {
    return EVENT_DISPLAY_LABELS[eventType];
  }
  return formatMetricLabel(String(event?.event || '-'));
};

const normalizeReasonForDisplay = (reason) =>
  String(reason || '')
    .replace(/\s+/g, ' ')
    .trim();

const classifyEventLane = (event) => {
  const lane = normalizeToken(event?.sim_lane);
  return lane || 'none';
};

const classifyEventScenario = (event) => {
  const fromField = String(event?.scenario_id || '').trim();
  if (fromField) return fromField;
  const reason = String(event?.reason || '');
  const scenarioMatch = /(?:scenario_id|scenario)=([a-zA-Z0-9_.:-]+)/i.exec(reason);
  if (scenarioMatch && scenarioMatch[1]) {
    return scenarioMatch[1];
  }
  const profile = String(event?.sim_profile || '').trim();
  if (profile) {
    return profile;
  }
  return 'unknown';
};

const classifyDefense = (event) => {
  const eventType = normalizeToken(event?.event);
  const reason = normalizeToken(event?.reason);
  const outcome = normalizeToken(event?.outcome);
  const outcomeCode = normalizeToken(event?.outcome_code);
  const combined = `${eventType} ${reason} ${outcomeCode} ${outcome}`;
  if (combined.includes('honeypot')) return 'honeypot';
  if (combined.includes('tarpit')) return 'tarpit';
  if (combined.includes('maze')) return 'maze';
  if (combined.includes('not_a_bot') || combined.includes('not-a-bot')) return 'not_a_bot';
  if (combined.includes('pow') || combined.includes('proof')) return 'pow';
  if (combined.includes('rate')) return 'rate_limit';
  if (combined.includes('geo')) return 'geo';
  if (combined.includes('cdp')) return 'cdp';
  if (combined.includes('fingerprint')) return 'fingerprint';
  if (combined.includes('challenge')) return 'challenge';
  if (combined.includes('ban') || combined.includes('deny_temp') || combined.includes('block')) return 'ban_path';
  if (String(event?.sim_run_id || '').trim().length > 0) return 'event_stream';
  return 'other';
};

const deriveDisplayedOutcome = (event) => {
  if (classifyExecutionMode(event) === 'shadow') {
    const action = classifyShadowAction(event);
    if (action && SHADOW_ACTION_LABELS[action]) {
      return {
        token: `would_${action}`,
        label: SHADOW_ACTION_LABELS[action]
      };
    }
    return {
      token: 'would_act',
      label: 'Would Act'
    };
  }

  const outcomeCode = normalizeToken(event?.outcome_code);
  if (outcomeCode) {
    return {
      token: outcomeCode,
      label: OUTCOME_CODE_LABELS[outcomeCode] || formatMetricLabel(outcomeCode)
    };
  }

  const token = normalizeToken(event?.outcome) || 'unknown';
  return {
    token,
    label: formatMetricLabel(token)
  };
};

const classifyShadowOutcomeBucket = (event) => {
  const action = classifyShadowAction(event);
  if (!action) return 'unknown';
  if (action === 'block' || action === 'tarpit' || action === 'drop_connection') {
    return 'fail';
  }
  if (
    action === 'challenge'
    || action === 'js_challenge'
    || action === 'not_a_bot'
    || action === 'maze'
    || action === 'redirect'
  ) {
    return 'escalate';
  }
  return 'unknown';
};

const classifyOutcomeBucket = (outcomeRaw) => {
  const outcome = normalizeToken(outcomeRaw);
  if (outcome.includes('success')) return 'pass';
  if (outcome.includes('failure')) return 'fail';
  if (outcome.includes('replay')) return 'fail';
  if (outcome.includes('allow') || outcome.includes('monitor') || outcome.includes('not-a-bot') || outcome === 'pass') {
    return 'pass';
  }
  if (outcome.includes('challenge') || outcome.includes('maze') || outcome.includes('tarpit') || outcome.includes('escalate')) {
    return 'escalate';
  }
  if (outcome.includes('deny') || outcome.includes('block') || outcome.includes('fail') || outcome.includes('ban')) {
    return 'fail';
  }
  return 'unknown';
};

const defenseSupportsOutcomeBuckets = (defense) =>
  OUTCOME_BUCKET_SUPPORTED_DEFENSES.has(String(defense || '').trim());

const classifyOutcomeBucketForDefense = (defense, event) => {
  if (!defenseSupportsOutcomeBuckets(defense)) {
    return 'unknown';
  }
  if (classifyExecutionMode(event) === 'shadow') {
    return classifyShadowOutcomeBucket(event);
  }
  const outcome = normalizeToken(event?.outcome_code || event?.outcome);
  const reason = normalizeToken(event?.reason);
  const combined = `${outcome} ${reason}`;
  if (defense === 'challenge') {
    if (
      combined.includes('incorrect')
      || combined.includes('expired')
      || combined.includes('forbidden')
      || combined.includes('invalid')
      || combined.includes('fail')
    ) {
      return 'fail';
    }
    if (combined.includes('solved') || combined.includes('pass')) {
      return 'pass';
    }
    if (combined.includes('challenge') || combined.includes('maze') || combined.includes('tarpit')) {
      return 'escalate';
    }
    return 'unknown';
  }
  if (defense === 'not_a_bot') {
    if (combined.includes('pass')) return 'pass';
    if (combined.includes('escalate')) return 'escalate';
    if (combined.includes('fail') || combined.includes('replay')) return 'fail';
    return 'unknown';
  }
  if (defense === 'rate_limit') {
    if (combined.includes('fallback_allow') || combined.includes('allow')) return 'pass';
    if (combined.includes('limited') || combined.includes('challenge') || combined.includes('maze')) {
      return 'escalate';
    }
    if (combined.includes('banned') || combined.includes('deny') || combined.includes('block')) {
      return 'fail';
    }
    return 'unknown';
  }
  if (defense === 'geo') {
    if (combined.includes('challenge') || combined.includes('maze')) return 'escalate';
    if (combined.includes('block') || combined.includes('deny') || combined.includes('ban')) return 'fail';
    if (combined.includes('allow') || combined.includes('monitor')) return 'pass';
    return 'unknown';
  }
  return classifyOutcomeBucket(combined);
};

const isBanOutcomeEvent = (event) => {
  if (classifyExecutionMode(event) === 'shadow') return false;
  const eventType = normalizeToken(event?.event);
  if (eventType === 'ban') return true;
  const outcome = normalizeToken(event?.outcome_code || event?.outcome);
  const reason = normalizeToken(event?.reason);
  return (
    outcome.includes('deny')
    || outcome.includes('block')
    || outcome.includes('banned')
    || reason.includes('ban')
  );
};

const toOptionRows = (values, labels = null) => values
  .filter((value) => String(value || '').trim().length > 0)
  .sort((left, right) => String(left).localeCompare(String(right)))
  .map((value) => {
    const normalized = String(value);
    return {
      value: normalized,
      label: labels && labels[normalized]
        ? labels[normalized]
        : normalized
    };
  });

export const deriveRecentEventFilterOptions = (events = []) => {
  const rows = Array.isArray(events) ? events : [];
  const origins = new Set();
  const modes = new Set();
  const scenarios = new Set();
  const lanes = new Set();
  const defenses = new Set();
  const outcomes = new Map();
  rows.forEach((event) => {
    origins.add(classifyEventOrigin(event));
    modes.add(classifyExecutionMode(event));
    scenarios.add(classifyEventScenario(event));
    lanes.add(classifyEventLane(event));
    defenses.add(classifyDefense(event));
    const outcome = deriveDisplayedOutcome(event);
    outcomes.set(outcome.token, outcome.label);
  });
  return {
    origins: toOptionRows(Array.from(origins), EVENT_ORIGIN_LABELS),
    modes: toOptionRows(Array.from(modes), EXECUTION_MODE_LABELS),
    scenarios: toOptionRows(Array.from(scenarios)),
    lanes: toOptionRows(Array.from(lanes)),
    defenses: toOptionRows(Array.from(defenses), DEFENSE_LABELS),
    outcomes: Array.from(outcomes.entries())
      .sort((left, right) => String(left[1] || '').localeCompare(String(right[1] || '')))
      .map(([value, label]) => ({ value, label }))
  };
};

export const filterRecentEvents = (events = [], filters = {}) => {
  const rows = Array.isArray(events) ? events : [];
  const sourceFilter = normalizeToken(filters.origin);
  const modeFilter = normalizeToken(filters.mode);
  const scenarioFilter = normalizeToken(filters.scenario);
  const laneFilter = normalizeToken(filters.lane);
  const defenseFilter = normalizeToken(filters.defense);
  const outcomeFilter = normalizeToken(filters.outcome);
  return rows.filter((event) => {
    if (sourceFilter && sourceFilter !== 'all' && classifyEventOrigin(event) !== sourceFilter) return false;
    if (modeFilter && modeFilter !== 'all' && classifyExecutionMode(event) !== modeFilter) return false;
    if (scenarioFilter && scenarioFilter !== 'all' && normalizeToken(classifyEventScenario(event)) !== scenarioFilter) return false;
    if (laneFilter && laneFilter !== 'all' && classifyEventLane(event) !== laneFilter) return false;
    if (defenseFilter && defenseFilter !== 'all' && classifyDefense(event) !== defenseFilter) return false;
    if (outcomeFilter && outcomeFilter !== 'all' && deriveDisplayedOutcome(event).token !== outcomeFilter) return false;
    return true;
  });
};

export const deriveDefenseTrendRows = (events = []) => {
  const rows = Array.isArray(events) ? events : [];
  const grouped = createDefenseTrendAccumulator();
  rows.forEach((event) => {
    appendDefenseTrendEvent(grouped, event);
  });
  return deriveDefenseTrendRowsFromAccumulator(grouped);
};

export const createDefenseTrendAccumulator = () =>
  new Map();

export const appendDefenseTrendEvent = (accumulator, event) => {
  const grouped = accumulator instanceof Map ? accumulator : createDefenseTrendAccumulator();
  const defense = classifyDefense(event);
  const source = classifyEventOrigin(event);
  const executionMode = classifyExecutionMode(event);
  const existing = grouped.get(defense) || {
    defense,
    triggerCount: 0,
    passCount: 0,
    failCount: 0,
    escalationCount: 0,
    banOutcomeCount: 0,
    executionCounts: {},
    sourceCounts: {}
  };
  existing.triggerCount += 1;
  const outcomeBucket = classifyOutcomeBucketForDefense(defense, event);
  if (outcomeBucket === 'pass') existing.passCount += 1;
  if (outcomeBucket === 'fail') existing.failCount += 1;
  if (outcomeBucket === 'escalate') existing.escalationCount += 1;
  if (executionMode === 'enforced' && isBanOutcomeEvent(event)) existing.banOutcomeCount += 1;
  existing.executionCounts[executionMode] = Number(existing.executionCounts[executionMode] || 0) + 1;
  existing.sourceCounts[source] = Number(existing.sourceCounts[source] || 0) + 1;
  grouped.set(defense, existing);
  return grouped;
};

export const deriveDefenseTrendRowsFromAccumulator = (accumulator) => {
  const grouped = accumulator instanceof Map ? accumulator : createDefenseTrendAccumulator();
  return Array.from(grouped.values())
    .sort((left, right) => right.triggerCount - left.triggerCount)
    .map((row) => ({
      defense: row.defense,
      label: DEFENSE_LABELS[row.defense] || row.defense,
      hasOutcomeBreakdown: defenseSupportsOutcomeBuckets(row.defense),
      triggerCount: row.triggerCount,
      passCount: row.passCount,
      failCount: row.failCount,
      escalationCount: row.escalationCount,
      banOutcomeCount: row.banOutcomeCount,
      modeRows: toOptionRows(Object.keys(row.executionCounts), EXECUTION_MODE_LABELS).map((entry) => ({
        mode: entry.value,
        label: entry.label,
        count: row.executionCounts[entry.value] || 0
      })),
      sourceRows: toOptionRows(Object.keys(row.sourceCounts), EVENT_ORIGIN_LABELS).map((entry) => ({
        source: entry.value,
        label: entry.label,
        count: row.sourceCounts[entry.value] || 0
      }))
    }));
};

const asDisplayValue = (value, fallback = '0') => {
  if (value === null || value === undefined || value === '') return fallback;
  if (typeof value === 'string') {
    const trimmed = value.trim();
    return trimmed || fallback;
  }
  const numeric = Number(value);
  if (Number.isFinite(numeric)) return formatCompactNumber(numeric, fallback);
  return String(value);
};

const asTopOffenderValue = (value) => {
  const normalized = String(value || '').trim();
  return normalized || 'None';
};

const createFactRow = (label, value) => ({
  label,
  value: asDisplayValue(value, 'None')
});

const asSummaryFact = (summary = {}, fallbackLabel = 'Top Offender') => {
  const value = String(summary?.value || '').trim();
  if (!value) return null;
  const label = String(summary?.label || '').trim() || fallbackLabel;
  return createFactRow(label, value);
};

const asTopCountEntryFact = (entries = [], label, fallbackMap = null) => {
  const firstEntry = Array.isArray(entries) && entries.length > 0 ? entries[0] : null;
  const key = Array.isArray(firstEntry) ? String(firstEntry[0] || '').trim() : '';
  const count = Array.isArray(firstEntry) ? Number(firstEntry[1] || 0) : 0;
  if (!key || !Number.isFinite(count) || count <= 0) return null;
  return createFactRow(
    `${label} (${formatCompactNumber(count, '0')})`,
    formatMetricLabel(key, fallbackMap)
  );
};

const asTopCountryFact = (rows = []) => {
  const firstRow = Array.isArray(rows) && rows.length > 0 ? rows[0] : null;
  const country = String(firstRow?.country || '').trim();
  const count = Number(firstRow?.count || 0);
  if (!country || !Number.isFinite(count) || count <= 0) return null;
  return createFactRow(`Top Country (${formatCompactNumber(count, '0')})`, country);
};

const asTopPathFact = (rows = []) => {
  const firstRow = Array.isArray(rows) && rows.length > 0 ? rows[0] : null;
  const path = String(firstRow?.path || '').trim();
  const count = Number(firstRow?.count || 0);
  if (!path || !Number.isFinite(count) || count <= 0) return null;
  return createFactRow(`Top Path (${formatCompactNumber(count, '0')})`, path);
};

const asCountMixFact = (rows = [], label) => {
  const normalizedRows = Array.isArray(rows) ? rows : [];
  if (normalizedRows.length === 0) return null;
  const value = normalizedRows
    .filter((row) => String(row?.label || '').trim().length > 0)
    .map((row) => `${row.label}: ${formatCompactNumber(row?.count || 0, '0')}`)
    .join(', ');
  if (!value) return null;
  return createFactRow(label, value);
};

const deriveTrendContextFacts = (trendRows = [], defense) => {
  const trendRow = (Array.isArray(trendRows) ? trendRows : []).find((row) => row?.defense === defense);
  if (!trendRow) return [];
  return [
    createFactRow('Recent Triggers', trendRow.triggerCount),
    trendRow.hasOutcomeBreakdown ? createFactRow('Recent Pass', trendRow.passCount) : null,
    trendRow.hasOutcomeBreakdown ? createFactRow('Recent Fail', trendRow.failCount) : null,
    trendRow.hasOutcomeBreakdown ? createFactRow('Recent Escalate', trendRow.escalationCount) : null,
    createFactRow('Recent Ban Outcomes', trendRow.banOutcomeCount),
    asCountMixFact(trendRow.modeRows, 'Recent Modes'),
    asCountMixFact(trendRow.sourceRows, 'Recent Sources')
  ].filter(Boolean);
};

const createDefenseBreakdownRow = (defense, label, factRows) => ({
  defense,
  label,
  factRows: (Array.isArray(factRows) ? factRows : []).filter((fact) =>
    fact
    && String(fact.label || '').trim().length > 0
    && String(fact.value || '').trim().length > 0
  )
});

export const deriveDefenseBreakdownRows = ({
  trendRows = [],
  cdpDetections = 0,
  cdpAutoBans = 0,
  cdpUaClientHintMismatch = 0,
  cdpUaTransportMismatch = 0,
  cdpFlowViolations = 0,
  mazeStats = {},
  tarpitSummary = {},
  monitoringSummary = {},
  ipRangeSummary = {}
} = {}) => [
  createDefenseBreakdownRow('cdp', DEFENSE_LABELS.cdp, [
    createFactRow('Detections', cdpDetections),
    createFactRow('Auto Bans', cdpAutoBans),
    createFactRow('UA/Hint Mismatch', cdpUaClientHintMismatch),
    createFactRow('UA/Transport Mismatch', cdpUaTransportMismatch),
    createFactRow('Flow Violations', cdpFlowViolations),
    ...deriveTrendContextFacts(trendRows, 'cdp')
  ]),
  createDefenseBreakdownRow('maze', DEFENSE_LABELS.maze, [
    createFactRow('Hits', mazeStats?.totalHits),
    createFactRow('Unique Crawlers', mazeStats?.uniqueCrawlers),
    createFactRow('Auto Bans', mazeStats?.mazeAutoBans),
    asSummaryFact(mazeStats?.topOffender),
    ...deriveTrendContextFacts(trendRows, 'maze')
  ]),
  createDefenseBreakdownRow('tarpit', DEFENSE_LABELS.tarpit, [
    createFactRow('Activations', tarpitSummary?.activationsProgressive),
    createFactRow('Progress Advanced', tarpitSummary?.progressAdvanced),
    createFactRow('Fallback Maze', tarpitSummary?.fallbackMaze),
    createFactRow('Fallback Block', tarpitSummary?.fallbackBlock),
    createFactRow('Escalation Short Ban', tarpitSummary?.escalationShortBan),
    createFactRow('Escalation Block', tarpitSummary?.escalationBlock),
    ...deriveTrendContextFacts(trendRows, 'tarpit')
  ]),
  createDefenseBreakdownRow('honeypot', DEFENSE_LABELS.honeypot, [
    createFactRow('Hits', monitoringSummary?.honeypot?.totalHits),
    createFactRow('Unique Crawlers', monitoringSummary?.honeypot?.uniqueCrawlers),
    asSummaryFact(monitoringSummary?.honeypot?.topOffender),
    asTopPathFact(monitoringSummary?.honeypot?.topPaths),
    ...deriveTrendContextFacts(trendRows, 'honeypot')
  ]),
  createDefenseBreakdownRow('challenge', DEFENSE_LABELS.challenge, [
    createFactRow('Failures', monitoringSummary?.challenge?.totalFailures),
    createFactRow('Unique Offenders', monitoringSummary?.challenge?.uniqueOffenders),
    asSummaryFact(monitoringSummary?.challenge?.topOffender),
    asTopCountEntryFact(monitoringSummary?.challenge?.reasons, 'Top Reason', CHALLENGE_REASON_LABELS),
    ...deriveTrendContextFacts(trendRows, 'challenge')
  ]),
  createDefenseBreakdownRow('not_a_bot', DEFENSE_LABELS.not_a_bot, [
    createFactRow('Served', monitoringSummary?.notABot?.served),
    createFactRow('Submitted', monitoringSummary?.notABot?.submitted),
    createFactRow('Pass', monitoringSummary?.notABot?.pass),
    createFactRow('Escalate', monitoringSummary?.notABot?.escalate),
    createFactRow('Fail', monitoringSummary?.notABot?.fail),
    createFactRow('Abandonment Rate', monitoringSummary?.notABot?.abandonmentRate),
    ...deriveTrendContextFacts(trendRows, 'not_a_bot')
  ]),
  createDefenseBreakdownRow('pow', DEFENSE_LABELS.pow, [
    createFactRow('Attempts', monitoringSummary?.pow?.totalAttempts),
    createFactRow('Successes', monitoringSummary?.pow?.totalSuccesses),
    createFactRow('Failures', monitoringSummary?.pow?.totalFailures),
    createFactRow('Success Rate', monitoringSummary?.pow?.successRate),
    createFactRow('Unique Offenders', monitoringSummary?.pow?.uniqueOffenders),
    asSummaryFact(monitoringSummary?.pow?.topOffender),
    ...deriveTrendContextFacts(trendRows, 'pow')
  ]),
  createDefenseBreakdownRow('rate_limit', DEFENSE_LABELS.rate_limit, [
    createFactRow('Violations', monitoringSummary?.rate?.totalViolations),
    createFactRow('Unique Offenders', monitoringSummary?.rate?.uniqueOffenders),
    asSummaryFact(monitoringSummary?.rate?.topOffender),
    asTopCountEntryFact(monitoringSummary?.rate?.outcomes, 'Top Outcome', RATE_OUTCOME_LABELS),
    ...deriveTrendContextFacts(trendRows, 'rate_limit')
  ]),
  createDefenseBreakdownRow('geo', DEFENSE_LABELS.geo, [
    createFactRow('Violations', monitoringSummary?.geo?.totalViolations),
    createFactRow('Blocks', monitoringSummary?.geo?.actionMix?.block),
    createFactRow('Challenge', monitoringSummary?.geo?.actionMix?.challenge),
    createFactRow('Maze', monitoringSummary?.geo?.actionMix?.maze),
    asTopCountryFact(monitoringSummary?.geo?.topCountries),
    ...deriveTrendContextFacts(trendRows, 'geo')
  ]),
  createDefenseBreakdownRow('ip_range', 'IP Range', [
    createFactRow('Mode', formatMetricLabel(ipRangeSummary?.mode || 'off')),
    createFactRow('Matches', ipRangeSummary?.totalMatches),
    createFactRow('Fallbacks', ipRangeSummary?.totalFallbacks),
    createFactRow('Source IDs', ipRangeSummary?.uniqueSourceIds),
    createFactRow('Custom Rules', ipRangeSummary?.catalog?.customRuleCount),
    createFactRow('Emergency Allowlist', ipRangeSummary?.catalog?.emergencyAllowlistCount),
    ...deriveTrendContextFacts(trendRows, 'ip_range')
  ])
];

const shapeAdversaryRunRows = (rows = [], activeBans = []) => {
  const normalizedRows = Array.isArray(rows) ? rows : [];
  const normalizedBans = Array.isArray(activeBans) ? activeBans : [];
  const runRows = normalizedRows
    .sort((left, right) => right.lastTs - left.lastTs)
    .map((row) => {
      const defenseCounts = row && typeof row.defenseCounts === 'object' ? row.defenseCounts : {};
      const defenseRows = Array.isArray(row?.defenseRows)
        ? row.defenseRows
        : Object.entries(defenseCounts)
          .sort((left, right) => Number(right[1]) - Number(left[1]))
          .map(([defense, count]) => ({
            defense,
            label: DEFENSE_LABELS[defense] || defense,
            count: Number(count || 0)
          }));
      const defenseDeltaCount = Number.isFinite(Number(row?.defenseDeltaCount))
        ? Number(row.defenseDeltaCount)
        : Object.keys(defenseCounts).length;
      const observedFulfillmentModes = Array.isArray(row?.observedFulfillmentModes)
        ? row.observedFulfillmentModes
        : [];
      const observedCategoryIds = Array.isArray(row?.observedCategoryIds)
        ? row.observedCategoryIds
        : [];
      const ownedSurfaceCoverage = row?.ownedSurfaceCoverage && typeof row.ownedSurfaceCoverage === 'object'
        ? row.ownedSurfaceCoverage
        : null;
      return {
        runId: row.runId,
        lane: row.lane,
        profile: row.profile,
        firstTs: row.firstTs,
        lastTs: row.lastTs,
        monitoringEventCount: row.monitoringEventCount,
        defenseDeltaCount,
        defenseRows,
        observedFulfillmentModes,
        observedCategoryIds,
        ownedSurfaceCoverage,
        banOutcomeCount: row.banOutcomeCount,
        monitoringHref: '#game-loop',
        ipBansHref: '#ip-bans'
      };
    });
  return {
    runRows,
    totalSimulationEvents: runRows.reduce((total, row) => total + row.monitoringEventCount, 0),
    activeBanCount: normalizedBans.length
  };
};

export const deriveAdversaryRunRows = (events = [], bans = []) => {
  const rows = Array.isArray(events) ? events : [];
  const activeBans = Array.isArray(bans) ? bans : [];
  /** @type {Map<string, {runId: string, lane: string, profile: string, firstTs: number, lastTs: number, monitoringEventCount: number, defenseCounts: Record<string, number>, banOutcomeCount: number}>} */
  const grouped = new Map();
  rows.forEach((event) => {
    const runId = String(event?.sim_run_id || '').trim();
    if (!runId) return;
    const lane = classifyEventLane(event);
    const profile = String(event?.sim_profile || '').trim() || 'unknown';
    const ts = Number(event?.ts || 0);
    const defense = classifyDefense(event);
    const existing = grouped.get(runId) || {
      runId,
      lane,
      profile,
      firstTs: Number.isFinite(ts) ? ts : 0,
      lastTs: Number.isFinite(ts) ? ts : 0,
      monitoringEventCount: 0,
      defenseCounts: {},
      banOutcomeCount: 0
    };
    existing.monitoringEventCount += 1;
    if (Number.isFinite(ts) && ts > 0) {
      existing.firstTs = existing.firstTs === 0 ? ts : Math.min(existing.firstTs, ts);
      existing.lastTs = Math.max(existing.lastTs, ts);
    }
    existing.defenseCounts[defense] = Number(existing.defenseCounts[defense] || 0) + 1;
    if (isBanOutcomeEvent(event)) {
      existing.banOutcomeCount += 1;
    }
    grouped.set(runId, existing);
  });
  return shapeAdversaryRunRows(Array.from(grouped.values()), activeBans);
};

const toSummaryStringArray = (value) =>
  (Array.isArray(value) ? value : [])
    .map((entry) => String(entry || '').trim())
    .filter(Boolean);

const shapeOwnedSurfaceCoverageReceipt = (receipt = {}) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  return {
    surfaceId: String(source.surfaceId || source.surface_id || '').trim(),
    successContract: String(source.successContract || source.success_contract || '').trim(),
    coverageStatus: String(source.coverageStatus || source.coverage_status || '').trim(),
    satisfied: source.satisfied === true,
    attemptCount: Number(source.attemptCount || source.attempt_count || 0),
    sampleRequestMethod: String(source.sampleRequestMethod || source.sample_request_method || '').trim(),
    sampleRequestPath: String(source.sampleRequestPath || source.sample_request_path || '').trim(),
    sampleResponseStatus:
      source.sampleResponseStatus === null || source.sampleResponseStatus === undefined || source.sampleResponseStatus === ''
        ? null
        : Number(source.sampleResponseStatus || source.sample_response_status || 0)
  };
};

const receiptWasExercised = (receipt = {}) => {
  const attemptCount = Number(receipt?.attemptCount || 0);
  const coverageStatus = String(receipt?.coverageStatus || '').trim();
  const sampleRequestMethod = String(receipt?.sampleRequestMethod || '').trim();
  const sampleRequestPath = String(receipt?.sampleRequestPath || '').trim();
  return Boolean(
    attemptCount > 0 ||
    receipt?.satisfied === true ||
    sampleRequestMethod ||
    sampleRequestPath ||
    (coverageStatus && coverageStatus !== 'unavailable')
  );
};

const shapeOwnedSurfaceCoverage = (coverage = {}) => {
  const source = coverage && typeof coverage === 'object' ? coverage : {};
  const requiredSurfaceIds = toSummaryStringArray(source.requiredSurfaceIds || source.required_surface_ids);
  const satisfiedSurfaceIds = toSummaryStringArray(source.satisfiedSurfaceIds || source.satisfied_surface_ids);
  const blockingSurfaceIds = toSummaryStringArray(source.blockingSurfaceIds || source.blocking_surface_ids);
  const receipts = (Array.isArray(source.receipts) ? source.receipts : [])
    .map((receipt) => shapeOwnedSurfaceCoverageReceipt(receipt))
    .filter((receipt) => receipt.surfaceId);
  if (
    !String(source.overallStatus || source.overall_status || '').trim() &&
    requiredSurfaceIds.length === 0 &&
    satisfiedSurfaceIds.length === 0 &&
    blockingSurfaceIds.length === 0 &&
    receipts.length === 0
  ) {
    return null;
  }
  const exercisedSurfaceCount = receipts.filter((receipt) => receiptWasExercised(receipt)).length;
  const expectedPassCount = receipts.filter(
    (receipt) => receipt.successContract === 'should_pass_some' && receipt.coverageStatus === 'pass_observed'
  ).length;
  const expectedFailCount = receipts.filter(
    (receipt) => receipt.successContract === 'should_fail' && receipt.coverageStatus === 'fail_observed'
  ).length;
  const mixedOutcomeCount = receipts.filter(
    (receipt) => receipt.successContract === 'mixed_outcomes' && receiptWasExercised(receipt)
  ).length;
  return {
    overallStatus: String(source.overallStatus || source.overall_status || '').trim(),
    requiredSurfaceIds,
    requiredSurfaceCount: requiredSurfaceIds.length,
    satisfiedSurfaceIds,
    satisfiedSurfaceCount: satisfiedSurfaceIds.length,
    blockingSurfaceIds,
    blockingSurfaceCount: blockingSurfaceIds.length,
    exercisedSurfaceCount,
    expectedPassCount,
    expectedFailCount,
    mixedOutcomeCount,
    receipts
  };
};

export const deriveAdversaryRunRowsFromSummaries = (summaries = [], bans = []) => {
  const rows = Array.isArray(summaries) ? summaries : [];
  const shapedRows = rows
    .map((summary) => ({
      runId: String(summary?.run_id || '').trim(),
      lane: String(summary?.lane || 'none').trim() || 'none',
      profile: String(summary?.profile || 'unknown').trim() || 'unknown',
      firstTs: Number(summary?.first_ts || 0),
      lastTs: Number(summary?.last_ts || 0),
      monitoringEventCount: Number(summary?.monitoring_event_count || 0),
      defenseDeltaCount: Number(summary?.defense_delta_count || 0),
      defenseRows: [],
      observedFulfillmentModes: toSummaryStringArray(
        summary?.observedFulfillmentModes || summary?.observed_fulfillment_modes
      ),
      observedCategoryIds: toSummaryStringArray(
        summary?.observedCategoryIds || summary?.observed_category_ids
      ),
      ownedSurfaceCoverage: shapeOwnedSurfaceCoverage(
        summary?.ownedSurfaceCoverage || summary?.owned_surface_coverage
      ),
      banOutcomeCount: Number(summary?.ban_outcome_count || 0)
    }))
    .filter((row) => row.runId.length > 0);
  return shapeAdversaryRunRows(shapedRows, bans);
};

export const deriveLatestScraplingEvidenceFromSummaries = (summaries = []) => {
  const recentRuns = deriveAdversaryRunRowsFromSummaries(summaries, []).runRows;
  return recentRuns.find((row) => row.lane === 'scrapling_traffic' && row.ownedSurfaceCoverage) || null;
};

const incrementCount = (target, key, amount = 1) => {
  const normalizedKey = String(key || '').trim() || 'unknown';
  target[normalizedKey] = Number(target[normalizedKey] || 0) + Number(amount || 0);
};

const toSortedCountEntries = (target) =>
  Object.entries(target && typeof target === 'object' ? target : {})
    .sort((left, right) => Number(right[1] || 0) - Number(left[1] || 0));

const normalizeMode = (value) => {
  const mode = String(value || '').trim().toLowerCase();
  if (mode === 'advisory' || mode === 'enforce' || mode === 'off') return mode;
  return 'off';
};

const TREND_POINT_LIMIT = 24;

export const deriveIpRangeMonitoringViewModel = (
  events = [],
  configSnapshot = {}
) => {
  const rows = Array.isArray(events) ? events : [];
  const config = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};

  const reasonCounts = {};
  const sourceCounts = {};
  const actionCounts = {};
  const detectionCounts = {};
  const sourceIdCounts = {};
  const fallbackCounts = {};
  const trendBuckets = {};
  let totalMatches = 0;

  rows.forEach((entry) => {
    const reason = String(entry?.reason || '').trim().toLowerCase();
    if (!isIpRangeReason(reason)) return;
    totalMatches += 1;

    const parsed = parseIpRangeOutcome(entry?.outcome, entry?.taxonomy);
    const source = String(parsed.source || 'unknown').toLowerCase();
    const action = String(parsed.action || 'unknown').toLowerCase();
    const detection = String(parsed.detection || 'unknown').toUpperCase();
    const sourceId = String(parsed.sourceId || 'unknown').toLowerCase();
    const fallback = classifyIpRangeFallback(reason, parsed);

    incrementCount(reasonCounts, reason);
    incrementCount(sourceCounts, source);
    incrementCount(actionCounts, action);
    incrementCount(detectionCounts, detection);
    incrementCount(sourceIdCounts, sourceId);
    incrementCount(fallbackCounts, fallback);

    const ts = Number(entry?.ts || 0);
    if (Number.isFinite(ts) && ts > 0) {
      const hour = Math.floor(ts / 3600) * 3600;
      trendBuckets[hour] = Number(trendBuckets[hour] || 0) + 1;
    }
  });

  const sortedHours = Object.keys(trendBuckets)
    .map((rawHour) => Number(rawHour))
    .filter((hour) => Number.isFinite(hour))
    .sort((left, right) => left - right);
  const trendStartIndex = Math.max(0, sortedHours.length - TREND_POINT_LIMIT);
  const trendHours = sortedHours.slice(trendStartIndex);
  const trend = {
    labels: trendHours.map((hour) => formatTrendTimestamp(hour)),
    data: trendHours.map((hour) => Number(trendBuckets[hour] || 0))
  };

  const customRules = Array.isArray(config.ip_range_custom_rules)
    ? config.ip_range_custom_rules
    : [];
  const emergencyAllowlist = Array.isArray(config.ip_range_emergency_allowlist)
    ? config.ip_range_emergency_allowlist
    : [];

  return {
    mode: normalizeMode(config.ip_range_policy_mode),
    totalMatches,
    totalFallbacks: toNonNegativeNumber(totalMatches - Number(fallbackCounts.none || 0)),
    uniqueSourceIds: Object.keys(sourceIdCounts).filter((key) => key !== 'unknown').length,
    reasons: toSortedCountEntries(reasonCounts),
    sources: toSortedCountEntries(sourceCounts),
    actions: toSortedCountEntries(actionCounts),
    detections: toSortedCountEntries(detectionCounts),
    sourceIds: toSortedCountEntries(sourceIdCounts),
    fallbacks: toSortedCountEntries(fallbackCounts),
    trend,
    catalog: {
      customRuleCount: customRules.length,
      customRuleEnabledCount: customRules.filter((rule) => rule?.enabled === true).length,
      emergencyAllowlistCount: emergencyAllowlist.length
    }
  };
};

export const formatMetricLabel = (key, fallbackMap) => {
  if (fallbackMap && fallbackMap[key]) return fallbackMap[key];
  return String(key || '-')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase());
};

export const deriveMazeStatsViewModel = (data = {}) => {
  const topCrawler =
    Array.isArray(data.top_crawlers) && data.top_crawlers.length ? data.top_crawlers[0] : null;
  const mazeAutoBans = data.maze_auto_bans === null || data.maze_auto_bans === undefined
    ? 'Unavailable'
    : formatCompactNumber(data.maze_auto_bans, '0');
  return {
    totalHits: formatCompactNumber(data.total_hits, '0'),
    uniqueCrawlers: formatCompactNumber(data.unique_crawlers, '0'),
    mazeAutoBans,
    topOffender: deriveTopOffenderViewModel(
      topCrawler?.ip,
      topCrawler?.hits,
      'page',
      'pages'
    )
  };
};

export const deriveTarpitViewModel = (data = {}) => {
  const source = data && typeof data === 'object' ? data : {};
  const metrics = source.metrics && typeof source.metrics === 'object' ? source.metrics : {};
  const activations = metrics.activations && typeof metrics.activations === 'object'
    ? metrics.activations
    : {};
  const progressOutcomes = metrics.progress_outcomes && typeof metrics.progress_outcomes === 'object'
    ? metrics.progress_outcomes
    : {};
  const budgetOutcomes = metrics.budget_outcomes && typeof metrics.budget_outcomes === 'object'
    ? metrics.budget_outcomes
    : {};
  const escalationOutcomes =
    metrics.escalation_outcomes && typeof metrics.escalation_outcomes === 'object'
      ? metrics.escalation_outcomes
      : {};

  return {
    activationsProgressive: formatCompactNumber(activations.progressive, '0'),
    progressAdvanced: formatCompactNumber(progressOutcomes.advanced, '0'),
    fallbackMaze: formatCompactNumber(budgetOutcomes.fallback_maze, '0'),
    fallbackBlock: formatCompactNumber(budgetOutcomes.fallback_block, '0'),
    escalationShortBan: formatCompactNumber(escalationOutcomes.short_ban, '0'),
    escalationBlock: formatCompactNumber(escalationOutcomes.block, '0'),
    progressOutcomes: sortCountEntries(progressOutcomes),
    budgetOutcomes: sortCountEntries(budgetOutcomes),
    escalationOutcomes: sortCountEntries(escalationOutcomes)
  };
};

export const deriveMonitoringSummaryViewModel = (summary = {}) => {
  const honeypot = summary.honeypot || {};
  const challenge = summary.challenge || {};
  const notABot = summary.not_a_bot || {};
  const pow = summary.pow || {};
  const rate = summary.rate || {};
  const geo = summary.geo || {};
  const honeypotTopPaths = Array.isArray(honeypot.top_paths)
    ? honeypot.top_paths.map((entry) => ({
      path: String(
        Array.isArray(entry)
          ? (entry[0] ?? '')
          : (entry?.path ?? entry?.label ?? '')
      ),
      count: toNonNegativeNumber(Array.isArray(entry) ? entry[1] : entry?.count)
    }))
    : [];
  const geoTopCountries = Array.isArray(geo.top_countries)
    ? geo.top_countries.map((entry) => ({
      country: String(
        Array.isArray(entry)
          ? (entry[0] ?? '')
          : (entry?.country ?? entry?.label ?? '')
      ),
      count: toNonNegativeNumber(Array.isArray(entry) ? entry[1] : entry?.count)
    }))
    : [];

  const topHoneypotCrawler =
    Array.isArray(honeypot.top_crawlers) && honeypot.top_crawlers.length
      ? honeypot.top_crawlers[0]
      : null;
  const topChallengeOffender =
    Array.isArray(challenge.top_offenders) && challenge.top_offenders.length
      ? challenge.top_offenders[0]
      : null;
  const topPowOffender =
    Array.isArray(pow.top_offenders) && pow.top_offenders.length
      ? pow.top_offenders[0]
      : null;
  const topRateOffender =
    Array.isArray(rate.top_offenders) && rate.top_offenders.length
      ? rate.top_offenders[0]
      : null;
  const powFailureTotal = toNonNegativeNumber(pow.total_failures);
  const powSuccessTotal = toNonNegativeNumber(pow.total_successes);
  const powAttemptFallback = powFailureTotal + powSuccessTotal;
  const powAttemptsTotal = Math.max(powAttemptFallback, toNonNegativeNumber(pow.total_attempts));
  const powRatioRaw = Number(pow.success_ratio);
  const powSuccessRatio = Number.isFinite(powRatioRaw)
    ? Math.min(1, Math.max(0, powRatioRaw))
    : (powAttemptsTotal > 0 ? Math.min(1, Math.max(0, powSuccessTotal / powAttemptsTotal)) : 0);
  const notABotServed = toNonNegativeNumber(notABot.served);
  const notABotSubmitted = toNonNegativeNumber(notABot.submitted);
  const notABotPass = toNonNegativeNumber(notABot.pass);
  const notABotEscalate = toNonNegativeNumber(notABot.escalate);
  const notABotFail = toNonNegativeNumber(notABot.fail);
  const notABotReplay = toNonNegativeNumber(notABot.replay);
  const notABotAbandonments = toNonNegativeNumber(notABot.abandonments_estimated);
  const notABotAbandonmentRatioRaw = Number(notABot.abandonment_ratio);
  const notABotAbandonmentRatio = Number.isFinite(notABotAbandonmentRatioRaw)
    ? Math.min(1, Math.max(0, notABotAbandonmentRatioRaw))
    : (notABotServed > 0
      ? Math.min(1, Math.max(0, notABotAbandonments / notABotServed))
      : 0);
  return {
    honeypot: {
      totalHits: formatCompactNumber(honeypot.total_hits, '0'),
      uniqueCrawlers: formatCompactNumber(honeypot.unique_crawlers, '0'),
      topOffender: deriveTopOffenderViewModel(
        topHoneypotCrawler?.label,
        topHoneypotCrawler?.count,
        'hit',
        'hits'
      ),
      topPaths: honeypotTopPaths
    },
    challenge: {
      totalFailures: formatCompactNumber(challenge.total_failures, '0'),
      uniqueOffenders: formatCompactNumber(challenge.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topChallengeOffender?.label,
        topChallengeOffender?.count,
        'hit',
        'hits'
      ),
      reasons: sortCountEntries(challenge.reasons),
      trend: deriveTrendSeries(challenge.trend)
    },
    notABot: {
      served: formatCompactNumber(notABotServed, '0'),
      submitted: formatCompactNumber(notABotSubmitted, '0'),
      pass: formatCompactNumber(notABotPass, '0'),
      escalate: formatCompactNumber(notABotEscalate, '0'),
      fail: formatCompactNumber(notABotFail, '0'),
      replay: formatCompactNumber(notABotReplay, '0'),
      abandonmentsEstimated: formatCompactNumber(notABotAbandonments, '0'),
      abandonmentRate: `${(notABotAbandonmentRatio * 100).toFixed(1)}%`,
      outcomes: sortCountEntries(notABot.outcomes),
      latencyBuckets: sortCountEntries(notABot.solve_latency_buckets)
    },
    pow: {
      totalFailures: formatCompactNumber(powFailureTotal, '0'),
      totalSuccesses: formatCompactNumber(powSuccessTotal, '0'),
      totalAttempts: formatCompactNumber(powAttemptsTotal, '0'),
      successRatio: powSuccessRatio,
      successRate: `${(powSuccessRatio * 100).toFixed(1)}%`,
      uniqueOffenders: formatCompactNumber(pow.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topPowOffender?.label,
        topPowOffender?.count,
        'hit',
        'hits'
      ),
      reasons: sortCountEntries(pow.reasons),
      outcomes: sortCountEntries(pow.outcomes),
      trend: deriveTrendSeries(pow.trend)
    },
    rate: {
      totalViolations: formatCompactNumber(rate.total_violations, '0'),
      uniqueOffenders: formatCompactNumber(rate.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topRateOffender?.label,
        topRateOffender?.count,
        'hit',
        'hits'
      ),
      outcomes: sortCountEntries(rate.outcomes)
    },
    geo: {
      totalViolations: formatCompactNumber(geo.total_violations, '0'),
      actionMix: {
        block: formatCompactNumber(geo.actions?.block || 0, '0'),
        challenge: formatCompactNumber(geo.actions?.challenge || 0, '0'),
        maze: formatCompactNumber(geo.actions?.maze || 0, '0')
      },
      topCountries: geoTopCountries
    }
  };
};

export const deriveMonitoringEventDisplay = (event = {}) => {
  const source = event && typeof event === 'object' ? event : {};
  const normalized = { ...source };
  const eventType = normalizeToken(source.event || '');
  if (eventType === 'challenge') {
    normalized.event = classifyChallengeDisplayLabel(source);
  }
  normalized.reason = normalizeReasonForDisplay(source.reason);
  const executionMode = classifyExecutionMode(source);
  normalized.executionMode = executionMode;
  normalized.executionModeLabel = EXECUTION_MODE_LABELS[executionMode] || 'Enforced';
  const outcome = deriveDisplayedOutcome(source);
  normalized.outcome = outcome.label;
  normalized.outcomeToken = outcome.token;
  return normalized;
};

export const derivePrometheusHelperViewModel = (prometheusData = {}, origin = '') => {
  const readString = (value) => (typeof value === 'string' ? value.trim() : '');
  const sanitizeExternalUrl = (value) => {
    const raw = readString(value);
    if (!/^https?:\/\//i.test(raw)) return '';
    try {
      const parsed = new URL(raw);
      if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return '';
      return parsed.href;
    } catch (_error) {
      return '';
    }
  };
  const endpoint =
    typeof prometheusData.endpoint === 'string' ? prometheusData.endpoint : '/metrics';
  const docs =
    prometheusData && typeof prometheusData.docs === 'object' ? prometheusData.docs : {};
  const notes = Array.isArray(prometheusData?.notes)
    ? prometheusData.notes.map(readString).filter((entry) => entry.length > 0)
    : [];
  const fallbackFacts = ['Monitoring guidance unavailable; see docs links below.'];
  const siteOrigin = origin || 'http://127.0.0.1:3000';

  return {
    exampleJs: readString(prometheusData?.example_js) || '// Example unavailable.',
    copyCurlText: `curl -sS '${siteOrigin}${endpoint}'`,
    facts: notes.length ? notes : fallbackFacts,
    exampleOutput: readString(prometheusData?.example_output) || '# Example unavailable.',
    exampleStats: readString(prometheusData?.example_stats) || '// Example unavailable.',
    exampleWindowed: readString(prometheusData?.example_windowed) || '// Example unavailable.',
    exampleSummaryStats:
      readString(prometheusData?.example_summary_stats) || '// Example unavailable.',
    observabilityLink: sanitizeExternalUrl(docs.observability),
    apiLink: sanitizeExternalUrl(docs.api)
  };
};

export {
  CHALLENGE_REASON_LABELS,
  EXECUTION_MODE_LABELS,
  IP_RANGE_ACTION_LABELS,
  IP_RANGE_FALLBACK_LABELS,
  IP_RANGE_SOURCE_LABELS,
  NOT_A_BOT_OUTCOME_LABELS,
  NOT_A_BOT_LATENCY_LABELS,
  POW_REASON_LABELS,
  RATE_OUTCOME_LABELS,
  SHADOW_ACTION_LABELS,
  normalizeOffenderBucketLabel
};
