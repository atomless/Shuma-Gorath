const ADVERSARY_SIM_LANE_LABELS = Object.freeze({
  synthetic_traffic: 'Synthetic Traffic',
  scrapling_traffic: 'Scrapling Traffic',
  bot_red_team: 'Agentic Traffic'
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

