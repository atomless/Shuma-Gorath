// @ts-check

let loadPromise = null;
let refCount = 0;
let adapterOwnedGlobal = false;
let runtimeWindowRef = null;

const chartConstructorFrom = (win) =>
  win && typeof win.Chart === 'function' ? win.Chart : null;

const resolveChartCtor = async (loader) => {
  const loaded = await loader();
  const ctor =
    (loaded && typeof loaded.Chart === 'function' ? loaded.Chart : null) ||
    (loaded && typeof loaded.default === 'function' ? loaded.default : null) ||
    (typeof loaded === 'function' ? loaded : null);
  if (typeof ctor === 'function') {
    return ctor;
  }
  throw new Error('Chart runtime loaded but Chart constructor is unavailable.');
};

const defaultLoader = () => import('chart.js/auto');

export async function acquireChartRuntime(options = {}) {
  const win = options.window || (typeof window !== 'undefined' ? window : null);
  if (!win) {
    throw new Error('Chart runtime requires browser window context.');
  }
  runtimeWindowRef = win;

  const existing = chartConstructorFrom(win);
  if (existing) {
    refCount += 1;
    return existing;
  }

  if (!loadPromise) {
    const loader = typeof options.loader === 'function' ? options.loader : defaultLoader;
    loadPromise = resolveChartCtor(loader).then((ctor) => {
      if (typeof win.Chart !== 'function') {
        win.Chart = ctor;
        adapterOwnedGlobal = true;
      }
      return chartConstructorFrom(win);
    });
  }

  try {
    const chart = await loadPromise;
    if (typeof chart !== 'function') {
      throw new Error('Chart runtime loaded but window.Chart is unavailable.');
    }
    refCount += 1;
    return chart;
  } catch (error) {
    loadPromise = null;
    throw error;
  }
}

export function getChartConstructor(options = {}) {
  const win = options.window || (typeof window !== 'undefined' ? window : null);
  return chartConstructorFrom(win);
}

export function releaseChartRuntime(options = {}) {
  if (refCount > 0) {
    refCount -= 1;
  }
  if (refCount > 0) return;

  loadPromise = null;
  const win =
    options.window ||
    runtimeWindowRef ||
    (typeof window !== 'undefined' ? window : null);

  if (adapterOwnedGlobal && win && Object.prototype.hasOwnProperty.call(win, 'Chart')) {
    try {
      delete win.Chart;
    } catch (_error) {
      win.Chart = undefined;
    }
  }

  adapterOwnedGlobal = false;
  runtimeWindowRef = null;
}
