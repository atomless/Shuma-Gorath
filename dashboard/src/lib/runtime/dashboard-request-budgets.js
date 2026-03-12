// @ts-check

const DEFAULT_MONITORING_REQUEST_TIMEOUT_MS = 12_000;
const DEFAULT_MONITORING_DELTA_TIMEOUT_MS = 12_000;
const DEFAULT_CONFIG_WRITE_TIMEOUT_MS = 12_000;
const DEFAULT_ADVERSARY_SIM_CONTROL_TIMEOUT_MS = 15_000;
const DEFAULT_ADVERSARY_SIM_ENABLE_TIMEOUT_MS = 45_000;
const DEFAULT_ADVERSARY_SIM_DISABLE_TIMEOUT_MS = 30_000;
const DEFAULT_ADVERSARY_SIM_STATUS_TIMEOUT_MS = 12_000;
const EDGE_MONITORING_REQUEST_TIMEOUT_MS = 45_000;
const EDGE_MONITORING_DELTA_TIMEOUT_MS = 20_000;
const EDGE_CONFIG_WRITE_TIMEOUT_MS = 45_000;
const EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_MS = 45_000;
const EDGE_ADVERSARY_SIM_ENABLE_TIMEOUT_MS = 90_000;
const EDGE_ADVERSARY_SIM_DISABLE_TIMEOUT_MS = 45_000;
const EDGE_ADVERSARY_SIM_STATUS_TIMEOUT_MS = 30_000;

/**
 * @param {unknown} value
 * @returns {'' | 'shared-server' | 'edge-fermyon'}
 */
export function normalizeGatewayDeploymentProfile(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === 'shared-server' || normalized === 'edge-fermyon') {
    return normalized;
  }
  return '';
}

/**
 * @param {unknown} value
 * @returns {boolean}
 */
export function isEdgeFermyonDeploymentProfile(value) {
  return normalizeGatewayDeploymentProfile(value) === 'edge-fermyon';
}

/**
 * @param {Record<string, unknown> | null | undefined} configSnapshot
 * @returns {{
 *   gatewayDeploymentProfile: '' | 'shared-server' | 'edge-fermyon',
 *   autoHydrateFullMonitoring: boolean,
 *   monitoringRequestTimeoutMs: number,
 *   monitoringDeltaTimeoutMs: number,
 *   configWriteTimeoutMs: number,
 *   adversarySimControlTimeoutMs: number,
 *   adversarySimEnableTimeoutMs: number,
 *   adversarySimDisableTimeoutMs: number,
 *   adversarySimStatusTimeoutMs: number
 * }}
 */
export function deriveDashboardRequestBudgets(configSnapshot = {}) {
  const source =
    configSnapshot && typeof configSnapshot === 'object'
      ? /** @type {Record<string, unknown>} */ (configSnapshot)
      : {};
  const gatewayDeploymentProfile = normalizeGatewayDeploymentProfile(
    source.gateway_deployment_profile
  );
  const edgeFermyon = gatewayDeploymentProfile === 'edge-fermyon';

  return {
    gatewayDeploymentProfile,
    autoHydrateFullMonitoring: edgeFermyon !== true,
    monitoringRequestTimeoutMs: edgeFermyon
      ? EDGE_MONITORING_REQUEST_TIMEOUT_MS
      : DEFAULT_MONITORING_REQUEST_TIMEOUT_MS,
    monitoringDeltaTimeoutMs: edgeFermyon
      ? EDGE_MONITORING_DELTA_TIMEOUT_MS
      : DEFAULT_MONITORING_DELTA_TIMEOUT_MS,
    configWriteTimeoutMs: edgeFermyon
      ? EDGE_CONFIG_WRITE_TIMEOUT_MS
      : DEFAULT_CONFIG_WRITE_TIMEOUT_MS,
    adversarySimControlTimeoutMs: edgeFermyon
      ? EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_MS
      : DEFAULT_ADVERSARY_SIM_CONTROL_TIMEOUT_MS,
    adversarySimEnableTimeoutMs: edgeFermyon
      ? EDGE_ADVERSARY_SIM_ENABLE_TIMEOUT_MS
      : DEFAULT_ADVERSARY_SIM_ENABLE_TIMEOUT_MS,
    adversarySimDisableTimeoutMs: edgeFermyon
      ? EDGE_ADVERSARY_SIM_DISABLE_TIMEOUT_MS
      : DEFAULT_ADVERSARY_SIM_DISABLE_TIMEOUT_MS,
    adversarySimStatusTimeoutMs: edgeFermyon
      ? EDGE_ADVERSARY_SIM_STATUS_TIMEOUT_MS
      : DEFAULT_ADVERSARY_SIM_STATUS_TIMEOUT_MS
  };
}
