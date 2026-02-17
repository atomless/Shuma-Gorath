// @ts-check

export const createRuntimeEffects = (options = {}) => {
  const win = options.window || window;
  const nav = options.navigator || navigator;
  const requestImpl =
    typeof options.request === 'function' ? options.request : win.fetch.bind(win);

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

  const request = (input, init = {}) => requestImpl(input, init);

  const setTimer = (task, ms = 0) => setTimeoutFn(task, ms);
  const clearTimer = (id) => clearTimeoutFn(id);

  return {
    request,
    copyText,
    setTimer,
    clearTimer
  };
};
