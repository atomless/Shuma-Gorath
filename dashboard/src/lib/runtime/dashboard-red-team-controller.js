// @ts-check

import {
  adversarySimStateMatchesDesired,
  normalizeAdversarySimStatus
} from './dashboard-adversary-sim.js';

const DEFAULT_DEBOUNCE_MS = 300;
const DEFAULT_POLL_INTERVAL_MS = 1000;
const DEFAULT_CONVERGENCE_TIMEOUT_MS = 30_000;

const asRecord = (value) =>
  value && typeof value === 'object'
    ? /** @type {Record<string, unknown>} */ (value)
    : {};

const unwrapStatus = (value) => {
  const source = asRecord(value);
  const nestedStatus = asRecord(source.status);
  return Object.keys(nestedStatus).length > 0 ? nestedStatus : source;
};

const defaultSchedule = (callback, delayMs = 0) =>
  setTimeout(callback, Math.max(0, Number(delayMs || 0)));
const defaultCancelScheduled = (handle) => clearTimeout(handle);
const defaultNowMs = () => Date.now();

/**
 * @typedef {'idle' | 'debouncing' | 'submitting' | 'converging' | 'error'} ControllerPhase
 */

/**
 * @param {{
 *   initialStatus?: unknown,
 *   debounceMs?: number,
 *   pollIntervalMs?: number,
 *   nowMs?: () => number,
 *   schedule?: (callback: () => void | Promise<void>, delayMs?: number) => unknown,
 *   cancelScheduled?: (handle: unknown) => void,
 *   submitControl?: (desiredEnabled: boolean) => Promise<unknown>,
 *   fetchStatus?: (reason?: string) => Promise<unknown>,
 *   normalizeStatus?: (payload: unknown) => ReturnType<typeof normalizeAdversarySimStatus>,
 *   stateMatchesDesired?: (payload: unknown, desiredEnabled: boolean) => boolean,
 *   isPollingAllowed?: () => boolean,
 *   resolveConvergenceTimeoutMs?: (desiredEnabled: boolean) => number,
 *   onControlAccepted?: (desiredEnabled: boolean, response: unknown) => void,
 *   onSettled?: (desiredEnabled: boolean, status: Record<string, unknown>) => void,
 *   onError?: (message: string, error: unknown) => void
 * }} [options]
 */
export function createDashboardRedTeamController(options = {}) {
  const normalizeStatus =
    typeof options.normalizeStatus === 'function'
      ? options.normalizeStatus
      : normalizeAdversarySimStatus;
  const stateMatchesDesired =
    typeof options.stateMatchesDesired === 'function'
      ? options.stateMatchesDesired
      : adversarySimStateMatchesDesired;
  const nowMs = typeof options.nowMs === 'function' ? options.nowMs : defaultNowMs;
  const schedule =
    typeof options.schedule === 'function' ? options.schedule : defaultSchedule;
  const cancelScheduled =
    typeof options.cancelScheduled === 'function'
      ? options.cancelScheduled
      : defaultCancelScheduled;
  const submitControl =
    typeof options.submitControl === 'function'
      ? options.submitControl
      : async (desiredEnabled) => ({
        status: {
          adversary_sim_enabled: desiredEnabled === true,
          generation_active: desiredEnabled === true,
          phase: desiredEnabled === true ? 'running' : 'off'
        }
      });
  const fetchStatus =
    typeof options.fetchStatus === 'function'
      ? options.fetchStatus
      : async () => unwrapStatus(options.initialStatus);
  const isPollingAllowed =
    typeof options.isPollingAllowed === 'function' ? options.isPollingAllowed : () => true;
  const resolveConvergenceTimeoutMs =
    typeof options.resolveConvergenceTimeoutMs === 'function'
      ? options.resolveConvergenceTimeoutMs
      : () => DEFAULT_CONVERGENCE_TIMEOUT_MS;
  const onControlAccepted =
    typeof options.onControlAccepted === 'function' ? options.onControlAccepted : () => {};
  const onSettled =
    typeof options.onSettled === 'function' ? options.onSettled : () => {};
  const onError =
    typeof options.onError === 'function' ? options.onError : () => {};

  const debounceMs = Math.max(0, Number(options.debounceMs || DEFAULT_DEBOUNCE_MS));
  const pollIntervalMs = Math.max(0, Number(options.pollIntervalMs || DEFAULT_POLL_INTERVAL_MS));
  const initialStatus = unwrapStatus(options.initialStatus);
  const initialNormalizedStatus = normalizeStatus(initialStatus);
  const listeners = new Set();

  /** @type {unknown} */
  let debounceTimer = null;
  /** @type {unknown} */
  let pollTimer = null;
  /** @type {Promise<unknown> | null} */
  let statusRequestInFlight = null;
  let convergenceDeadlineAt = 0;

  let state = {
    backendStatus: initialStatus,
    uiDesiredEnabled: initialNormalizedStatus.enabled === true,
    lastBackendDesiredEnabled: initialNormalizedStatus.enabled === true,
    lastSubmittedDesiredEnabled: null,
    inFlightDesiredEnabled: null,
    queuedDesiredEnabled: null,
    controllerPhase: /** @type {ControllerPhase} */ ('idle'),
    lastError: '',
    statusPrimed: Object.keys(initialStatus).length > 0,
    bootstrapInFlight: false
  };

  const snapshot = () => ({ ...state, backendStatus: asRecord(state.backendStatus) });

  const emit = () => {
    const current = snapshot();
    listeners.forEach((listener) => listener(current));
    return current;
  };

  const clearDebounceTimer = () => {
    if (debounceTimer === null) return;
    cancelScheduled(debounceTimer);
    debounceTimer = null;
  };

  const clearPollTimer = () => {
    if (pollTimer === null) return;
    cancelScheduled(pollTimer);
    pollTimer = null;
  };

  const localIntentOwnsUi = () =>
    state.controllerPhase === 'debouncing' ||
    state.inFlightDesiredEnabled !== null ||
    state.queuedDesiredEnabled !== null;

  const setBackendStatus = (status, syncUi = false) => {
    const nextStatus = unwrapStatus(status);
    const normalizedStatus = normalizeStatus(nextStatus);
    state = {
      ...state,
      backendStatus: nextStatus,
      lastBackendDesiredEnabled: normalizedStatus.enabled === true,
      statusPrimed: true
    };
    if (syncUi === true && !localIntentOwnsUi()) {
      state = {
        ...state,
        uiDesiredEnabled: normalizedStatus.enabled === true
      };
    }
    return normalizedStatus;
  };

  const shouldPoll = () => {
    if (!isPollingAllowed()) return false;
    const normalizedStatus = normalizeStatus(state.backendStatus);
    return (
      state.inFlightDesiredEnabled !== null ||
      normalizedStatus.enabled === true ||
      normalizedStatus.phase === 'running' ||
      normalizedStatus.phase === 'stopping'
    );
  };

  const handleFailure = (error, fallbackMessage) => {
    clearDebounceTimer();
    clearPollTimer();
    const message = String(error?.message || fallbackMessage || 'Adversary simulation update failed.').trim();
    state = {
      ...state,
      uiDesiredEnabled: state.lastBackendDesiredEnabled,
      inFlightDesiredEnabled: null,
      queuedDesiredEnabled: null,
      controllerPhase: 'error',
      lastError: message
    };
    emit();
    onError(message, error);
  };

  const schedulePoll = (delayMs = pollIntervalMs, reason = 'poll') => {
    clearPollTimer();
    if (!shouldPoll()) return;
    pollTimer = schedule(async () => {
      pollTimer = null;
      try {
        await refreshStatus(reason);
      } catch (_error) {}
    }, delayMs);
  };

  const maybeAdvance = async () => {
    if (state.inFlightDesiredEnabled !== null) {
      if (stateMatchesDesired(state.backendStatus, state.inFlightDesiredEnabled)) {
        const backendDesiredEnabled = state.lastBackendDesiredEnabled;
        const queuedDesiredEnabled = state.queuedDesiredEnabled;
        const settledStatus = asRecord(state.backendStatus);
        state = {
          ...state,
          inFlightDesiredEnabled: null,
          queuedDesiredEnabled: null,
          controllerPhase: 'idle',
          uiDesiredEnabled: queuedDesiredEnabled === null ? backendDesiredEnabled : state.uiDesiredEnabled
        };
        emit();
        onSettled(backendDesiredEnabled, settledStatus);
        if (
          queuedDesiredEnabled !== null &&
          queuedDesiredEnabled !== backendDesiredEnabled
        ) {
          await submitDesired(queuedDesiredEnabled);
          return;
        }
      } else if (nowMs() >= convergenceDeadlineAt) {
        handleFailure(
          new Error('Adversary simulation did not settle before the convergence timeout elapsed.'),
          'Adversary simulation did not settle before the convergence timeout elapsed.'
        );
        return;
      }
    }
    schedulePoll();
  };

  async function refreshStatus(reason = 'manual') {
    if (statusRequestInFlight) {
      return statusRequestInFlight;
    }

    if (reason === 'bootstrap') {
      state = { ...state, bootstrapInFlight: true };
      emit();
    }

    statusRequestInFlight = (async () => {
      const status = await fetchStatus(reason);
      setBackendStatus(status, state.inFlightDesiredEnabled === null && state.queuedDesiredEnabled === null);
      emit();
      await maybeAdvance();
      return snapshot();
    })()
      .catch((error) => {
        if (state.inFlightDesiredEnabled !== null) {
          handleFailure(error, 'Failed to refresh adversary simulation status.');
        }
        throw error;
      })
      .finally(() => {
        statusRequestInFlight = null;
        if (reason === 'bootstrap') {
          state = { ...state, bootstrapInFlight: false };
          emit();
        }
      });

    return statusRequestInFlight;
  }

  async function submitDesired(desiredEnabled) {
    clearPollTimer();
    state = {
      ...state,
      lastSubmittedDesiredEnabled: desiredEnabled,
      inFlightDesiredEnabled: desiredEnabled,
      controllerPhase: 'submitting',
      lastError: ''
    };
    emit();

    try {
      const response = await submitControl(desiredEnabled === true);
      const controlStatus = unwrapStatus(response);
      setBackendStatus(controlStatus, false);
      state = {
        ...state,
        controllerPhase: 'converging'
      };
      convergenceDeadlineAt = nowMs() + Math.max(
        1_000,
        Number(resolveConvergenceTimeoutMs(desiredEnabled === true) || DEFAULT_CONVERGENCE_TIMEOUT_MS)
      );
      emit();
      onControlAccepted(desiredEnabled === true, response);
      await maybeAdvance();
    } catch (error) {
      handleFailure(error, 'Failed to toggle adversary simulation.');
    }
  }

  const flushDesiredIntent = async () => {
    clearDebounceTimer();
    const desiredEnabled = state.uiDesiredEnabled === true;

    if (state.inFlightDesiredEnabled !== null) {
      state = {
        ...state,
        queuedDesiredEnabled:
          desiredEnabled === state.inFlightDesiredEnabled ? null : desiredEnabled
      };
      emit();
      return;
    }

    if (desiredEnabled === state.lastBackendDesiredEnabled) {
      state = {
        ...state,
        controllerPhase: 'idle',
        lastError: ''
      };
      emit();
      schedulePoll();
      return;
    }

    await submitDesired(desiredEnabled);
  };

  return {
    getState() {
      return snapshot();
    },
    subscribe(listener) {
      if (typeof listener !== 'function') {
        return () => {};
      }
      listeners.add(listener);
      listener(snapshot());
      return () => {
        listeners.delete(listener);
      };
    },
    async bootstrap() {
      if (state.statusPrimed === true) {
        schedulePoll();
        return snapshot();
      }
      return refreshStatus('bootstrap');
    },
    async refreshStatus(reason = 'manual') {
      return refreshStatus(reason);
    },
    replaceBackendStatus(status) {
      setBackendStatus(status, true);
      emit();
      schedulePoll();
      return snapshot();
    },
    handleToggleIntent(nextEnabled) {
      const desiredEnabled = nextEnabled === true;
      clearDebounceTimer();
      state = {
        ...state,
        uiDesiredEnabled: desiredEnabled,
        lastError: '',
        controllerPhase:
          state.inFlightDesiredEnabled !== null ? state.controllerPhase : 'debouncing'
      };
      emit();
      debounceTimer = schedule(async () => {
        await flushDesiredIntent();
      }, debounceMs);
    },
    handleTabActivated() {
      if (state.statusPrimed !== true || !shouldPoll()) {
        void refreshStatus('tab-activated');
      }
    },
    handleVisibilityResume() {
      if (isPollingAllowed()) {
        void refreshStatus('visibility-resume');
      }
    },
    dispose() {
      clearDebounceTimer();
      clearPollTimer();
      listeners.clear();
    }
  };
}
