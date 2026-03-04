// @ts-check

export const REQUEST_OUTCOMES = Object.freeze({
  success: 'success',
  failure: 'failure'
});

export const REQUEST_FAILURE_CLASSES = Object.freeze({
  cancelled: 'cancelled',
  timeout: 'timeout',
  transport: 'transport',
  http: 'http'
});

/**
 * @param {unknown} value
 * @returns {number}
 */
const toStatusCode = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return 0;
  return Math.floor(numeric);
};

/**
 * @param {unknown} error
 * @returns {boolean}
 */
export const isAbortLikeError = (error) => {
  if (!error) return false;
  const name = String(error && typeof error === 'object' ? error.name || '' : '');
  const message = String(error && typeof error === 'object' ? error.message || '' : '');
  return name === 'AbortError' || message.toLowerCase().includes('abort');
};

/**
 * @param {unknown} error
 * @param {{ didTimeout?: boolean, statusCode?: number }} [options]
 * @returns {'cancelled' | 'timeout' | 'transport' | 'http'}
 */
export const classifyRequestFailure = (error, options = {}) => {
  const didTimeout = options && options.didTimeout === true;
  if (didTimeout) {
    return REQUEST_FAILURE_CLASSES.timeout;
  }
  const statusCode = toStatusCode(options && options.statusCode);
  if (statusCode > 0) {
    return REQUEST_FAILURE_CLASSES.http;
  }
  const errorStatus = toStatusCode(error && typeof error === 'object' ? error.status : 0);
  if (errorStatus > 0) {
    return REQUEST_FAILURE_CLASSES.http;
  }
  if (isAbortLikeError(error)) {
    return REQUEST_FAILURE_CLASSES.cancelled;
  }
  return REQUEST_FAILURE_CLASSES.transport;
};
