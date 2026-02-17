// @ts-check

const cloneValue = (value) => {
  if (value === null || value === undefined) return value;
  if (typeof value !== 'object') return value;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (_e) {
    return value;
  }
};

const stableStringify = (value) => {
  const normalize = (input) => {
    if (Array.isArray(input)) return input.map(normalize);
    if (!input || typeof input !== 'object') return input;
    const out = {};
    Object.keys(input).sort().forEach((key) => {
      out[key] = normalize(input[key]);
    });
    return out;
  };
  return JSON.stringify(normalize(value));
};

export const create = (initial = {}) => {
  const snapshots = new Map();
  const fingerprints = new Map();

  Object.entries(initial || {}).forEach(([section, value]) => {
    snapshots.set(section, cloneValue(value));
    fingerprints.set(section, stableStringify(value));
  });

  const set = (section, value) => {
    snapshots.set(section, cloneValue(value));
    fingerprints.set(section, stableStringify(value));
  };

  const get = (section, fallback = null) => {
    if (!snapshots.has(section)) {
      return cloneValue(fallback);
    }
    return cloneValue(snapshots.get(section));
  };

  const isDirty = (section, currentValue) => {
    const previous = fingerprints.has(section) ? fingerprints.get(section) : undefined;
    return previous !== stableStringify(currentValue);
  };

  return {
    set,
    get,
    isDirty
  };
};
