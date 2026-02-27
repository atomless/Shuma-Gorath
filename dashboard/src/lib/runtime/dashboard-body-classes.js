// @ts-check

const RUNTIME_DEV_CLASS = 'runtime-dev';
const RUNTIME_PROD_CLASS = 'runtime-prod';
const ADVERSARY_SIM_CLASS = 'adversary-sim';
const RUNTIME_CLASSES = Object.freeze([RUNTIME_DEV_CLASS, RUNTIME_PROD_CLASS]);

const asRecord = (value) =>
  value && typeof value === 'object' ? /** @type {Record<string, unknown>} */ (value) : {};

const parseBoolLike = (value, fallback = false) => {
  if (typeof value === 'boolean') return value;
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') {
    return true;
  }
  if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') {
    return false;
  }
  return fallback;
};

const normalizeRuntimeClass = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  return normalized === RUNTIME_DEV_CLASS ? RUNTIME_DEV_CLASS : RUNTIME_PROD_CLASS;
};

const getBodyClassList = (doc) =>
  doc &&
  typeof doc === 'object' &&
  doc.body &&
  doc.body.classList &&
  typeof doc.body.classList.add === 'function' &&
  typeof doc.body.classList.remove === 'function'
    ? doc.body.classList
    : null;

/**
 * @param {unknown} configSnapshot
 * @returns {{ runtimeClass: 'runtime-dev' | 'runtime-prod', adversarySimEnabled: boolean }}
 */
export function deriveDashboardBodyClassState(configSnapshot = {}) {
  const source = asRecord(configSnapshot);
  return {
    runtimeClass: normalizeRuntimeClass(source.runtime_environment),
    adversarySimEnabled: parseBoolLike(source.adversary_sim_enabled, false)
  };
}

/**
 * @param {unknown} doc
 * @param {{ runtimeClass?: unknown, adversarySimEnabled?: unknown }} state
 * @returns {{ runtimeClass: 'runtime-dev' | 'runtime-prod', adversarySimEnabled: boolean }}
 */
export function syncDashboardBodyClasses(doc, state = {}) {
  const classList = getBodyClassList(doc);
  const normalizedState = {
    runtimeClass: normalizeRuntimeClass(state.runtimeClass),
    adversarySimEnabled: parseBoolLike(state.adversarySimEnabled, false)
  };
  if (!classList) return normalizedState;

  for (const runtimeClass of RUNTIME_CLASSES) {
    classList.remove(runtimeClass);
  }
  classList.add(normalizedState.runtimeClass);
  classList.toggle(ADVERSARY_SIM_CLASS, normalizedState.adversarySimEnabled);
  return normalizedState;
}

/**
 * @param {unknown} doc
 */
export function clearDashboardBodyClasses(doc) {
  const classList = getBodyClassList(doc);
  if (!classList) return;

  for (const runtimeClass of RUNTIME_CLASSES) {
    classList.remove(runtimeClass);
  }
  classList.remove(ADVERSARY_SIM_CLASS);
}
