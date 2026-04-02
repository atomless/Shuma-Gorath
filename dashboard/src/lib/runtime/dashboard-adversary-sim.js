// @ts-check

import { formatAdversarySimTransitionReasonCopy } from '../domain/adversary-sim.js';

const DEFAULT_DURATION_SECONDS = 180;
const ADVERSARY_SIM_LANES = Object.freeze([
  'synthetic_traffic',
  'scrapling_traffic',
  'bot_red_team',
  'parallel_mixed_traffic'
]);
const hasOwn = (value, key) => Object.prototype.hasOwnProperty.call(value, key);

/**
 * @param {unknown} value
 * @returns {'' | 'runtime-dev' | 'runtime-prod'}
 */
function normalizeRuntimeEnvironment(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === 'runtime-dev' || normalized === 'runtime-prod') {
    return normalized;
  }
  return '';
}

/**
 * @param {Record<string, unknown>} source
 * @param {string} snakeCaseKey
 * @param {string} camelCaseKey
 * @param {unknown} fallback
 * @returns {unknown}
 */
function pick(source, snakeCaseKey, camelCaseKey, fallback = undefined) {
  if (hasOwn(source, snakeCaseKey)) {
    return source[snakeCaseKey];
  }
  if (hasOwn(source, camelCaseKey)) {
    return source[camelCaseKey];
  }
  return fallback;
}

/**
 * @param {unknown} value
 * @param {number} fallback
 * @returns {number}
 */
function toSafeNumber(value, fallback = 0) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return numeric;
}

/**
 * @param {unknown} value
 * @returns {Record<string, unknown>}
 */
function asRecord(value) {
  return value && typeof value === 'object'
    ? /** @type {Record<string, unknown>} */ (value)
    : {};
}

/**
 * @param {unknown} value
 * @param {string} fallback
 * @returns {string}
 */
function normalizeLane(value, fallback = 'scrapling_traffic') {
  const normalized = String(value || '').trim().toLowerCase();
  return ADVERSARY_SIM_LANES.includes(normalized) ? normalized : fallback;
}

/**
 * @param {unknown} value
 * @returns {string}
 */
function normalizeOptionalLane(value) {
  const normalized = String(value || '').trim().toLowerCase();
  return ADVERSARY_SIM_LANES.includes(normalized) ? normalized : '';
}

/**
 * @param {unknown} value
 * @returns {Record<string, number>}
 */
function normalizeCountMap(value) {
  const source = asRecord(value);
  return Object.entries(source).reduce((next, [key, rawValue]) => {
    const normalizedKey = String(key || '').trim();
    if (!normalizedKey) return next;
    next[normalizedKey] = Math.max(0, Math.floor(toSafeNumber(rawValue, 0)));
    return next;
  }, /** @type {Record<string, number>} */ ({}));
}

/**
 * @param {Record<string, unknown>} source
 * @returns {{
 *   beatAttempts: number,
 *   beatSuccesses: number,
 *   beatFailures: number,
 *   generatedRequests: number,
 *   blockedRequests: number,
 *   offsiteRequests: number,
 *   responseBytes: number,
 *   responseStatusCount: Record<string, number>,
 *   lastGeneratedAt: number,
 *   lastError: string
 * }}
 */
function normalizeLaneCounterState(source = {}) {
  return {
    beatAttempts: Math.max(0, Math.floor(toSafeNumber(pick(source, 'beat_attempts', 'beatAttempts', 0), 0))),
    beatSuccesses: Math.max(0, Math.floor(toSafeNumber(pick(source, 'beat_successes', 'beatSuccesses', 0), 0))),
    beatFailures: Math.max(0, Math.floor(toSafeNumber(pick(source, 'beat_failures', 'beatFailures', 0), 0))),
    generatedRequests: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'generated_requests', 'generatedRequests', 0), 0))
    ),
    blockedRequests: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'blocked_requests', 'blockedRequests', 0), 0))
    ),
    offsiteRequests: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'offsite_requests', 'offsiteRequests', 0), 0))
    ),
    responseBytes: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'response_bytes', 'responseBytes', 0), 0))
    ),
    responseStatusCount: normalizeCountMap(
      pick(source, 'response_status_count', 'responseStatusCount', {})
    ),
    lastGeneratedAt: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'last_generated_at', 'lastGeneratedAt', 0), 0))
    ),
    lastError: String(pick(source, 'last_error', 'lastError', '') || '')
  };
}

/**
 * @param {Record<string, unknown>} source
 * @returns {{ count: number, lastSeenAt: number }}
 */
function normalizeFailureClassCounter(source = {}) {
  return {
    count: Math.max(0, Math.floor(toSafeNumber(pick(source, 'count', 'count', 0), 0))),
    lastSeenAt: Math.max(0, Math.floor(toSafeNumber(pick(source, 'last_seen_at', 'lastSeenAt', 0), 0)))
  };
}

/**
 * @param {unknown} value
 * @returns {{
 *   schemaVersion: string,
 *   lanes: {
 *     syntheticTraffic: ReturnType<typeof normalizeLaneCounterState>,
 *     scraplingTraffic: ReturnType<typeof normalizeLaneCounterState>,
 *     botRedTeam: ReturnType<typeof normalizeLaneCounterState>
 *   },
 *   requestFailureClasses: {
 *     cancelled: ReturnType<typeof normalizeFailureClassCounter>,
 *     timeout: ReturnType<typeof normalizeFailureClassCounter>,
 *     transport: ReturnType<typeof normalizeFailureClassCounter>,
 *     http: ReturnType<typeof normalizeFailureClassCounter>
 *   }
 * }}
 */
function normalizeLaneDiagnostics(value) {
  const source = asRecord(value);
  const lanesRaw = asRecord(pick(source, 'lanes', 'lanes', {}));
  const failureClassesRaw = asRecord(
    pick(source, 'request_failure_classes', 'requestFailureClasses', {})
  );
  return {
    schemaVersion: String(pick(source, 'schema_version', 'schemaVersion', '') || ''),
    truthBasis: String(pick(source, 'truth_basis', 'truthBasis', '') || ''),
    lanes: {
      syntheticTraffic: normalizeLaneCounterState(
        asRecord(pick(lanesRaw, 'synthetic_traffic', 'syntheticTraffic', {}))
      ),
      scraplingTraffic: normalizeLaneCounterState(
        asRecord(pick(lanesRaw, 'scrapling_traffic', 'scraplingTraffic', {}))
      ),
      botRedTeam: normalizeLaneCounterState(
        asRecord(pick(lanesRaw, 'bot_red_team', 'botRedTeam', {}))
      )
    },
    requestFailureClasses: {
      cancelled: normalizeFailureClassCounter(
        asRecord(pick(failureClassesRaw, 'cancelled', 'cancelled', {}))
      ),
      timeout: normalizeFailureClassCounter(
        asRecord(pick(failureClassesRaw, 'timeout', 'timeout', {}))
      ),
      transport: normalizeFailureClassCounter(
        asRecord(pick(failureClassesRaw, 'transport', 'transport', {}))
      ),
      http: normalizeFailureClassCounter(
        asRecord(pick(failureClassesRaw, 'http', 'http', {}))
      )
    }
  };
}

/**
 * @param {unknown} value
 * @returns {{
 *   runId: string,
 *   lane: string,
 *   profile: string,
 *   monitoringEventCount: number,
 *   defenseDeltaCount: number,
 *   banOutcomeCount: number,
 *   firstObservedAt: number,
 *   lastObservedAt: number,
 *   truthBasis: string
 * } | null}
 */
function normalizePersistedEventEvidence(value) {
  if (!value || typeof value !== 'object') return null;
  const source = /** @type {Record<string, unknown>} */ (value);
  return {
    runId: String(pick(source, 'run_id', 'runId', '') || ''),
    lane: normalizeOptionalLane(pick(source, 'lane', 'lane', '')),
    profile: String(pick(source, 'profile', 'profile', '') || ''),
    monitoringEventCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'monitoring_event_count', 'monitoringEventCount', 0), 0))
    ),
    defenseDeltaCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'defense_delta_count', 'defenseDeltaCount', 0), 0))
    ),
    banOutcomeCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'ban_outcome_count', 'banOutcomeCount', 0), 0))
    ),
    firstObservedAt: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'first_observed_at', 'firstObservedAt', 0), 0))
    ),
    lastObservedAt: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'last_observed_at', 'lastObservedAt', 0), 0))
    ),
    truthBasis: String(pick(source, 'truth_basis', 'truthBasis', '') || '')
  };
}

function normalizeStringArray(value) {
  if (!Array.isArray(value)) return [];
  return value
    .map((entry) => String(entry || '').trim())
    .filter(Boolean);
}

function normalizeRepresentativenessLaneReadiness(value) {
  const source = asRecord(value);
  return {
    status: String(pick(source, 'status', 'status', '') || ''),
    summary: String(pick(source, 'summary', 'summary', '') || ''),
    blockers: normalizeStringArray(pick(source, 'blockers', 'blockers', []))
  };
}

function normalizeRepresentativenessReadiness(value) {
  const source = asRecord(value);
  const prerequisites = asRecord(pick(source, 'prerequisites', 'prerequisites', {}));
  const laneStatuses = asRecord(pick(source, 'lane_statuses', 'laneStatuses', {}));
  return {
    status: String(pick(source, 'status', 'status', '') || ''),
    summary: String(pick(source, 'summary', 'summary', '') || ''),
    blockers: normalizeStringArray(pick(source, 'blockers', 'blockers', [])),
    representativeHostileLaneCount: Math.max(
      0,
      Math.floor(
        toSafeNumber(
          pick(source, 'representative_hostile_lane_count', 'representativeHostileLaneCount', 0),
          0
        )
      )
    ),
    partiallyRepresentativeHostileLaneCount: Math.max(
      0,
      Math.floor(
        toSafeNumber(
          pick(
            source,
            'partially_representative_hostile_lane_count',
            'partiallyRepresentativeHostileLaneCount',
            0
          ),
          0
        )
      )
    ),
    hostileLaneCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'hostile_lane_count', 'hostileLaneCount', 0), 0))
    ),
    prerequisites: {
      trustedIngressConfigured: pick(
        prerequisites,
        'trusted_ingress_configured',
        'trustedIngressConfigured',
        false
      ) === true,
      scraplingRequestProxyPoolCount: Math.max(
        0,
        Math.floor(
          toSafeNumber(
            pick(
              prerequisites,
              'scrapling_request_proxy_pool_count',
              'scraplingRequestProxyPoolCount',
              0
            ),
            0
          )
        )
      ),
      scraplingBrowserProxyPoolCount: Math.max(
        0,
        Math.floor(
          toSafeNumber(
            pick(
              prerequisites,
              'scrapling_browser_proxy_pool_count',
              'scraplingBrowserProxyPoolCount',
              0
            ),
            0
          )
        )
      ),
      agenticRequestProxyPoolCount: Math.max(
        0,
        Math.floor(
          toSafeNumber(
            pick(
              prerequisites,
              'agentic_request_proxy_pool_count',
              'agenticRequestProxyPoolCount',
              0
            ),
            0
          )
        )
      ),
      scraplingRequestProxyConfigured: pick(
        prerequisites,
        'scrapling_request_proxy_configured',
        'scraplingRequestProxyConfigured',
        false
      ) === true,
      scraplingBrowserProxyConfigured: pick(
        prerequisites,
        'scrapling_browser_proxy_configured',
        'scraplingBrowserProxyConfigured',
        false
      ) === true
    },
    laneStatuses: {
      syntheticTraffic: normalizeRepresentativenessLaneReadiness(
        pick(laneStatuses, 'synthetic_traffic', 'syntheticTraffic', {})
      ),
      scraplingTraffic: normalizeRepresentativenessLaneReadiness(
        pick(laneStatuses, 'scrapling_traffic', 'scraplingTraffic', {})
      ),
      botRedTeam: normalizeRepresentativenessLaneReadiness(
        pick(laneStatuses, 'bot_red_team', 'botRedTeam', {})
      ),
      parallelMixedTraffic: normalizeRepresentativenessLaneReadiness(
        pick(laneStatuses, 'parallel_mixed_traffic', 'parallelMixedTraffic', {})
      )
    }
  };
}

/**
 * @param {unknown} value
 * @returns {'off' | 'running' | 'stopping'}
 */
function normalizePhase(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === 'running' || normalized === 'stopping') return normalized;
  return 'off';
}

/**
 * @param {unknown} payload
 * @returns {{
 *   runtimeEnvironment: string,
 *   available: boolean,
 *   enabled: boolean,
 *   generationActive: boolean,
 *   historicalDataVisible: boolean,
 *   historyRetentionHours: number,
 *   historyCleanupSupported: boolean,
 *   historyCleanupCommand: string,
 *   phase: 'off' | 'running' | 'stopping',
 *   runId: string,
 *   startedAt: number,
 *   endsAt: number,
 *   durationSeconds: number,
 *   remainingSeconds: number,
 *   activeRunCount: number,
 *   activeLaneCount: number,
 *   desiredLane: string,
 *   activeLane: string,
 *   laneSwitchSeq: number,
 *   lastLaneSwitchAt: number,
 *   lastLaneSwitchReason: string,
 *   lastTransitionReason: string,
 *   queuePolicy: string,
 *   laneDiagnostics: ReturnType<typeof normalizeLaneDiagnostics>,
 *   supervisor: {
 *     owner: string,
 *     cadenceSeconds: number,
 *     maxCatchupTicksPerInvocation: number,
 *     heartbeatActive: boolean,
 *     workerActive: boolean,
 *     lastHeartbeatAt: number,
 *     idleSeconds: number,
 *     offStateInert: boolean,
 *     triggerSurface: string
 *   },
 *   generationDiagnostics: {
 *     health: string,
 *     reason: string,
 *     recommendedAction: string,
 *     generatedTickCount: number,
 *     generatedRequestCount: number,
 *     lastGeneratedAt: number,
 *     lastGenerationError: string,
 *     truthBasis: string
 *   },
 *   persistedEventEvidence: ReturnType<typeof normalizePersistedEventEvidence>,
 *   representativenessReadiness: ReturnType<typeof normalizeRepresentativenessReadiness>
 * }}
 */
export function normalizeAdversarySimStatus(payload) {
  const source = payload && typeof payload === 'object'
    ? /** @type {Record<string, unknown>} */ (payload)
    : {};
  const historyRetentionRaw = pick(source, 'history_retention', 'historyRetention', {});
  const historyRetention = historyRetentionRaw && typeof historyRetentionRaw === 'object'
    ? /** @type {Record<string, unknown>} */ (historyRetentionRaw)
    : {};
  const generationDiagnosticsRaw = pick(source, 'generation_diagnostics', 'generationDiagnostics', {});
  const generationDiagnostics = generationDiagnosticsRaw && typeof generationDiagnosticsRaw === 'object'
    ? /** @type {Record<string, unknown>} */ (generationDiagnosticsRaw)
    : {};
  const laneDiagnosticsRaw = pick(source, 'lane_diagnostics', 'laneDiagnostics', {});
  const supervisorRaw = pick(source, 'supervisor', 'supervisor', {});
  const supervisor = supervisorRaw && typeof supervisorRaw === 'object'
    ? /** @type {Record<string, unknown>} */ (supervisorRaw)
    : {};
  const representativenessReadinessRaw = pick(
    source,
    'representativeness_readiness',
    'representativenessReadiness',
    {}
  );
  const phase = normalizePhase(pick(source, 'phase', 'phase', 'off'));
  const durationSeconds = Math.max(
    1,
    Math.floor(toSafeNumber(
      pick(source, 'duration_seconds', 'durationSeconds', DEFAULT_DURATION_SECONDS),
      DEFAULT_DURATION_SECONDS
    ))
  );
  const startedAt = Math.max(
    0,
    Math.floor(toSafeNumber(pick(source, 'started_at', 'startedAt', 0), 0))
  );
  const endsAt = Math.max(
    0,
    Math.floor(toSafeNumber(pick(source, 'ends_at', 'endsAt', 0), 0))
  );
  const runIdValue = pick(source, 'run_id', 'runId', '');
  return {
    runtimeEnvironment: String(pick(source, 'runtime_environment', 'runtimeEnvironment', '') || ''),
    available: pick(source, 'adversary_sim_available', 'available', false) === true,
    enabled: pick(source, 'adversary_sim_enabled', 'enabled', false) === true,
    generationActive: pick(source, 'generation_active', 'generationActive', phase === 'running') === true,
    historicalDataVisible: pick(source, 'historical_data_visible', 'historicalDataVisible', true) !== false,
    historyRetentionHours: Math.max(
      0,
      Math.floor(toSafeNumber(pick(historyRetention, 'retention_hours', 'retentionHours', 168), 168))
    ),
    historyCleanupSupported: pick(historyRetention, 'cleanup_supported', 'cleanupSupported', false) === true,
    historyCleanupCommand: String(pick(historyRetention, 'cleanup_command', 'cleanupCommand', '') || ''),
    phase,
    runId: typeof runIdValue === 'string' ? runIdValue : '',
    startedAt,
    endsAt,
    durationSeconds,
    remainingSeconds: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'remaining_seconds', 'remainingSeconds', 0), 0))
    ),
    activeRunCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'active_run_count', 'activeRunCount', 0), 0))
    ),
    activeLaneCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'active_lane_count', 'activeLaneCount', 0), 0))
    ),
    desiredLane: normalizeLane(pick(source, 'desired_lane', 'desiredLane', 'scrapling_traffic')),
    activeLane: normalizeOptionalLane(pick(source, 'active_lane', 'activeLane', '')),
    laneSwitchSeq: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'lane_switch_seq', 'laneSwitchSeq', 0), 0))
    ),
    lastLaneSwitchAt: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'last_lane_switch_at', 'lastLaneSwitchAt', 0), 0))
    ),
    lastLaneSwitchReason: String(
      pick(source, 'last_lane_switch_reason', 'lastLaneSwitchReason', '') || ''
    ),
    lastTransitionReason: String(
      pick(source, 'last_transition_reason', 'lastTransitionReason', '') || ''
    ),
    queuePolicy: String(pick(source, 'queue_policy', 'queuePolicy', '') || ''),
    laneDiagnostics: normalizeLaneDiagnostics(laneDiagnosticsRaw),
    supervisor: {
      owner: String(pick(supervisor, 'owner', 'owner', '') || ''),
      cadenceSeconds: Math.max(
        1,
        Math.floor(toSafeNumber(pick(supervisor, 'cadence_seconds', 'cadenceSeconds', 1), 1))
      ),
      maxCatchupTicksPerInvocation: Math.max(
        1,
        Math.floor(
          toSafeNumber(
            pick(
              supervisor,
              'max_catchup_ticks_per_invocation',
              'maxCatchupTicksPerInvocation',
              1
            ),
            1
          )
        )
      ),
      heartbeatActive: pick(supervisor, 'heartbeat_active', 'heartbeatActive', phase === 'running') === true,
      workerActive: pick(supervisor, 'worker_active', 'workerActive', phase === 'running') === true,
      lastHeartbeatAt: Math.max(
        0,
        Math.floor(toSafeNumber(pick(supervisor, 'last_heartbeat_at', 'lastHeartbeatAt', 0), 0))
      ),
      idleSeconds: Math.max(
        0,
        Math.floor(toSafeNumber(pick(supervisor, 'idle_seconds', 'idleSeconds', 0), 0))
      ),
      offStateInert: pick(supervisor, 'off_state_inert', 'offStateInert', false) === true,
      triggerSurface: String(pick(supervisor, 'trigger_surface', 'triggerSurface', '') || '')
    },
    generationDiagnostics: {
      health: String(pick(generationDiagnostics, 'health', 'health', '') || ''),
      reason: String(pick(generationDiagnostics, 'reason', 'reason', '') || ''),
      recommendedAction: String(
        pick(generationDiagnostics, 'recommended_action', 'recommendedAction', '') || ''
      ),
      generatedTickCount: Math.max(
        0,
        Math.floor(toSafeNumber(pick(generationDiagnostics, 'generated_tick_count', 'generatedTickCount', 0), 0))
      ),
      generatedRequestCount: Math.max(
        0,
        Math.floor(toSafeNumber(pick(generationDiagnostics, 'generated_request_count', 'generatedRequestCount', 0), 0))
      ),
      lastGeneratedAt: Math.max(
        0,
        Math.floor(toSafeNumber(pick(generationDiagnostics, 'last_generated_at', 'lastGeneratedAt', 0), 0))
      ),
      lastGenerationError: String(
        pick(generationDiagnostics, 'last_generation_error', 'lastGenerationError', '') || ''
      ),
      truthBasis: String(
        pick(generationDiagnostics, 'truth_basis', 'truthBasis', '') || ''
      )
    },
    persistedEventEvidence: normalizePersistedEventEvidence(
      pick(source, 'persisted_event_evidence', 'persistedEventEvidence', null)
    ),
    representativenessReadiness: normalizeRepresentativenessReadiness(
      representativenessReadinessRaw
    )
  };
}

/**
 * @param {{ configRuntimeSnapshot?: Record<string, unknown>, adversarySimStatus?: unknown }} source
 * @returns {{ runtimeEnvironment: '' | 'runtime-dev' | 'runtime-prod', surfaceAvailable: boolean, controlAvailable: boolean }}
 */
export function deriveAdversarySimControlState(source = {}) {
  const configRuntimeSnapshot =
    source && source.configRuntimeSnapshot && typeof source.configRuntimeSnapshot === 'object'
      ? /** @type {Record<string, unknown>} */ (source.configRuntimeSnapshot)
      : {};
  const normalizedStatus = normalizeAdversarySimStatus(source?.adversarySimStatus);
  const runtimeEnvironment = normalizeRuntimeEnvironment(
    normalizedStatus.runtimeEnvironment || configRuntimeSnapshot.runtime_environment || ''
  );
  const surfaceAvailable =
    normalizedStatus.available === true || configRuntimeSnapshot.adversary_sim_available === true;
  return {
    runtimeEnvironment,
    surfaceAvailable,
    controlAvailable: surfaceAvailable === true
  };
}

/**
 * @param {{
 *   status?: unknown,
 *   controllerState?: Record<string, unknown> | null
 * }} source
 * @returns {string}
 */
export function deriveAdversarySimLifecycleCopy(source = {}) {
  const normalizedStatus = normalizeAdversarySimStatus(source?.status);
  const controllerState =
    source && source.controllerState && typeof source.controllerState === 'object'
      ? /** @type {Record<string, unknown>} */ (source.controllerState)
      : {};
  const controllerPhase = String(controllerState.controllerPhase || '').trim().toLowerCase();
  const uiDesiredEnabled = controllerState.uiDesiredEnabled === true;

  if (
    controllerPhase === 'debouncing' ||
    controllerPhase === 'submitting' ||
    controllerPhase === 'converging'
  ) {
    return uiDesiredEnabled
      ? 'Starting adversary simulation. Awaiting backend convergence.'
      : 'Stopping adversary simulation. Awaiting backend convergence.';
  }

  const generationDiagnostics = normalizedStatus.generationDiagnostics || {};
  const transitionReasonCopy = formatAdversarySimTransitionReasonCopy(
    normalizedStatus.lastTransitionReason,
    ''
  );
  if (normalizedStatus.generationActive) {
    const activePrefix = transitionReasonCopy
      ? `Generation active. ${transitionReasonCopy} `
      : 'Generation active. ';
    return String(generationDiagnostics.health || '') === 'ok'
      ? `${activePrefix}Auto-off stops new simulation traffic only; retained telemetry stays visible.`
      : `${activePrefix}${
        String(generationDiagnostics.recommendedAction || '').trim() ||
        'No observable traffic yet. Check supervisor diagnostics for stalled heartbeat state.'
      }`;
  }

  const retentionHours = Math.max(0, Number(normalizedStatus.historyRetentionHours || 0));
  const cleanupCommand =
    String(normalizedStatus.historyCleanupCommand || '').trim() || 'make telemetry-clean';
  const inactiveBase = normalizedStatus.historicalDataVisible
    ? `Generation inactive. Retained telemetry remains visible for ${retentionHours}h or until ${cleanupCommand} is run.`
    : 'Generation inactive.';
  return transitionReasonCopy ? `${inactiveBase} ${transitionReasonCopy}` : inactiveBase;
}

/**
 * @param {{
 *   status?: unknown,
 *   controllerState?: Record<string, unknown> | null,
 *   nowMs?: number
 * }} source
 * @returns {{ active: boolean, progressPercent: number, remainingMs: number }}
 */
export function deriveAdversarySimProgressState(source = {}) {
  const normalizedStatus = normalizeAdversarySimStatus(source?.status);
  const controllerState =
    source && source.controllerState && typeof source.controllerState === 'object'
      ? /** @type {Record<string, unknown>} */ (source.controllerState)
      : {};
  const controllerPhase = String(controllerState.controllerPhase || '').trim().toLowerCase();
  const uiDesiredEnabled = controllerState.uiDesiredEnabled === true;

  if (
    uiDesiredEnabled !== true ||
    controllerPhase === 'error' ||
    normalizedStatus.generationActive !== true ||
    normalizedStatus.phase !== 'running'
  ) {
    return {
      active: false,
      progressPercent: 0,
      remainingMs: 0
    };
  }

  const durationMs = Math.max(1, normalizedStatus.durationSeconds * 1000);
  const nowMs = Math.max(0, Number(source?.nowMs || Date.now()) || 0);
  const startedAtMs = normalizedStatus.startedAt > 0 ? normalizedStatus.startedAt * 1000 : 0;
  const endsAtMs = normalizedStatus.endsAt > 0 ? normalizedStatus.endsAt * 1000 : 0;

  let remainingMs = Math.max(0, normalizedStatus.remainingSeconds * 1000);
  let progressPercent = Math.max(0, Math.min(100, ((durationMs - remainingMs) / durationMs) * 100));

  if (startedAtMs > 0 && endsAtMs > startedAtMs) {
    const boundedNowMs = Math.min(Math.max(nowMs, startedAtMs), endsAtMs);
    const elapsedMs = boundedNowMs - startedAtMs;
    remainingMs = Math.max(0, endsAtMs - boundedNowMs);
    progressPercent = Math.max(0, Math.min(100, (elapsedMs / (endsAtMs - startedAtMs)) * 100));
  }

  return {
    active: true,
    progressPercent,
    remainingMs
  };
}

/**
 * @param {unknown} payload
 * @param {boolean} desiredEnabled
 * @returns {boolean}
 */
export function adversarySimStateMatchesDesired(payload, desiredEnabled) {
  const normalized = normalizeAdversarySimStatus(payload);
  const desired = desiredEnabled === true;
  if (desired) {
    return normalized.enabled === true;
  }
  return normalized.enabled !== true && normalized.generationActive !== true && normalized.phase === 'off';
}

/**
 * @param {unknown} error
 * @returns {boolean}
 */
export function isRetryableAdversarySimControlError(error) {
  const status = Number(error?.status || 0);
  return status === 409 || status === 429;
}

/**
 * @param {unknown} error
 * @returns {number}
 */
export function adversarySimControlRetryDelayMs(error) {
  const status = Number(error?.status || 0);
  const retryAfterSeconds = Number(error?.retryAfterSeconds || 0);
  if (Number.isFinite(retryAfterSeconds) && retryAfterSeconds > 0) {
    return retryAfterSeconds * 1000 + 250;
  }
  return status === 409 ? 2_000 : 1_100;
}

/**
 * @param {(desiredEnabled: boolean) => Promise<unknown>} controlFn
 * @param {boolean} desiredEnabled
 * @param {{ timeoutMs?: number, sleep?: (ms: number) => Promise<void> }} [options]
 * @returns {Promise<unknown>}
 */
export async function controlAdversarySimWithRetry(
  controlFn,
  desiredEnabled,
  options = {}
) {
  if (typeof controlFn !== 'function') {
    throw new Error('Adversary sim control callback is required.');
  }
  const timeoutMs = Math.max(1_000, Number(options.timeoutMs || 0) || 30_000);
  const sleep = typeof options.sleep === 'function'
    ? options.sleep
    : (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      return await controlFn(desiredEnabled === true);
    } catch (error) {
      lastError = error;
      if (!isRetryableAdversarySimControlError(error)) {
        throw error;
      }
      const remainingMs = deadline - Date.now();
      if (remainingMs <= 0) break;
      const delayMs = Math.min(adversarySimControlRetryDelayMs(error), remainingMs);
      await sleep(delayMs);
    }
  }

  throw lastError || new Error('Adversary simulation control retry budget exhausted.');
}
