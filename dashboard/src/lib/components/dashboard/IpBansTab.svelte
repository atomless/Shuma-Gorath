<script>
  import { browser } from '$app/environment';
  import { onDestroy, onMount } from 'svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TableEmptyRow from './primitives/TableEmptyRow.svelte';
  import TableWrapper from './primitives/TableWrapper.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TextareaField from './primitives/TextareaField.svelte';
  import {
    formatListTextarea,
    normalizeListTextareaForCompare,
    parseListTextarea
  } from '../../domain/config-form-utils.js';
  import { normalizeIpRangePolicyMode } from '../../domain/config-tab-helpers.js';
  import {
    durationPartsFromSeconds as durationPartsFromTotalSeconds,
    formatUnixSecondsLocal
  } from '../../domain/core/date-time.js';
  import {
    classifyIpRangeFallback,
    formatIpRangeReasonLabel,
    isIpRangeBanLike,
    isIpRangeReason,
    parseIpRangeOutcome
  } from '../../domain/ip-range-policy.js';
  import { resolveMonitoringChartTheme } from '../../domain/monitoring-chart-presets.js';
  import HalfDoughnutChart from './primitives/HalfDoughnutChart.svelte';
  import {
    buildHalfDoughnutSeries,
    EMPTY_HALF_DOUGHNUT_READOUT,
    HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN,
    buildHalfDoughnutOptions,
    syncHalfDoughnutReadout
  } from '../../domain/half-doughnut-chart.js';
  import { isAdminConfigWritable } from '../../domain/config-runtime.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let bansSnapshot = null;
  export let ipRangeSuggestionsSnapshot = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let configVersion = 0;
  export let ipRangeSuggestionsVersion = 0;
  export let onSaveConfig = null;
  export let onBan = null;
  export let onUnban = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  const MANUAL_BAN_FALLBACK_SECONDS = 21600;
  const CHART_RESIZE_REDRAW_DEBOUNCE_MS = 180;
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
  let hasConfigSnapshot = false;
  let lastAppliedConfigVersion = -1;
  let lastAppliedSuggestionsVersion = -1;
  let banReasonCanvas = null;
  let banReasonChart = null;
  let banReasonReadout = EMPTY_HALF_DOUGHNUT_READOUT;
  let chartRefreshNonce = 0;
  let resizeRedrawTimer = null;
  let wasActive = false;

  let bypassAllowlistsEnabled = true;
  let networkAllowlist = '';
  let bypassAllowlistsBaseline = {
    enabled: true,
    network: ''
  };

  let ipRangePolicyMode = 'off';
  let ipRangeEmergencyAllowlist = '';
  let ipRangeCustomRulesJson = '';
  let ipRangeSuggestions = [];
  let ipRangeSuggestionsSummary = {
    suggestionsTotal: 0,
    lowRisk: 0,
    mediumRisk: 0,
    highRisk: 0
  };
  let ipRangeBaseline = {
    mode: 'off',
    emergencyAllowlist: '',
    customRulesJson: '[]'
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
  let banReasonEntries = [];

  const formatTimestamp = (rawTs) => formatUnixSecondsLocal(rawTs, '-');
  const clearTimer = (timerId) => {
    if (timerId === null) return null;
    clearTimeout(timerId);
    return null;
  };
  const scheduleChartRefresh = (delayMs = 0) => {
    resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    resizeRedrawTimer = setTimeout(() => {
      resizeRedrawTimer = null;
      if (!isActive) return;
      chartRefreshNonce += 1;
    }, Math.max(0, Number(delayMs) || 0));
  };
  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    const onResize = () => {
      scheduleChartRefresh(CHART_RESIZE_REDRAW_DEBOUNCE_MS);
    };
    window.addEventListener('beforeunload', handleBeforeUnload);
    window.addEventListener('resize', onResize, { passive: true });
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
      window.removeEventListener('resize', onResize);
      resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    };
  });

  onDestroy(() => {
    resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    if (banReasonChart && typeof banReasonChart.destroy === 'function') {
      banReasonChart.destroy();
    }
    banReasonChart = null;
  });

  const getChartConstructor = () => {
    if (!browser || !window || typeof window.Chart !== 'function') return null;
    return window.Chart;
  };

  const chartNeedsRefresh = (chart, refreshNonce) =>
    Number(chart?.__shumaRefreshNonce || 0) !== Number(refreshNonce || 0);

  const stampChartRefresh = (chart, refreshNonce) => {
    if (chart && typeof chart === 'object') {
      chart.__shumaRefreshNonce = Number(refreshNonce || 0);
    }
    return chart;
  };

  const resizeChartIfNeeded = (chart, needsRefresh) => {
    if (!needsRefresh) return;
    if (chart && typeof chart.resize === 'function') {
      chart.resize();
    }
  };

  const sameSeries = (chart, nextLabels, nextData) => {
    if (!chart || !chart.data || !Array.isArray(chart.data.datasets) || chart.data.datasets.length === 0) {
      return false;
    }
    const currentLabels = Array.isArray(chart.data.labels) ? chart.data.labels : [];
    const currentData = Array.isArray(chart.data.datasets[0].data) ? chart.data.datasets[0].data : [];
    return (
      currentLabels.length === nextLabels.length &&
      currentData.length === nextData.length &&
      currentLabels.every((label, index) => String(label) === String(nextLabels[index])) &&
      currentData.every((value, index) => Number(value) === Number(nextData[index]))
    );
  };

  const sameColorSeries = (currentColors, nextColors) => {
    const current = Array.isArray(currentColors)
      ? currentColors.map((color) => String(color || ''))
      : [String(currentColors || '')];
    const next = Array.isArray(nextColors)
      ? nextColors.map((color) => String(color || ''))
      : [String(nextColors || '')];
    return (
      current.length === next.length &&
      current.every((value, index) => value === next[index])
    );
  };

  const normalizeBanReasonLabel = (reason) => {
    const normalized = String(reason || '').trim();
    return normalized || 'unknown';
  };

  const canvasHasRenderableSize = (canvas) => {
    if (!canvas || typeof canvas.getBoundingClientRect !== 'function') return false;
    const rect = canvas.getBoundingClientRect();
    return Number.isFinite(rect.width) && Number.isFinite(rect.height) && rect.width > 0 && rect.height > 0;
  };

  const updateBanReasonChart = (
    chart,
    canvas,
    entries,
    refreshNonce = 0,
    onReadoutChange = null
  ) => {
    const chartCtor = getChartConstructor();
    if (!chartCtor || !canvas || !Array.isArray(entries) || entries.length === 0) {
      return chart;
    }
    if (!canvasHasRenderableSize(canvas)) {
      return chart;
    }
    const chartTheme = resolveMonitoringChartTheme();
    const { labels, values } = buildHalfDoughnutSeries(entries);
    const colors = labels.map((_, index) => chartTheme.palette[index % chartTheme.palette.length]);
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    if (!chart) {
      const nextChart = stampChartRefresh(new chartCtor(ctx, {
        type: 'doughnut',
        plugins: [HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN],
        data: {
          labels,
          datasets: [{
            data: values,
            backgroundColor: colors,
            borderColor: 'rgba(0, 0, 0, 0)',
            borderWidth: 0,
            hoverBorderWidth: 0
          }]
        },
        options: buildHalfDoughnutOptions({
          legendColor: chartTheme.legendColor,
          maintainAspectRatio: false,
          resizeDelay: 0,
          onReadoutChange
        })
      }), refreshNonce);
      syncHalfDoughnutReadout(nextChart, onReadoutChange);
      return nextChart;
    }
    const needsRefresh = chartNeedsRefresh(chart, refreshNonce);
    const hasSameSeries = sameSeries(chart, labels, values);
    const hasSameColors = sameColorSeries(chart.data.datasets?.[0]?.backgroundColor, colors);
    if (needsRefresh || !hasSameSeries || !hasSameColors) {
      resizeChartIfNeeded(chart, needsRefresh);
      chart.data.labels = labels;
      chart.data.datasets[0].data = values;
      chart.data.datasets[0].backgroundColor = colors;
      chart.data.datasets[0].borderColor = 'rgba(0, 0, 0, 0)';
      chart.data.datasets[0].borderWidth = 0;
      chart.data.datasets[0].hoverBorderWidth = 0;
      const halfDoughnutOptions = buildHalfDoughnutOptions({
        legendColor: chartTheme.legendColor,
        maintainAspectRatio: false,
        resizeDelay: 0,
        onReadoutChange
      });
      chart.options.rotation = halfDoughnutOptions.rotation;
      chart.options.circumference = halfDoughnutOptions.circumference;
      chart.options.cutout = halfDoughnutOptions.cutout;
      chart.options.maintainAspectRatio = halfDoughnutOptions.maintainAspectRatio;
      chart.options.onHover = halfDoughnutOptions.onHover;
      chart.options.resizeDelay = halfDoughnutOptions.resizeDelay;
      if (chart.options?.plugins?.tooltip) {
        chart.options.plugins.tooltip.enabled = false;
      }
      if (chart.options?.plugins?.legend) {
        chart.options.plugins.legend.position = halfDoughnutOptions.plugins.legend.position;
      }
      if (chart.options?.plugins?.legend?.labels) {
        chart.options.plugins.legend.labels.color = chartTheme.legendColor;
      }
      chart.update('none');
    }
    const nextChart = stampChartRefresh(chart, refreshNonce);
    syncHalfDoughnutReadout(nextChart, onReadoutChange);
    return nextChart;
  };

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
    network: formatListTextarea(config.allowlist)
  });

  const currentBypassAllowlistsBaseline = () => ({
    enabled: bypassAllowlistsEnabled === true,
    network: bypassAllowlistsNetworkNormalized
  });

  const applyBypassAllowlistsConfig = (config = {}) => {
    const next = readBypassAllowlistsConfig(config);
    bypassAllowlistsEnabled = next.enabled;
    networkAllowlist = next.network;
    bypassAllowlistsBaseline = {
      enabled: next.enabled === true,
      network: normalizeListTextareaForCompare(next.network)
    };
  };

  const readIpRangeConfig = (config = {}) => ({
    mode: normalizeIpRangePolicyMode(config.ip_range_policy_mode),
    emergencyAllowlist: formatListTextarea(config.ip_range_emergency_allowlist),
    customRulesJson: formatJsonObjectLines(config.ip_range_custom_rules)
  });

  const currentIpRangeBaseline = () => ({
    mode: ipRangeModeNormalized,
    emergencyAllowlist: ipRangeEmergencyAllowlistNormalized,
    customRulesJson: ipRangeCustomRulesValidation.normalized
  });

  const applyIpRangeConfig = (config = {}) => {
    const next = readIpRangeConfig(config);
    const parsedEmergencyAllowlist = parseEmergencyAllowlistField(next.emergencyAllowlist);
    const parsedCustomRules = parseJsonObjectLinesField(
      next.customRulesJson,
      'Custom rules',
      validateCustomRuleShape
    );
    ipRangePolicyMode = next.mode;
    ipRangeEmergencyAllowlist = next.emergencyAllowlist;
    ipRangeCustomRulesJson = next.customRulesJson;
    ipRangeBaseline = {
      mode: next.mode,
      emergencyAllowlist: parsedEmergencyAllowlist.normalized,
      customRulesJson: parsedCustomRules.normalized
    };
  };

  const normalizeSuggestionCidr = (value) => String(value || '').trim();

  const normalizeSuggestionRiskBand = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'low') return 'low';
    if (normalized === 'medium') return 'medium';
    return 'high';
  };

  const normalizeSuggestionAction = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'deny_temp') return 'deny_temp';
    if (normalized === 'tarpit') return 'tarpit';
    return 'logging-only';
  };

  const normalizeSuggestionMode = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return normalized === 'enforce' ? 'enforce' : 'logging-only';
  };

  const toPercentLabel = (value) => `${(Number(value || 0) * 100).toFixed(1)}%`;

  const suggestionActionLabel = (action) => {
    if (action === 'deny_temp') return 'deny_temp';
    if (action === 'tarpit') return 'tarpit';
    return 'logging-only';
  };

  const suggestionRuleAction = (action) => {
    if (action === 'deny_temp') return 'honeypot';
    if (action === 'tarpit') return 'tarpit';
    return 'tarpit';
  };

  const suggestionRuleIdBase = (cidr, action) => {
    const actionPart = String(action || 'tarpit').toLowerCase();
    const cidrPart = String(cidr || '')
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 42);
    return `suggested-${actionPart}-${cidrPart || 'range'}`;
  };

  const normalizeCidrs = (value) => (
    Array.isArray(value)
      ? value
        .map((entry) => String(entry || '').trim())
        .filter((entry) => entry.length > 0)
        .sort()
      : []
  );

  const suggestionAlreadyMapped = (rule, existingRules) => {
    const targetCidrs = normalizeCidrs(rule.cidrs);
    return existingRules.some((candidate) => {
      if (!candidate || typeof candidate !== 'object') return false;
      if (String(candidate.id || '').trim() === String(rule.id || '').trim()) return true;
      const candidateCidrs = normalizeCidrs(candidate.cidrs);
      if (candidateCidrs.length !== targetCidrs.length) return false;
      if (candidateCidrs.join(',') !== targetCidrs.join(',')) return false;
      return String(candidate.action || '').trim() === String(rule.action || '').trim();
    });
  };

  const ensureUniqueRuleId = (baseId, existingRules) => {
    const takenIds = new Set(
      existingRules
        .map((rule) => (rule && typeof rule === 'object' ? String(rule.id || '').trim() : ''))
        .filter((value) => value.length > 0)
    );
    if (!takenIds.has(baseId)) return baseId;
    let suffix = 2;
    let candidate = `${baseId}-${suffix}`;
    while (takenIds.has(candidate)) {
      suffix += 1;
      candidate = `${baseId}-${suffix}`;
    }
    return candidate;
  };

  const normalizeIpRangeSuggestionsSnapshot = (snapshot = {}) => {
    const source = snapshot && typeof snapshot === 'object' ? snapshot : {};
    const summary = source.summary && typeof source.summary === 'object' ? source.summary : {};
    const suggestions = Array.isArray(source.suggestions) ? source.suggestions : [];
    return {
      summary: {
        suggestionsTotal: Number(summary.suggestions_total || suggestions.length || 0),
        lowRisk: Number(summary.low_risk || 0),
        mediumRisk: Number(summary.medium_risk || 0),
        highRisk: Number(summary.high_risk || 0)
      },
      suggestions: suggestions
        .map((entry, index) => {
          const candidate = entry && typeof entry === 'object' ? entry : {};
          const cidr = normalizeSuggestionCidr(candidate.cidr);
          if (!cidr) return null;
          const recommendedAction = normalizeSuggestionAction(candidate.recommended_action);
          const recommendedMode = normalizeSuggestionMode(candidate.recommended_mode);
          const riskBand = normalizeSuggestionRiskBand(candidate.risk_band);
          const key = `${cidr}|${recommendedAction}|${recommendedMode}|${index}`;
          return {
            key,
            cidr,
            ipFamily: String(candidate.ip_family || '').toLowerCase() === 'ipv6' ? 'ipv6' : 'ipv4',
            botEvidenceScore: Number(candidate.bot_evidence_score || 0),
            humanEvidenceScore: Number(candidate.human_evidence_score || 0),
            collateralRisk: Number(candidate.collateral_risk || 0),
            confidence: Number(candidate.confidence || 0),
            riskBand,
            recommendedAction,
            recommendedMode,
            saferAlternatives: Array.isArray(candidate.safer_alternatives)
              ? candidate.safer_alternatives.map((value) => String(value || '')).filter((value) => value)
              : [],
            guardrailNotes: Array.isArray(candidate.guardrail_notes)
              ? candidate.guardrail_notes.map((value) => String(value || '')).filter((value) => value)
              : []
          };
        })
        .filter(Boolean)
    };
  };

  const buildRuleFromSuggestion = (suggestion, applyMode, existingRules) => {
    const recommendedAction = suggestionRuleAction(suggestion.recommendedAction);
    const baseId = suggestionRuleIdBase(suggestion.cidr, suggestion.recommendedAction);
    return {
      id: ensureUniqueRuleId(baseId, existingRules),
      enabled: applyMode === 'enforce',
      cidrs: [suggestion.cidr],
      action: recommendedAction
    };
  };

  const hasSuggestionRule = (suggestion) => {
    const existingRules = Array.isArray(ipRangeCustomRulesValidation.parsed)
      ? ipRangeCustomRulesValidation.parsed
      : [];
    const draft = {
      id: suggestionRuleIdBase(suggestion.cidr, suggestion.recommendedAction),
      cidrs: [suggestion.cidr],
      action: suggestionRuleAction(suggestion.recommendedAction)
    };
    return suggestionAlreadyMapped(draft, existingRules);
  };

  const applySuggestionToCustomRules = (suggestion, applyMode) => {
    if (!writable || !suggestion) return;
    const existingRules = Array.isArray(ipRangeCustomRulesValidation.parsed)
      ? [...ipRangeCustomRulesValidation.parsed]
      : [];
    const rule = buildRuleFromSuggestion(suggestion, applyMode, existingRules);
    if (suggestionAlreadyMapped(rule, existingRules)) return;
    existingRules.push(rule);
    ipRangeCustomRulesJson = formatJsonObjectLines(existingRules);
  };

  const toKey = (ban, index) =>
    `${String(ban?.ip || '-')}:${String(ban?.reason || '-')}:${String(ban?.banned_at || 0)}:${String(ban?.expires || 0)}:${index}`;

  const isExpanded = (key) => expandedRows[key] === true;

  const formatIpRangeSourceLabel = (source) => {
    const normalized = String(source || '').trim().toLowerCase();
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
        allowlist: parseListTextarea(networkAllowlist)
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
        ip_range_custom_rules: ipRangeCustomRulesValidation.parsed
      };
      const nextConfig = await onSaveConfig(payload, { successMessage: 'IP range policy saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyIpRangeConfig(nextConfig);
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

  $: writable = isAdminConfigWritable(configRuntimeSnapshot);
  $: hasConfigSnapshot = configSnapshot && typeof configSnapshot === 'object' && Object.keys(configSnapshot).length > 0;
  $: bypassAllowlistsNetworkNormalized = normalizeListTextareaForCompare(networkAllowlist);
  $: bypassAllowlistsDirty = (
    (bypassAllowlistsEnabled === true) !== bypassAllowlistsBaseline.enabled ||
    bypassAllowlistsNetworkNormalized !== bypassAllowlistsBaseline.network
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
  $: ipRangeCustomRulesValid = ipRangeCustomRulesValidation.valid;
  $: ipRangeValid = ipRangeEmergencyAllowlistValid && ipRangeCustomRulesValid;
  $: ipRangeDirty = (
    ipRangeModeNormalized !== ipRangeBaseline.mode ||
    ipRangeEmergencyAllowlistNormalized !== ipRangeBaseline.emergencyAllowlist ||
    ipRangeCustomRulesValidation.normalized !== ipRangeBaseline.customRulesJson
  );
  $: ipRangeInvalidSummary = !ipRangeEmergencyAllowlistValid
    ? ipRangeEmergencyAllowlistValidation.error
    : (!ipRangeCustomRulesValid
      ? ipRangeCustomRulesValidation.error
      : '');
  $: saveIpRangeDisabled = !writable || !ipRangeDirty || !ipRangeValid || savingIpRange;
  $: saveIpRangeLabel = savingIpRange ? 'Saving...' : 'Save IP range policy';
  $: saveIpRangeSummary = ipRangeDirty
    ? 'IP range policy has unsaved changes'
    : 'No unsaved changes';
  $: warnOnUnload = writable && (ipRangeDirty || bypassAllowlistsDirty);
  $: banSnapshotStatus = String(bansSnapshot?.status || 'available').trim().toLowerCase() || 'available';
  $: banSnapshotUnavailableMessage = banSnapshotStatus === 'unavailable'
    ? String(bansSnapshot?.message || '').trim()
    : '';
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
  $: {
    const reasonCounts = new Map();
    filteredBanRows.forEach((row) => {
      const reason = normalizeBanReasonLabel(row?.ban?.reason);
      reasonCounts.set(reason, Number(reasonCounts.get(reason) || 0) + 1);
    });
    banReasonEntries = buildHalfDoughnutSeries(Array.from(reasonCounts.entries())).entries;
  }
  $: banDurationSeconds = (
    (Number(banDurationDays) * 24 * 60 * 60) +
    (Number(banDurationHours) * 60 * 60) +
    (Number(banDurationMinutes) * 60)
  );
  $: canBan = isValidIp(banIp) && banDurationSeconds > 0 && !banning;
  $: canUnban = isValidIp(unbanIp) && !unbanning;
  $: {
    const nextActive = isActive === true;
    if (browser && nextActive && !wasActive) {
      chartRefreshNonce += 1;
    }
    wasActive = nextActive;
  }
  $: {
    if (!banReasonCanvas || banReasonEntries.length === 0 || !isActive) {
      banReasonReadout = EMPTY_HALF_DOUGHNUT_READOUT;
      if (banReasonChart && typeof banReasonChart.destroy === 'function') {
        banReasonChart.destroy();
      }
      banReasonChart = null;
    } else if (!canvasHasRenderableSize(banReasonCanvas)) {
      // Defer first render until layout is stable when tab visibility flips.
      scheduleChartRefresh(0);
    } else {
      banReasonChart = updateBanReasonChart(
        banReasonChart,
        banReasonCanvas,
        banReasonEntries,
        chartRefreshNonce,
        (nextReadout) => {
          banReasonReadout = nextReadout;
        }
      );
    }
  }

  $: {
    const nextSuggestionsVersion = Number(ipRangeSuggestionsVersion || 0);
    if (nextSuggestionsVersion !== lastAppliedSuggestionsVersion) {
      lastAppliedSuggestionsVersion = nextSuggestionsVersion;
      const normalized = normalizeIpRangeSuggestionsSnapshot(ipRangeSuggestionsSnapshot || {});
      ipRangeSuggestions = normalized.suggestions;
      ipRangeSuggestionsSummary = normalized.summary;
    }
  }

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      const nextConfig = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};
      applyConfiguredBanDuration(nextConfig);
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
  class="admin-group"
  data-dashboard-tab-panel="ip-bans"
  aria-labelledby="dashboard-tab-ip-bans"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
  <TabStateMessage tab="ip-bans" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="chart-container panel-soft panel-border pad-md">
    <h2>Ban Reason Spread</h2>
    <p class="control-desc text-muted">
      Distribution by reason for the current ban view.
    </p>
    {#if banReasonEntries.length === 0}
      <p class="control-desc text-muted">No active ban reasons in this view.</p>
    {:else}
      <HalfDoughnutChart
        canvasId="ip-ban-reasons-chart"
        ariaLabel="IP ban reason spread chart"
        shellClass="chart-canvas-shell--ip-bans"
        bind:canvas={banReasonCanvas}
        readout={banReasonReadout}
      />
    {/if}
  </div>
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
    {#if banSnapshotUnavailableMessage}
      <p id="ip-bans-state-unavailable" class="message warning">
        {banSnapshotUnavailableMessage}
      </p>
    {/if}
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
            {#if banSnapshotUnavailableMessage}
              Authoritative active-ban state unavailable
            {:else if banFilter === 'ip-range'}
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

  <div class="control-group panel-soft pad-md">
    <div class="panel-heading-with-control">
      <h3>Suggested Ranges (Last 24h)</h3>
      <span class="text-muted">
        {ipRangeSuggestionsSummary.suggestionsTotal} candidate{ipRangeSuggestionsSummary.suggestionsTotal === 1 ? '' : 's'}
      </span>
    </div>
    <p class="control-desc text-muted">
      Suggestions are computed from recent abuse and likely-human evidence.
      Apply as <code>logging-only</code> first when uncertain, then promote to enforce after validation.
    </p>
    <p class="control-desc text-muted">
      <strong>Recommendation mapping:</strong> <code>deny_temp</code> suggestions map to the existing <code>honeypot</code> rule action on apply.
    </p>
    {#if ipRangeSuggestions.length === 0}
      <p class="control-desc text-muted">No suggestions available for the current telemetry window.</p>
    {:else}
      <TableWrapper>
        <table id="ip-range-suggestions-table" class="panel panel-border bans-table-admin">
          <thead>
            <tr>
              <th class="caps-label"><abbr title="Classless Inter-Domain Routing">CIDR</abbr></th>
              <th class="caps-label">Risk</th>
              <th class="caps-label">Evidence</th>
              <th class="caps-label">Recommendation</th>
              <th class="caps-label">Apply</th>
            </tr>
          </thead>
          <tbody>
            {#each ipRangeSuggestions as suggestion (suggestion.key)}
              {@const alreadyMapped = hasSuggestionRule(suggestion)}
              <tr>
                <td>
                  <code>{suggestion.cidr}</code>
                  <div class="ban-detail-content">
                    <span class="text-muted">{suggestion.ipFamily}</span>
                  </div>
                </td>
                <td>
                  <span class={`ban-signal-badge suggestion-risk-badge suggestion-risk-badge--${suggestion.riskBand}`}>{suggestion.riskBand}</span>
                  <div class="ban-detail-content">
                    <span>confidence {toPercentLabel(suggestion.confidence)}</span>
                    <span>collateral {toPercentLabel(suggestion.collateralRisk)}</span>
                  </div>
                </td>
                <td>
                  <div class="ban-detail-content">
                    <span>bot score {suggestion.botEvidenceScore.toFixed(2)}</span>
                    <span>human score {suggestion.humanEvidenceScore.toFixed(2)}</span>
                    {#if suggestion.saferAlternatives.length > 0}
                      <span>safer: {suggestion.saferAlternatives.join(', ')}</span>
                    {/if}
                    {#if suggestion.guardrailNotes.length > 0}
                      <span class="text-muted">{suggestion.guardrailNotes.join(' · ')}</span>
                    {/if}
                  </div>
                </td>
                <td>
                  <div class="ban-detail-content">
                    <span><code>{suggestionActionLabel(suggestion.recommendedAction)}</code></span>
                    <span class="text-muted">{suggestion.recommendedMode}</span>
                  </div>
                </td>
                <td>
                  <div class="suggestion-actions">
                    <button
                      type="button"
                      class="btn btn-subtle"
                      disabled={!writable || alreadyMapped}
                      on:click={() => applySuggestionToCustomRules(suggestion, 'logging-only')}
                    >Add as logging-only</button>
                    <button
                      type="button"
                      class="btn btn-subtle"
                      disabled={!writable || alreadyMapped}
                      on:click={() => applySuggestionToCustomRules(suggestion, 'enforce')}
                    >Add as enforce</button>
                    {#if alreadyMapped}
                      <span class="text-muted">Already added to custom rules.</span>
                    {/if}
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </TableWrapper>
    {/if}
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
        <option value="advisory">logging-only</option>
        <option value="enforce">enforce</option>
      </select>
    </div>
    <p class="control-desc text-muted">
      Use this to control what happens when traffic matches configured IP ranges.
      Start in <code>logging-only</code> first so you can observe outcomes before enabling enforcement.
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
    </div>
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
    <p class="control-desc text-muted">Define trusted IP/CIDR bypass entries. Use one entry per line.</p>
    <p class="control-desc text-muted">
      If a legitimate visitor is blocked by IP range policy, their specific IP will not be in the IP Ban list so unbanning it will not help. If they still match the same range rule, they will be blocked again on the next request. You will need to add their known-to-be-safe IP or CIDR to the IP/CIDR Allowlist below, or change the matching IP range rule to avoid their IP.
    </p>
    <div class="admin-controls">
      <TextareaField id="network-allowlist" label='<abbr title="Internet Protocol">IP</abbr>/<abbr title="Classless Inter-Domain Routing">CIDR</abbr> Allowlist' rows="3" ariaLabel="Internet Protocol and Classless Inter-Domain Routing allowlist" spellcheck={false} disabled={!writable} bind:value={networkAllowlist} />
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
