// @ts-check

const DEFAULT_DURATION_SECONDS = 180;
const hasOwn = (value, key) => Object.prototype.hasOwnProperty.call(value, key);

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
 *   phase: 'off' | 'running' | 'stopping',
 *   runId: string,
 *   startedAt: number,
 *   endsAt: number,
 *   durationSeconds: number,
 *   remainingSeconds: number,
 *   activeRunCount: number,
 *   activeLaneCount: number,
 *   queuePolicy: string
 * }}
 */
export function normalizeAdversarySimStatus(payload) {
  const source = payload && typeof payload === 'object'
    ? /** @type {Record<string, unknown>} */ (payload)
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
    phase,
    runId: typeof runIdValue === 'string' ? runIdValue : '',
    startedAt: Math.max(0, Math.floor(toSafeNumber(pick(source, 'started_at', 'startedAt', 0), 0))),
    endsAt: Math.max(0, Math.floor(toSafeNumber(pick(source, 'ends_at', 'endsAt', 0), 0))),
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
    queuePolicy: String(pick(source, 'queue_policy', 'queuePolicy', '') || '')
  };
}

/**
 * @param {{
 *   phase?: string,
 *   startedAt?: number,
 *   endsAt?: number,
 *   durationSeconds?: number,
 *   remainingSeconds?: number
 * }} status
 * @param {number} [nowMs]
 * @returns {{ visible: boolean, widthPercent: number, remainingSeconds: number, complete: boolean }}
 */
export function deriveAdversarySimProgress(status = {}, nowMs = Date.now()) {
  const normalized = normalizeAdversarySimStatus(status);
  if (normalized.phase !== 'running') {
    return {
      visible: false,
      widthPercent: 0,
      remainingSeconds: 0,
      complete: false
    };
  }

  const nowSeconds = Math.max(0, Math.floor(toSafeNumber(nowMs, 0) / 1000));
  const hasWindow = normalized.startedAt > 0 && normalized.endsAt > normalized.startedAt;
  if (!hasWindow) {
    return {
      visible: true,
      widthPercent: 0,
      remainingSeconds: normalized.remainingSeconds,
      complete: normalized.remainingSeconds <= 0
    };
  }

  const total = Math.max(1, normalized.endsAt - normalized.startedAt);
  const elapsed = Math.max(0, Math.min(total, nowSeconds - normalized.startedAt));
  const remaining = Math.max(0, normalized.endsAt - nowSeconds);
  const widthPercent = Math.max(0, Math.min(100, (elapsed / total) * 100));
  return {
    visible: true,
    widthPercent,
    remainingSeconds: remaining,
    complete: remaining <= 0 || widthPercent >= 100
  };
}
