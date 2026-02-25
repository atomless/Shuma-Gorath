<script>
  import { onMount } from 'svelte';
  import ConfigWriteModeMessage from './primitives/ConfigWriteModeMessage.svelte';
  import NumericInputRow from './primitives/NumericInputRow.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TableEmptyRow from './primitives/TableEmptyRow.svelte';
  import TableWrapper from './primitives/TableWrapper.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TextareaField from './primitives/TextareaField.svelte';
  import ToggleRow from './primitives/ToggleRow.svelte';
  import {
    formatListTextarea,
    normalizeListTextareaForCompare,
    parseListTextarea
  } from '../../domain/config-form-utils.js';
  import {
    normalizeIpRangePolicyMode,
    parseInteger
  } from '../../domain/config-tab-helpers.js';
  import {
    durationPartsFromSeconds as durationPartsFromTotalSeconds,
    formatUnixSecondsLocal
  } from '../../domain/core/date-time.js';
  import { inRange } from '../../domain/core/validation.js';
  import {
    classifyIpRangeFallback,
    formatIpRangeReasonLabel,
    isIpRangeBanLike,
    isIpRangeReason,
    parseIpRangeOutcome
  } from '../../domain/ip-range-policy.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let bansSnapshot = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let onBan = null;
  export let onUnban = null;

  const MANUAL_BAN_FALLBACK_SECONDS = 21600;
  const IP_RANGE_MANAGED_STALENESS_MIN = 1;
  const IP_RANGE_MANAGED_STALENESS_MAX = 2160;
  const VALID_IP_RANGE_ACTIONS = new Set([
    'forbidden_403',
    'custom_message',
    'drop_connection',
    'redirect_308',
    'rate_limit',
    'honeypot',
    'maze',
    'tarpit'
  ]);
  const EMPTY_JSON_ARRAY = Object.freeze([]);

  let expandedRows = {};
  let banIp = '';
  let unbanIp = '';
  let banDurationDays = 0;
  let banDurationHours = 6;
  let banDurationMinutes = 0;
  let banning = false;
  let unbanning = false;
  let banFilter = 'all';
  let savingBypassAllowlists = false;
  let savingIpRange = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let bypassAllowlistsEnabled = true;
  let networkWhitelist = '';
  let pathWhitelist = '';
  let bypassAllowlistsBaseline = {
    enabled: true,
    network: '',
    path: ''
  };

  let ipRangePolicyMode = 'off';
  let ipRangeEmergencyAllowlist = '';
  let ipRangeCustomRulesJson = '';
  let ipRangeManagedPoliciesJson = '';
  let ipRangeManagedMaxStalenessHours = 168;
  let ipRangeAllowStaleManagedEnforce = false;
  let ipRangeCatalogVersion = '-';
  let ipRangeCatalogGeneratedAt = '-';
  let ipRangeCatalogAgeHours = null;
  let ipRangeManagedSetRows = [];
  let ipRangeBaseline = {
    mode: 'off',
    emergencyAllowlist: '',
    customRulesJson: '[]',
    managedPoliciesJson: '[]',
    managedMaxStalenessHours: 168,
    allowStaleManagedEnforce: false
  };

  let ipRangeCustomRulesValidation = {
    valid: true,
    parsed: EMPTY_JSON_ARRAY,
    normalized: '[]',
    error: ''
  };
  let ipRangeEmergencyAllowlistValidation = {
    valid: true,
    parsed: EMPTY_JSON_ARRAY,
    normalized: '',
    error: ''
  };
  let ipRangeManagedPoliciesValidation = {
    valid: true,
    parsed: EMPTY_JSON_ARRAY,
    normalized: '[]',
    error: ''
  };

  const formatTimestamp = (rawTs) => formatUnixSecondsLocal(rawTs, '-');

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  const isValidIpv4 = (value) => {
    const segments = String(value || '').split('.');
    if (segments.length !== 4) return false;
    return segments.every((segment) => {
      if (!/^\d{1,3}$/.test(segment)) return false;
      const numeric = Number(segment);
      return Number.isInteger(numeric) && numeric >= 0 && numeric <= 255;
    });
  };

  const isValidIpv6 = (value) => {
    const source = String(value || '').trim();
    if (!source.includes(':')) return false;
    return /^[0-9a-fA-F:]+$/.test(source);
  };

  const isValidIp = (value) => {
    const trimmed = String(value || '').trim();
    if (!trimmed || trimmed.length > 45) return false;
    return isValidIpv4(trimmed) || isValidIpv6(trimmed);
  };

  const applyConfiguredBanDuration = (config) => {
    const rawAdminDuration = config && typeof config === 'object' && config.ban_durations
      ? config.ban_durations.admin
      : undefined;
    const parts = durationPartsFromTotalSeconds(rawAdminDuration, MANUAL_BAN_FALLBACK_SECONDS);
    banDurationDays = parts.days;
    banDurationHours = parts.hours;
    banDurationMinutes = parts.minutes;
  };

  const splitLineEntries = (raw) => String(raw || '')
    .split('\n')
    .map((line, index) => ({
      line: index + 1,
      value: String(line || '').trim()
    }))
    .filter((entry) => entry.value.length > 0);

  const isValidIpv6CidrAddress = (value) => {
    const source = String(value || '').trim();
    if (!source || !source.includes(':')) return false;
    if (!/^[0-9a-fA-F:]+$/.test(source)) return false;
    if (source.includes(':::')) return false;
    const halves = source.split('::');
    if (halves.length > 2) return false;
    const parseHalf = (half) => (
      half === ''
        ? []
        : half.split(':').filter((segment) => segment.length > 0)
    );
    const left = parseHalf(halves[0]);
    const right = halves.length === 2 ? parseHalf(halves[1]) : [];
    const allSegments = [...left, ...right];
    if (allSegments.some((segment) => !/^[0-9a-fA-F]{1,4}$/.test(segment))) return false;
    if (halves.length === 1) return allSegments.length === 8;
    return allSegments.length < 8;
  };

  const isValidCidrNotation = (value) => {
    const source = String(value || '').trim();
    const slashIndex = source.indexOf('/');
    if (slashIndex <= 0 || slashIndex === source.length - 1) return false;
    if (source.indexOf('/', slashIndex + 1) !== -1) return false;
    const network = source.slice(0, slashIndex);
    const prefixText = source.slice(slashIndex + 1);
    if (!/^\d+$/.test(prefixText)) return false;
    const prefix = Number(prefixText);
    if (network.includes(':')) {
      return prefix >= 0 && prefix <= 128 && isValidIpv6CidrAddress(network);
    }
    return prefix >= 0 && prefix <= 32 && isValidIpv4(network);
  };

  const formatJsonObjectLines = (values) => {
    if (!Array.isArray(values) || values.length === 0) return '';
    return values
      .filter((value) => value && typeof value === 'object' && !Array.isArray(value))
      .map((value) => JSON.stringify(value))
      .join('\n');
  };

  const parseEmergencyAllowlistField = (raw) => {
    const entries = splitLineEntries(raw);
    const seen = new Set();
    const parsed = [];
    for (const entry of entries) {
      if (entry.value.includes(',')) {
        return {
          valid: false,
          parsed: EMPTY_JSON_ARRAY,
          normalized: entries.map((item) => item.value).join('\n'),
          error: `Emergency allowlist line ${entry.line} is invalid: use one CIDR per line (no commas).`
        };
      }
      if (!isValidCidrNotation(entry.value)) {
        return {
          valid: false,
          parsed: EMPTY_JSON_ARRAY,
          normalized: entries.map((item) => item.value).join('\n'),
          error: `Emergency allowlist line ${entry.line} is invalid: '${entry.value}' is not valid CIDR notation (example: 203.0.113.0/24).`
        };
      }
      if (seen.has(entry.value)) continue;
      seen.add(entry.value);
      parsed.push(entry.value);
    }
    return {
      valid: true,
      parsed,
      normalized: parsed.join('\n'),
      error: ''
    };
  };

  const validateCustomRuleShape = (rule, line) => {
    if (typeof rule.id !== 'string' || !rule.id.trim()) {
      return `Custom rules line ${line} is invalid: required key 'id' must be a non-empty string.`;
    }
    if (typeof rule.enabled !== 'boolean') {
      return `Custom rules line ${line} is invalid: required key 'enabled' must be true or false.`;
    }
    if (!Array.isArray(rule.cidrs) || rule.cidrs.length === 0) {
      return `Custom rules line ${line} is invalid: required key 'cidrs' must be a non-empty array.`;
    }
    if (rule.cidrs.some((cidr) => typeof cidr !== 'string' || !isValidCidrNotation(cidr))) {
      return `Custom rules line ${line} is invalid: each entry in 'cidrs' must be valid CIDR notation.`;
    }
    if (typeof rule.action !== 'string' || !VALID_IP_RANGE_ACTIONS.has(rule.action)) {
      return `Custom rules line ${line} is invalid: 'action' must be one of ${Array.from(VALID_IP_RANGE_ACTIONS).join(', ')}.`;
    }
    if (rule.action === 'redirect_308' && (typeof rule.redirect_url !== 'string' || !rule.redirect_url.trim())) {
      return `Custom rules line ${line} is invalid: 'redirect_url' is required when action is redirect_308.`;
    }
    if (rule.action === 'custom_message' && (typeof rule.custom_message !== 'string' || !rule.custom_message.trim())) {
      return `Custom rules line ${line} is invalid: 'custom_message' is required when action is custom_message.`;
    }
    return '';
  };

  const validateManagedPolicyShape = (rule, line) => {
    if (typeof rule.set_id !== 'string' || !rule.set_id.trim()) {
      return `Managed policies line ${line} is invalid: required key 'set_id' must be a non-empty string.`;
    }
    if (typeof rule.enabled !== 'boolean') {
      return `Managed policies line ${line} is invalid: required key 'enabled' must be true or false.`;
    }
    if (typeof rule.action !== 'string' || !VALID_IP_RANGE_ACTIONS.has(rule.action)) {
      return `Managed policies line ${line} is invalid: 'action' must be one of ${Array.from(VALID_IP_RANGE_ACTIONS).join(', ')}.`;
    }
    if (rule.action === 'redirect_308' && (typeof rule.redirect_url !== 'string' || !rule.redirect_url.trim())) {
      return `Managed policies line ${line} is invalid: 'redirect_url' is required when action is redirect_308.`;
    }
    if (rule.action === 'custom_message' && (typeof rule.custom_message !== 'string' || !rule.custom_message.trim())) {
      return `Managed policies line ${line} is invalid: 'custom_message' is required when action is custom_message.`;
    }
    return '';
  };

  const parseJsonObjectLinesField = (raw, fieldLabel, validateShape) => {
    const entries = splitLineEntries(raw);
    const parsed = [];
    for (const entry of entries) {
      if (entry.value === '[' || entry.value === ']') continue;
      let payload = entry.value;
      if (payload.endsWith(',')) {
        payload = payload.slice(0, -1).trim();
      }
      if (!payload) continue;
      let parsedValue;
      try {
        parsedValue = JSON.parse(payload);
      } catch (error) {
        const detail = error && error.message ? String(error.message) : 'Invalid JSON.';
        return {
          valid: false,
          parsed: EMPTY_JSON_ARRAY,
          normalized: entries.map((item) => item.value).join('\n'),
          error: `${fieldLabel} line ${entry.line} is invalid JSON: ${detail}`
        };
      }
      if (!parsedValue || typeof parsedValue !== 'object' || Array.isArray(parsedValue)) {
        return {
          valid: false,
          parsed: EMPTY_JSON_ARRAY,
          normalized: entries.map((item) => item.value).join('\n'),
          error: `${fieldLabel} line ${entry.line} is invalid: each line must be one JSON object.`
        };
      }
      const shapeError = validateShape(parsedValue, entry.line);
      if (shapeError) {
        return {
          valid: false,
          parsed: EMPTY_JSON_ARRAY,
          normalized: entries.map((item) => item.value).join('\n'),
          error: shapeError
        };
      }
      parsed.push(parsedValue);
    }
    return {
      valid: true,
      parsed,
      normalized: JSON.stringify(parsed),
      error: ''
    };
  };

  const readBypassAllowlistsConfig = (config = {}) => ({
    enabled: config.bypass_allowlists_enabled !== false,
    network: formatListTextarea(config.whitelist),
    path: formatListTextarea(config.path_whitelist)
  });

  const currentBypassAllowlistsBaseline = () => ({
    enabled: bypassAllowlistsEnabled === true,
    network: bypassAllowlistsNetworkNormalized,
    path: bypassAllowlistsPathNormalized
  });

  const applyBypassAllowlistsConfig = (config = {}) => {
    const next = readBypassAllowlistsConfig(config);
    bypassAllowlistsEnabled = next.enabled;
    networkWhitelist = next.network;
    pathWhitelist = next.path;
    bypassAllowlistsBaseline = {
      enabled: next.enabled === true,
      network: normalizeListTextareaForCompare(next.network),
      path: normalizeListTextareaForCompare(next.path)
    };
  };

  const readIpRangeConfig = (config = {}) => ({
    mode: normalizeIpRangePolicyMode(config.ip_range_policy_mode),
    emergencyAllowlist: formatListTextarea(config.ip_range_emergency_allowlist),
    customRulesJson: formatJsonObjectLines(config.ip_range_custom_rules),
    managedPoliciesJson: formatJsonObjectLines(config.ip_range_managed_policies),
    managedMaxStalenessHours: parseInteger(config.ip_range_managed_max_staleness_hours, 168),
    allowStaleManagedEnforce: config.ip_range_allow_stale_managed_enforce === true
  });

  const readIpRangeCatalog = (config = {}) => {
    const generatedAtUnix = Number(config.ip_range_managed_catalog_generated_at_unix || 0);
    let catalogAgeHours = null;
    if (Number.isFinite(generatedAtUnix) && generatedAtUnix > 0) {
      const nowUnix = Math.floor(Date.now() / 1000);
      catalogAgeHours = nowUnix >= generatedAtUnix
        ? Math.floor((nowUnix - generatedAtUnix) / 3600)
        : 0;
    }
    return {
      version: String(config.ip_range_managed_catalog_version || '-'),
      generatedAt: String(config.ip_range_managed_catalog_generated_at || '-'),
      ageHours: catalogAgeHours,
      managedSets: Array.isArray(config.ip_range_managed_sets) ? config.ip_range_managed_sets : []
    };
  };

  const currentIpRangeBaseline = () => ({
    mode: ipRangeModeNormalized,
    emergencyAllowlist: ipRangeEmergencyAllowlistNormalized,
    customRulesJson: ipRangeCustomRulesValidation.normalized,
    managedPoliciesJson: ipRangeManagedPoliciesValidation.normalized,
    managedMaxStalenessHours: Number(ipRangeManagedMaxStalenessHours),
    allowStaleManagedEnforce: ipRangeAllowStaleManagedEnforce === true
  });

  const applyIpRangeConfig = (config = {}) => {
    const next = readIpRangeConfig(config);
    const parsedEmergencyAllowlist = parseEmergencyAllowlistField(next.emergencyAllowlist);
    const parsedCustomRules = parseJsonObjectLinesField(
      next.customRulesJson,
      'Custom rules',
      validateCustomRuleShape
    );
    const parsedManagedPolicies = parseJsonObjectLinesField(
      next.managedPoliciesJson,
      'Managed policies',
      validateManagedPolicyShape
    );
    ipRangePolicyMode = next.mode;
    ipRangeEmergencyAllowlist = next.emergencyAllowlist;
    ipRangeCustomRulesJson = next.customRulesJson;
    ipRangeManagedPoliciesJson = next.managedPoliciesJson;
    ipRangeManagedMaxStalenessHours = next.managedMaxStalenessHours;
    ipRangeAllowStaleManagedEnforce = next.allowStaleManagedEnforce;
    ipRangeBaseline = {
      mode: next.mode,
      emergencyAllowlist: parsedEmergencyAllowlist.normalized,
      customRulesJson: parsedCustomRules.normalized,
      managedPoliciesJson: parsedManagedPolicies.normalized,
      managedMaxStalenessHours: Number(next.managedMaxStalenessHours),
      allowStaleManagedEnforce: next.allowStaleManagedEnforce === true
    };
  };

  const applyIpRangeCatalog = (config = {}) => {
    const catalog = readIpRangeCatalog(config);
    ipRangeCatalogVersion = catalog.version;
    ipRangeCatalogGeneratedAt = catalog.generatedAt;
    ipRangeCatalogAgeHours = catalog.ageHours;
    ipRangeManagedSetRows = catalog.managedSets;
  };

  const toKey = (ban, index) =>
    `${String(ban?.ip || '-')}:${String(ban?.reason || '-')}:${String(ban?.banned_at || 0)}:${String(ban?.expires || 0)}:${index}`;

  const isExpanded = (key) => expandedRows[key] === true;

  const formatIpRangeSourceLabel = (source) => {
    const normalized = String(source || '').trim().toLowerCase();
    if (normalized === 'managed') return 'Managed Set';
    if (normalized === 'custom') return 'Custom Rule';
    return normalized ? normalized : '-';
  };

  const deriveIpRangeBanMeta = (ban = {}) => {
    const reason = String(ban?.reason || '').trim();
    const parsed = parseIpRangeOutcome(ban?.fingerprint?.summary);
    const fallback = classifyIpRangeFallback(reason, parsed);
    const isIpRange = isIpRangeBanLike(ban) || isIpRangeReason(reason);
    return {
      isIpRange,
      reasonLabel: isIpRange ? formatIpRangeReasonLabel(reason) : '',
      source: parsed.source || '',
      sourceLabel: formatIpRangeSourceLabel(parsed.source),
      sourceId: parsed.sourceId || '',
      action: parsed.action || '',
      matchedCidr: parsed.matchedCidr || '',
      detection: parsed.detection || '',
      fallback: fallback !== 'none' ? fallback : ''
    };
  };

  function toggleDetails(key) {
    expandedRows = {
      ...expandedRows,
      [key]: !isExpanded(key)
    };
  }

  async function saveBypassAllowlistsConfig() {
    if (!bypassAllowlistsDirty || !writable || typeof onSaveConfig !== 'function') return;
    savingBypassAllowlists = true;
    try {
      const payload = {
        bypass_allowlists_enabled: bypassAllowlistsEnabled === true,
        whitelist: parseListTextarea(networkWhitelist),
        path_whitelist: parseListTextarea(pathWhitelist)
      };
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Bypass allowlists saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyBypassAllowlistsConfig(nextConfig);
      } else {
        bypassAllowlistsBaseline = currentBypassAllowlistsBaseline();
      }
    } finally {
      savingBypassAllowlists = false;
    }
  }

  async function saveIpRangeConfig() {
    if (!ipRangeDirty || !ipRangeValid || !writable || typeof onSaveConfig !== 'function') return;
    savingIpRange = true;
    try {
      const payload = {
        ip_range_policy_mode: ipRangeModeNormalized,
        ip_range_emergency_allowlist: ipRangeEmergencyAllowlistValidation.parsed,
        ip_range_custom_rules: ipRangeCustomRulesValidation.parsed,
        ip_range_managed_policies: ipRangeManagedPoliciesValidation.parsed,
        ip_range_managed_max_staleness_hours: Number(ipRangeManagedMaxStalenessHours),
        ip_range_allow_stale_managed_enforce: ipRangeAllowStaleManagedEnforce === true
      };
      const nextConfig = await onSaveConfig(payload, { successMessage: 'IP range policy saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyIpRangeConfig(nextConfig);
        applyIpRangeCatalog(nextConfig);
      } else {
        ipRangeBaseline = currentIpRangeBaseline();
      }
    } finally {
      savingIpRange = false;
    }
  }

  async function submitBan() {
    if (!canBan || typeof onBan !== 'function') return;
    banning = true;
    try {
      await onBan({
        ip: String(banIp || '').trim(),
        duration: Number(banDurationSeconds)
      });
      banIp = '';
    } finally {
      banning = false;
    }
  }

  async function submitUnban() {
    if (!canUnban || typeof onUnban !== 'function') return;
    unbanning = true;
    try {
      await onUnban({
        ip: String(unbanIp || '').trim()
      });
      unbanIp = '';
    } finally {
      unbanning = false;
    }
  }

  $: writable = configSnapshot && configSnapshot.admin_config_write_enabled === true;
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;
  $: bypassAllowlistsNetworkNormalized = normalizeListTextareaForCompare(networkWhitelist);
  $: bypassAllowlistsPathNormalized = normalizeListTextareaForCompare(pathWhitelist);
  $: bypassAllowlistsDirty = (
    (bypassAllowlistsEnabled === true) !== bypassAllowlistsBaseline.enabled ||
    bypassAllowlistsNetworkNormalized !== bypassAllowlistsBaseline.network ||
    bypassAllowlistsPathNormalized !== bypassAllowlistsBaseline.path
  );
  $: saveBypassAllowlistsDisabled = !writable || !bypassAllowlistsDirty || savingBypassAllowlists;
  $: saveBypassAllowlistsLabel = savingBypassAllowlists ? 'Saving...' : 'Save bypass allowlists';
  $: saveBypassAllowlistsSummary = bypassAllowlistsDirty
    ? 'Bypass allowlists have unsaved changes'
    : 'No unsaved changes';
  $: ipRangeModeNormalized = normalizeIpRangePolicyMode(ipRangePolicyMode);
  $: ipRangeEmergencyAllowlistValidation = parseEmergencyAllowlistField(ipRangeEmergencyAllowlist);
  $: ipRangeEmergencyAllowlistNormalized = ipRangeEmergencyAllowlistValidation.normalized;
  $: ipRangeEmergencyAllowlistValid = ipRangeEmergencyAllowlistValidation.valid;
  $: ipRangeCustomRulesValidation = parseJsonObjectLinesField(
    ipRangeCustomRulesJson,
    'Custom rules',
    validateCustomRuleShape
  );
  $: ipRangeManagedPoliciesValidation = parseJsonObjectLinesField(
    ipRangeManagedPoliciesJson,
    'Managed policies',
    validateManagedPolicyShape
  );
  $: ipRangeCustomRulesValid = ipRangeCustomRulesValidation.valid;
  $: ipRangeManagedPoliciesValid = ipRangeManagedPoliciesValidation.valid;
  $: ipRangeManagedMaxStalenessValid = inRange(
    ipRangeManagedMaxStalenessHours,
    IP_RANGE_MANAGED_STALENESS_MIN,
    IP_RANGE_MANAGED_STALENESS_MAX
  );
  $: ipRangeValid = (
    ipRangeEmergencyAllowlistValid &&
    ipRangeCustomRulesValid &&
    ipRangeManagedPoliciesValid &&
    ipRangeManagedMaxStalenessValid
  );
  $: ipRangeDirty = (
    ipRangeModeNormalized !== ipRangeBaseline.mode ||
    ipRangeEmergencyAllowlistNormalized !== ipRangeBaseline.emergencyAllowlist ||
    ipRangeCustomRulesValidation.normalized !== ipRangeBaseline.customRulesJson ||
    ipRangeManagedPoliciesValidation.normalized !== ipRangeBaseline.managedPoliciesJson ||
    Number(ipRangeManagedMaxStalenessHours) !== ipRangeBaseline.managedMaxStalenessHours ||
    (ipRangeAllowStaleManagedEnforce === true) !== ipRangeBaseline.allowStaleManagedEnforce
  );
  $: ipRangeManagedSetStaleCount = ipRangeManagedSetRows.filter((set) => set?.stale === true).length;
  $: ipRangeCatalogStale = (
    (Number.isFinite(ipRangeCatalogAgeHours)
      ? Number(ipRangeCatalogAgeHours) > Number(ipRangeManagedMaxStalenessHours)
      : false) ||
    ipRangeManagedSetStaleCount > 0
  );
  $: ipRangeInvalidSummary = !ipRangeEmergencyAllowlistValid
    ? ipRangeEmergencyAllowlistValidation.error
    : (!ipRangeCustomRulesValid
      ? ipRangeCustomRulesValidation.error
      : (!ipRangeManagedPoliciesValid
      ? ipRangeManagedPoliciesValidation.error
      : (!ipRangeManagedMaxStalenessValid
        ? `Managed max staleness must be between ${IP_RANGE_MANAGED_STALENESS_MIN} and ${IP_RANGE_MANAGED_STALENESS_MAX} hours.`
        : '')));
  $: saveIpRangeDisabled = !writable || !ipRangeDirty || !ipRangeValid || savingIpRange;
  $: saveIpRangeLabel = savingIpRange ? 'Saving...' : 'Save IP range policy';
  $: saveIpRangeSummary = ipRangeDirty
    ? 'IP range policy has unsaved changes'
    : 'No unsaved changes';
  $: warnOnUnload = writable && (ipRangeDirty || bypassAllowlistsDirty);
  $: bans = Array.isArray(bansSnapshot?.bans) ? bansSnapshot.bans : [];
  $: banRows = bans.map((ban, index) => ({
    ban,
    key: toKey(ban, index),
    originalIndex: index,
    meta: deriveIpRangeBanMeta(ban)
  }));
  $: filteredBanRows = banFilter === 'ip-range'
    ? banRows.filter((row) => row.meta.isIpRange)
    : banRows;
  $: banDurationSeconds = (
    (Number(banDurationDays) * 24 * 60 * 60) +
    (Number(banDurationHours) * 60 * 60) +
    (Number(banDurationMinutes) * 60)
  );
  $: canBan = isValidIp(banIp) && banDurationSeconds > 0 && !banning;
  $: canUnban = isValidIp(unbanIp) && !unbanning;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      const nextConfig = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};
      applyConfiguredBanDuration(nextConfig);
      applyIpRangeCatalog(nextConfig);
      if (!bypassAllowlistsDirty && !savingBypassAllowlists) {
        applyBypassAllowlistsConfig(nextConfig);
      }
      if (!ipRangeDirty && !savingIpRange) {
        applyIpRangeConfig(nextConfig);
      }
    }
  }
</script>

<section
  id="dashboard-panel-ip-bans"
  class="admin-group admin-group--status"
  data-dashboard-tab-panel="ip-bans"
  aria-labelledby="dashboard-tab-ip-bans"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
  <TabStateMessage tab="ip-bans" status={tabStatus} />
  <ConfigWriteModeMessage
    id="ip-bans-mode-subtitle"
    controlsLabel="IP-ban policy controls"
    loading={tabStatus?.loading === true}
    {hasConfigSnapshot}
    {writable}
  />
  <div class="control-group panel-soft pad-sm">
    <div class="input-row">
      <label class="control-label control-label--wide" for="ip-ban-filter">Ban View</label>
      <select id="ip-ban-filter" class="input-field" aria-label="Filter ban table" bind:value={banFilter}>
        <option value="all">All Active Bans</option>
        <option value="ip-range"><abbr title="Internet Protocol">IP</abbr> Range Policy Only</option>
      </select>
    </div>
    <p class="control-desc text-muted">
      {filteredBanRows.length} shown of {bans.length}.
    </p>
  </div>

  <TableWrapper>
    <table id="bans-table" class="panel panel-border bans-table-admin">
      <thead>
        <tr>
          <th class="caps-label"><abbr title="Internet Protocol">IP</abbr> Address</th>
          <th class="caps-label">Reason</th>
          <th class="caps-label">Banned At</th>
          <th class="caps-label">Expires</th>
          <th class="caps-label">Signals</th>
          <th class="caps-label">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#if filteredBanRows.length === 0}
          <TableEmptyRow colspan={6}>
            {#if banFilter === 'ip-range'}
              No active <abbr title="Internet Protocol">IP</abbr> range policy bans
            {:else}
              No active bans
            {/if}
          </TableEmptyRow>
        {:else}
          {#each filteredBanRows as row (row.key)}
            {@const ban = row.ban}
            {@const meta = row.meta}
            {@const rowKey = row.key}
            {@const detailVisible = isExpanded(rowKey)}
            {@const detailsId = `ban-detail-${row.originalIndex}`}
            {@const signals = Array.isArray(ban?.fingerprint?.signals) ? ban.fingerprint.signals : []}
            {@const expiresTs = Number(ban?.expires || 0)}
            {@const isExpired = Number.isFinite(expiresTs) && expiresTs > 0
              ? expiresTs < Math.floor(Date.now() / 1000)
              : false}
            <tr class="ban-summary-row">
              <td><code>{ban?.ip || '-'}</code></td>
              <td>
                <code>{ban?.reason || '-'}</code>
                {#if meta.isIpRange}
                  <div class="ban-detail-content">
                    <span class="ban-signal-badge"><abbr title="Internet Protocol">IP</abbr> range</span>
                    <span class="text-muted">{meta.reasonLabel}</span>
                    {#if meta.sourceId}
                      <span><code>{meta.sourceId}</code></span>
                    {/if}
                  </div>
                {/if}
              </td>
              <td>{formatTimestamp(ban?.banned_at)}</td>
              <td class={isExpired ? 'expired' : ''}>
                {isExpired ? 'Expired' : formatTimestamp(expiresTs)}
              </td>
              <td>
                {#if signals.length === 0}
                  <span class="text-muted">none</span>
                {:else}
                  {#each signals as signal}
                    <span class="ban-signal-badge">{signal}</span>
                  {/each}
                {/if}
              </td>
              <td class="ban-action-cell">
                <button
                  class="ban-details-toggle"
                  type="button"
                  aria-expanded={detailVisible ? 'true' : 'false'}
                  aria-controls={detailsId}
                  on:click={() => toggleDetails(rowKey)}
                >{detailVisible ? 'Hide' : 'Details'}</button>
              </td>
            </tr>
            {#if detailVisible}
              <tr id={detailsId} class="ban-detail-row">
                <td colspan="6">
                  <div class="ban-detail-content">
                    <div><strong>Score:</strong> {Number.isFinite(Number(ban?.fingerprint?.score)) ? Number(ban.fingerprint.score) : 'n/a'}</div>
                    <div><strong>Summary:</strong> {ban?.fingerprint?.summary || 'No additional fingerprint details.'}</div>
                    {#if meta.isIpRange}
                      <div><strong><abbr title="Internet Protocol">IP</abbr> Range Source:</strong> {meta.sourceLabel}</div>
                      <div><strong>Source <abbr title="Identifier">ID</abbr>:</strong> {meta.sourceId ? meta.sourceId : '-'}</div>
                      <div><strong>Policy Action:</strong> {meta.action ? meta.action : '-'}</div>
                      <div><strong>Matched <abbr title="Classless Inter-Domain Routing">CIDR</abbr>:</strong> {meta.matchedCidr ? meta.matchedCidr : '-'}</div>
                      <div><strong>Detection:</strong> {meta.detection ? meta.detection : '-'}</div>
                      {#if meta.fallback}
                        <div><strong>Fallback:</strong> {meta.fallback}</div>
                      {/if}
                    {/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>

  <div class="controls-grid controls-grid--manual">
    <div class="control-group panel-soft pad-md">
      <h3>Ban <abbr title="Internet Protocol">IP</abbr></h3>
      <input id="ban-ip" class="input-field" type="text" placeholder="Internet Protocol address" aria-label="Internet Protocol address to ban" maxlength="45" spellcheck="false" autocomplete="off" bind:value={banIp} />
      <input id="ban-reason" class="input-field" type="text" value="manual_ban" aria-label="Ban reason (fixed)" readonly disabled />
      <label class="control-label" for="ban-duration-days">Duration</label>
      <div class="duration-inputs">
        <label class="duration-input" for="ban-duration-days">
          <input id="ban-duration-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" aria-label="Manual ban duration days" bind:value={banDurationDays} />
          <span class="input-unit">days</span>
        </label>
        <label class="duration-input" for="ban-duration-hours">
          <input id="ban-duration-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" aria-label="Manual ban duration hours" bind:value={banDurationHours} />
          <span class="input-unit">hrs</span>
        </label>
        <label class="duration-input" for="ban-duration-minutes">
          <input id="ban-duration-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" aria-label="Manual ban duration minutes" bind:value={banDurationMinutes} />
          <span class="input-unit">mins</span>
        </label>
      </div>
      <button id="ban-btn" class="btn btn-submit" disabled={!canBan} on:click={submitBan}>Ban</button>
    </div>
    <div class="control-group panel-soft pad-md">
      <h3>Unban <abbr title="Internet Protocol">IP</abbr></h3>
      <input id="unban-ip" class="input-field" type="text" placeholder="Internet Protocol address" aria-label="Internet Protocol address to unban" maxlength="45" spellcheck="false" autocomplete="off" bind:value={unbanIp} />
      <button id="unban-btn" class="btn btn-submit" disabled={!canUnban} on:click={submitUnban}>Unban</button>
    </div>
  </div>

  <div class="control-group panel-soft pad-md config-edit-pane" class:config-edit-pane--dirty={ipRangeDirty}>
    <div class="panel-heading-with-control">
      <h3><abbr title="Internet Protocol">IP</abbr> Range Policy</h3>
      <select
        class="input-field panel-heading-select"
        id="ip-range-policy-mode"
        aria-label="Internet Protocol range policy mode"
        bind:value={ipRangePolicyMode}
        disabled={!writable}
      >
        <option value="off">off</option>
        <option value="advisory">advisory</option>
        <option value="enforce">enforce</option>
      </select>
    </div>
    <p class="control-desc text-muted">
      Use this to control what happens when traffic matches configured IP ranges.
      Start in <code>advisory</code> first so you can observe outcomes before enabling enforcement.
    </p>
    <p class="control-desc text-muted">
      <strong>Mode behavior:</strong> <code>off</code> = disabled, <code>advisory</code> = log only, <code>enforce</code> = apply rule action.
    </p>
    <div class="admin-controls">
      <TextareaField
        id="ip-range-emergency-allowlist"
        label='Emergency Allowlist <abbr title="Classless Inter-Domain Routing">CIDRs</abbr>'
        rows="3"
        ariaLabel="Internet Protocol range emergency allowlist"
        spellcheck={false}
        disabled={!writable}
        bind:value={ipRangeEmergencyAllowlist}
      />
      <p class="control-desc text-muted">
        One CIDR per line. Do not separate entries with commas.
        Matching entries bypass IP range policy actions, so this is your fastest false-positive safety control.
      </p>
      <p class="control-desc text-muted"><strong>Example valid formats:</strong> <code>203.0.113.0/24</code> or <code>2001:db8:abcd::/48</code></p>
      {#if !ipRangeEmergencyAllowlistValid}
        <p id="ip-range-emergency-allowlist-error" class="field-error visible">{ipRangeEmergencyAllowlistValidation.error}</p>
      {/if}
      <TextareaField
        id="ip-range-custom-rules-json"
        label='Custom Rules (one <abbr title="JavaScript Object Notation">JSON</abbr> object per line)'
        rows="8"
        ariaLabel="Internet Protocol range custom rules JavaScript Object Notation"
        spellcheck={false}
        monospace={true}
        ariaInvalid={ipRangeCustomRulesValid ? 'false' : 'true'}
        disabled={!writable}
        bind:value={ipRangeCustomRulesJson}
      />
      <p class="control-desc text-muted">
        Enter one JSON object per line. Do not wrap lines in square brackets, and no trailing comma is required.
        Rules are checked top-to-bottom; first match wins.
      </p>
      <p class="control-desc text-muted">
        Required keys per line: <code>id</code>, <code>enabled</code>, <code>cidrs</code>, <code>action</code>.
        For <code>redirect_308</code>, include <code>redirect_url</code>. For <code>custom_message</code>, include <code>custom_message</code>.
      </p>
      <p class="control-desc text-muted">
        Available <code>action</code> values: <code>forbidden_403</code>, <code>custom_message</code>,
        <code>drop_connection</code>, <code>redirect_308</code>, <code>rate_limit</code>,
        <code>honeypot</code>, <code>maze</code>, <code>tarpit</code>.
      </p>
      <p class="control-desc text-muted">
        <strong>Example line:</strong>
        <code>&#123;"id":"corp-proxy","enabled":true,"cidrs":["198.51.100.0/24"],"action":"rate_limit"&#125;</code>
      </p>
      <p class="control-desc text-muted">
        <strong>Second example line:</strong>
        <code>&#123;"id":"known-bad","enabled":true,"cidrs":["203.0.113.44/32"],"action":"forbidden_403"&#125;</code>
      </p>
      {#if !ipRangeCustomRulesValid}
        <p id="ip-range-custom-rules-error" class="field-error visible">{ipRangeCustomRulesValidation.error}</p>
      {/if}
      <TextareaField
        id="ip-range-managed-policies-json"
        label='Managed Policies (one <abbr title="JavaScript Object Notation">JSON</abbr> object per line)'
        rows="6"
        ariaLabel="Internet Protocol range managed policies JavaScript Object Notation"
        spellcheck={false}
        monospace={true}
        ariaInvalid={ipRangeManagedPoliciesValid ? 'false' : 'true'}
        disabled={!writable}
        bind:value={ipRangeManagedPoliciesJson}
      />
      <p class="control-desc text-muted">
        Enter one JSON object per line. Do not wrap lines in square brackets, and no trailing comma is required.
      </p>
      <p class="control-desc text-muted">
        Each line maps a managed <code>set_id</code> to an action.
        Common set IDs include <code>openai_gptbot</code>, <code>openai_oai_searchbot</code>,
        <code>openai_chatgpt_user</code>, and <code>github_copilot</code>.
      </p>
      <p class="control-desc text-muted">
        Available <code>action</code> values: <code>forbidden_403</code>, <code>custom_message</code>,
        <code>drop_connection</code>, <code>redirect_308</code>, <code>rate_limit</code>,
        <code>honeypot</code>, <code>maze</code>, <code>tarpit</code>.
      </p>
      <p class="control-desc text-muted">
        <strong>Example line:</strong>
        <code>&#123;"set_id":"openai_gptbot","enabled":true,"action":"forbidden_403"&#125;</code>
      </p>
      <p class="control-desc text-muted">
        <strong>Second example line:</strong>
        <code>&#123;"set_id":"openai_chatgpt_user","enabled":true,"action":"rate_limit"&#125;</code>
      </p>
      {#if !ipRangeManagedPoliciesValid}
        <p id="ip-range-managed-policies-error" class="field-error visible">{ipRangeManagedPoliciesValidation.error}</p>
      {/if}
      <NumericInputRow
        id="ip-range-managed-max-staleness"
        label="Managed Max Staleness (hours)"
        labelClass="control-label control-label--wide"
        min={IP_RANGE_MANAGED_STALENESS_MIN}
        max={IP_RANGE_MANAGED_STALENESS_MAX}
        step="1"
        inputmode="numeric"
        ariaLabel="Internet Protocol range managed max staleness hours"
        ariaInvalid={ipRangeManagedMaxStalenessValid ? 'false' : 'true'}
        disabled={!writable}
        bind:value={ipRangeManagedMaxStalenessHours}
      />
      <p class="control-desc text-muted">
        How old the managed IP catalog snapshot can be before managed-set rules are considered stale.
      </p>
      <ToggleRow
        id="ip-range-allow-stale-enforce"
        label="Allow stale managed enforce"
        labelClass="control-label control-label--wide"
        ariaLabel="Allow stale managed enforce"
        disabled={!writable}
        bind:checked={ipRangeAllowStaleManagedEnforce}
      />
    </div>
    <div class="info-panel">
      <h4>Managed Catalog Snapshot</h4>
      <div class="info-row">
        <span class="info-label text-muted">Version</span>
        <span><code>{ipRangeCatalogVersion}</code></span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Generated At</span>
        <span>{ipRangeCatalogGeneratedAt}</span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Catalog Age</span>
        <span>
          {#if Number.isFinite(ipRangeCatalogAgeHours)}
            {ipRangeCatalogAgeHours}h
          {:else}
            -
          {/if}
        </span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Managed Sets (stale)</span>
        <span>{ipRangeManagedSetRows.length} ({ipRangeManagedSetStaleCount})</span>
      </div>
    </div>
    {#if ipRangeManagedSetRows.length > 0}
      <TableWrapper>
        <table id="ip-range-config-managed-sets-table" class="panel panel-border">
          <thead>
            <tr>
              <th class="caps-label">Set</th>
              <th class="caps-label">Provider</th>
              <th class="caps-label">Version</th>
              <th class="caps-label">Entries</th>
              <th class="caps-label">Stale</th>
            </tr>
          </thead>
          <tbody>
            {#each ipRangeManagedSetRows as set}
              <tr>
                <td><code>{set?.set_id || '-'}</code></td>
                <td>{set?.provider || '-'}</td>
                <td><code>{set?.version || '-'}</code></td>
                <td>{set?.entry_count ?? 0}</td>
                <td>{set?.stale === true ? 'YES' : 'NO'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </TableWrapper>
    {/if}
    {#if ipRangeCatalogStale}
      <p class="message warning">
        The managed IP catalog snapshot is older than your staleness limit. Managed-set enforcement may be skipped until the catalog is refreshed.
      </p>
    {/if}
    <SaveChangesBar
      containerId="ip-range-policy-save-bar"
      isHidden={!writable || !ipRangeDirty}
      summaryId="ip-range-policy-unsaved-summary"
      summaryText={saveIpRangeSummary}
      summaryClass="text-unsaved-changes"
      invalidId="ip-range-policy-invalid-summary"
      invalidText={ipRangeInvalidSummary}
      buttonId="save-ip-range-policy"
      buttonLabel={saveIpRangeLabel}
      buttonDisabled={saveIpRangeDisabled}
      onSave={saveIpRangeConfig}
    />
  </div>

  <div class="control-group panel-soft pad-md config-edit-pane" class:config-edit-pane--dirty={bypassAllowlistsDirty}>
    <div class="panel-heading-with-control">
      <h3>Bypass Allowlists</h3>
      <label class="toggle-switch" for="bypass-allowlists-toggle">
        <input type="checkbox" id="bypass-allowlists-toggle" aria-label="Enable bypass allowlists" bind:checked={bypassAllowlistsEnabled} disabled={!writable}>
        <span class="toggle-slider"></span>
      </label>
    </div>
    <p class="control-desc text-muted">Define trusted bypass entries. Use one entry per line.</p>
    <p class="control-desc text-muted">
      If a legitimate visitor is blocked by IP range policy, their specific IP will not be in the ban list so unbanning it will not help. If they still match the same range rule, they will be blocked again on the next request. You will need to add their known-to-be-safe IP or CIDR to the IP/CIDR Allowlist below, or change the matching IP range rule to avoid their IP.
    </p>
    <div class="admin-controls">
      <TextareaField id="network-whitelist" label='<abbr title="Internet Protocol">IP</abbr>/<abbr title="Classless Inter-Domain Routing">CIDR</abbr> Allowlist' rows="3" ariaLabel="Internet Protocol and Classless Inter-Domain Routing allowlist" spellcheck={false} disabled={!writable} bind:value={networkWhitelist} />
      <TextareaField id="path-whitelist" label="Path Allowlist" rows="3" ariaLabel="Path allowlist" spellcheck={false} disabled={!writable} bind:value={pathWhitelist} />
    </div>
    <SaveChangesBar
      containerId="bypass-allowlists-save-bar"
      isHidden={!writable || !bypassAllowlistsDirty}
      summaryId="bypass-allowlists-unsaved-summary"
      summaryText={saveBypassAllowlistsSummary}
      summaryClass="text-unsaved-changes"
      buttonId="save-bypass-allowlists"
      buttonLabel={saveBypassAllowlistsLabel}
      buttonDisabled={saveBypassAllowlistsDisabled}
      onSave={saveBypassAllowlistsConfig}
    />
  </div>
</section>
