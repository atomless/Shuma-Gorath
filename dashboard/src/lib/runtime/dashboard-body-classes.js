// @ts-check

const RUNTIME_DEV_CLASS = 'runtime-dev';
const RUNTIME_PROD_CLASS = 'runtime-prod';
const SHADOW_MODE_CLASS = 'shadow-mode';
const ADVERSARY_SIM_CLASS = 'adversary-sim';
const CONNECTED_CLASS = 'connected';
const DEGRADED_CLASS = 'degraded';
const DISCONNECTED_CLASS = 'disconnected';
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
  if (normalized === RUNTIME_DEV_CLASS) return RUNTIME_DEV_CLASS;
  if (normalized === RUNTIME_PROD_CLASS) return RUNTIME_PROD_CLASS;
  return '';
};

const normalizeConnectionState = (value) => {
  const normalized = String(value || '').trim().toLowerCase();
  if (
    normalized === CONNECTED_CLASS ||
    normalized === DEGRADED_CLASS ||
    normalized === DISCONNECTED_CLASS
  ) {
    return normalized;
  }
  return DISCONNECTED_CLASS;
};

const applyRuntimeClass = (classList, runtimeClass) => {
  if (!classList || typeof classList.toggle !== 'function') return;
  classList.toggle(RUNTIME_DEV_CLASS, runtimeClass === RUNTIME_DEV_CLASS);
  classList.toggle(RUNTIME_PROD_CLASS, runtimeClass === RUNTIME_PROD_CLASS);
};

const getClassList = (target) =>
  target &&
  typeof target === 'object' &&
  target.classList &&
  typeof target.classList.add === 'function' &&
  typeof target.classList.remove === 'function'
    ? target.classList
    : null;

const getDashboardClassLists = (doc) => ({
  bodyClassList: getClassList(doc?.body),
  rootClassList: getClassList(doc?.documentElement)
});

/**
 * @param {unknown} configSnapshot
 * @param {{ backendConnected?: unknown, backendConnectionState?: unknown, runtimeClassHint?: unknown, shadowModeEnabled?: unknown, adversarySimEnabled?: unknown }} [options]
 * @returns {{ runtimeClass: '' | 'runtime-dev' | 'runtime-prod', shadowModeEnabled: boolean, adversarySimEnabled: boolean, connectionState: 'connected' | 'degraded' | 'disconnected' }}
 */
export function deriveDashboardBodyClassState(configSnapshot = {}, options = {}) {
  const source = asRecord(configSnapshot);
  const optionConnectedState = options && typeof options === 'object'
    ? options.backendConnectionState
    : undefined;
  const optionConnected = options && typeof options === 'object'
    ? options.backendConnected
    : undefined;
  const optionRuntimeClassHint = options && typeof options === 'object'
    ? options.runtimeClassHint
    : undefined;
  const optionTestModeEnabled = options && typeof options === 'object'
    ? options.shadowModeEnabled
    : undefined;
  const optionAdversarySimEnabled = options && typeof options === 'object'
    ? options.adversarySimEnabled
    : undefined;
  const runtimeEnvironment = normalizeRuntimeClass(source.runtime_environment);
  const runtimeClass = runtimeEnvironment
    ? runtimeEnvironment
    : normalizeRuntimeClass(optionRuntimeClassHint);
  const configConnected = source.backend_connected;
  const connectionState = optionConnectedState !== undefined
    ? normalizeConnectionState(optionConnectedState)
    : optionConnected !== undefined
      ? (parseBoolLike(optionConnected, false) ? CONNECTED_CLASS : DISCONNECTED_CLASS)
      : configConnected !== undefined
        ? (parseBoolLike(configConnected, false) ? CONNECTED_CLASS : DISCONNECTED_CLASS)
        : DISCONNECTED_CLASS;
  const guardedConnectionState = runtimeClass ? connectionState : DISCONNECTED_CLASS;
  return {
    runtimeClass,
    shadowModeEnabled: optionTestModeEnabled !== undefined
      ? parseBoolLike(optionTestModeEnabled, false)
      : parseBoolLike(source.shadow_mode, false),
    // The root adversary-sim class must follow backend truth only, never config intent.
    adversarySimEnabled: parseBoolLike(optionAdversarySimEnabled, false),
    connectionState: guardedConnectionState
  };
}

/**
 * @param {unknown} doc
 * @param {{ runtimeClass?: unknown, shadowModeEnabled?: unknown, adversarySimEnabled?: unknown, backendConnected?: unknown, connectionState?: unknown }} state
 * @returns {{ runtimeClass: '' | 'runtime-dev' | 'runtime-prod', shadowModeEnabled: boolean, adversarySimEnabled: boolean, connectionState: 'connected' | 'degraded' | 'disconnected' }}
 */
export function syncDashboardBodyClasses(doc, state = {}) {
  const { bodyClassList, rootClassList } = getDashboardClassLists(doc);
  const fallbackConnectionState = state.backendConnected === undefined
    ? DISCONNECTED_CLASS
    : (parseBoolLike(state.backendConnected, false) ? CONNECTED_CLASS : DISCONNECTED_CLASS);
  const normalizedState = {
    runtimeClass: normalizeRuntimeClass(state.runtimeClass),
    shadowModeEnabled: parseBoolLike(state.shadowModeEnabled, false),
    adversarySimEnabled: parseBoolLike(state.adversarySimEnabled, false),
    connectionState: normalizeConnectionState(
      state.connectionState === undefined ? fallbackConnectionState : state.connectionState
    )
  };
  if (!normalizedState.runtimeClass) {
    normalizedState.connectionState = DISCONNECTED_CLASS;
  }
  if (!bodyClassList && !rootClassList) return normalizedState;

  if (rootClassList) {
    applyRuntimeClass(rootClassList, normalizedState.runtimeClass);
    rootClassList.toggle(SHADOW_MODE_CLASS, normalizedState.shadowModeEnabled);
    rootClassList.toggle(ADVERSARY_SIM_CLASS, normalizedState.adversarySimEnabled);
    rootClassList.toggle(CONNECTED_CLASS, normalizedState.connectionState === CONNECTED_CLASS);
    rootClassList.toggle(DEGRADED_CLASS, normalizedState.connectionState === DEGRADED_CLASS);
    rootClassList.toggle(DISCONNECTED_CLASS, normalizedState.connectionState === DISCONNECTED_CLASS);
  }

  if (bodyClassList) {
    applyRuntimeClass(bodyClassList, '');
    bodyClassList.remove(SHADOW_MODE_CLASS);
    bodyClassList.remove(CONNECTED_CLASS);
    bodyClassList.remove(DEGRADED_CLASS);
    bodyClassList.remove(DISCONNECTED_CLASS);
    bodyClassList.remove(ADVERSARY_SIM_CLASS);
  }
  return normalizedState;
}

/**
 * @param {unknown} doc
 */
export function clearDashboardBodyClasses(doc) {
  const { bodyClassList, rootClassList } = getDashboardClassLists(doc);
  if (!bodyClassList && !rootClassList) return;

  if (rootClassList) {
    for (const runtimeClass of RUNTIME_CLASSES) {
      rootClassList.remove(runtimeClass);
    }
    rootClassList.remove(SHADOW_MODE_CLASS);
    rootClassList.remove(ADVERSARY_SIM_CLASS);
    rootClassList.remove(CONNECTED_CLASS);
    rootClassList.remove(DEGRADED_CLASS);
    rootClassList.remove(DISCONNECTED_CLASS);
  }

  if (bodyClassList) {
    for (const runtimeClass of RUNTIME_CLASSES) {
      bodyClassList.remove(runtimeClass);
    }
    bodyClassList.remove(SHADOW_MODE_CLASS);
    bodyClassList.remove(ADVERSARY_SIM_CLASS);
    bodyClassList.remove(CONNECTED_CLASS);
    bodyClassList.remove(DEGRADED_CLASS);
    bodyClassList.remove(DISCONNECTED_CLASS);
  }
}
