// @ts-check

const CONTROL_CHARS_RE = /[\u0000-\u001f\u007f]/g;

/**
 * @param {unknown} value
 */
export const normalizeTrimmed = (value) => String(value || '').trim();

/**
 * @param {unknown} value
 */
export const normalizeLowerTrimmed = (value) => normalizeTrimmed(value).toLowerCase();

/**
 * @param {unknown} value
 * @param {string} fallback
 */
export const sanitizeDisplayText = (value, fallback = '-') => {
  const text = String(value || '').replace(CONTROL_CHARS_RE, '').trim();
  return text || fallback;
};

/**
 * @param {unknown} value
 */
export const formatUnknownForDisplay = (value) => {
  if (value === undefined) return '';
  if (value === null) return 'null';
  if (typeof value === 'string') return `"${value}"`;
  try {
    return JSON.stringify(value);
  } catch (_error) {
    return String(value);
  }
};
