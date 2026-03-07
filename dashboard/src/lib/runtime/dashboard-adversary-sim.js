// @ts-check

const DEFAULT_DURATION_SECONDS = 180;
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
 *   durationSeconds: number,
 *   activeRunCount: number,
 *   activeLaneCount: number,
 *   queuePolicy: string,
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
 *     lastGenerationError: string
 *   }
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
  const supervisorRaw = pick(source, 'supervisor', 'supervisor', {});
  const supervisor = supervisorRaw && typeof supervisorRaw === 'object'
    ? /** @type {Record<string, unknown>} */ (supervisorRaw)
    : {};
  const phase = normalizePhase(pick(source, 'phase', 'phase', 'off'));
  const durationSeconds = Math.max(
    1,
    Math.floor(toSafeNumber(
      pick(source, 'duration_seconds', 'durationSeconds', DEFAULT_DURATION_SECONDS),
      DEFAULT_DURATION_SECONDS
    ))
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
    durationSeconds,
    activeRunCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'active_run_count', 'activeRunCount', 0), 0))
    ),
    activeLaneCount: Math.max(
      0,
      Math.floor(toSafeNumber(pick(source, 'active_lane_count', 'activeLaneCount', 0), 0))
    ),
    queuePolicy: String(pick(source, 'queue_policy', 'queuePolicy', '') || ''),
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
      )
    }
  };
}

/**
 * @param {{ configSnapshot?: Record<string, unknown>, adversarySimStatus?: unknown }} source
 * @returns {{ runtimeEnvironment: '' | 'runtime-dev' | 'runtime-prod', surfaceAvailable: boolean, controlAvailable: boolean }}
 */
export function deriveAdversarySimControlState(source = {}) {
  const configSnapshot =
    source && source.configSnapshot && typeof source.configSnapshot === 'object'
      ? /** @type {Record<string, unknown>} */ (source.configSnapshot)
      : {};
  const normalizedStatus = normalizeAdversarySimStatus(source?.adversarySimStatus);
  const runtimeEnvironment = normalizeRuntimeEnvironment(
    normalizedStatus.runtimeEnvironment || configSnapshot.runtime_environment || ''
  );
  const surfaceAvailable =
    normalizedStatus.available === true || configSnapshot.adversary_sim_available === true;
  return {
    runtimeEnvironment,
    surfaceAvailable,
    controlAvailable: surfaceAvailable === true
  };
}
