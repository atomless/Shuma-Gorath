// @ts-check

import {
  classifyRequestFailure,
  REQUEST_FAILURE_CLASSES
} from './core/request-failure.js';

/**
 * @typedef {Object} AdminContext
 * @property {string} endpoint
 * @property {string} apikey
 * @property {boolean} [sessionAuth]
 * @property {string} [csrfToken]
 */

/**
 * @typedef {Object} RequestOptions
 * @property {string} [method]
 * @property {HeadersInit} [headers]
 * @property {unknown} [json]
 * @property {BodyInit | null} [body]
 * @property {AbortSignal} [signal]
 * @property {number} [timeoutMs]
 * @property {RequestCache} [cache]
 * @property {HTMLElement | null} [messageTarget]
 * @property {{ tab?: string, reason?: string, source?: string }} [telemetry]
 */

const JSON_CONTENT_TYPE = 'application/json';
const DEFAULT_REQUEST_TIMEOUT_MS = 12000;
const isWriteMethod = (method) => {
  const upper = String(method || 'GET').toUpperCase();
  return upper === 'POST' || upper === 'PUT' || upper === 'PATCH' || upper === 'DELETE';
};

const newIdempotencyKey = () => {
  if (typeof crypto !== 'undefined' && crypto && typeof crypto.randomUUID === 'function') {
    return `dash-${crypto.randomUUID()}`;
  }
  const randomPart = Math.floor(Math.random() * 0x1_0000_0000).toString(16).padStart(8, '0');
  return `dash-${Date.now().toString(16)}-${randomPart}`;
};

/**
 * @param {string} message
 * @param {number} status
 * @param {string} path
 * @param {string} method
 */
export function DashboardApiError(message, status, path, method) {
  const error = new Error(message);
  error.name = 'DashboardApiError';
  /** @type {number} */
  error.status = Number.isInteger(status) ? status : 0;
  /** @type {string} */
  error.path = String(path || '');
  /** @type {string} */
  error.method = String(method || 'GET').toUpperCase();
  /** @type {number | null} */
  error.retryAfterSeconds = null;
  return error;
}

/**
 * @param {string | null} value
 * @returns {number | null}
 */
const parseRetryAfterSeconds = (value) => {
  const normalized = String(value || '').trim();
  if (!normalized) return null;
  const seconds = Number.parseInt(normalized, 10);
  if (!Number.isFinite(seconds) || seconds <= 0) return null;
  return seconds;
};

/**
 * @param {unknown} value
 * @returns {Record<string, unknown>}
 */
const asRecord = (value) =>
  value && typeof value === 'object' ? /** @type {Record<string, unknown>} */ (value) : {};

const ADVERSARY_SIM_LANES = Object.freeze([
  'synthetic_traffic',
  'scrapling_traffic',
  'bot_red_team'
]);

/**
 * @param {unknown} value
 * @param {string} fallback
 * @returns {string}
 */
const normalizeAdversarySimLane = (value, fallback = 'scrapling_traffic') => {
  const normalized = String(value || '').trim().toLowerCase();
  return ADVERSARY_SIM_LANES.includes(normalized) ? normalized : fallback;
};

/**
 * @param {unknown} value
 * @returns {string}
 */
const normalizeOptionalAdversarySimLane = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  return ADVERSARY_SIM_LANES.includes(normalized) ? normalized : '';
};

/**
 * @param {unknown} value
 * @returns {number | null}
 */
const adaptOptionalNumber = (value) => {
  if (value === null || value === undefined || value === '') return null;
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : null;
};

/**
 * @param {unknown} value
 * @returns {Record<string, number>}
 */
const adaptCountMap = (value) => {
  const source = asRecord(value);
  return Object.entries(source).reduce((next, [key, rawValue]) => {
    const normalizedKey = String(key || '').trim();
    if (!normalizedKey) return next;
    next[normalizedKey] = Number(rawValue || 0);
    return next;
  }, /** @type {Record<string, number>} */ ({}));
};

/**
 * @param {unknown} value
 * @returns {Array<{ label: string, count: number }>}
 */
const adaptCountEntries = (value) => {
  if (!Array.isArray(value)) return [];
  return value
    .filter((entry) => entry && typeof entry === 'object')
    .map((entry) => {
      const record = asRecord(entry);
      return {
        label: String(record.label || ''),
        count: Number(record.count || 0)
      };
    })
    .filter((entry) => entry.label);
};

/**
 * @param {unknown} value
 */
const adaptNonHumanClassificationReadiness = (value) => {
  const source = asRecord(value);
  return {
    status: String(source.status || ''),
    blockers: Array.isArray(source.blockers)
      ? source.blockers.map((entry) => String(entry || ''))
      : [],
    live_receipt_count: Number(source.live_receipt_count || 0),
    adversary_sim_receipt_count: Number(source.adversary_sim_receipt_count || 0)
  };
};

/**
 * @param {unknown} value
 */
const adaptNonHumanCoverageSummary = (value) => {
  const source = asRecord(value);
  return {
    overall_status: String(source.overall_status || ''),
    blocking_reasons: Array.isArray(source.blocking_reasons)
      ? source.blocking_reasons.map((entry) => String(entry || ''))
      : [],
    blocking_category_ids: Array.isArray(source.blocking_category_ids)
      ? source.blocking_category_ids.map((entry) => String(entry || ''))
      : []
  };
};

/**
 * @param {unknown} value
 */
const adaptRecognitionComparisonRow = (value) => {
  const source = asRecord(value);
  return {
    category_id: String(source.category_id || ''),
    category_label: String(source.category_label || ''),
    inference_capability_status: String(source.inference_capability_status || ''),
    comparison_status: String(source.comparison_status || ''),
    inferred_category_id: String(source.inferred_category_id || ''),
    inferred_category_label: String(source.inferred_category_label || ''),
    exactness: String(source.exactness || ''),
    basis: String(source.basis || ''),
    note: String(source.note || ''),
    evidence_references: Array.isArray(source.evidence_references)
      ? source.evidence_references.map((entry) => String(entry || ''))
      : []
  };
};

/**
 * @param {unknown} value
 */
const adaptSimulatorGroundTruthCategory = (value) => {
  const source = asRecord(value);
  return {
    category_id: String(source.category_id || ''),
    category_label: String(source.category_label || ''),
    recent_run_count: Number(source.recent_run_count || 0),
    evidence_references: Array.isArray(source.evidence_references)
      ? source.evidence_references.map((entry) => String(entry || ''))
      : []
  };
};

/**
 * @param {unknown} value
 */
const adaptSimulatorGroundTruthSummary = (value) => {
  const source = asRecord(value);
  return {
    status: String(source.status || ''),
    recent_sim_run_count: Number(source.recent_sim_run_count || 0),
    categories: asObjectArray(source.categories).map(adaptSimulatorGroundTruthCategory)
  };
};

/**
 * @param {unknown} value
 */
const adaptLaneCounterState = (value) => {
  const source = asRecord(value);
  return {
    beat_attempts: Number(source.beat_attempts || 0),
    beat_successes: Number(source.beat_successes || 0),
    beat_failures: Number(source.beat_failures || 0),
    generated_requests: Number(source.generated_requests || 0),
    blocked_requests: Number(source.blocked_requests || 0),
    offsite_requests: Number(source.offsite_requests || 0),
    response_bytes: Number(source.response_bytes || 0),
    response_status_count: adaptCountMap(source.response_status_count),
    last_generated_at: adaptOptionalNumber(source.last_generated_at),
    last_error: String(source.last_error || '')
  };
};

/**
 * @param {unknown} value
 */
const adaptFailureClassCounter = (value) => {
  const source = asRecord(value);
  return {
    count: Number(source.count || 0),
    last_seen_at: adaptOptionalNumber(source.last_seen_at)
  };
};

/**
 * @param {unknown} value
 */
const adaptLaneDiagnostics = (value) => {
  const source = asRecord(value);
  const lanes = asRecord(source.lanes);
  const requestFailureClasses = asRecord(source.request_failure_classes);
  return {
    schema_version: String(source.schema_version || ''),
    truth_basis: String(source.truth_basis || ''),
    lanes: {
      synthetic_traffic: adaptLaneCounterState(lanes.synthetic_traffic),
      scrapling_traffic: adaptLaneCounterState(lanes.scrapling_traffic),
      bot_red_team: adaptLaneCounterState(lanes.bot_red_team)
    },
    request_failure_classes: {
      cancelled: adaptFailureClassCounter(requestFailureClasses.cancelled),
      timeout: adaptFailureClassCounter(requestFailureClasses.timeout),
      transport: adaptFailureClassCounter(requestFailureClasses.transport),
      http: adaptFailureClassCounter(requestFailureClasses.http)
    }
  };
};

/**
 * @param {unknown} value
 * @param {number} fallback
 * @returns {number}
 */
const normalizeTimeoutMs = (value, fallback = DEFAULT_REQUEST_TIMEOUT_MS) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return fallback;
  return Math.max(1, Math.floor(numeric));
};

/**
 * @param {AbortSignal | undefined} externalSignal
 * @param {number} timeoutMs
 */
const createRequestSignal = (externalSignal, timeoutMs) => {
  if (typeof AbortController !== 'function') {
    return {
      signal: externalSignal,
      didTimeout: () => false,
      cleanup: () => {}
    };
  }

  const controller = new AbortController();
  let didTimeout = false;
  let timeoutId = null;

  const abortFromExternal = () => {
    if (!controller.signal.aborted) {
      controller.abort();
    }
  };

  if (externalSignal) {
    if (externalSignal.aborted) {
      abortFromExternal();
    } else {
      externalSignal.addEventListener('abort', abortFromExternal, { once: true });
    }
  }

  const safeTimeoutMs = normalizeTimeoutMs(timeoutMs, DEFAULT_REQUEST_TIMEOUT_MS);
  timeoutId = setTimeout(() => {
    didTimeout = true;
    if (!controller.signal.aborted) {
      controller.abort();
    }
  }, safeTimeoutMs);

  return {
    signal: controller.signal,
    didTimeout: () => didTimeout,
    cleanup: () => {
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
        timeoutId = null;
      }
      if (externalSignal) {
        externalSignal.removeEventListener('abort', abortFromExternal);
      }
    }
  };
};

/**
 * @param {unknown} value
 * @returns {string}
 */
const errorMessageFromPayload = (value) => {
  if (typeof value === 'string' && value.trim()) return value.trim();
  if (value && typeof value === 'object') {
    const body = /** @type {Record<string, unknown>} */ (value);
    if (typeof body.error === 'string' && body.error.trim()) return body.error.trim();
    if (typeof body.message === 'string' && body.message.trim()) return body.message.trim();
    if (typeof body.detail === 'string' && body.detail.trim()) return body.detail.trim();
  }
  return 'Request failed';
};

/**
 * @param {unknown} value
 * @returns {Array<Record<string, unknown>>}
 */
const asObjectArray = (value) => {
  if (!Array.isArray(value)) return [];
  return value.filter((entry) => entry && typeof entry === 'object');
};

/**
 * @param {unknown} value
 * @returns {Array<[string, number]>}
 */
const adaptTopIps = (value) => {
  if (!Array.isArray(value)) return [];
  return value
    .filter((entry) => Array.isArray(entry) && entry.length >= 2)
    .map((entry) => [String(entry[0] || ''), Number(entry[1] || 0)]);
};

/**
 * @param {unknown} payload
 */
export const adaptAnalytics = (payload) => {
  const source = asRecord(payload);
  const rawBanCount = source.ban_count;
  const banCount = rawBanCount === null || rawBanCount === undefined || rawBanCount === ''
    ? null
    : Number(rawBanCount || 0);
  return {
    ban_count: Number.isFinite(banCount) && banCount >= 0 ? banCount : null,
    ban_store_status: String(source.ban_store_status || 'available'),
    ban_store_message: String(source.ban_store_message || ''),
    shadow_mode: source.shadow_mode === true,
    fail_mode: source.fail_mode || 'open'
  };
};

/**
 * @param {unknown} payload
 */
export const adaptEvents = (payload) => {
  const source = asRecord(payload);
  return {
    recent_events: asObjectArray(source.recent_events),
    recent_sim_runs: asObjectArray(source.recent_sim_runs),
    event_counts: asRecord(source.event_counts),
    top_ips: adaptTopIps(source.top_ips),
    unique_ips: Number(source.unique_ips || 0)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptBans = (payload) => {
  const source = asRecord(payload);
  return {
    bans: asObjectArray(source.bans),
    status: String(source.status || 'available'),
    message: String(source.message || '')
  };
};

/**
 * @param {unknown} payload
 */
export const adaptMaze = (payload) => {
  const source = asRecord(payload);
  const rawMazeAutoBans = source.maze_auto_bans;
  const mazeAutoBans = rawMazeAutoBans === null || rawMazeAutoBans === undefined || rawMazeAutoBans === ''
    ? null
    : Number(rawMazeAutoBans || 0);
  return {
    total_hits: Number(source.total_hits || 0),
    unique_crawlers: Number(source.unique_crawlers || 0),
    maze_auto_bans: Number.isFinite(mazeAutoBans) && mazeAutoBans >= 0 ? mazeAutoBans : null,
    top_crawlers: Array.isArray(source.top_crawlers) ? source.top_crawlers : []
  };
};

/**
 * @param {unknown} payload
 */
export const adaptCdp = (payload) => {
  const source = asRecord(payload);
  return {
    stats: asRecord(source.stats),
    config: asRecord(source.config),
    fingerprint_stats: asRecord(source.fingerprint_stats)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptCdpEvents = (payload) => {
  const source = asRecord(payload);
  return {
    events: asObjectArray(source.events)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptMonitoring = (payload) => {
  const source = asRecord(payload);
  const detailsSource = asRecord(source.details);
  const details = {
    analytics: adaptAnalytics(detailsSource.analytics),
    events: adaptEvents(detailsSource.events),
    bans: adaptBans(detailsSource.bans),
    maze: adaptMaze(detailsSource.maze),
    tarpit: asRecord(detailsSource.tarpit),
    cdp: adaptCdp(detailsSource.cdp),
    cdp_events: adaptCdpEvents(detailsSource.cdp_events || detailsSource.cdpEvents)
  };
  return {
    summary: asRecord(source.summary),
    prometheus: asRecord(source.prometheus),
    freshness_slo: asRecord(source.freshness_slo),
    load_envelope: asRecord(source.load_envelope),
    freshness: asRecord(source.freshness),
    retention_health: asRecord(source.retention_health),
    window_end_cursor: String(source.window_end_cursor || ''),
    details
  };
};

/**
 * @param {unknown} payload
 */
export const adaptCursorDelta = (payload) => {
  const source = asRecord(payload);
  return {
    cursor_contract: asRecord(source.cursor_contract),
    freshness_slo: asRecord(source.freshness_slo),
    load_envelope: asRecord(source.load_envelope),
    stream_contract: asRecord(source.stream_contract),
    hours: Number(source.hours || 24),
    limit: Number(source.limit || 100),
    after_cursor: String(source.after_cursor || ''),
    window_end_cursor: String(source.window_end_cursor || ''),
    next_cursor: String(source.next_cursor || ''),
    has_more: source.has_more === true,
    overflow: String(source.overflow || 'none'),
    events: asObjectArray(source.events),
    recent_sim_runs: asObjectArray(source.recent_sim_runs),
    active_bans: asObjectArray(source.active_bans),
    active_bans_status: String(source.active_bans_status || 'available'),
    active_bans_message: String(source.active_bans_message || ''),
    freshness: asRecord(source.freshness),
    stream_supported: source.stream_supported === true,
    stream_endpoint: String(source.stream_endpoint || '')
  };
};

/**
 * @param {unknown} payload
 */
export const adaptIpRangeSuggestions = (payload) => {
  const source = asRecord(payload);
  const summarySource = asRecord(source.summary);
  const suggestions = asObjectArray(source.suggestions).map((entry) => {
    const record = asRecord(entry);
    return {
      cidr: String(record.cidr || ''),
      ip_family: String(record.ip_family || ''),
      bot_evidence_score: Number(record.bot_evidence_score || 0),
      human_evidence_score: Number(record.human_evidence_score || 0),
      collateral_risk: Number(record.collateral_risk || 0),
      confidence: Number(record.confidence || 0),
      risk_band: String(record.risk_band || 'high'),
      recommended_action: String(record.recommended_action || 'logging-only'),
      recommended_mode: String(record.recommended_mode || 'logging-only'),
      evidence_counts: asRecord(record.evidence_counts),
      safer_alternatives: Array.isArray(record.safer_alternatives)
        ? record.safer_alternatives.map((value) => String(value || ''))
        : [],
      guardrail_notes: Array.isArray(record.guardrail_notes)
        ? record.guardrail_notes.map((value) => String(value || ''))
        : []
    };
  });
  return {
    generated_at: Number(source.generated_at || 0),
    hours: Number(source.hours || 24),
    summary: {
      suggestions_total: Number(summarySource.suggestions_total || 0),
      low_risk: Number(summarySource.low_risk || 0),
      medium_risk: Number(summarySource.medium_risk || 0),
      high_risk: Number(summarySource.high_risk || 0)
    },
    suggestions
  };
};

/**
 * @param {unknown} payload
 * @returns {{ config: Record<string, unknown>, runtime: Record<string, unknown> }}
 */
export const adaptConfigEnvelope = (payload) => {
  const source = asRecord(payload);
  return {
    config: asRecord(source.config),
    runtime: asRecord(source.runtime)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptOperatorSnapshot = (payload) => {
  const source = asRecord(payload);
  const objectives = asRecord(source.objectives);
  const runtimePosture = asRecord(source.runtime_posture);
  const liveTraffic = asRecord(source.live_traffic);
  const liveTrafficHumanFriction = asRecord(liveTraffic.human_friction);
  const shadowMode = asRecord(source.shadow_mode);
  const adversarySim = asRecord(source.adversary_sim);
  const recentChanges = asRecord(source.recent_changes);
  const verifiedIdentity = asRecord(source.verified_identity);
  const nonHumanTraffic = asRecord(source.non_human_traffic);
  const restrictionReadiness = asRecord(nonHumanTraffic.restriction_readiness);
  const recognitionEvaluation = asRecord(nonHumanTraffic.recognition_evaluation);
  const taxonomyAlignment = asRecord(verifiedIdentity.taxonomy_alignment);
  const policyTranche = asRecord(verifiedIdentity.policy_tranche);
  const effectiveNonHumanPolicy = asRecord(verifiedIdentity.effective_non_human_policy);
  return {
    schema_version: String(source.schema_version || ''),
    generated_at: Number(source.generated_at || 0),
    section_metadata: asRecord(source.section_metadata),
    objectives: {
      profile_id: String(objectives.profile_id || ''),
      revision: String(objectives.revision || ''),
      window_hours: Number(objectives.window_hours || 0),
      category_postures: asObjectArray(objectives.category_postures).map((entry) => {
        const record = asRecord(entry);
        return {
          category_id: String(record.category_id || ''),
          posture: String(record.posture || '')
        };
      })
    },
    runtime_posture: {
      shadow_mode: runtimePosture.shadow_mode === true,
      fail_mode: String(runtimePosture.fail_mode || ''),
      runtime_environment: String(runtimePosture.runtime_environment || ''),
      gateway_deployment_profile: String(runtimePosture.gateway_deployment_profile || ''),
      adversary_sim_available: runtimePosture.adversary_sim_available === true
    },
    live_traffic: {
      traffic_origin: String(liveTraffic.traffic_origin || ''),
      execution_mode: String(liveTraffic.execution_mode || ''),
      total_requests: Number(liveTraffic.total_requests || 0),
      forwarded_requests: Number(liveTraffic.forwarded_requests || 0),
      short_circuited_requests: Number(liveTraffic.short_circuited_requests || 0),
      human_friction: {
        friction_rate: Number(liveTrafficHumanFriction.friction_rate || 0)
      }
    },
    shadow_mode: {
      enabled: shadowMode.enabled === true,
      total_actions: Number(shadowMode.total_actions || 0),
      pass_through_total: Number(shadowMode.pass_through_total || 0)
    },
    adversary_sim: {
      traffic_origin: String(adversarySim.traffic_origin || ''),
      execution_mode: String(adversarySim.execution_mode || ''),
      total_requests: Number(adversarySim.total_requests || 0),
      forwarded_requests: Number(adversarySim.forwarded_requests || 0),
      short_circuited_requests: Number(adversarySim.short_circuited_requests || 0),
      recent_runs: asObjectArray(adversarySim.recent_runs)
    },
    recent_changes: {
      lookback_seconds: Number(recentChanges.lookback_seconds || 0),
      watch_window_seconds: Number(recentChanges.watch_window_seconds || 0),
      rows: asObjectArray(recentChanges.rows).map((entry) => {
        const record = asRecord(entry);
        return {
          changed_at_ts: Number(record.changed_at_ts || 0),
          change_summary: String(record.change_summary || record.summary || ''),
          change_reason: String(record.change_reason || ''),
          source: String(record.source || ''),
          watch_window_status: String(record.watch_window_status || ''),
          expected_impact_summary: String(record.expected_impact_summary || '')
        };
      })
    },
    non_human_traffic: {
      availability: String(nonHumanTraffic.availability || ''),
      restriction_readiness: adaptNonHumanClassificationReadiness(restrictionReadiness),
      recognition_evaluation: {
        comparison_status: String(recognitionEvaluation.comparison_status || ''),
        current_exact_match_count: Number(recognitionEvaluation.current_exact_match_count || 0),
        degraded_match_count: Number(recognitionEvaluation.degraded_match_count || 0),
        collapsed_to_unknown_count: Number(
          recognitionEvaluation.collapsed_to_unknown_count || 0
        ),
        not_materialized_count: Number(recognitionEvaluation.not_materialized_count || 0),
        simulator_ground_truth: adaptSimulatorGroundTruthSummary(
          recognitionEvaluation.simulator_ground_truth
        ),
        readiness: adaptNonHumanClassificationReadiness(recognitionEvaluation.readiness),
        coverage: adaptNonHumanCoverageSummary(recognitionEvaluation.coverage),
        comparison_rows: asObjectArray(recognitionEvaluation.comparison_rows).map(
          adaptRecognitionComparisonRow
        )
      }
    },
    verified_identity: {
      availability: String(verifiedIdentity.availability || ''),
      enabled: verifiedIdentity.enabled === true,
      native_web_bot_auth_enabled: verifiedIdentity.native_web_bot_auth_enabled === true,
      provider_assertions_enabled: verifiedIdentity.provider_assertions_enabled === true,
      effective_non_human_policy: {
        schema_version: String(effectiveNonHumanPolicy.schema_version || ''),
        profile_id: String(effectiveNonHumanPolicy.profile_id || ''),
        objective_revision: String(effectiveNonHumanPolicy.objective_revision || ''),
        verified_identity_override_mode: String(
          effectiveNonHumanPolicy.verified_identity_override_mode || ''
        ),
        rows: asObjectArray(effectiveNonHumanPolicy.rows).map((entry) => {
          const row = asRecord(entry);
          return {
            category_id: String(row.category_id || ''),
            base_posture: String(row.base_posture || ''),
            effective_posture: String(row.effective_posture || ''),
            verified_identity_handling: String(row.verified_identity_handling || ''),
            authority: String(row.authority || '')
          };
        })
      },
      named_policy_count: Number(verifiedIdentity.named_policy_count || 0),
      service_profile_count: Number(verifiedIdentity.service_profile_count || 0),
      attempts: Number(verifiedIdentity.attempts || 0),
      verified: Number(verifiedIdentity.verified || 0),
      failed: Number(verifiedIdentity.failed || 0),
      unique_verified_identities: Number(verifiedIdentity.unique_verified_identities || 0),
      top_failure_reasons: adaptCountEntries(verifiedIdentity.top_failure_reasons),
      top_schemes: adaptCountEntries(verifiedIdentity.top_schemes),
      top_categories: adaptCountEntries(verifiedIdentity.top_categories),
      top_provenance: adaptCountEntries(verifiedIdentity.top_provenance),
      taxonomy_alignment: {
        status: String(taxonomyAlignment.status || ''),
        aligned_count: Number(taxonomyAlignment.aligned_count || 0),
        mismatched_count: Number(taxonomyAlignment.mismatched_count || 0),
        insufficient_evidence_count: Number(taxonomyAlignment.insufficient_evidence_count || 0),
        receipts: Array.isArray(taxonomyAlignment.receipts) ? taxonomyAlignment.receipts : []
      },
      policy_tranche: {
        total_requests: Number(policyTranche.total_requests || 0),
        forwarded_requests: Number(policyTranche.forwarded_requests || 0),
        short_circuited_requests: Number(policyTranche.short_circuited_requests || 0)
      }
    }
  };
};

const adaptOversightApply = (value) => {
  const source = asRecord(value);
  return {
    stage: String(source.stage || ''),
    summary: String(source.summary || ''),
    refusal_reasons: Array.isArray(source.refusal_reasons)
      ? source.refusal_reasons.map((entry) => String(entry || ''))
      : [],
    patch_family: String(source.patch_family || ''),
    watch_window_seconds: adaptOptionalNumber(source.watch_window_seconds),
    watch_window_started_at: adaptOptionalNumber(source.watch_window_started_at),
    watch_window_end_at: adaptOptionalNumber(source.watch_window_end_at),
    baseline_generated_at: adaptOptionalNumber(source.baseline_generated_at),
    candidate_generated_at: adaptOptionalNumber(source.candidate_generated_at),
    comparison_status: String(source.comparison_status || ''),
    rollback_reason: String(source.rollback_reason || '')
  };
};

const adaptHomeostasisRestartBaseline = (value) => {
  const source = asRecord(value);
  return {
    source: String(source.source || ''),
    generated_at: adaptOptionalNumber(source.generated_at),
    note: String(source.note || '')
  };
};

const adaptOperatorDecisionEvidenceReference = (value) => {
  const source = asRecord(value);
  return {
    kind: String(source.kind || ''),
    reference: String(source.reference || ''),
    note: String(source.note || '')
  };
};

const adaptOversightEpisodeProposal = (value) => {
  const source = asRecord(value);
  return {
    patch_family: String(source.patch_family || ''),
    expected_impact: String(source.expected_impact || ''),
    confidence: String(source.confidence || ''),
    note: String(source.note || '')
  };
};

const adaptOversightEpisodeArchiveRow = (value) => {
  const source = asRecord(value);
  return {
    episode_id: String(source.episode_id || ''),
    completed_at_ts: adaptOptionalNumber(source.completed_at_ts),
    proposal_status: String(source.proposal_status || ''),
    watch_window_result: String(source.watch_window_result || ''),
    retain_or_rollback: String(source.retain_or_rollback || ''),
    judged_lane_ids: Array.isArray(source.judged_lane_ids)
      ? source.judged_lane_ids.map((entry) => String(entry || ''))
      : [],
    judged_run_ids: Array.isArray(source.judged_run_ids)
      ? source.judged_run_ids.map((entry) => String(entry || ''))
      : [],
    proposal: adaptOversightEpisodeProposal(source.proposal),
    cycle_judgment: String(source.cycle_judgment || ''),
    homeostasis_eligible: source.homeostasis_eligible === true,
    benchmark_urgency_status: String(source.benchmark_urgency_status || ''),
    homeostasis_break_status: String(source.homeostasis_break_status || ''),
    homeostasis_break_reasons: Array.isArray(source.homeostasis_break_reasons)
      ? source.homeostasis_break_reasons.map((entry) => String(entry || ''))
      : [],
    restart_baseline: adaptHomeostasisRestartBaseline(source.restart_baseline),
    evidence_references: asObjectArray(source.evidence_references).map(
      adaptOperatorDecisionEvidenceReference
    )
  };
};

const adaptOversightRequiredLaneRun = (value) => {
  const source = asRecord(value);
  return {
    lane: String(source.lane || ''),
    status: String(source.status || ''),
    requested_at_ts: adaptOptionalNumber(source.requested_at_ts),
    requested_duration_seconds: adaptOptionalNumber(source.requested_duration_seconds),
    follow_on_run_id: String(source.follow_on_run_id || ''),
    follow_on_started_at: adaptOptionalNumber(source.follow_on_started_at),
    materialized_at_ts: adaptOptionalNumber(source.materialized_at_ts)
  };
};

const adaptOversightCandidateWindow = (value) => {
  const source = asRecord(value);
  return {
    status: String(source.status || ''),
    canary_id: String(source.canary_id || ''),
    patch_family: String(source.patch_family || ''),
    requested_lane: String(source.requested_lane || ''),
    requested_duration_seconds: adaptOptionalNumber(source.requested_duration_seconds),
    requested_at_ts: adaptOptionalNumber(source.requested_at_ts),
    watch_window_end_at: adaptOptionalNumber(source.watch_window_end_at),
    follow_on_run_id: String(source.follow_on_run_id || ''),
    follow_on_started_at: adaptOptionalNumber(source.follow_on_started_at),
    materialized_at_ts: adaptOptionalNumber(source.materialized_at_ts),
    required_runs: asObjectArray(source.required_runs).map(adaptOversightRequiredLaneRun)
  };
};

const adaptOversightContinuationRun = (value) => {
  const source = asRecord(value);
  return {
    status: String(source.status || ''),
    requested_lane: String(source.requested_lane || ''),
    requested_duration_seconds: adaptOptionalNumber(source.requested_duration_seconds),
    requested_at_ts: adaptOptionalNumber(source.requested_at_ts),
    source_decision_id: String(source.source_decision_id || ''),
    source_decision_outcome: String(source.source_decision_outcome || ''),
    continue_reason: String(source.continue_reason || ''),
    stop_reason: String(source.stop_reason || ''),
    follow_on_run_id: String(source.follow_on_run_id || ''),
    follow_on_started_at: adaptOptionalNumber(source.follow_on_started_at),
    required_runs: asObjectArray(source.required_runs).map(adaptOversightRequiredLaneRun)
  };
};

const adaptOversightEpisodeArchive = (value) => {
  const source = asRecord(value);
  const homeostasis = asRecord(source.homeostasis);
  return {
    schema_version: String(source.schema_version || ''),
    homeostasis: {
      status: String(homeostasis.status || ''),
      judged_cycle_count: Number(homeostasis.judged_cycle_count || 0),
      minimum_completed_cycles_for_homeostasis: Number(
        homeostasis.minimum_completed_cycles_for_homeostasis || 0
      ),
      urgency_status: String(homeostasis.urgency_status || ''),
      break_status: String(homeostasis.break_status || ''),
      break_reasons: Array.isArray(homeostasis.break_reasons)
        ? homeostasis.break_reasons.map((entry) => String(entry || ''))
        : [],
      restart_baseline: adaptHomeostasisRestartBaseline(homeostasis.restart_baseline),
      note: String(homeostasis.note || '')
    },
    rows: asObjectArray(source.rows).map(adaptOversightEpisodeArchiveRow)
  };
};

const adaptOversightObserverRoundRunRow = (value) => {
  const source = asRecord(value);
  return {
    run_id: String(source.run_id || ''),
    lane: String(source.lane || ''),
    profile: String(source.profile || ''),
    observed_fulfillment_modes: Array.isArray(source.observed_fulfillment_modes)
      ? source.observed_fulfillment_modes.map((entry) => String(entry || ''))
      : [],
    observed_category_ids: Array.isArray(source.observed_category_ids)
      ? source.observed_category_ids.map((entry) => String(entry || ''))
      : [],
    monitoring_event_count: Number(source.monitoring_event_count || 0),
    defense_delta_count: Number(source.defense_delta_count || 0),
    ban_outcome_count: Number(source.ban_outcome_count || 0)
  };
};

const adaptOversightObserverRoundSurfaceRow = (value) => {
  const source = asRecord(value);
  return {
    run_id: String(source.run_id || ''),
    surface_id: String(source.surface_id || ''),
    surface_state: String(source.surface_state || ''),
    coverage_status: String(source.coverage_status || ''),
    success_contract: String(source.success_contract || ''),
    dependency_kind: String(source.dependency_kind || ''),
    dependency_surface_ids: Array.isArray(source.dependency_surface_ids)
      ? source.dependency_surface_ids.map((entry) => String(entry || ''))
      : [],
    attempt_count: Number(source.attempt_count || 0),
    sample_request_method: String(source.sample_request_method || ''),
    sample_request_path: String(source.sample_request_path || ''),
    sample_response_status: adaptOptionalNumber(source.sample_response_status)
  };
};

const adaptOversightObserverRoundArchiveRow = (value) => {
  const source = asRecord(value);
  return {
    episode_id: String(source.episode_id || ''),
    completed_at_ts: adaptOptionalNumber(source.completed_at_ts),
    basis_status: String(source.basis_status || ''),
    missing_run_ids: Array.isArray(source.missing_run_ids)
      ? source.missing_run_ids.map((entry) => String(entry || ''))
      : [],
    run_rows: asObjectArray(source.run_rows).map(adaptOversightObserverRoundRunRow),
    scrapling_surface_rows: asObjectArray(source.scrapling_surface_rows).map(
      adaptOversightObserverRoundSurfaceRow
    ),
    llm_surface_rows: asObjectArray(source.llm_surface_rows).map(
      adaptOversightObserverRoundSurfaceRow
    )
  };
};

const adaptOversightObserverRoundArchive = (value) => {
  const source = asRecord(value);
  return {
    schema_version: String(source.schema_version || ''),
    rows: asObjectArray(source.rows).map(adaptOversightObserverRoundArchiveRow)
  };
};

const adaptOversightDecision = (value) => {
  const source = asRecord(value);
  const proposal = asRecord(source.proposal);
  return {
    decision_id: String(source.decision_id || ''),
    recorded_at_ts: Number(source.recorded_at_ts || 0),
    trigger_source: String(source.trigger_source || ''),
    outcome: String(source.outcome || ''),
    summary: String(source.summary || ''),
    benchmark_overall_status: String(source.benchmark_overall_status || ''),
    improvement_status: String(source.improvement_status || ''),
    replay_promotion_availability: String(source.replay_promotion_availability || ''),
    trigger_family_ids: Array.isArray(source.trigger_family_ids)
      ? source.trigger_family_ids.map((entry) => String(entry || ''))
      : [],
    candidate_action_families: Array.isArray(source.candidate_action_families)
      ? source.candidate_action_families.map((entry) => String(entry || ''))
      : [],
    refusal_reasons: Array.isArray(source.refusal_reasons)
      ? source.refusal_reasons.map((entry) => String(entry || ''))
      : [],
    validation_status: String(source.validation_status || ''),
    validation_issues: Array.isArray(source.validation_issues)
      ? source.validation_issues.map((entry) => String(entry || ''))
      : [],
    latest_sim_run_id: String(source.latest_sim_run_id || ''),
    proposal: {
      patch_family: String(proposal.patch_family || ''),
      expected_impact: String(proposal.expected_impact || ''),
      confidence: String(proposal.confidence || ''),
      note: String(proposal.note || '')
    },
    apply: adaptOversightApply(source.apply)
  };
};

export const adaptOversightHistory = (payload) => {
  const source = asRecord(payload);
  return {
    schema_version: String(source.schema_version || ''),
    episode_archive: adaptOversightEpisodeArchive(source.episode_archive),
    observer_round_archive: adaptOversightObserverRoundArchive(source.observer_round_archive),
    rows: asObjectArray(source.rows).map(adaptOversightDecision)
  };
};

const adaptOversightAgentRun = (value) => {
  const source = asRecord(value);
  const execution = asRecord(source.execution);
  return {
    run_id: String(source.run_id || ''),
    trigger_kind: String(source.trigger_kind || ''),
    requested_at_ts: Number(source.requested_at_ts || 0),
    started_at_ts: Number(source.started_at_ts || 0),
    completed_at_ts: Number(source.completed_at_ts || 0),
    execution: {
      apply: adaptOversightApply(execution.apply)
    }
  };
};

export const adaptOversightAgentStatus = (payload) => {
  const source = asRecord(payload);
  const periodicTrigger = asRecord(source.periodic_trigger);
  const postSimTrigger = asRecord(source.post_sim_trigger);
  return {
    schema_version: String(source.schema_version || ''),
    execution_boundary: String(source.execution_boundary || ''),
    periodic_trigger: {
      surface: String(periodicTrigger.surface || ''),
      wrapper_command: String(periodicTrigger.wrapper_command || ''),
      default_interval_seconds: Number(periodicTrigger.default_interval_seconds || 0)
    },
    post_sim_trigger: {
      surface: String(postSimTrigger.surface || ''),
      qualifying_completion: String(postSimTrigger.qualifying_completion || ''),
      dedupe_key: String(postSimTrigger.dedupe_key || '')
    },
    candidate_window: adaptOversightCandidateWindow(source.candidate_window),
    continuation_run: adaptOversightContinuationRun(source.continuation_run),
    episode_archive: adaptOversightEpisodeArchive(source.episode_archive),
    latest_run: adaptOversightAgentRun(source.latest_run),
    latest_decision: adaptOversightDecision(source.latest_decision),
    recent_runs: asObjectArray(source.recent_runs).map(adaptOversightAgentRun)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptAdversarySimStatus = (payload) => {
  const source = asRecord(payload);
  const guardrails = asRecord(source.guardrails);
  const lanes = asRecord(source.lanes);
  const historyRetention = asRecord(source.history_retention);
  const generationDiagnostics = asRecord(source.generation_diagnostics);
  const persistedEventEvidence = asRecord(source.persisted_event_evidence);
  const supervisor = asRecord(source.supervisor);
  return {
    runtime_environment: String(source.runtime_environment || ''),
    adversary_sim_available: source.adversary_sim_available === true,
    adversary_sim_enabled: source.adversary_sim_enabled === true,
    generation_active: source.generation_active === true,
    historical_data_visible: source.historical_data_visible !== false,
    phase: String(source.phase || 'off'),
    run_id: typeof source.run_id === 'string' ? source.run_id : '',
    started_at: Number(source.started_at || 0),
    ends_at: Number(source.ends_at || 0),
    duration_seconds: Number(source.duration_seconds || 0),
    remaining_seconds: Number(source.remaining_seconds || 0),
    active_run_count: Number(source.active_run_count || 0),
    active_lane_count: Number(source.active_lane_count || 0),
    desired_lane: normalizeAdversarySimLane(source.desired_lane),
    active_lane: normalizeOptionalAdversarySimLane(source.active_lane),
    lane_switch_seq: Number(source.lane_switch_seq || 0),
    last_lane_switch_at: adaptOptionalNumber(source.last_lane_switch_at),
    last_lane_switch_reason: String(source.last_lane_switch_reason || ''),
    lanes: {
      deterministic: String(lanes.deterministic || 'off'),
      containerized: String(lanes.containerized || 'off')
    },
    lane_diagnostics: adaptLaneDiagnostics(source.lane_diagnostics),
    queue_policy: String(source.queue_policy || ''),
    guardrails: {
      max_duration_seconds: Number(guardrails.max_duration_seconds || 0),
      max_concurrent_runs: Number(guardrails.max_concurrent_runs || 0),
      cpu_cap_millicores: Number(guardrails.cpu_cap_millicores || 0),
      memory_cap_mib: Number(guardrails.memory_cap_mib || 0),
      queue_policy: String(guardrails.queue_policy || '')
    },
    last_transition_reason:
      typeof source.last_transition_reason === 'string' ? source.last_transition_reason : '',
    last_terminal_failure_reason:
      typeof source.last_terminal_failure_reason === 'string'
        ? source.last_terminal_failure_reason
        : '',
    last_run_id: typeof source.last_run_id === 'string' ? source.last_run_id : '',
    history_retention: {
      retention_hours: Number(historyRetention.retention_hours || 0),
      cleanup_supported: historyRetention.cleanup_supported === true,
      cleanup_endpoint: String(historyRetention.cleanup_endpoint || ''),
      cleanup_command: String(historyRetention.cleanup_command || '')
    },
    supervisor: {
      owner: String(supervisor.owner || ''),
      cadence_seconds: Number(supervisor.cadence_seconds || 0),
      max_catchup_ticks_per_invocation: Number(supervisor.max_catchup_ticks_per_invocation || 0),
      heartbeat_active: supervisor.heartbeat_active === true,
      worker_active: supervisor.worker_active === true,
      last_heartbeat_at: Number(supervisor.last_heartbeat_at || 0),
      idle_seconds: Number(supervisor.idle_seconds || 0),
      off_state_inert: supervisor.off_state_inert === true,
      trigger_surface: String(supervisor.trigger_surface || '')
    },
    generation_diagnostics: {
      health: String(generationDiagnostics.health || ''),
      reason: String(generationDiagnostics.reason || ''),
      recommended_action: String(generationDiagnostics.recommended_action || ''),
      generated_tick_count: Number(generationDiagnostics.generated_tick_count || 0),
      generated_request_count: Number(generationDiagnostics.generated_request_count || 0),
      last_generated_at: Number(generationDiagnostics.last_generated_at || 0),
      last_generation_error: String(generationDiagnostics.last_generation_error || ''),
      truth_basis: String(generationDiagnostics.truth_basis || '')
    },
    persisted_event_evidence: source.persisted_event_evidence == null
      ? null
      : {
        run_id: String(persistedEventEvidence.run_id || ''),
        lane: normalizeOptionalAdversarySimLane(persistedEventEvidence.lane),
        profile: String(persistedEventEvidence.profile || ''),
        monitoring_event_count: Number(persistedEventEvidence.monitoring_event_count || 0),
        defense_delta_count: Number(persistedEventEvidence.defense_delta_count || 0),
        ban_outcome_count: Number(persistedEventEvidence.ban_outcome_count || 0),
        first_observed_at: Number(persistedEventEvidence.first_observed_at || 0),
        last_observed_at: Number(persistedEventEvidence.last_observed_at || 0),
        truth_basis: String(persistedEventEvidence.truth_basis || '')
      }
  };
};

/**
 * @param {unknown} payload
 * @returns {{ valid: boolean, issues: Array<Record<string, unknown>> }}
 */
export const adaptConfigValidation = (payload) => {
  const source = asRecord(payload);
  return {
    valid: source.valid === true,
    issues: asObjectArray(source.issues)
  };
};

/**
 * @param {unknown} payload
 * @returns {{ content: string }}
 */
const adaptRobots = (payload) => {
  if (payload && typeof payload === 'object') {
    const source = /** @type {Record<string, unknown>} */ (payload);
    return {
      content: typeof source.preview === 'string' ? source.preview : ''
    };
  }
  return {
    content: typeof payload === 'string' ? payload : ''
  };
};

export const create = (options = {}) => {
  const getAdminContext =
    typeof options.getAdminContext === 'function' ? options.getAdminContext : null;
  const onUnauthorized =
    typeof options.onUnauthorized === 'function' ? options.onUnauthorized : null;
  const onApiError = typeof options.onApiError === 'function' ? options.onApiError : null;
  const onRequestTelemetry =
    typeof options.onRequestTelemetry === 'function' ? options.onRequestTelemetry : null;
  const onBackendConnected =
    typeof options.onBackendConnected === 'function' ? options.onBackendConnected : null;
  const onBackendDisconnected =
    typeof options.onBackendDisconnected === 'function' ? options.onBackendDisconnected : null;
  const requestImpl =
    typeof options.request === 'function'
      ? options.request
      : fetch.bind(globalThis);

  /**
   * Parse response payloads defensively because some local/runtime paths may
   * omit content-type headers even when returning JSON.
   *
   * @param {Response} response
   * @returns {Promise<unknown>}
   */
  const parseResponsePayload = async (response) => {
    const contentType = String(response.headers.get('content-type') || '').toLowerCase();
    if (contentType.includes(JSON_CONTENT_TYPE)) {
      try {
        return await response.json();
      } catch (_e) {
        return await response.text();
      }
    }

    const text = await response.text();
    if (!text) return '';
    const trimmed = text.trim();
    if (!trimmed) return '';
    if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
      try {
        return JSON.parse(trimmed);
      } catch (_e) {
        return text;
      }
    }
    return text;
  };

  /**
   * @param {string} path
   * @param {RequestOptions} [options]
   */
  const request = async (path, options = {}) => {
    if (!getAdminContext) {
      throw new DashboardApiError('API client is not configured', 0, path, options.method || 'GET');
    }

    /** @type {AdminContext | null} */
    const context = getAdminContext(options.messageTarget || null);
    if (!context) {
      throw new DashboardApiError(
        'Login required. Go to /shuma/dashboard/login.html.',
        0,
        path,
        options.method || 'GET'
      );
    }

    const method = String(options.method || (options.json ? 'POST' : 'GET')).toUpperCase();
    const requestId = newIdempotencyKey();
    const startedAtMs = Date.now();
    const startedAtIso = new Date(startedAtMs).toISOString();
    const telemetryContext = asRecord(options.telemetry);
    const telemetryTab = typeof telemetryContext.tab === 'string'
      ? telemetryContext.tab
      : '';
    const telemetryReason = typeof telemetryContext.reason === 'string'
      ? telemetryContext.reason
      : '';
    const telemetrySource = typeof telemetryContext.source === 'string' && telemetryContext.source.trim()
      ? telemetryContext.source.trim()
      : 'api-client';
    const emitRequestTelemetry = (event = {}) => {
      if (!onRequestTelemetry) return;
      const failureClass = typeof event.failureClass === 'string'
        ? event.failureClass
        : '';
      onRequestTelemetry({
        requestId,
        path,
        method,
        tab: telemetryTab,
        reason: telemetryReason,
        source: telemetrySource,
        startedAt: startedAtIso,
        durationMs: Math.max(0, Date.now() - startedAtMs),
        outcome: event.outcome === 'success' ? 'success' : 'failure',
        failureClass,
        statusCode: Number(event.statusCode || 0),
        aborted: event.aborted === true,
        errorMessage: String(event.errorMessage || '')
      });
    };
    const headers = new Headers(options.headers || {});
    if (!headers.has('Accept')) headers.set('Accept', JSON_CONTENT_TYPE);
    if (!headers.has('Authorization') && String(context.apikey || '').trim()) {
      headers.set('Authorization', `Bearer ${String(context.apikey).trim()}`);
    }
    const authHeader = headers.get('Authorization') || headers.get('authorization') || '';
    if (/^Bearer\s*$/i.test(authHeader.trim())) {
      headers.delete('Authorization');
      headers.delete('authorization');
    }
    if (
      context &&
      context.sessionAuth === true &&
      isWriteMethod(method) &&
      String(context.csrfToken || '').trim()
    ) {
      headers.set('X-Shuma-CSRF', String(context.csrfToken).trim());
    }

    /** @type {BodyInit | null | undefined} */
    let body = options.body;
    if (options.json !== undefined) {
      if (!headers.has('Content-Type')) headers.set('Content-Type', JSON_CONTENT_TYPE);
      body = JSON.stringify(options.json);
    }

    const timeoutMs = normalizeTimeoutMs(options.timeoutMs, DEFAULT_REQUEST_TIMEOUT_MS);
    const requestSignal = createRequestSignal(options.signal, timeoutMs);
    let response;
    try {
      response = await requestImpl(`${context.endpoint}${path}`, {
        method,
        headers,
        cache: options.cache,
        credentials: context && context.sessionAuth === true ? 'same-origin' : undefined,
        body: method === 'GET' || method === 'HEAD' ? undefined : body,
        signal: requestSignal.signal
      });
    } catch (error) {
      if (requestSignal.didTimeout()) {
        const timeoutError = new DashboardApiError(
          `Request timed out after ${timeoutMs}ms`,
          0,
          path,
          method
        );
        emitRequestTelemetry({
          outcome: 'failure',
          failureClass: classifyRequestFailure(timeoutError, { didTimeout: true }),
          statusCode: 0,
          aborted: true,
          errorMessage: timeoutError.message
        });
        if (onBackendDisconnected) onBackendDisconnected(timeoutError);
        if (onApiError) onApiError(timeoutError);
        throw timeoutError;
      }
      const failureClass = classifyRequestFailure(error, { didTimeout: false });
      emitRequestTelemetry({
        outcome: 'failure',
        failureClass,
        statusCode: Number(error && typeof error === 'object' ? error.status || 0 : 0),
        aborted: failureClass === REQUEST_FAILURE_CLASSES.cancelled,
        errorMessage: String(error && typeof error === 'object' ? error.message || '' : '')
      });
      if (onBackendDisconnected) {
        const transportError = error instanceof Error
          ? error
          : new DashboardApiError('Network request failed', 0, path, method);
        if (failureClass !== REQUEST_FAILURE_CLASSES.cancelled) {
          onBackendDisconnected(transportError);
        }
      }
      throw error;
    } finally {
      requestSignal.cleanup();
    }
    if (onBackendConnected) {
      onBackendConnected({
        status: Number(response.status || 0),
        path,
        method
      });
    }

    const payload = await parseResponsePayload(response);

    if (response.status === 401) {
      if (onUnauthorized) onUnauthorized();
      const unauthorizedError = new DashboardApiError(
        'Unauthorized',
        response.status,
        path,
        method
      );
      emitRequestTelemetry({
        outcome: 'failure',
        failureClass: classifyRequestFailure(unauthorizedError, { statusCode: response.status }),
        statusCode: response.status,
        aborted: false,
        errorMessage: unauthorizedError.message
      });
      if (onApiError) onApiError(unauthorizedError);
      throw unauthorizedError;
    }

    if (!response.ok) {
      const apiError = new DashboardApiError(
        errorMessageFromPayload(payload),
        response.status,
        path,
        method
      );
      apiError.retryAfterSeconds = parseRetryAfterSeconds(response.headers.get('retry-after'));
      emitRequestTelemetry({
        outcome: 'failure',
        failureClass: classifyRequestFailure(apiError, { statusCode: response.status }),
        statusCode: response.status,
        aborted: false,
        errorMessage: apiError.message
      });
      if (onApiError) onApiError(apiError);
      throw apiError;
    }

    emitRequestTelemetry({
      outcome: 'success',
      statusCode: Number(response.status || 0),
      aborted: false
    });
    return payload;
  };

  /**
   * @param {string} path
   * @param {{hours?: number, limit?: number, after_cursor?: string}} [options]
   */
  const buildCursorPath = (path, options = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 100;
    const afterCursor = typeof options.after_cursor === 'string'
      ? options.after_cursor
      : '';
    const params = new URLSearchParams();
    params.set('hours', String(hours));
    params.set('limit', String(limit));
    if (afterCursor.trim()) {
      params.set('after_cursor', afterCursor);
    }
    return `${path}?${params.toString()}`;
  };

  /**
   * @param {string} path
   * @param {{hours?: number, limit?: number, after_cursor?: string}} [options]
   */
  const buildEventStreamUrl = (path, options = {}) => {
    if (!getAdminContext) {
      throw new DashboardApiError('API client is not configured', 0, path, 'GET');
    }
    const context = getAdminContext(null);
    if (!context || typeof context.endpoint !== 'string' || !context.endpoint.trim()) {
      throw new DashboardApiError('Login required. Go to /shuma/dashboard/login.html.', 0, path, 'GET');
    }
    return `${context.endpoint}${buildCursorPath(path, options)}`;
  };

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getAnalytics = async (requestOptions = {}) =>
    adaptAnalytics(await request('/shuma/admin/analytics', requestOptions));

  /**
   * @param {number} hours
   * @param {RequestOptions} [requestOptions]
   */
  const getEvents = async (hours = 24, requestOptions = {}) =>
    adaptEvents(await request(`/shuma/admin/events?hours=${encodeURIComponent(String(hours))}`, requestOptions));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getBans = async (requestOptions = {}) => adaptBans(await request('/shuma/admin/ban', requestOptions));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getMaze = async (requestOptions = {}) => adaptMaze(await request('/shuma/admin/maze', requestOptions));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getCdp = async (requestOptions = {}) => adaptCdp(await request('/shuma/admin/cdp', requestOptions));

  /**
   * @param {{hours?: number, limit?: number}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getCdpEvents = async (options = {}, requestOptions = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 500;
    return adaptCdpEvents(
      await request(
        `/shuma/admin/cdp/events?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}`,
        requestOptions
      )
    );
  };

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getConfig = async (requestOptions = {}) =>
    adaptConfigEnvelope(await request('/shuma/admin/config', {
      ...requestOptions,
      cache: 'no-store'
    }));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getOperatorSnapshot = async (requestOptions = {}) =>
    adaptOperatorSnapshot(await request('/shuma/admin/operator-snapshot', {
      ...requestOptions,
      cache: 'no-store'
    }));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getOversightHistory = async (requestOptions = {}) =>
    adaptOversightHistory(await request('/shuma/admin/oversight/history', {
      ...requestOptions,
      cache: 'no-store'
    }));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getOversightAgentStatus = async (requestOptions = {}) =>
    adaptOversightAgentStatus(await request('/shuma/admin/oversight/agent/status', {
      ...requestOptions,
      cache: 'no-store'
    }));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getAdversarySimStatus = async (requestOptions = {}) =>
    adaptAdversarySimStatus(await request('/shuma/admin/adversary-sim/status', {
      ...requestOptions,
      cache: 'no-store'
    }));

  /**
   * @param {{hours?: number, limit?: number}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getMonitoring = async (options = {}, requestOptions = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 10;
    return adaptMonitoring(
      await request(
        `/shuma/admin/monitoring?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}`,
        requestOptions
      )
    );
  };

  /**
   * @param {{hours?: number, limit?: number}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getMonitoringBootstrap = async (options = {}, requestOptions = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 10;
    return adaptMonitoring(
      await request(
        `/shuma/admin/monitoring?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}&bootstrap=1`,
        requestOptions
      )
    );
  };

  /**
   * @param {{hours?: number, limit?: number, after_cursor?: string}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getMonitoringDelta = async (options = {}, requestOptions = {}) =>
    adaptCursorDelta(
      await request(
        buildCursorPath('/shuma/admin/monitoring/delta', options),
        requestOptions
      )
    );

  /**
   * @param {{hours?: number, limit?: number, after_cursor?: string}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getIpBansDelta = async (options = {}, requestOptions = {}) =>
    adaptCursorDelta(
      await request(
        buildCursorPath('/shuma/admin/ip-bans/delta', options),
        requestOptions
      )
    );

  /**
   * @param {{hours?: number, limit?: number, after_cursor?: string}} [options]
   */
  const getMonitoringStreamUrl = (options = {}) =>
    buildEventStreamUrl('/shuma/admin/monitoring/stream', options);

  /**
   * @param {{hours?: number, limit?: number, after_cursor?: string}} [options]
   */
  const getIpBansStreamUrl = (options = {}) =>
    buildEventStreamUrl('/shuma/admin/ip-bans/stream', options);

  /**
   * @param {{hours?: number, limit?: number}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getIpRangeSuggestions = async (options = {}, requestOptions = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 20;
    return adaptIpRangeSuggestions(
      await request(
        `/shuma/admin/ip-range/suggestions?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}`,
        requestOptions
      )
    );
  };

  /**
   * @param {Record<string, unknown> | null} [previewPatch]
   * @param {RequestOptions} [requestOptions]
   */
  const getRobotsPreview = async (previewPatch = null, requestOptions = {}) => {
    if (previewPatch && typeof previewPatch === 'object' && !Array.isArray(previewPatch)) {
      return adaptRobots(
        await request('/shuma/admin/robots/preview', {
          ...requestOptions,
          method: 'POST',
          json: previewPatch
        })
      );
    }
    return adaptRobots(await request('/shuma/admin/robots', requestOptions));
  };

  /**
   * @param {Record<string, unknown>} configPatch
   * @param {RequestOptions} [requestOptions]
   */
  const updateConfig = async (configPatch, requestOptions = {}) => {
    const payload = asRecord(
      await request('/shuma/admin/config', {
        ...requestOptions,
        method: 'POST',
        json: configPatch
      })
    );
    const configEnvelope = adaptConfigEnvelope(payload);
    return {
      status: typeof payload.status === 'string' ? payload.status : '',
      config: configEnvelope.config,
      runtime: configEnvelope.runtime
    };
  };

  /**
   * @param {Record<string, unknown>} configPatch
   * @param {RequestOptions} [requestOptions]
   */
  const validateConfigPatch = async (configPatch, requestOptions = {}) =>
    adaptConfigValidation(
      await request('/shuma/admin/config/validate', {
        ...requestOptions,
        method: 'POST',
        json: configPatch
      })
    );

  /**
   * @param {boolean} enabled
   * @param {RequestOptions & { lane?: string }} [requestOptions]
   */
  const controlAdversarySim = async (enabled, requestOptions = {}) => {
    const controlOptions = asRecord(requestOptions);
    const headers = controlOptions.headers ? controlOptions.headers : {};
    const normalizedLane = normalizeOptionalAdversarySimLane(controlOptions.lane);
    const payloadBody = normalizedLane
      ? {
        enabled: enabled === true,
        lane: normalizedLane
      }
      : {
        enabled: enabled === true
      };
    const idempotencyKey = newIdempotencyKey();
    const payload = asRecord(
      await request('/shuma/admin/adversary-sim/control', {
        ...requestOptions,
        method: 'POST',
        headers: {
          ...headers,
          'Idempotency-Key': idempotencyKey
        },
        json: payloadBody
      })
    );
    return {
      requested_enabled:
        payload.requested_enabled === true ||
        (payload.requested_state && payload.requested_state.enabled === true),
      operation_id: typeof payload.operation_id === 'string' ? payload.operation_id : '',
      status: adaptAdversarySimStatus(payload.status),
      config: asRecord(payload.config),
      runtime: asRecord(payload.runtime)
    };
  };

  /**
   * @param {string} ip
   * @param {number} duration
   * @param {string} [reason]
   * @param {RequestOptions} [requestOptions]
   */
  const banIp = async (ip, duration, reason = 'manual_ban', requestOptions = {}) =>
    request('/shuma/admin/ban', {
      ...requestOptions,
      method: 'POST',
      json: {
        ip: String(ip || ''),
        reason: String(reason || 'manual_ban'),
        duration: Number(duration || 0)
      }
    });

  /**
   * @param {string} ip
   * @param {RequestOptions} [requestOptions]
   */
  const unbanIp = async (ip, requestOptions = {}) =>
    request(`/shuma/admin/unban?ip=${encodeURIComponent(String(ip || ''))}`, {
      ...requestOptions,
      method: 'POST'
    });

  return {
    request,
    getAnalytics,
    getEvents,
    getBans,
    getMaze,
    getCdp,
    getCdpEvents,
    getMonitoring,
    getMonitoringBootstrap,
    getMonitoringDelta,
    getIpBansDelta,
    getMonitoringStreamUrl,
    getIpBansStreamUrl,
    getIpRangeSuggestions,
    getConfig,
    getOperatorSnapshot,
    getOversightHistory,
    getOversightAgentStatus,
    getAdversarySimStatus,
    getRobotsPreview,
    updateConfig,
    validateConfigPatch,
    controlAdversarySim,
    banIp,
    unbanIp
  };
};
