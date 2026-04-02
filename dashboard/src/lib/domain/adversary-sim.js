const ADVERSARY_SIM_LANE_LABELS = Object.freeze({
  synthetic_traffic: 'Synthetic Traffic',
  scrapling_traffic: 'Scrapling Traffic',
  bot_red_team: 'Agentic Traffic',
  parallel_mixed_traffic: 'Scrapling + Agentic'
});

const REPRESENTATIVENESS_STATUS_LABELS = Object.freeze({
  representative: 'Representative',
  partially_representative: 'Partially Representative',
  degraded: 'Degraded'
});

const TRANSITION_REASON_COPY = Object.freeze({
  manual_on: 'This run started manually.',
  candidate_window_follow_on:
    'This run started automatically to materialize a candidate-window follow-on.',
  loop_continuation_follow_on:
    'This run started automatically to continue the oversight loop after a still-outside-budget judgment.',
  auto_window_expired: 'The latest run ended after reaching its configured duration window.',
  manual_off: 'The latest run was stopped manually.',
  config_disabled: 'The latest run stopped because adversary simulation was disabled.',
  process_restart:
    'The latest run stopped because runtime ownership moved to a different process.',
  forced_kill_timeout:
    'The latest run was force-stopped after the shutdown timeout elapsed.'
});

const TOKEN_ACRONYMS = Object.freeze({
  ai: 'AI',
  api: 'API',
  cdp: 'CDP',
  http: 'HTTP',
  id: 'ID',
  ip: 'IP',
  js: 'JS',
  llm: 'LLM',
  pow: 'PoW',
  sim: 'Sim',
  ui: 'UI'
});

export const humanizeAdversarySimToken = (value, fallback = '-') => {
  const normalized = String(value || '').trim().replace(/[-]+/g, '_');
  if (!normalized) return fallback;
  return normalized
    .split(/[_\s]+/)
    .filter(Boolean)
    .map((word) => {
      const lowered = word.toLowerCase();
      if (TOKEN_ACRONYMS[lowered]) return TOKEN_ACRONYMS[lowered];
      return lowered.charAt(0).toUpperCase() + lowered.slice(1);
    })
    .join(' ');
};

export const formatAdversarySimLaneLabel = (value, fallback = '-') => {
  const normalized = String(value || '').trim().toLowerCase();
  if (!normalized) return fallback;
  return ADVERSARY_SIM_LANE_LABELS[normalized] || humanizeAdversarySimToken(normalized, fallback);
};

export const formatRepresentativenessStatusLabel = (value, fallback = '-') => {
  const normalized = String(value || '').trim().toLowerCase();
  if (!normalized) return fallback;
  return REPRESENTATIVENESS_STATUS_LABELS[normalized]
    || humanizeAdversarySimToken(normalized, fallback);
};

export const formatAdversarySimTransitionReasonCopy = (value, fallback = '') => {
  const normalized = String(value || '').trim().toLowerCase();
  if (!normalized) return fallback;
  return TRANSITION_REASON_COPY[normalized] || fallback;
};
