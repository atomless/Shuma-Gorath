// @ts-check

export const MONITORING_CHART_PALETTE = Object.freeze([
  'rgb(255,205,235)',
  'rgb(225,175,205)',
  'rgb(205, 155, 185)',
  'rgb(190, 140, 170)',
  'rgb(175, 125, 155)',
  'rgb(160, 110, 140)',
  'rgb(147, 97, 127)',
  'rgb(135, 85, 115)'
]);

export const MONITORING_TIME_SERIES_FILL = Object.freeze({
  events: MONITORING_CHART_PALETTE[0],
  challenge: MONITORING_CHART_PALETTE[1],
  pow: MONITORING_CHART_PALETTE[2]
});

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
