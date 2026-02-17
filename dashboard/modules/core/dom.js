// @ts-check

export const createCache = (options = {}) => {
  const doc = options.document || document;
  const idCache = new Map();
  const queryCache = new Map();

  const byId = (id) => {
    if (idCache.has(id)) return idCache.get(id);
    const el = doc.getElementById(id);
    idCache.set(id, el || null);
    return el || null;
  };

  const query = (selector) => {
    if (queryCache.has(selector)) return queryCache.get(selector);
    const el = doc.querySelector(selector);
    queryCache.set(selector, el || null);
    return el || null;
  };

  const queryAll = (selector) => Array.from(doc.querySelectorAll(selector));

  return {
    byId,
    query,
    queryAll,
    clear() {
      idCache.clear();
      queryCache.clear();
    }
  };
};

export const setText = (el, nextText) => {
  if (!el) return;
  const next = String(nextText ?? '');
  if (el.textContent !== next) {
    el.textContent = next;
  }
};

export const setHtml = (el, nextHtml) => {
  if (!el) return;
  const next = String(nextHtml ?? '');
  if (el.innerHTML !== next) {
    el.innerHTML = next;
  }
};

export const setValue = (el, nextValue) => {
  if (!el) return;
  const next = String(nextValue ?? '');
  if (el.value !== next) {
    el.value = next;
  }
};

export const setChecked = (el, checked) => {
  if (!el) return;
  const next = checked === true;
  if (el.checked !== next) {
    el.checked = next;
  }
};

export const createWriteScheduler = (options = {}) => {
  const win = options.window || globalThis;
  const queue = [];
  let scheduled = false;

  const flush = () => {
    scheduled = false;
    const tasks = queue.splice(0, queue.length);
    tasks.forEach((task) => {
      try {
        task();
      } catch (err) {
        console.error('Dashboard write task failed:', err);
      }
    });
  };

  const schedule = (task) => {
    if (typeof task !== 'function') return;
    queue.push(task);
    if (scheduled) return;
    scheduled = true;
    const raf = typeof win.requestAnimationFrame === 'function'
      ? win.requestAnimationFrame.bind(win)
      : (cb) => win.setTimeout(cb, 0);
    raf(flush);
  };

  return { schedule };
};
