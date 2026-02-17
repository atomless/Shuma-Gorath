// @ts-check

export const createRuntimeEffects = (options = {}) => {
  const win = options.window || window;
  const nav = options.navigator || navigator;
  const requestImpl =
    typeof options.request === 'function' ? options.request : null;

  const setTimeoutFn =
    typeof options.setTimeout === 'function'
      ? options.setTimeout
      : win.setTimeout.bind(win);
  const clearTimeoutFn =
    typeof options.clearTimeout === 'function'
      ? options.clearTimeout
      : win.clearTimeout.bind(win);

  const copyText = async (text = '') => {
    const value = String(text || '');
    if (!nav || !nav.clipboard || typeof nav.clipboard.writeText !== 'function') {
      throw new Error('Clipboard API unavailable');
    }
    await nav.clipboard.writeText(value);
  };

  // Resolve window.fetch at call time so late-installed wrappers (for example
  // admin session CSRF injection) are respected.
  const request = (input, init = {}) => {
    if (requestImpl) return requestImpl(input, init);
    return win.fetch(input, init);
  };

  const setTimer = (task, ms = 0) => setTimeoutFn(task, ms);
  const clearTimer = (id) => clearTimeoutFn(id);

  return {
    request,
    copyText,
    setTimer,
    clearTimer
  };
};
