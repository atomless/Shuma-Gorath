// @ts-check

const DEFAULT_RUNTIME_CLASS = 'runtime-dev';
const DEFAULT_LEGEND_FALLBACK_SATURATION = 20;
const DEFAULT_LEGEND_FALLBACK_LIGHTNESS = 33;

export const MONITORING_RUNTIME_HUES = Object.freeze({
  'runtime-dev': 310,
  'runtime-prod': 210
});

export const MONITORING_CHART_PALETTE_STOPS = Object.freeze([
  Object.freeze({ saturation: 100, lightness: 90 }),
  Object.freeze({ saturation: 45, lightness: 78 }),
  Object.freeze({ saturation: 33, lightness: 71 }),
  Object.freeze({ saturation: 28, lightness: 65 }),
  Object.freeze({ saturation: 24, lightness: 59 }),
  Object.freeze({ saturation: 21, lightness: 53 }),
  Object.freeze({ saturation: 20, lightness: 48 }),
  Object.freeze({ saturation: 23, lightness: 43 })
]);

const normalizeHue = (value, fallback) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  const wrapped = Math.round(numeric) % 360;
  return wrapped < 0 ? wrapped + 360 : wrapped;
};

const buildTimeSeriesFill = (palette) =>
  Object.freeze({
    events: String(palette[0] || ''),
    challenge: String(palette[1] || ''),
    pow: String(palette[2] || '')
  });

const resolveRuntimeHueFromBodyClass = (documentRef) => {
  const classList = documentRef?.body?.classList;
  if (!classList || typeof classList.contains !== 'function') return null;
  if (classList.contains('runtime-prod')) return MONITORING_RUNTIME_HUES['runtime-prod'];
  if (classList.contains('runtime-dev')) return MONITORING_RUNTIME_HUES['runtime-dev'];
  return null;
};

const resolveCssHueOverride = (documentRef, windowRef) => {
  if (!documentRef?.body || !windowRef || typeof windowRef.getComputedStyle !== 'function') {
    return null;
  }
  const computed = windowRef.getComputedStyle(documentRef.body);
  const raw = computed && typeof computed.getPropertyValue === 'function'
    ? computed.getPropertyValue('--hue')
    : '';
  const trimmed = String(raw || '').trim();
  if (!trimmed) return null;
  const numeric = Number(trimmed);
  return Number.isFinite(numeric) ? numeric : null;
};

const resolveLegendColor = (documentRef, windowRef, hue) => {
  if (!documentRef?.body || !windowRef || typeof windowRef.getComputedStyle !== 'function') {
    return `hsl(${hue}, ${DEFAULT_LEGEND_FALLBACK_SATURATION}%, ${DEFAULT_LEGEND_FALLBACK_LIGHTNESS}%)`;
  }
  const computed = windowRef.getComputedStyle(documentRef.body);
  const color = computed && typeof computed.getPropertyValue === 'function'
    ? String(computed.getPropertyValue('--muted-fg') || '').trim()
    : '';
  if (color) return color;
  return `hsl(${hue}, ${DEFAULT_LEGEND_FALLBACK_SATURATION}%, ${DEFAULT_LEGEND_FALLBACK_LIGHTNESS}%)`;
};

export const buildMonitoringChartPalette = (hue = MONITORING_RUNTIME_HUES[DEFAULT_RUNTIME_CLASS]) => {
  const normalizedHue = normalizeHue(hue, MONITORING_RUNTIME_HUES[DEFAULT_RUNTIME_CLASS]);
  return Object.freeze(
    MONITORING_CHART_PALETTE_STOPS.map(
      (stop) => `hsl(${normalizedHue}, ${stop.saturation}%, ${stop.lightness}%)`
    )
  );
};

export const MONITORING_CHART_PALETTE = buildMonitoringChartPalette(
  MONITORING_RUNTIME_HUES[DEFAULT_RUNTIME_CLASS]
);

export const MONITORING_TIME_SERIES_FILL = buildTimeSeriesFill(MONITORING_CHART_PALETTE);

export const resolveMonitoringChartTheme = (options = {}) => {
  const documentRef = options.documentRef || (typeof document !== 'undefined' ? document : null);
  const windowRef = options.windowRef || (typeof window !== 'undefined' ? window : null);
  const runtimeHue = resolveRuntimeHueFromBodyClass(documentRef);
  const cssHue = resolveCssHueOverride(documentRef, windowRef);
  const fallbackHue = MONITORING_RUNTIME_HUES[DEFAULT_RUNTIME_CLASS];
  const hue = normalizeHue(
    cssHue === null ? (runtimeHue === null ? options.hue : runtimeHue) : cssHue,
    fallbackHue
  );
  const palette = buildMonitoringChartPalette(hue);
  return Object.freeze({
    hue,
    palette,
    timeSeriesFill: buildTimeSeriesFill(palette),
    legendColor: resolveLegendColor(documentRef, windowRef, hue)
  });
};

export const MONITORING_TIME_SERIES_TICK_MAX = 8;
export const MONITORING_TIME_SERIES_TICK_PADDING = 10;
export const MONITORING_TIME_SERIES_AXIS_HEIGHT_PX = 34;
export const MONITORING_TIME_SERIES_TICK_MAX_CHARS = 12;
export const MONITORING_TIME_SERIES_OMIT_FINAL_TICK_LABEL = true;

const toBoundedInteger = (value, fallback, min, max) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return Math.min(max, Math.max(min, Math.floor(numeric)));
};

const normalizeTickLabel = (value, maxChars) => {
  const text = String(value || '').trim();
  if (!text) return '-';
  if (text.length <= maxChars) return text;
  if (maxChars <= 3) return text.slice(0, maxChars);
  return `${text.slice(0, maxChars - 3)}...`;
};

export const buildMonitoringTimeSeriesXAxis = (options = {}) => {
  const maxTicksLimit = toBoundedInteger(
    options.maxTicksLimit,
    MONITORING_TIME_SERIES_TICK_MAX,
    2,
    20
  );
  const tickPadding = toBoundedInteger(
    options.tickPadding,
    MONITORING_TIME_SERIES_TICK_PADDING,
    0,
    40
  );
  const axisHeightPx = toBoundedInteger(
    options.axisHeightPx,
    MONITORING_TIME_SERIES_AXIS_HEIGHT_PX,
    20,
    80
  );
  const maxChars = toBoundedInteger(
    options.maxLabelChars,
    MONITORING_TIME_SERIES_TICK_MAX_CHARS,
    4,
    40
  );
  const omitFinalTickLabel = options.omitFinalTickLabel !== false;

  return {
    ticks: {
      autoSkip: true,
      maxTicksLimit,
      autoSkipPadding: tickPadding,
      minRotation: 0,
      maxRotation: 0,
      callback(value, index, ticks) {
        if (
          omitFinalTickLabel &&
          Array.isArray(ticks) &&
          ticks.length > 0 &&
          Number(index) === ticks.length - 1
        ) {
          return '';
        }
        const raw =
          this && typeof this.getLabelForValue === 'function'
            ? this.getLabelForValue(value)
            : value;
        return normalizeTickLabel(raw, maxChars);
      }
    },
    afterFit(scale) {
      if (!scale) return;
      scale.height = axisHeightPx;
    }
  };
};
