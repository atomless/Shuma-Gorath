// @ts-check

const asRecord = (value) =>
  value && typeof value === 'object' ? /** @type {Record<string, unknown>} */ (value) : {};

export function normalizeConfigRuntimeSnapshot(value = {}) {
  return asRecord(value);
}

export function isAdminConfigWritable(runtimeSnapshot = {}) {
  const source = normalizeConfigRuntimeSnapshot(runtimeSnapshot);
  return source.admin_config_write_enabled === true;
}

export function isAkamaiEdgeAvailable(runtimeSnapshot = {}) {
  const source = normalizeConfigRuntimeSnapshot(runtimeSnapshot);
  return source.akamai_edge_available === true;
}

export function getRuntimeEnvironment(runtimeSnapshot = {}) {
  const source = normalizeConfigRuntimeSnapshot(runtimeSnapshot);
  return String(source.runtime_environment || '');
}

export function getGatewayDeploymentProfile(runtimeSnapshot = {}) {
  const source = normalizeConfigRuntimeSnapshot(runtimeSnapshot);
  return String(source.gateway_deployment_profile || '');
}

export function getFrontierProviderCount(runtimeSnapshot = {}) {
  const source = normalizeConfigRuntimeSnapshot(runtimeSnapshot);
  const numeric = Number(source.frontier_provider_count || 0);
  if (!Number.isFinite(numeric) || numeric < 0) return 0;
  return Math.floor(numeric);
}

export function isAdversarySimSurfaceAvailable(runtimeSnapshot = {}) {
  const source = normalizeConfigRuntimeSnapshot(runtimeSnapshot);
  return source.adversary_sim_available === true;
}
