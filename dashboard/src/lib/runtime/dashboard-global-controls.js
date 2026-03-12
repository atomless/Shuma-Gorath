// @ts-check

import { normalizeAdversarySimStatus } from './dashboard-adversary-sim.js';

/**
 * @param {{
 *   runtimeMounted?: boolean,
 *   loggingOut?: boolean,
 *   saving?: boolean,
 *   authenticated?: boolean,
 *   adminConfigWritable?: boolean,
 *   surfaceAvailable?: boolean
 * }} source
 * @returns {boolean}
 */
export function deriveGlobalControlDisabled(source = {}) {
  return !(
    source.runtimeMounted === true &&
    source.loggingOut !== true &&
    source.saving !== true &&
    source.authenticated === true &&
    source.adminConfigWritable === true &&
    source.surfaceAvailable !== false
  );
}

/**
 * @param {{
 *   pendingEnabled?: boolean | null,
 *   adversarySimStatus?: unknown,
 *   configSnapshot?: Record<string, unknown>
 * }} source
 * @returns {boolean}
 */
export function deriveAdversarySimToggleEnabled(source = {}) {
  if (source.pendingEnabled === true) return true;
  if (source.pendingEnabled === false) return false;

  const statusSource =
    source.adversarySimStatus && typeof source.adversarySimStatus === 'object'
      ? /** @type {Record<string, unknown>} */ (source.adversarySimStatus)
      : {};
  if (Object.keys(statusSource).length > 0) {
    return normalizeAdversarySimStatus(statusSource).enabled === true;
  }

  const configSnapshot =
    source.configSnapshot && typeof source.configSnapshot === 'object'
      ? /** @type {Record<string, unknown>} */ (source.configSnapshot)
      : {};
  return configSnapshot.adversary_sim_enabled === true;
}

/**
 * @param {unknown} value
 * @returns {boolean}
 */
export function hasExplicitAdversarySimStatus(value) {
  const source =
    value && typeof value === 'object'
      ? /** @type {Record<string, unknown>} */ (value)
      : {};
  return (
    Object.prototype.hasOwnProperty.call(source, 'adversary_sim_enabled') ||
    Object.prototype.hasOwnProperty.call(source, 'adversary_sim_available') ||
    Object.prototype.hasOwnProperty.call(source, 'phase') ||
    Object.prototype.hasOwnProperty.call(source, 'runtime_environment')
  );
}

/**
 * @param {{
 *   runtimeMounted?: boolean,
 *   authenticated?: boolean,
 *   bootstrapInFlight?: boolean,
 *   configSnapshot?: Record<string, unknown>,
 *   adversarySimStatus?: unknown
 * }} source
 * @returns {boolean}
 */
export function shouldPrimeAdversarySimStatus(source = {}) {
  if (source.runtimeMounted !== true) return false;
  if (source.authenticated !== true) return false;
  if (source.bootstrapInFlight === true) return false;
  if (hasExplicitAdversarySimStatus(source.adversarySimStatus)) return false;
  return (
    source.configSnapshot &&
    typeof source.configSnapshot === 'object' &&
    source.configSnapshot.adversary_sim_available === true
  );
}
