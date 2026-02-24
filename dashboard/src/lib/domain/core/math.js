// @ts-check

/**
 * @param {unknown} value
 * @param {number} fallback
 */
export const parseInteger = (value, fallback) => {
  const parsed = Number.parseInt(String(value), 10);
  return Number.isInteger(parsed) ? parsed : fallback;
};

/**
 * @param {unknown} value
 * @param {number} fallback
 */
export const parseFloatNumber = (value, fallback) => {
  const parsed = Number.parseFloat(String(value));
  return Number.isFinite(parsed) ? parsed : fallback;
};

/**
 * @param {unknown} value
 * @param {number} max
 */
export const toBoundedNonNegativeInteger = (value, max) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric < 0) return 0;
  return Math.min(max, Math.floor(numeric));
};
