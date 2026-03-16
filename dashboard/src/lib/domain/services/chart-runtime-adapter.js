// @ts-check

let loadPromise = null;
let refCount = 0;
let adapterOwnedGlobal = false;
let runtimeWindowRef = null;

export const SHUMA_CHART_ANIMATION_DURATION_MS = 0;

const chartConstructorFrom = (win) =>
  win && typeof win.Chart === 'function' ? win.Chart : null;

const applySharedChartDefaults = (ctor) => {
  if (typeof ctor !== 'function') {
    return ctor;
  }
  const defaults =
    ctor.defaults && typeof ctor.defaults === 'object'
      ? ctor.defaults
      : (ctor.defaults = {});
  const animation =
    defaults.animation && typeof defaults.animation === 'object'
      ? { ...defaults.animation }
      : {};
  animation.duration = SHUMA_CHART_ANIMATION_DURATION_MS;
  defaults.animation = animation;
  return ctor;
};

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
    return applySharedChartDefaults(existing);
  }

  if (!loadPromise) {
    const loader = typeof options.loader === 'function' ? options.loader : defaultLoader;
    loadPromise = resolveChartCtor(loader).then((ctor) => {
      if (typeof win.Chart !== 'function') {
        win.Chart = ctor;
        adapterOwnedGlobal = true;
      }
      return applySharedChartDefaults(chartConstructorFrom(win));
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
