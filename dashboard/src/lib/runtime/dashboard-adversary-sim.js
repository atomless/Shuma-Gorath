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
 *   startedAt: number,
 *   endsAt: number,
 *   durationSeconds: number,
 *   remainingSeconds: number,
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
  if (normalizedStatus.generationActive) {
    return String(generationDiagnostics.health || '') === 'ok'
      ? 'Generation active. Auto-off stops new simulation traffic only; retained telemetry stays visible.'
      : `Generation active. ${
        String(generationDiagnostics.recommendedAction || '').trim() ||
        'No observable traffic yet. Check supervisor diagnostics for stalled heartbeat state.'
      }`;
  }

  const retentionHours = Math.max(0, Number(normalizedStatus.historyRetentionHours || 0));
  const cleanupCommand =
    String(normalizedStatus.historyCleanupCommand || '').trim() || 'make telemetry-clean';
  return normalizedStatus.historicalDataVisible
    ? `Generation inactive. Retained telemetry remains visible for ${retentionHours}h or until ${cleanupCommand} is run.`
    : 'Generation inactive.';
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
