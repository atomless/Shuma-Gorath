// @ts-check

import { normalizeTrimmed } from './core/strings.js';

const IP_RANGE_REASON_PREFIX = 'ip_range_';

const IP_RANGE_REASON_LABELS = Object.freeze({
  ip_range_emergency_allowlist: 'Emergency Allowlist',
  ip_range_policy_advisory: 'Logging-Only Match',
  ip_range_policy_forbidden: '403 Forbidden',
  ip_range_policy_custom_message: 'Custom Message',
  ip_range_policy_drop_connection: 'Drop Connection',
  ip_range_policy_redirect: '308 Redirect',
  ip_range_policy_redirect_missing_url: 'Redirect Missing URL',
  ip_range_policy_rate_limit: 'Rate Limited',
  ip_range_policy_honeypot: 'Honeypot Ban',
  ip_range_policy_maze: 'Maze',
  ip_range_policy_maze_fallback_challenge: 'Maze Fallback Challenge',
  ip_range_policy_maze_fallback_block: 'Maze Fallback Block',
  ip_range_policy_tarpit: 'Tarpit',
  ip_range_policy_tarpit_fallback_maze: 'Tarpit Fallback Maze',
  ip_range_policy_tarpit_fallback_block: 'Tarpit Fallback Block'
});

const parseKeyValuePairs = (text) => {
  const source = String(text || '');
  const pairs = {};
  const matcher = /([a-z_]+)=([^\s\]]+)/gi;
  let match = matcher.exec(source);
  while (match) {
    const key = String(match[1] || '').trim().toLowerCase();
    const value = normalizeTrimmed(match[2]);
    if (key && value) {
      pairs[key] = value;
    }
    match = matcher.exec(source);
  }
  return pairs;
};

const normalizeSignals = (rawSignals) => {
  const source = normalizeTrimmed(rawSignals);
  if (!source) return [];
  return source
    .split(',')
    .map((entry) => normalizeTrimmed(entry))
    .filter((entry) => entry.length > 0);
};

export const isIpRangeReason = (reason) =>
  normalizeTrimmed(reason).toLowerCase().startsWith(IP_RANGE_REASON_PREFIX);

export const formatIpRangeReasonLabel = (reason) => {
  const key = normalizeTrimmed(reason).toLowerCase();
  if (Object.prototype.hasOwnProperty.call(IP_RANGE_REASON_LABELS, key)) {
    return IP_RANGE_REASON_LABELS[key];
  }
  if (!key) return '-';
  return key
    .replace(/_/g, ' ')
    .replace(/\b[a-z]/g, (char) => char.toUpperCase());
};

export const parseIpRangeOutcome = (outcome, taxonomy = null) => {
  const rawOutcome = normalizeTrimmed(outcome);
  const taxonomyMatch = /taxonomy\[([^\]]+)\]/i.exec(rawOutcome);
  const taxonomyPairs = taxonomy && typeof taxonomy === 'object'
    ? {
      detection: normalizeTrimmed(taxonomy.detection),
      level: normalizeTrimmed(taxonomy.level),
      action: normalizeTrimmed(taxonomy.action),
      signals: Array.isArray(taxonomy.signals)
        ? taxonomy.signals
          .map((signal) => normalizeTrimmed(signal))
          .filter((signal) => signal.length > 0)
          .join(',')
        : ''
    }
    : parseKeyValuePairs(taxonomyMatch ? taxonomyMatch[1] : '');
  const outcomeWithoutTaxonomy = taxonomyMatch
    ? rawOutcome.replace(taxonomyMatch[0], '').trim()
    : rawOutcome;
  const outcomePairs = parseKeyValuePairs(outcomeWithoutTaxonomy);
  return {
    source: normalizeTrimmed(outcomePairs.source),
    sourceId: normalizeTrimmed(outcomePairs.source_id),
    action: normalizeTrimmed(outcomePairs.action),
    matchedCidr: normalizeTrimmed(outcomePairs.matched_cidr),
    fallback: normalizeTrimmed(outcomePairs.fallback),
    location: normalizeTrimmed(outcomePairs.location),
    detection: normalizeTrimmed(taxonomyPairs.detection),
    level: normalizeTrimmed(taxonomyPairs.level),
    actionId: normalizeTrimmed(taxonomyPairs.action),
    signals: normalizeSignals(taxonomyPairs.signals),
    rawOutcome
  };
};

export const classifyIpRangeFallback = (reason, parsedOutcome = {}) => {
  const fallback = normalizeTrimmed(parsedOutcome.fallback).toLowerCase();
  if (fallback) return fallback;

  const reasonKey = normalizeTrimmed(reason).toLowerCase();
  if (reasonKey.includes('fallback_maze')) return 'maze';
  if (reasonKey.includes('fallback_challenge')) return 'challenge';
  if (reasonKey.includes('fallback_block')) return 'block';
  if (reasonKey.includes('redirect_missing_url')) return 'block_missing_redirect';
  return 'none';
};

export const isIpRangeBanLike = (ban = {}) => {
  const reason = normalizeTrimmed(ban?.reason).toLowerCase();
  if (isIpRangeReason(reason)) return true;
  const fingerprintSignals = Array.isArray(ban?.fingerprint?.signals)
    ? ban.fingerprint.signals
    : [];
  if (
    fingerprintSignals.some((signal) => normalizeTrimmed(signal).toLowerCase().includes('ip_range'))
  ) {
    return true;
  }
  const parsed = parseIpRangeOutcome(ban?.fingerprint?.summary);
  return Boolean(parsed.source || parsed.sourceId || parsed.action || parsed.matchedCidr);
};
