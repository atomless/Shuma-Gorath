// @ts-check

const asRecord = (value) =>
  value && typeof value === 'object' ? /** @type {Record<string, unknown>} */ (value) : {};

export function normalizeConfigSnapshot(value = {}) {
  return asRecord(value);
}

export function isConfigSnapshotEmpty(configSnapshot = {}) {
  return Object.keys(normalizeConfigSnapshot(configSnapshot)).length === 0;
}

export function hasRequiredSharedConfigRuntimeTruth(runtimeSnapshot = {}) {
  const source = asRecord(runtimeSnapshot);
  if (!Object.prototype.hasOwnProperty.call(source, 'admin_config_write_enabled')) {
    return false;
  }
  return typeof source.runtime_environment === 'string' && source.runtime_environment.trim().length > 0;
}

export function isConfigRuntimeSnapshotEmpty(runtimeSnapshot = {}) {
  const source = asRecord(runtimeSnapshot);
  return Object.keys(source).length === 0 || !hasRequiredSharedConfigRuntimeTruth(source);
}

export function hasHydratedConfigEnvelope(configSnapshot = {}, runtimeSnapshot = {}) {
  return !isConfigSnapshotEmpty(configSnapshot) && !isConfigRuntimeSnapshotEmpty(runtimeSnapshot);
}
