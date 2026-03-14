// @ts-check

/**
 * @param {{
 *   runtimeMounted?: boolean,
 *   loggingOut?: boolean,
 *   saving?: boolean,
 *   authenticated?: boolean,
 *   adminConfigWritable?: boolean,
 *   surfaceAvailable?: boolean
 * }} source
 * @returns {boolean}
 */
export function deriveGlobalControlDisabled(source = {}) {
  return !(
    source.runtimeMounted === true &&
    source.loggingOut !== true &&
    source.saving !== true &&
    source.authenticated === true &&
    source.adminConfigWritable === true &&
    source.surfaceAvailable !== false
  );
}
