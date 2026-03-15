// @ts-check

import { formatCompactNumber } from './core/format.js';

const DEFAULT_INACTIVE_LABEL = '';
const DEFAULT_ACTIVE_LABEL = 'Unknown';

export const HALF_DOUGHNUT_ROTATION = -90;
export const HALF_DOUGHNUT_CIRCUMFERENCE = 180;
export const HALF_DOUGHNUT_CUTOUT = '72%';
export const HALF_DOUGHNUT_LEGEND_OFFSET_PX = 5;

export const EMPTY_HALF_DOUGHNUT_READOUT = Object.freeze({
  label: DEFAULT_INACTIVE_LABEL,
  value: '',
  active: false
});

const shiftLegendHitBoxes = (legendHitBoxes = [], offsetPx) => {
  if (!Array.isArray(legendHitBoxes)) return;
  legendHitBoxes.forEach((hitBox) => {
    if (!hitBox || !Number.isFinite(Number(hitBox.top))) return;
    hitBox.top += offsetPx;
  });
};

export const HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN = Object.freeze({
  id: 'shuma-half-doughnut-legend-offset',
  afterUpdate(chart) {
    const offsetPx = Number(HALF_DOUGHNUT_LEGEND_OFFSET_PX);
    const legend = chart?.legend;
    if (!Number.isFinite(offsetPx) || offsetPx === 0 || !legend || legend.options?.position !== 'bottom') {
      return;
    }
    legend.top += offsetPx;
    legend.bottom += offsetPx;
    shiftLegendHitBoxes(legend.legendHitBoxes, offsetPx);
  }
});

const toHalfDoughnutValue = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric < 0) return null;
  return numeric;
};

const compareHalfDoughnutEntries = (left, right) => {
  if (right.value !== left.value) return right.value - left.value;
  return left.label.localeCompare(right.label);
};

const normalizeHalfDoughnutEntry = (entry) => {
  if (Array.isArray(entry)) {
    const value = toHalfDoughnutValue(entry[1]);
    if (value === null) return null;
    return {
      label: normalizeText(entry[0], DEFAULT_ACTIVE_LABEL),
      value
    };
  }
  if (!entry || typeof entry !== 'object') {
    return null;
  }
  const value = toHalfDoughnutValue(entry.value);
  if (value === null) return null;
  return {
    label: normalizeText(entry.label, DEFAULT_ACTIVE_LABEL),
    value
  };
};

export const buildHalfDoughnutSeries = (source = {}) => {
  const normalizedEntries = Array.isArray(source)
    ? source
        .map((entry) => normalizeHalfDoughnutEntry(entry))
        .filter(Boolean)
    : Object.entries(source || {})
        .map(([label, value]) => normalizeHalfDoughnutEntry([label, value]))
        .filter(Boolean);

  normalizedEntries.sort(compareHalfDoughnutEntries);

  return {
    entries: normalizedEntries,
    labels: normalizedEntries.map((entry) => entry.label),
    values: normalizedEntries.map((entry) => entry.value)
  };
};

const normalizeText = (value, fallback) => {
  const normalized = String(value || '').trim();
  return normalized || fallback;
};

const normalizeActiveIndex = (activeElements = []) => {
  if (!Array.isArray(activeElements) || activeElements.length === 0) {
    return -1;
  }
  const index = Number(activeElements[0]?.index);
  return Number.isInteger(index) ? index : -1;
};

export const buildHalfDoughnutReadout = (
  labels = [],
  values = [],
  activeElements = [],
  options = {}
) => {
  const inactiveLabel =
    typeof options.inactiveLabel === 'string'
      ? options.inactiveLabel
      : DEFAULT_INACTIVE_LABEL;
  const activeIndex = normalizeActiveIndex(activeElements);
  if (activeIndex < 0 || activeIndex >= values.length) {
    return {
      ...EMPTY_HALF_DOUGHNUT_READOUT,
      label: inactiveLabel
    };
  }
  const formatValue = typeof options.formatValue === 'function' ? options.formatValue : formatCompactNumber;
  return {
    label: normalizeText(labels[activeIndex], DEFAULT_ACTIVE_LABEL),
    value: formatValue(values[activeIndex], '0'),
    active: true
  };
};

export const syncHalfDoughnutReadout = (chart, onReadoutChange, options = {}) => {
  if (typeof onReadoutChange !== 'function') return;
  const labels = Array.isArray(chart?.data?.labels) ? chart.data.labels : [];
  const values = Array.isArray(chart?.data?.datasets?.[0]?.data) ? chart.data.datasets[0].data : [];
  const activeElements = typeof chart?.getActiveElements === 'function' ? chart.getActiveElements() : [];
  onReadoutChange(buildHalfDoughnutReadout(labels, values, activeElements, options));
};

export const buildHalfDoughnutOptions = (options = {}) => {
  const onReadoutChange =
    typeof options.onReadoutChange === 'function' ? options.onReadoutChange : null;
  const readoutOptions = {
    formatValue: options.formatValue,
    inactiveLabel: options.inactiveLabel
  };
  const chartOptions = {
    responsive: true,
    maintainAspectRatio: options.maintainAspectRatio !== false,
    aspectRatio: Number.isFinite(Number(options.aspectRatio))
      ? Number(options.aspectRatio)
      : 2.2,
    rotation: HALF_DOUGHNUT_ROTATION,
    circumference: HALF_DOUGHNUT_CIRCUMFERENCE,
    cutout: normalizeText(options.cutout, HALF_DOUGHNUT_CUTOUT),
    plugins: {
      legend: {
        position: normalizeText(options.legendPosition, 'bottom'),
        labels: {
          color: String(options.legendColor || ''),
          usePointStyle: true,
          pointStyle: 'circle'
        }
      },
      tooltip: {
        enabled: false
      }
    }
  };

  if (typeof options.resizeDelay !== 'undefined') {
    chartOptions.resizeDelay = options.resizeDelay;
  }
  if (typeof options.animation !== 'undefined') {
    chartOptions.animation = options.animation;
  }
  if (onReadoutChange) {
    chartOptions.onHover = (_event, activeElements, chart) => {
      const labels = Array.isArray(chart?.data?.labels) ? chart.data.labels : [];
      const values = Array.isArray(chart?.data?.datasets?.[0]?.data)
        ? chart.data.datasets[0].data
        : [];
      onReadoutChange(buildHalfDoughnutReadout(labels, values, activeElements, readoutOptions));
    };
  }

  return chartOptions;
};
