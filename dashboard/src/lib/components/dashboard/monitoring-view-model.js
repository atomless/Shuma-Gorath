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

const CANONICAL_ADVERSARY_COVERAGE_SURFACE_IDS = Object.freeze([
  'public_ingress',
  'challenge_puzzle',
  'not_a_bot',
  'js_verification',
  'maze',
  'tarpit',
  'rate_pressure',
  'geo_ip_policy',
  'honeypot',
  'pow_verify',
  'ban_path',
  'browser_automation_detection'
]);

const CANONICAL_ADVERSARY_COVERAGE_SURFACE_ID_SET = new Set(
  CANONICAL_ADVERSARY_COVERAGE_SURFACE_IDS
);

const RECEIPT_PROJECTED_CANONICAL_SURFACE_IDS = Object.freeze({
  public_ingress: 'public_ingress',
  public_path_traversal: 'public_ingress',
  challenge_puzzle: 'challenge_puzzle',
  challenge_routing: 'challenge_puzzle',
  puzzle_submit_or_escalation: 'challenge_puzzle',
  not_a_bot: 'not_a_bot',
  not_a_bot_submit: 'not_a_bot',
  js_verification: 'js_verification',
  js_verification_execution: 'js_verification',
  maze: 'maze',
  maze_navigation: 'maze',
  tarpit: 'tarpit',
  tarpit_progress_abuse: 'tarpit',
  rate_pressure: 'rate_pressure',
  geo_ip_policy: 'geo_ip_policy',
  honeypot: 'honeypot',
  pow_verify: 'pow_verify',
  pow_verify_abuse: 'pow_verify',
  ban_path: 'ban_path',
  browser_automation_detection: 'browser_automation_detection'
});

const canonicalizeAdversaryCoverageSurfaceIds = (surfaceIds = []) =>
  Array.from(new Set(
    (Array.isArray(surfaceIds) ? surfaceIds : [])
      .map((surfaceId) => RECEIPT_PROJECTED_CANONICAL_SURFACE_IDS[normalizeToken(surfaceId)] || '')
      .filter((surfaceId) => CANONICAL_ADVERSARY_COVERAGE_SURFACE_ID_SET.has(surfaceId))
  ));

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
      const llmSurfaceCoverage = row?.llmSurfaceCoverage && typeof row.llmSurfaceCoverage === 'object'
        ? row.llmSurfaceCoverage
        : null;
      const observedSurfaceCoverage =
        row?.observedSurfaceCoverage && typeof row.observedSurfaceCoverage === 'object'
          ? row.observedSurfaceCoverage
          : null;
      const latestScraplingRealismReceipt =
        row?.latestScraplingRealismReceipt && typeof row.latestScraplingRealismReceipt === 'object'
          ? row.latestScraplingRealismReceipt
          : null;
      const identitySummary = row?.identitySummary && typeof row.identitySummary === 'object'
        ? row.identitySummary
        : null;
      const transportSummary = row?.transportSummary && typeof row.transportSummary === 'object'
        ? row.transportSummary
        : null;
      const llmRuntimeSummary = row?.llmRuntimeSummary && typeof row.llmRuntimeSummary === 'object'
        ? row.llmRuntimeSummary
        : null;
      const monitoringEventCount = Number.isFinite(Number(row?.monitoringEventCount))
        ? Number(row.monitoringEventCount)
        : 0;
      const hasSharedObservedTelemetry =
        Boolean(observedSurfaceCoverage)
        || monitoringEventCount > 0
        || defenseDeltaCount > 0;
      return {
        runId: row.runId,
        lane: row.lane,
        profile: row.profile,
        firstTs: row.firstTs,
        lastTs: row.lastTs,
        scraplingActivityCount: row.scraplingActivityCount,
        monitoringEventCount,
        defenseDeltaCount,
        hasSharedObservedTelemetry,
        defenseRows,
        observedFulfillmentModes,
        observedCategoryIds,
        observedSurfaceCoverage,
        ownedSurfaceCoverage,
        llmSurfaceCoverage,
        coverageSummary: deriveCoverageSummary(
          observedSurfaceCoverage,
          ownedSurfaceCoverage,
          llmSurfaceCoverage
        ),
        latestScraplingRealismReceipt,
        identitySummary,
        transportSummary,
        llmRuntimeSummary,
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

const toSummaryLabelMap = (value) => {
  const source = value && typeof value === 'object' ? value : {};
  return Object.fromEntries(
    Object.entries(source)
      .map(([key, label]) => [String(key || '').trim(), String(label || '').trim()])
      .filter(([key, label]) => key && label)
  );
};

const formatOwnedSurfaceLabelFallback = (surfaceId) => String(surfaceId || '')
  .trim()
  .replace(/[_-]+/g, ' ')
  .replace(/\s+/g, ' ')
  .split(' ')
  .filter(Boolean)
  .map((word) => {
    const lowered = word.toLowerCase();
    if (lowered === 'ai') return 'AI';
    if (lowered === 'cdp') return 'CDP';
    if (lowered === 'http') return 'HTTP';
    if (lowered === 'ip') return 'IP';
    if (lowered === 'js') return 'JavaScript';
    if (lowered === 'pow') return 'PoW';
    return lowered.charAt(0).toUpperCase() + lowered.slice(1);
  })
  .join(' ');

const resolveOwnedSurfaceLabel = (surfaceId, surfaceLabels = {}) =>
  String(surfaceLabels?.[surfaceId] || '').trim() || formatOwnedSurfaceLabelFallback(surfaceId);

const deriveRequiredSurfaceState = (receipt = {}, satisfiedFallback = false) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  const explicitState = String(source.surfaceState || source.surface_state || '').trim();
  if (explicitState) return explicitState;
  const satisfied = source.satisfied === true || satisfiedFallback === true;
  if (satisfied) return 'satisfied';
  const attemptCount = Number(source.attemptCount || source.attempt_count || 0);
  return attemptCount > 0 ? 'attempted_blocked' : 'unreached';
};

const formatOwnedSurfaceStateLabel = (state = '', blockedBySurfaceIds = [], surfaceLabels = {}) => {
  switch (String(state || '').trim()) {
    case 'satisfied':
      return 'satisfied';
    case 'attempted_blocked':
      return 'attempted and blocked';
    case 'blocked_by_prerequisite': {
      const blockedBy = (Array.isArray(blockedBySurfaceIds) ? blockedBySurfaceIds : [])
        .map((surfaceId) => resolveOwnedSurfaceLabel(surfaceId, surfaceLabels))
        .filter(Boolean);
      return blockedBy.length > 0
        ? `blocked by prerequisite: ${blockedBy.join(', ')}`
        : 'blocked by prerequisite';
    }
    case 'unreached':
      return 'required but unreached';
    case 'not_required':
      return 'not required this run';
    default:
      return 'state unavailable';
  }
};

const formatOwnedSurfaceDependencyLabel = (dependencyKind = '', dependencySurfaceIds = [], surfaceLabels = {}) => {
  switch (String(dependencyKind || '').trim()) {
    case 'independent':
      return 'independent surface';
    case 'co_materialized': {
      const related = (Array.isArray(dependencySurfaceIds) ? dependencySurfaceIds : [])
        .map((surfaceId) => resolveOwnedSurfaceLabel(surfaceId, surfaceLabels))
        .filter(Boolean);
      return related.length > 0
        ? `co-materialized with ${related.join(', ')}`
        : 'co-materialized surface';
    }
    case 'requires_prior_surface_pass': {
      const required = (Array.isArray(dependencySurfaceIds) ? dependencySurfaceIds : [])
        .map((surfaceId) => resolveOwnedSurfaceLabel(surfaceId, surfaceLabels))
        .filter(Boolean);
      return required.length > 0
        ? `requires prior ${required.join(', ')}`
        : 'requires prior surface';
    }
    default:
      return '';
  }
};

const shapeOwnedSurfaceCoverageReceipt = (receipt = {}, surfaceLabels = {}) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  const surfaceId = String(source.surfaceId || source.surface_id || '').trim();
  const surfaceState = deriveRequiredSurfaceState(source, source.satisfied === true);
  const blockedBySurfaceIds = toSummaryStringArray(source.blockedBySurfaceIds || source.blocked_by_surface_ids);
  const dependencyKind = String(source.dependencyKind || source.dependency_kind || '').trim();
  const dependencySurfaceIds = toSummaryStringArray(source.dependencySurfaceIds || source.dependency_surface_ids);
  return {
    surfaceId,
    surfaceLabel: resolveOwnedSurfaceLabel(surfaceId, surfaceLabels),
    successContract: String(source.successContract || source.success_contract || '').trim(),
    dependencyKind,
    dependencySurfaceIds,
    coverageStatus: String(source.coverageStatus || source.coverage_status || '').trim(),
    satisfied: source.satisfied === true,
    surfaceState,
    surfaceStateLabel: formatOwnedSurfaceStateLabel(surfaceState, blockedBySurfaceIds, surfaceLabels),
    blockedBySurfaceIds,
    dependencyLabel: formatOwnedSurfaceDependencyLabel(
      dependencyKind,
      dependencySurfaceIds,
      surfaceLabels
    ),
    attemptCount: Number(source.attemptCount || source.attempt_count || 0),
    sampleRequestMethod: String(source.sampleRequestMethod || source.sample_request_method || '').trim(),
    sampleRequestPath: String(source.sampleRequestPath || source.sample_request_path || '').trim(),
    sampleResponseStatus:
      source.sampleResponseStatus === null || source.sampleResponseStatus === undefined || source.sampleResponseStatus === ''
        ? null
        : Number(source.sampleResponseStatus || source.sample_response_status || 0)
  };
};

const buildSurfaceChecklistRows = (
  canonicalSurfaceIds = [],
  surfaceLabels = {},
  requiredSurfaceIds = [],
  satisfiedSurfaceIds = [],
  receipts = []
) => {
  const requiredSurfaceSet = new Set(requiredSurfaceIds);
  const satisfiedSurfaceSet = new Set(satisfiedSurfaceIds);
  const receiptBySurfaceId = new Map(
    (Array.isArray(receipts) ? receipts : [])
      .filter((receipt) => receipt && receipt.surfaceId)
      .map((receipt) => [receipt.surfaceId, receipt])
  );
  return canonicalSurfaceIds.map((surfaceId) => {
    const receipt = receiptBySurfaceId.get(surfaceId);
    const state = !requiredSurfaceSet.has(surfaceId)
      ? 'not_required'
      : deriveRequiredSurfaceState(receipt, satisfiedSurfaceSet.has(surfaceId));
    const dependencyLabel =
      state === 'not_required' || state === 'satisfied'
        ? ''
        : String(receipt?.dependencyLabel || '').trim();
    const row = {
      surfaceId,
      surfaceLabel: resolveOwnedSurfaceLabel(surfaceId, surfaceLabels),
      state,
      stateLabel: formatOwnedSurfaceStateLabel(
        state,
        receipt?.blockedBySurfaceIds,
        surfaceLabels
      )
    };
    if (dependencyLabel) {
      row.dependencyLabel = dependencyLabel;
    }
    return row;
  });
};

const shapeOwnedSurfaceCoverage = (coverage = {}) => {
  const source = coverage && typeof coverage === 'object' ? coverage : {};
  const canonicalSurfaceIds = toSummaryStringArray(source.canonicalSurfaceIds || source.canonical_surface_ids);
  const surfaceLabels = toSummaryLabelMap(source.surfaceLabels || source.surface_labels);
  const requiredSurfaceIds = toSummaryStringArray(source.requiredSurfaceIds || source.required_surface_ids);
  const satisfiedSurfaceIds = toSummaryStringArray(source.satisfiedSurfaceIds || source.satisfied_surface_ids);
  const blockingSurfaceIds = toSummaryStringArray(source.blockingSurfaceIds || source.blocking_surface_ids);
  const receipts = (Array.isArray(source.receipts) ? source.receipts : [])
    .map((receipt) => shapeOwnedSurfaceCoverageReceipt(receipt, surfaceLabels))
    .filter((receipt) => receipt.surfaceId);
  if (
    !String(source.overallStatus || source.overall_status || '').trim() &&
    canonicalSurfaceIds.length === 0 &&
    Object.keys(surfaceLabels).length === 0 &&
    requiredSurfaceIds.length === 0 &&
    satisfiedSurfaceIds.length === 0 &&
    blockingSurfaceIds.length === 0 &&
    receipts.length === 0
  ) {
    return null;
  }
  return {
    overallStatus: String(source.overallStatus || source.overall_status || '').trim(),
    canonicalSurfaceIds,
    surfaceLabels,
    requiredSurfaceIds,
    requiredSurfaceCount: requiredSurfaceIds.length,
    satisfiedSurfaceIds,
    satisfiedSurfaceCount: satisfiedSurfaceIds.length,
    blockingSurfaceIds,
    blockingSurfaceCount: blockingSurfaceIds.length,
    surfaceChecklistRows: buildSurfaceChecklistRows(
      canonicalSurfaceIds,
      surfaceLabels,
      requiredSurfaceIds,
      satisfiedSurfaceIds,
      receipts
    ),
    receipts
  };
};

const shapeObservedSurfaceCoverageReceipt = (receipt = {}, surfaceLabels = {}) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  const surfaceId = String(source.surfaceId || source.surface_id || '').trim();
  return {
    surfaceId,
    surfaceLabel: surfaceLabels[surfaceId] || formatMetricLabel(surfaceId),
    coverageStatus: String(source.coverageStatus || source.coverage_status || '').trim(),
    surfaceState: String(source.surfaceState || source.surface_state || '').trim(),
    attemptCount: Number(source.attemptCount || source.attempt_count || 0),
    sampleResponseKind: String(source.sampleResponseKind || source.sample_response_kind || '').trim(),
    sampleHttpStatus:
      source.sampleHttpStatus === null || source.sampleHttpStatus === undefined || source.sampleHttpStatus === ''
        ? null
        : Number(source.sampleHttpStatus || source.sample_http_status || 0)
  };
};

const shapeObservedSurfaceCoverage = (coverage = {}) => {
  const source = coverage && typeof coverage === 'object' ? coverage : {};
  const observedSurfaceIds = toSummaryStringArray(source.observedSurfaceIds || source.observed_surface_ids);
  const responseSurfaceIds = toSummaryStringArray(source.responseSurfaceIds || source.response_surface_ids);
  const progressSurfaceIds = toSummaryStringArray(source.progressSurfaceIds || source.progress_surface_ids);
  const surfaceLabels = toSummaryLabelMap(source.surfaceLabels || source.surface_labels);
  const receipts = (Array.isArray(source.receipts) ? source.receipts : [])
    .map((receipt) => shapeObservedSurfaceCoverageReceipt(receipt, surfaceLabels))
    .filter((receipt) => receipt.surfaceId);
  if (
    !String(source.overallStatus || source.overall_status || '').trim()
    && observedSurfaceIds.length === 0
    && responseSurfaceIds.length === 0
    && progressSurfaceIds.length === 0
    && Object.keys(surfaceLabels).length === 0
    && receipts.length === 0
  ) {
    return null;
  }
  return {
    overallStatus: String(source.overallStatus || source.overall_status || '').trim(),
    observedSurfaceIds,
    observedSurfaceCount: observedSurfaceIds.length,
    responseSurfaceIds,
    responseSurfaceCount: responseSurfaceIds.length,
    progressSurfaceIds,
    progressSurfaceCount: progressSurfaceIds.length,
    surfaceLabels,
    receipts
  };
};

const shapeLlmSurfaceCoverageReceipt = (receipt = {}, surfaceLabels = {}) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  const surfaceId = String(source.surfaceId || source.surface_id || '').trim();
  return {
    surfaceId,
    surfaceLabel: surfaceLabels[surfaceId] || llmSurfaceLabel(surfaceId),
    coverageStatus: String(source.coverageStatus || source.coverage_status || '').trim(),
    surfaceState: String(source.surfaceState || source.surface_state || '').trim(),
    attemptCount: Number(source.attemptCount || source.attempt_count || 0),
    sampleRequestMethod: String(source.sampleRequestMethod || source.sample_request_method || '').trim(),
    sampleRequestPath: String(source.sampleRequestPath || source.sample_request_path || '').trim(),
    sampleResponseStatus:
      source.sampleResponseStatus === null || source.sampleResponseStatus === undefined || source.sampleResponseStatus === ''
        ? null
        : Number(source.sampleResponseStatus || source.sample_response_status || 0)
  };
};

const shapeLlmRuntimeActionReceipt = (receipt = {}) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  return {
    actionIndex: Number(source.actionIndex || source.action_index || 0),
    actionType: String(source.actionType || source.action_type || '').trim(),
    path: String(source.path || '').trim(),
    label: String(source.label || '').trim(),
    status:
      source.status === null || source.status === undefined || source.status === ''
        ? null
        : Number(source.status || 0),
    error: String(source.error || '').trim()
  };
};

const shapeIdentityRealismReceipt = (receipt = {}) => {
  const source = receipt && typeof receipt === 'object' ? receipt : {};
  const hasAnyValue = Object.keys(source).length > 0;
  if (!hasAnyValue) return null;
  return {
    profileId: String(source.profileId || source.profile_id || '').trim(),
    transportProfile: String(source.transportProfile || source.transport_profile || '').trim(),
    transportRealismClass: String(
      source.transportRealismClass || source.transport_realism_class || ''
    ).trim(),
    transportEmissionBasis: String(
      source.transportEmissionBasis || source.transport_emission_basis || ''
    ).trim(),
    transportDegradedReason: String(
      source.transportDegradedReason || source.transport_degraded_reason || ''
    ).trim(),
    activityCount: Number(source.activityCount || source.activity_count || 0),
    identityRealismStatus: String(
      source.identityRealismStatus || source.identity_realism_status || ''
    ).trim(),
    identityProvenanceMode: String(
      source.identityProvenanceMode || source.identity_provenance_mode || ''
    ).trim(),
    observedCountryCodes: toSummaryStringArray(
      source.observedCountryCodes || source.observed_country_codes
    )
  };
};

const shapeIdentitySummary = (summary = {}) => {
  const source = summary && typeof summary === 'object' ? summary : {};
  const hasAnyValue = Object.keys(source).length > 0;
  if (!hasAnyValue) return null;
  const modes = toSummaryStringArray(source.modes);
  const observedCountryCodes = toSummaryStringArray(
    source.observedCountryCodes || source.observed_country_codes
  );
  if (modes.length === 0 && observedCountryCodes.length === 0) return null;
  return {
    modes,
    observedCountryCodes
  };
};

const shapeTransportSummary = (summary = {}) => {
  const source = summary && typeof summary === 'object' ? summary : {};
  const hasAnyValue = Object.keys(source).length > 0;
  if (!hasAnyValue) return null;
  const modes = toSummaryStringArray(source.modes);
  const degradedReasons = toSummaryStringArray(
    source.degradedReasons || source.degraded_reasons
  );
  if (modes.length === 0 && degradedReasons.length === 0) return null;
  return {
    modes,
    degradedReasons
  };
};

const shapeLlmRuntimeSummary = (summary = {}) => {
  const source = summary && typeof summary === 'object' ? summary : {};
  const hasAnyValue = Object.keys(source).length > 0;
  if (!hasAnyValue) return null;
  return {
    receiptCount: Number(source.receiptCount || source.receipt_count || 0),
    fulfillmentMode: String(source.fulfillmentMode || source.fulfillment_mode || '').trim(),
    categoryTargets: toSummaryStringArray(source.categoryTargets || source.category_targets),
    backendKind: String(source.backendKind || source.backend_kind || '').trim(),
    backendState: String(source.backendState || source.backend_state || '').trim(),
    generationSource: String(source.generationSource || source.generation_source || '').trim(),
    provider: String(source.provider || '').trim(),
    modelId: String(source.modelId || source.model_id || '').trim(),
    fallbackReason: String(source.fallbackReason || source.fallback_reason || '').trim(),
    generatedActionCount: Number(source.generatedActionCount || source.generated_action_count || 0),
    executedActionCount: Number(source.executedActionCount || source.executed_action_count || 0),
    failedActionCount: Number(source.failedActionCount || source.failed_action_count || 0),
    passedTickCount: Number(source.passedTickCount || source.passed_tick_count || 0),
    failedTickCount: Number(source.failedTickCount || source.failed_tick_count || 0),
    lastResponseStatus:
      source.lastResponseStatus === null || source.lastResponseStatus === undefined || source.lastResponseStatus === ''
        ? null
        : Number(source.lastResponseStatus || source.last_response_status || 0),
    failureClass: String(source.failureClass || source.failure_class || '').trim(),
    error: String(source.error || '').trim(),
    terminalFailure: String(source.terminalFailure || source.terminal_failure || '').trim(),
    latestRealismReceipt: shapeIdentityRealismReceipt(
      source.latestRealismReceipt || source.latest_realism_receipt
    ),
    latestActionReceipts: (Array.isArray(source.latestActionReceipts || source.latest_action_receipts)
      ? source.latestActionReceipts || source.latest_action_receipts
      : [])
      .map((receipt) => shapeLlmRuntimeActionReceipt(receipt))
      .filter((receipt) => receipt.actionIndex > 0 || receipt.actionType || receipt.path)
  };
};

const llmReceiptSurfaceId = (receipt = {}) => {
  const path = String(receipt?.path || '').trim();
  if (!path) return '';
  if (
    path === '/'
    || path === '/about'
    || path.startsWith('/about/')
    || path === '/research'
    || path.startsWith('/research/')
    || path === '/plans'
    || path.startsWith('/plans/')
    || path === '/work'
    || path.startsWith('/work/')
    || path === '/page'
    || path.startsWith('/page/')
    || path === '/sitemaps'
    || path.startsWith('/sitemaps/')
    || path.startsWith('/detail/')
    || path.startsWith('/search')
  ) {
    return 'public_path_traversal';
  }
  if (path.startsWith('/challenge')) return 'challenge_routing';
  if (path.startsWith('/maze')) return 'maze_navigation';
  if (path.startsWith('/pow')) return 'pow_verify_abuse';
  if (path.startsWith('/tarpit')) return 'tarpit_progress_abuse';
  return '';
};

const llmSurfaceLabel = (surfaceId) => {
  switch (String(surfaceId || '').trim()) {
    case 'public_path_traversal':
      return 'Public Path Traversal';
    case 'challenge_routing':
      return 'Challenge Routing';
    case 'maze_navigation':
      return 'Maze Navigation';
    case 'pow_verify_abuse':
      return 'PoW Verify Abuse';
    case 'tarpit_progress_abuse':
      return 'Tarpit Progress Abuse';
    default:
      return formatMetricLabel(String(surfaceId || '').trim());
  }
};

const shapeLlmSurfaceCoverage = (coverage = {}) => {
  const source = coverage && typeof coverage === 'object' ? coverage : {};
  const observedSurfaceIds = toSummaryStringArray(source.observedSurfaceIds || source.observed_surface_ids);
  const responseSurfaceIds = toSummaryStringArray(source.responseSurfaceIds || source.response_surface_ids);
  const progressSurfaceIds = toSummaryStringArray(source.progressSurfaceIds || source.progress_surface_ids);
  const surfaceLabels = toSummaryLabelMap(source.surfaceLabels || source.surface_labels);
  const receipts = (Array.isArray(source.receipts) ? source.receipts : [])
    .map((receipt) => shapeLlmSurfaceCoverageReceipt(receipt, surfaceLabels))
    .filter((receipt) => receipt.surfaceId);
  if (
    !String(source.overallStatus || source.overall_status || '').trim()
    && observedSurfaceIds.length === 0
    && responseSurfaceIds.length === 0
    && progressSurfaceIds.length === 0
    && Object.keys(surfaceLabels).length === 0
    && receipts.length === 0
  ) {
    return null;
  }
  return {
    overallStatus: String(source.overallStatus || source.overall_status || '').trim(),
    observedSurfaceIds,
    observedSurfaceCount: observedSurfaceIds.length,
    responseSurfaceIds,
    responseSurfaceCount: responseSurfaceIds.length,
    progressSurfaceIds,
    progressSurfaceCount: progressSurfaceIds.length,
    surfaceLabels,
    receipts
  };
};

const llmSurfaceState = (receipt = {}) => {
  const status = Number.isFinite(Number(receipt?.status)) ? Number(receipt.status) : null;
  if (!String(receipt?.error || '').trim() && status !== null && status >= 200 && status < 400) {
    return 'leaked';
  }
  if (status !== null) return 'held';
  return 'attempted';
};

const mergeLlmSurfaceState = (current, next) => {
  if (current === 'leaked' || next === 'leaked') return 'leaked';
  if (current === 'held' || next === 'held') return 'held';
  return 'attempted';
};

const llmRequestMethod = (actionType) => {
  const normalized = String(actionType || '').trim().toLowerCase();
  if (normalized === 'http_get' || normalized === 'browser_navigate') return 'GET';
  return String(actionType || '').trim().toUpperCase() || 'GET';
};

export const deriveLlmSurfaceRowsFromRuntimeSummary = (summary = null, runId = '') => {
  const shapedSummary = summary && typeof summary === 'object' ? summary : null;
  const rowsBySurfaceId = new Map();
  const latestActionReceipts = Array.isArray(shapedSummary?.latestActionReceipts)
    ? shapedSummary.latestActionReceipts
    : [];
  latestActionReceipts.forEach((receipt) => {
    const surfaceId = llmReceiptSurfaceId(receipt);
    if (!surfaceId) return;
    const key = `${String(runId || 'unknown').trim()}:${surfaceId}`;
    const nextState = llmSurfaceState(receipt);
    const entry = rowsBySurfaceId.get(key) || {
      key,
      runId: String(runId || '').trim(),
      surfaceId,
      surfaceLabel: llmSurfaceLabel(surfaceId),
      surfaceState: nextState,
      coverageStatus: 'attempt_observed',
      successContract: 'runtime_action_observed',
      dependencyKind: 'independent',
      dependencySurfaceIds: [],
      attemptCount: 0,
      sampleRequestMethod: llmRequestMethod(receipt.actionType),
      sampleRequestPath: String(receipt.path || '').trim(),
      sampleResponseStatus:
        Number.isFinite(Number(receipt.status)) ? Number(receipt.status) : null
    };
    entry.attemptCount += 1;
    entry.surfaceState = mergeLlmSurfaceState(entry.surfaceState, nextState);
    if (!entry.sampleRequestPath && String(receipt.path || '').trim()) {
      entry.sampleRequestMethod = llmRequestMethod(receipt.actionType);
      entry.sampleRequestPath = String(receipt.path || '').trim();
      entry.sampleResponseStatus =
        Number.isFinite(Number(receipt.status)) ? Number(receipt.status) : null;
    } else if (entry.sampleResponseStatus === null && Number.isFinite(Number(receipt.status))) {
      entry.sampleResponseStatus = Number(receipt.status);
    }
    rowsBySurfaceId.set(key, entry);
  });
  return Array.from(rowsBySurfaceId.values());
};

const summarizeLlmSurfaceCoverageRows = (rows = []) => {
  const normalizedRows = Array.isArray(rows) ? rows : [];
  if (normalizedRows.length === 0) return null;
  const rowsBySurfaceId = new Map();
  normalizedRows.forEach((row) => {
    const surfaceId = String(row?.surfaceId || '').trim();
    if (!surfaceId) return;
    const entry = rowsBySurfaceId.get(surfaceId) || {
      surfaceId,
      surfaceLabel: String(row?.surfaceLabel || '').trim() || llmSurfaceLabel(surfaceId),
      coverageStatus: String(row?.coverageStatus || '').trim() || 'attempt_observed',
      surfaceState: String(row?.surfaceState || '').trim() || 'attempted',
      attemptCount: 0,
      sampleRequestMethod: String(row?.sampleRequestMethod || '').trim(),
      sampleRequestPath: String(row?.sampleRequestPath || '').trim(),
      sampleResponseStatus: Number.isFinite(Number(row?.sampleResponseStatus))
        ? Number(row.sampleResponseStatus)
        : null
    };
    entry.attemptCount += Number(row?.attemptCount || 0);
    if (entry.surfaceState !== 'leaked' && String(row?.surfaceState || '').trim() === 'leaked') {
      entry.surfaceState = 'leaked';
      entry.coverageStatus = 'progress_observed';
    } else if (
      entry.surfaceState === 'attempted'
      && String(row?.surfaceState || '').trim() === 'held'
    ) {
      entry.surfaceState = 'held';
      if (entry.coverageStatus !== 'progress_observed') {
        entry.coverageStatus = 'response_observed';
      }
    }
    if (!entry.sampleRequestPath && String(row?.sampleRequestPath || '').trim()) {
      entry.sampleRequestMethod = String(row?.sampleRequestMethod || '').trim();
      entry.sampleRequestPath = String(row?.sampleRequestPath || '').trim();
      entry.sampleResponseStatus = Number.isFinite(Number(row?.sampleResponseStatus))
        ? Number(row.sampleResponseStatus)
        : null;
    } else if (entry.sampleResponseStatus === null && Number.isFinite(Number(row?.sampleResponseStatus))) {
      entry.sampleResponseStatus = Number(row.sampleResponseStatus);
    }
    rowsBySurfaceId.set(surfaceId, entry);
  });
  if (rowsBySurfaceId.size === 0) return null;
  const receipts = Array.from(rowsBySurfaceId.values());
  const observedSurfaceIds = receipts.map((receipt) => receipt.surfaceId);
  const responseSurfaceIds = receipts
    .filter((receipt) => receipt.coverageStatus !== 'attempt_observed')
    .map((receipt) => receipt.surfaceId);
  const progressSurfaceIds = receipts
    .filter((receipt) => receipt.surfaceState === 'leaked')
    .map((receipt) => receipt.surfaceId);
  let overallStatus = 'attempt_observed';
  if (progressSurfaceIds.length === observedSurfaceIds.length) overallStatus = 'progress_observed';
  else if (progressSurfaceIds.length > 0) overallStatus = 'partial_progress';
  else if (responseSurfaceIds.length > 0) overallStatus = 'response_observed';
  return {
    overallStatus,
    observedSurfaceIds,
    observedSurfaceCount: observedSurfaceIds.length,
    responseSurfaceIds,
    responseSurfaceCount: responseSurfaceIds.length,
    progressSurfaceIds,
    progressSurfaceCount: progressSurfaceIds.length,
    surfaceLabels: Object.fromEntries(receipts.map((receipt) => [receipt.surfaceId, receipt.surfaceLabel])),
    receipts
  };
};

const deriveCoverageSummary = (
  observedSurfaceCoverage = null,
  ownedSurfaceCoverage = null,
  llmSurfaceCoverage = null
) => {
  const totalSurfaceCount = CANONICAL_ADVERSARY_COVERAGE_SURFACE_IDS.length;
  if (observedSurfaceCoverage && typeof observedSurfaceCoverage === 'object') {
    const touchedSurfaceCount = canonicalizeAdversaryCoverageSurfaceIds(
      observedSurfaceCoverage.observedSurfaceIds
    ).length;
    return {
      kind: 'shared_observed_surface',
      overallStatus: observedSurfaceCoverage.overallStatus,
      touchedSurfaceCount,
      totalSurfaceCount,
      evidenceLabel: ''
    };
  }
  if (ownedSurfaceCoverage && typeof ownedSurfaceCoverage === 'object') {
    const touchedSurfaceIds = canonicalizeAdversaryCoverageSurfaceIds(
      (Array.isArray(ownedSurfaceCoverage.receipts) ? ownedSurfaceCoverage.receipts : [])
        .filter((receipt) => (
          Number(receipt?.attemptCount || 0) > 0
          || (
            String(receipt?.coverageStatus || '').trim()
            && String(receipt?.coverageStatus || '').trim() !== 'unavailable'
          )
        ))
        .map((receipt) => receipt.surfaceId)
    );
    return {
      kind: 'required_surface_closure',
      overallStatus: ownedSurfaceCoverage.overallStatus,
      touchedSurfaceCount: touchedSurfaceIds.length,
      totalSurfaceCount,
      evidenceLabel: 'Receipt projected only'
    };
  }
  if (llmSurfaceCoverage && typeof llmSurfaceCoverage === 'object') {
    const touchedSurfaceCount = canonicalizeAdversaryCoverageSurfaceIds(
      llmSurfaceCoverage.observedSurfaceIds
    ).length;
    return {
      kind: 'llm_surface_observation',
      overallStatus: llmSurfaceCoverage.overallStatus,
      touchedSurfaceCount,
      totalSurfaceCount,
      evidenceLabel: 'Receipt projected only'
    };
  }
  return null;
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
      scraplingActivityCount: Number(
        summary?.scraplingActivityCount || summary?.scrapling_activity_count || 0
      ),
      monitoringEventCount: Number(summary?.monitoring_event_count || 0),
      defenseDeltaCount: Number(summary?.defense_delta_count || 0),
      defenseRows: [],
      observedFulfillmentModes: toSummaryStringArray(
        summary?.observedFulfillmentModes || summary?.observed_fulfillment_modes
      ),
      observedCategoryIds: toSummaryStringArray(
        summary?.observedCategoryIds || summary?.observed_category_ids
      ),
      observedSurfaceCoverage: shapeObservedSurfaceCoverage(
        summary?.observedSurfaceCoverage || summary?.observed_surface_coverage
      ),
      ownedSurfaceCoverage: shapeOwnedSurfaceCoverage(
        summary?.ownedSurfaceCoverage || summary?.owned_surface_coverage
      ),
      identitySummary: shapeIdentitySummary(
        summary?.identitySummary || summary?.identity_summary
      ),
      transportSummary: shapeTransportSummary(
        summary?.transportSummary || summary?.transport_summary
      ),
      latestScraplingRealismReceipt: shapeIdentityRealismReceipt(
        summary?.latestScraplingRealismReceipt || summary?.latest_scrapling_realism_receipt
      ),
      llmRuntimeSummary: shapeLlmRuntimeSummary(
        summary?.llmRuntimeSummary || summary?.llm_runtime_summary
      ),
      llmSurfaceCoverage: shapeLlmSurfaceCoverage(
        summary?.llmSurfaceCoverage || summary?.llm_surface_coverage
      ),
      banOutcomeCount: Number(summary?.ban_outcome_count || 0)
    }))
    .map((row) => ({
      ...row,
      llmSurfaceCoverage:
        row.llmSurfaceCoverage
        || summarizeLlmSurfaceCoverageRows(
          deriveLlmSurfaceRowsFromRuntimeSummary(row.llmRuntimeSummary, row.runId)
        )
    }))
    .filter((row) => row.runId.length > 0);
  return shapeAdversaryRunRows(shapedRows, bans);
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

const IDENTITY_PROVENANCE_LABELS = Object.freeze({
  trusted_ingress_backed: 'Trusted ingress IP observed',
  pool_backed: 'Pool-backed proxy identity',
  fixed_proxy: 'Fixed proxy identity',
  bucket_only: 'Bucketed identity only',
  degraded_local: 'Degraded local identity'
});

export const formatIdentityRealismSummary = (receipt = null) => {
  const shaped = receipt && typeof receipt === 'object' ? receipt : null;
  if (!shaped) return '';
  const provenanceMode = String(shaped.identityProvenanceMode || shaped.identity_provenance_mode || '').trim();
  const identityStatus = String(shaped.identityRealismStatus || shaped.identity_realism_status || '').trim();
  const modes = toSummaryStringArray(shaped.modes).length > 0
    ? toSummaryStringArray(shaped.modes)
    : toSummaryStringArray([provenanceMode || identityStatus]);
  const labels = Array.from(new Set(
    modes
      .map((mode) => (
        IDENTITY_PROVENANCE_LABELS[mode]
        || (mode ? formatMetricLabel(mode) : '')
      ))
      .filter(Boolean)
  ));
  const countries = toSummaryStringArray(
    shaped.observedCountryCodes || shaped.observed_country_codes
  );
  if (labels.length === 0 && countries.length === 0) return '';
  if (labels.length === 0) return countries.join(', ');
  const summary = labels.length === 1 ? labels[0] : `Mixed: ${labels.join(', ')}`;
  return countries.length > 0 ? `${summary} (${countries.join(', ')})` : summary;
};

export const formatTransportRealismSummary = (receipt = null) => {
  const shaped = receipt && typeof receipt === 'object' ? receipt : null;
  if (!shaped) return '';
  const formatTransportLabel = (value) => formatMetricLabel(value)
    .replace(/\bAi\b/g, 'AI')
    .replace(/\bCdp\b/g, 'CDP')
    .replace(/\bHttp\b/g, 'HTTP')
    .replace(/\bIp\b/g, 'IP')
    .replace(/\bJs\b/g, 'JS')
    .replace(/\bTls\b/g, 'TLS');
  const realismClass = String(
    shaped.transportRealismClass || shaped.transport_realism_class || ''
  ).trim();
  const transportProfile = String(
    shaped.transportProfile || shaped.transport_profile || ''
  ).trim();
  const modes = toSummaryStringArray(shaped.modes).length > 0
    ? toSummaryStringArray(shaped.modes)
    : toSummaryStringArray([realismClass || transportProfile]);
  const labels = Array.from(new Set(
    modes.map((mode) => formatTransportLabel(mode)).filter(Boolean)
  ));
  const degradedReasons = toSummaryStringArray(
    shaped.degradedReasons || shaped.degraded_reasons || [
      shaped.transportDegradedReason || shaped.transport_degraded_reason || ''
    ]
  );
  const degradedLabels = Array.from(new Set(
    degradedReasons.map((reason) => formatTransportLabel(reason)).filter(Boolean)
  ));
  if (labels.length === 0 && degradedLabels.length === 0) return '';
  const summary = labels.length === 0
    ? ''
    : (labels.length === 1 ? labels[0] : `Mixed: ${labels.join(', ')}`);
  if (degradedLabels.length === 0) return summary;
  const degradedSummary = degradedLabels.length === 1
    ? degradedLabels[0]
    : `Mixed: ${degradedLabels.join(', ')}`;
  if (!summary) return degradedSummary;
  return `${summary} (${degradedSummary})`;
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
    typeof prometheusData.endpoint === 'string' ? prometheusData.endpoint : '/shuma/metrics';
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
