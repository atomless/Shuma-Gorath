<script>
  import { onDestroy, onMount } from 'svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import {
    formatBrowserRulesTextarea,
    formatListTextarea,
    normalizeBrowserRulesForCompare,
    normalizeCountryCodesForCompare,
    normalizeListTextareaForCompare,
    parseBrowserRulesTextarea,
    parseCountryCodesStrict,
    parseHoneypotPathsTextarea,
    parseListTextarea
  } from '../../domain/config-form-utils.js';
  import { advancedConfigTemplatePaths } from '../../domain/config-schema.js';
  import {
    buildTemplateFromPaths,
    normalizeJsonObjectForCompare
  } from '../../domain/core/json-object.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let onValidateConfig = null;
  export let onFetchRobotsPreview = null;

  const MAX_DURATION_SECONDS = 31536000;
  const MIN_DURATION_SECONDS = 60;
  const EDGE_MODES = new Set(['off', 'advisory', 'authoritative']);
  const IP_RANGE_POLICY_MODES = new Set(['off', 'advisory', 'enforce']);
  const IP_RANGE_MANAGED_STALENESS_MIN = 1;
  const IP_RANGE_MANAGED_STALENESS_MAX = 2160;
  const EXPORT_STATUS_RESET_MS = 4000;
  const ADVANCED_VALIDATE_DEBOUNCE_MS = 350;
  const ADVANCED_JSON_DISALLOWED_KEYS = new Set(['test_mode']);

  let writable = false;
  let hasConfigSnapshot = false;
  let configLoaded = true;
  let lastAppliedConfigVersion = -1;
  let skipNextConfigVersionApply = false;
  let savingTestMode = false;
  let savingAll = false;
  let warnOnUnload = false;

  let robotsPreviewOpen = false;
  let robotsPreviewLoading = false;
  let robotsPreviewContent = '';

  let testMode = false;

  let jsRequiredEnforced = true;

  let powEnabled = true;
  let powDifficulty = 15;
  let powTtl = 90;

  let challengePuzzleEnabled = true;
  let challengePuzzleTransformCount = 6;
  let notABotEnabled = true;
  let notABotScorePassMin = 7;
  let notABotScoreEscalateMin = 4;
  let notABotNonceTtl = 120;
  let notABotMarkerTtl = 600;
  let notABotAttemptLimit = 6;
  let notABotAttemptWindow = 300;

  let ipRangePolicyMode = 'off';
  let ipRangeEmergencyAllowlist = '';
  let ipRangeCustomRulesJson = '[]';
  let ipRangeManagedPoliciesJson = '[]';
  let ipRangeManagedMaxStalenessHours = 168;
  let ipRangeAllowStaleManagedEnforce = false;
  let ipRangeManagedSets = [];
  let ipRangeCatalogVersion = '-';
  let ipRangeCatalogGeneratedAt = '-';
  let ipRangeCatalogAgeHours = null;
  let ipRangeCatalogStale = false;

  let rateLimitThreshold = 80;

  let honeypotEnabled = true;
  let honeypotPaths = '';

  let browserBlockRules = '';
  let browserWhitelistRules = '';

  let networkWhitelist = '';
  let pathWhitelist = '';

  let mazeEnabled = true;
  let mazeAutoBan = true;
  let mazeThreshold = 50;

  let cdpEnabled = true;
  let cdpAutoBan = true;
  let cdpThreshold = 0.6;

  let edgeIntegrationMode = 'off';

  let geoRiskList = '';
  let geoAllowList = '';
  let geoChallengeList = '';
  let geoMazeList = '';
  let geoBlockList = '';

  let durHoneypotDays = 1;
  let durHoneypotHours = 0;
  let durHoneypotMinutes = 0;
  let durRateLimitDays = 0;
  let durRateLimitHours = 1;
  let durRateLimitMinutes = 0;
  let durBrowserDays = 0;
  let durBrowserHours = 6;
  let durBrowserMinutes = 0;
  let durCdpDays = 0;
  let durCdpHours = 12;
  let durCdpMinutes = 0;
  let durAdminDays = 0;
  let durAdminHours = 6;
  let durAdminMinutes = 0;

  let robotsEnabled = true;
  let robotsCrawlDelay = 2;
  let robotsBlockTraining = true;
  let robotsBlockSearch = false;
  let restrictSearchEngines = false;

  let advancedConfigJson = '{}';
  let advancedValidationPending = false;
  let advancedValidationError = '';
  let advancedValidationIssues = [];
  let advancedValidationTimer = null;
  let advancedValidationRequestId = 0;
  let exportConfigStatus = '';
  let exportConfigStatusKind = 'info';
  let exportConfigStatusTimer = null;

  let baseline = {
    testMode: { enabled: false },
    jsRequired: { enforced: true },
    pow: { enabled: true, difficulty: 15, ttl: 90 },
    challenge: { enabled: true, count: 6 },
    notABot: {
      enabled: true,
      scorePassMin: 7,
      scoreEscalateMin: 4,
      nonceTtl: 120,
      markerTtl: 600,
      attemptLimit: 6,
      attemptWindow: 300
    },
    ipRange: {
      mode: 'off',
      emergencyAllowlist: '',
      customRulesJson: '[]',
      managedPoliciesJson: '[]',
      managedMaxStalenessHours: 168,
      allowStaleManagedEnforce: false
    },
    rateLimit: { value: 80 },
    honeypot: { enabled: true, values: '' },
    browserPolicy: { block: '', whitelist: '' },
    whitelist: { network: '', path: '' },
    maze: { enabled: true, autoBan: true, threshold: 50 },
    cdp: { enabled: true, autoBan: true, threshold: 0.6 },
    edgeMode: { mode: 'off' },
    geo: { risk: '', allow: '', challenge: '', maze: '', block: '' },
    durations: {
      honeypot: 86400,
      rateLimit: 3600,
      browser: 21600,
      cdp: 43200,
      admin: 21600
    },
    robots: { enabled: true, crawlDelay: 2 },
    aiPolicy: { blockTraining: true, blockSearch: false, restrictSearchEngines: false },
    advanced: { normalized: '{}' }
  };

  const parseInteger = (value, fallback) => {
    const parsed = Number.parseInt(value, 10);
    return Number.isInteger(parsed) ? parsed : fallback;
  };

  const parseFloatNumber = (value, fallback) => {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  };

  const formatCountryCodes = (values) => {
    if (!Array.isArray(values) || values.length === 0) return '';
    return values
      .map((value) => String(value || '').trim().toUpperCase())
      .filter(Boolean)
      .join(',');
  };

  const normalizeEdgeMode = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return EDGE_MODES.has(normalized) ? normalized : 'off';
  };

  const normalizeIpRangePolicyMode = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return IP_RANGE_POLICY_MODES.has(normalized) ? normalized : 'off';
  };

  const normalizeJsonArrayForCompare = (value) => {
    try {
      const parsed = JSON.parse(String(value || '[]'));
      if (!Array.isArray(parsed)) return null;
      return JSON.stringify(parsed);
    } catch (_error) {
      return null;
    }
  };

  const durationPartsFromSeconds = (seconds, fallbackSeconds) => {
    const source = Number.parseInt(seconds, 10);
    const safe = Number.isFinite(source) && source > 0 ? source : fallbackSeconds;
    const days = Math.floor(safe / 86400);
    const remainingAfterDays = safe - (days * 86400);
    const hours = Math.floor(remainingAfterDays / 3600);
    const remainingAfterHours = remainingAfterDays - (hours * 3600);
    const minutes = Math.floor(remainingAfterHours / 60);
    return {
      days,
      hours,
      minutes
    };
  };

  const durationSeconds = (days, hours, minutes) => {
    const d = parseInteger(days, 0);
    const h = parseInteger(hours, 0);
    const m = parseInteger(minutes, 0);
    return (d * 86400) + (h * 3600) + (m * 60);
  };

  const inRange = (value, min, max) => {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) && parsed >= min && parsed <= max;
  };

  const isDurationTupleValid = (days, hours, minutes) => {
    if (!inRange(days, 0, 365)) return false;
    if (!inRange(hours, 0, 23)) return false;
    if (!inRange(minutes, 0, 59)) return false;
    const total = durationSeconds(days, hours, minutes);
    return total >= MIN_DURATION_SECONDS && total <= MAX_DURATION_SECONDS;
  };

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  const clearExportStatusTimer = () => {
    if (exportConfigStatusTimer) {
      clearTimeout(exportConfigStatusTimer);
      exportConfigStatusTimer = null;
    }
  };

  const scheduleExportStatusReset = () => {
    clearExportStatusTimer();
    exportConfigStatusTimer = setTimeout(() => {
      exportConfigStatus = '';
      exportConfigStatusKind = 'info';
      exportConfigStatusTimer = null;
    }, EXPORT_STATUS_RESET_MS);
  };

  const clearAdvancedValidationTimer = () => {
    if (advancedValidationTimer) {
      clearTimeout(advancedValidationTimer);
      advancedValidationTimer = null;
    }
  };

  const resetAdvancedValidationState = () => {
    clearAdvancedValidationTimer();
    advancedValidationPending = false;
    advancedValidationError = '';
    advancedValidationIssues = [];
  };

  const normalizeAdvancedValidationIssues = (issues) => {
    if (!Array.isArray(issues)) return [];
    return issues
      .filter((issue) => issue && typeof issue === 'object')
      .map((issue) => {
        const source = /** @type {Record<string, unknown>} */ (issue);
        return {
          field: typeof source.field === 'string' ? source.field : '',
          message: typeof source.message === 'string' ? source.message : 'Invalid value.',
          expected: typeof source.expected === 'string' ? source.expected : '',
          received: Object.prototype.hasOwnProperty.call(source, 'received')
            ? source.received
            : undefined
        };
      });
  };

  const formatIssueReceived = (value) => {
    if (value === undefined) return '';
    if (value === null) return 'null';
    if (typeof value === 'string') return `"${value}"`;
    try {
      return JSON.stringify(value);
    } catch (_error) {
      return String(value);
    }
  };

  async function runAdvancedServerValidation(advancedPatch, requestId) {
    if (typeof onValidateConfig !== 'function') {
      if (requestId !== advancedValidationRequestId) return;
      advancedValidationPending = false;
      advancedValidationError = '';
      advancedValidationIssues = [];
      return;
    }

    try {
      const result = await onValidateConfig(advancedPatch);
      if (requestId !== advancedValidationRequestId) return;
      const issues = normalizeAdvancedValidationIssues(result && result.issues);
      advancedValidationIssues = issues;
      advancedValidationError = '';
      advancedValidationPending = false;
    } catch (error) {
      if (requestId !== advancedValidationRequestId) return;
      advancedValidationIssues = [];
      advancedValidationPending = false;
      advancedValidationError = error && error.message
        ? String(error.message)
        : 'Unable to validate Advanced JSON right now.';
    }
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  onDestroy(() => {
    clearExportStatusTimer();
    clearAdvancedValidationTimer();
  });

  function resetRobotsPreview() {
    robotsPreviewOpen = false;
    robotsPreviewLoading = false;
    robotsPreviewContent = '';
  }

  function applyConfig(config = {}) {
    configLoaded = true;
    hasConfigSnapshot = config && typeof config === 'object' && Object.keys(config).length > 0;
    writable = config.admin_config_write_enabled === true;

    testMode = config.test_mode === true;
    jsRequiredEnforced = config.js_required_enforced !== false;

    powEnabled = config.pow_enabled !== false;
    powDifficulty = parseInteger(config.pow_difficulty, 15);
    powTtl = parseInteger(config.pow_ttl_seconds, 90);

    challengePuzzleEnabled = config.challenge_puzzle_enabled !== false;
    challengePuzzleTransformCount = parseInteger(config.challenge_puzzle_transform_count, 6);
    notABotEnabled = config.not_a_bot_enabled !== false;
    notABotScorePassMin = parseInteger(config.not_a_bot_score_pass_min, 7);
    notABotScoreEscalateMin = parseInteger(config.not_a_bot_score_escalate_min, 4);
    notABotNonceTtl = parseInteger(config.not_a_bot_nonce_ttl_seconds, 120);
    notABotMarkerTtl = parseInteger(config.not_a_bot_marker_ttl_seconds, 600);
    notABotAttemptLimit = parseInteger(config.not_a_bot_attempt_limit_per_window, 6);
    notABotAttemptWindow = parseInteger(config.not_a_bot_attempt_window_seconds, 300);

    ipRangePolicyMode = normalizeIpRangePolicyMode(config.ip_range_policy_mode);
    ipRangeEmergencyAllowlist = formatListTextarea(config.ip_range_emergency_allowlist);
    ipRangeCustomRulesJson = JSON.stringify(
      Array.isArray(config.ip_range_custom_rules) ? config.ip_range_custom_rules : [],
      null,
      2
    );
    ipRangeManagedPoliciesJson = JSON.stringify(
      Array.isArray(config.ip_range_managed_policies) ? config.ip_range_managed_policies : [],
      null,
      2
    );
    ipRangeManagedMaxStalenessHours = parseInteger(config.ip_range_managed_max_staleness_hours, 168);
    ipRangeAllowStaleManagedEnforce = config.ip_range_allow_stale_managed_enforce === true;
    ipRangeManagedSets = Array.isArray(config.ip_range_managed_sets) ? config.ip_range_managed_sets : [];
    ipRangeCatalogVersion = String(config.ip_range_managed_catalog_version || '-');
    ipRangeCatalogGeneratedAt = String(config.ip_range_managed_catalog_generated_at || '-');
    const catalogGeneratedAtUnix = Number(config.ip_range_managed_catalog_generated_at_unix || 0);
    if (Number.isFinite(catalogGeneratedAtUnix) && catalogGeneratedAtUnix > 0) {
      const nowUnix = Math.floor(Date.now() / 1000);
      ipRangeCatalogAgeHours = nowUnix >= catalogGeneratedAtUnix
        ? Math.floor((nowUnix - catalogGeneratedAtUnix) / 3600)
        : 0;
    } else {
      ipRangeCatalogAgeHours = null;
    }
    const staleByAge = Number.isFinite(ipRangeCatalogAgeHours)
      ? Number(ipRangeCatalogAgeHours) > Number(ipRangeManagedMaxStalenessHours)
      : false;
    ipRangeCatalogStale =
      staleByAge || ipRangeManagedSets.some((set) => set && set.stale === true);

    rateLimitThreshold = parseInteger(config.rate_limit, 80);

    honeypotEnabled = config.honeypot_enabled !== false;
    honeypotPaths = formatListTextarea(config.honeypots);

    browserBlockRules = formatBrowserRulesTextarea(config.browser_block);
    browserWhitelistRules = formatBrowserRulesTextarea(config.browser_whitelist);

    networkWhitelist = formatListTextarea(config.whitelist);
    pathWhitelist = formatListTextarea(config.path_whitelist);

    mazeEnabled = config.maze_enabled !== false;
    mazeAutoBan = config.maze_auto_ban !== false;
    mazeThreshold = parseInteger(config.maze_auto_ban_threshold, 50);

    cdpEnabled = config.cdp_detection_enabled !== false;
    cdpAutoBan = config.cdp_auto_ban !== false;
    cdpThreshold = Number(parseFloatNumber(config.cdp_detection_threshold, 0.6).toFixed(1));

    edgeIntegrationMode = normalizeEdgeMode(config.edge_integration_mode);

    geoRiskList = formatCountryCodes(config.geo_risk);
    geoAllowList = formatCountryCodes(config.geo_allow);
    geoChallengeList = formatCountryCodes(config.geo_challenge);
    geoMazeList = formatCountryCodes(config.geo_maze);
    geoBlockList = formatCountryCodes(config.geo_block);

    const banDurations = config && typeof config.ban_durations === 'object'
      ? config.ban_durations
      : {};

    const honeypotParts = durationPartsFromSeconds(banDurations.honeypot, 86400);
    durHoneypotDays = honeypotParts.days;
    durHoneypotHours = honeypotParts.hours;
    durHoneypotMinutes = honeypotParts.minutes;

    const rateLimitParts = durationPartsFromSeconds(banDurations.rate_limit, 3600);
    durRateLimitDays = rateLimitParts.days;
    durRateLimitHours = rateLimitParts.hours;
    durRateLimitMinutes = rateLimitParts.minutes;

    const browserParts = durationPartsFromSeconds(banDurations.browser, 21600);
    durBrowserDays = browserParts.days;
    durBrowserHours = browserParts.hours;
    durBrowserMinutes = browserParts.minutes;

    const cdpParts = durationPartsFromSeconds(banDurations.cdp, 43200);
    durCdpDays = cdpParts.days;
    durCdpHours = cdpParts.hours;
    durCdpMinutes = cdpParts.minutes;

    const adminParts = durationPartsFromSeconds(banDurations.admin, 21600);
    durAdminDays = adminParts.days;
    durAdminHours = adminParts.hours;
    durAdminMinutes = adminParts.minutes;

    robotsEnabled = config.robots_enabled !== false;
    robotsCrawlDelay = parseInteger(config.robots_crawl_delay, 2);

    robotsBlockTraining = (config.ai_policy_block_training ?? config.robots_block_ai_training) !== false;
    robotsBlockSearch = (config.ai_policy_block_search ?? config.robots_block_ai_search) === true;
    const aiAllowSearchEngines = config.ai_policy_allow_search_engines ?? config.robots_allow_search_engines;
    restrictSearchEngines = aiAllowSearchEngines === undefined ? false : aiAllowSearchEngines !== true;

    const advancedTemplate = buildTemplateFromPaths(config, advancedConfigTemplatePaths || []);
    const advancedText = JSON.stringify(advancedTemplate, null, 2);
    advancedConfigJson = advancedText;

    baseline = {
      testMode: { enabled: testMode },
      jsRequired: { enforced: jsRequiredEnforced },
      pow: {
        enabled: powEnabled,
        difficulty: Number(powDifficulty),
        ttl: Number(powTtl)
      },
      challenge: {
        enabled: challengePuzzleEnabled,
        count: Number(challengePuzzleTransformCount)
      },
      notABot: {
        enabled: notABotEnabled,
        scorePassMin: Number(notABotScorePassMin),
        scoreEscalateMin: Number(notABotScoreEscalateMin),
        nonceTtl: Number(notABotNonceTtl),
        markerTtl: Number(notABotMarkerTtl),
        attemptLimit: Number(notABotAttemptLimit),
        attemptWindow: Number(notABotAttemptWindow)
      },
      ipRange: {
        mode: normalizeIpRangePolicyMode(ipRangePolicyMode),
        emergencyAllowlist: normalizeListTextareaForCompare(ipRangeEmergencyAllowlist),
        customRulesJson: normalizeJsonArrayForCompare(ipRangeCustomRulesJson) || '[]',
        managedPoliciesJson: normalizeJsonArrayForCompare(ipRangeManagedPoliciesJson) || '[]',
        managedMaxStalenessHours: Number(ipRangeManagedMaxStalenessHours),
        allowStaleManagedEnforce: ipRangeAllowStaleManagedEnforce === true
      },
      rateLimit: { value: Number(rateLimitThreshold) },
      honeypot: {
        enabled: honeypotEnabled,
        values: normalizeListTextareaForCompare(honeypotPaths)
      },
      browserPolicy: {
        block: normalizeBrowserRulesForCompare(browserBlockRules),
        whitelist: normalizeBrowserRulesForCompare(browserWhitelistRules)
      },
      whitelist: {
        network: normalizeListTextareaForCompare(networkWhitelist),
        path: normalizeListTextareaForCompare(pathWhitelist)
      },
      maze: {
        enabled: mazeEnabled,
        autoBan: mazeAutoBan,
        threshold: Number(mazeThreshold)
      },
      cdp: {
        enabled: cdpEnabled,
        autoBan: cdpAutoBan,
        threshold: Number(cdpThreshold)
      },
      edgeMode: {
        mode: edgeIntegrationMode
      },
      geo: {
        risk: normalizeCountryCodesForCompare(geoRiskList),
        allow: normalizeCountryCodesForCompare(geoAllowList),
        challenge: normalizeCountryCodesForCompare(geoChallengeList),
        maze: normalizeCountryCodesForCompare(geoMazeList),
        block: normalizeCountryCodesForCompare(geoBlockList)
      },
      durations: {
        honeypot: durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes),
        rateLimit: durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes),
        browser: durationSeconds(durBrowserDays, durBrowserHours, durBrowserMinutes),
        cdp: durationSeconds(durCdpDays, durCdpHours, durCdpMinutes),
        admin: durationSeconds(durAdminDays, durAdminHours, durAdminMinutes)
      },
      robots: {
        enabled: robotsEnabled,
        crawlDelay: Number(robotsCrawlDelay)
      },
      aiPolicy: {
        blockTraining: robotsBlockTraining,
        blockSearch: robotsBlockSearch,
        restrictSearchEngines
      },
      advanced: {
        normalized: normalizeJsonObjectForCompare(advancedText) || '{}'
      }
    };

    clearExportStatusTimer();
    exportConfigStatus = '';
    exportConfigStatusKind = 'info';
    resetAdvancedValidationState();
    resetRobotsPreview();
  }

  async function onTestModeToggleChange(event) {
    const target = event && event.currentTarget ? event.currentTarget : null;
    const nextValue = target && target.checked === true;
    const previousValue = !nextValue;
    if (testModeToggleDisabled) {
      if (target) target.checked = testMode === true;
      return;
    }
    if (typeof onSaveConfig !== 'function') {
      if (target) target.checked = testMode === true;
      return;
    }
    if (savingTestMode) {
      if (target) target.checked = testMode === true;
      return;
    }
    savingTestMode = true;
    skipNextConfigVersionApply = true;
    try {
      const nextConfig = await onSaveConfig(
        { test_mode: nextValue },
        {
          successMessage: `Test mode ${nextValue ? 'enabled (logging only)' : 'disabled (blocking active)'}`,
          refresh: false
        }
      );
      const persistedValue = nextConfig && typeof nextConfig === 'object'
        ? nextConfig.test_mode === true
        : nextValue;
      testMode = persistedValue;
      baseline = {
        ...baseline,
        testMode: {
          enabled: persistedValue
        }
      };
      if (target) target.checked = persistedValue;
    } catch (_error) {
      skipNextConfigVersionApply = false;
      testMode = previousValue;
      if (target) target.checked = previousValue;
    } finally {
      savingTestMode = false;
    }
  }

  const parseAdvancedPatchObject = () => {
    const advancedPatch = JSON.parse(advancedConfigJson);
    if (!advancedPatch || typeof advancedPatch !== 'object' || Array.isArray(advancedPatch)) {
      throw new Error('Advanced config JSON patch must be an object.');
    }
    const sanitizedPatch = { ...advancedPatch };
    ADVANCED_JSON_DISALLOWED_KEYS.forEach((key) => {
      if (Object.prototype.hasOwnProperty.call(sanitizedPatch, key)) {
        delete sanitizedPatch[key];
      }
    });
    return sanitizedPatch;
  };

  const buildConfigPatch = ({ includeAll = false, includeAdvanced = true } = {}) => {
    const patch = {};
    if (includeAll) {
      if (includeAdvanced && advancedValid) {
        Object.assign(patch, parseAdvancedPatchObject());
      }
    } else if (advancedDirty) {
      Object.assign(patch, parseAdvancedPatchObject());
    }
    if (includeAll || jsRequiredDirty) {
      patch.js_required_enforced = jsRequiredEnforced;
    }
    if (includeAll || powDirty) {
      patch.pow_enabled = powEnabled;
      if (includeAll) {
        patch.pow_difficulty = Number(powDifficulty);
        patch.pow_ttl_seconds = Number(powTtl);
      }
    }
    if (includeAll || challengePuzzleDirty) {
      patch.challenge_puzzle_enabled = challengePuzzleEnabled;
      patch.challenge_puzzle_transform_count = Number(challengePuzzleTransformCount);
    }
    if (includeAll || notABotDirty) {
      patch.not_a_bot_enabled = notABotEnabled;
      patch.not_a_bot_score_pass_min = Number(notABotScorePassMin);
      patch.not_a_bot_score_escalate_min = Number(notABotScoreEscalateMin);
      patch.not_a_bot_nonce_ttl_seconds = Number(notABotNonceTtl);
      patch.not_a_bot_marker_ttl_seconds = Number(notABotMarkerTtl);
      patch.not_a_bot_attempt_limit_per_window = Number(notABotAttemptLimit);
      patch.not_a_bot_attempt_window_seconds = Number(notABotAttemptWindow);
    }
    if (includeAll || ipRangeDirty) {
      patch.ip_range_policy_mode = ipRangeModeNormalized;
      patch.ip_range_emergency_allowlist = parseListTextarea(ipRangeEmergencyAllowlist);
      patch.ip_range_custom_rules = JSON.parse(ipRangeCustomRulesJson);
      patch.ip_range_managed_policies = JSON.parse(ipRangeManagedPoliciesJson);
      patch.ip_range_managed_max_staleness_hours = Number(ipRangeManagedMaxStalenessHours);
      patch.ip_range_allow_stale_managed_enforce = ipRangeAllowStaleManagedEnforce === true;
    }
    if (includeAll || rateLimitDirty) {
      patch.rate_limit = Number(rateLimitThreshold);
    }
    if (includeAll || honeypotDirty) {
      patch.honeypot_enabled = honeypotEnabled;
      patch.honeypots = parseHoneypotPathsTextarea(honeypotPaths);
    }
    if (includeAll || browserPolicyDirty) {
      patch.browser_block = parseBrowserRulesTextarea(browserBlockRules);
      patch.browser_whitelist = parseBrowserRulesTextarea(browserWhitelistRules);
    }
    if (includeAll || whitelistDirty) {
      patch.whitelist = parseListTextarea(networkWhitelist);
      patch.path_whitelist = parseListTextarea(pathWhitelist);
    }
    if (includeAll || mazeDirty) {
      patch.maze_enabled = mazeEnabled;
      patch.maze_auto_ban = mazeAutoBan;
      patch.maze_auto_ban_threshold = Number(mazeThreshold);
    }
    if (includeAll || cdpDirty) {
      patch.cdp_detection_enabled = cdpEnabled;
      patch.cdp_auto_ban = cdpAutoBan;
      patch.cdp_detection_threshold = Number(cdpThreshold);
    }
    if (includeAll || edgeModeDirty) {
      patch.edge_integration_mode = edgeIntegrationMode;
    }
    if (includeAll || geoScoringDirty) {
      patch.geo_risk = parseCountryCodesStrict(geoRiskList);
    }
    if (includeAll || geoRoutingDirty) {
      patch.geo_allow = parseCountryCodesStrict(geoAllowList);
      patch.geo_challenge = parseCountryCodesStrict(geoChallengeList);
      patch.geo_maze = parseCountryCodesStrict(geoMazeList);
      patch.geo_block = parseCountryCodesStrict(geoBlockList);
    }
    if (includeAll || durationsDirty) {
      patch.ban_durations = {
        honeypot: durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes),
        rate_limit: durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes),
        browser: durationSeconds(durBrowserDays, durBrowserHours, durBrowserMinutes),
        cdp: durationSeconds(durCdpDays, durCdpHours, durCdpMinutes),
        admin: durationSeconds(durAdminDays, durAdminHours, durAdminMinutes)
      };
    }
    if (includeAll || robotsDirty) {
      patch.robots_enabled = robotsEnabled;
      patch.robots_crawl_delay = Number(robotsCrawlDelay);
    }
    if (includeAll || aiPolicyDirty) {
      patch.ai_policy_block_training = robotsBlockTraining;
      patch.ai_policy_block_search = robotsBlockSearch;
      patch.ai_policy_allow_search_engines = !restrictSearchEngines;
    }
    return patch;
  };

  const downloadJsonFile = (filename, payload) => {
    if (typeof window === 'undefined' || typeof document === 'undefined') return false;
    const blob = new Blob([payload], { type: 'application/json' });
    const url = window.URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    anchor.rel = 'noopener';
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.URL.revokeObjectURL(url);
    return true;
  };

  async function exportCurrentConfigJson(event) {
    if (event && typeof event.preventDefault === 'function') {
      event.preventDefault();
    }
    if (exportConfigDisabled) return;

    try {
      const includeAdvancedPatch = advancedValid === true;
      const payload = buildConfigPatch({ includeAll: true, includeAdvanced: includeAdvancedPatch });
      const text = JSON.stringify(payload, null, 2);
      const stamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = `shuma-config-${stamp}.json`;
      const downloaded = downloadJsonFile(filename, text);
      let copied = false;
      if (
        typeof window !== 'undefined' &&
        window.isSecureContext === true &&
        typeof navigator !== 'undefined' &&
        navigator.clipboard
      ) {
        try {
          await navigator.clipboard.writeText(text);
          copied = true;
        } catch (_error) {}
      }

      if (downloaded && copied) {
        exportConfigStatus = 'Exported config JSON downloaded and copied to clipboard.';
      } else if (downloaded) {
        exportConfigStatus = 'Exported config JSON downloaded.';
      } else if (copied) {
        exportConfigStatus = 'Exported config JSON copied to clipboard.';
      } else {
        exportConfigStatus = 'Exported config JSON generated.';
      }
      if (!includeAdvancedPatch) {
        exportConfigStatus += ' Advanced JSON editor input was invalid and excluded.';
      }
      exportConfigStatusKind = 'success';
      scheduleExportStatusReset();
    } catch (error) {
      exportConfigStatus = error && error.message
        ? error.message
        : 'Failed to export config JSON.';
      exportConfigStatusKind = 'error';
      scheduleExportStatusReset();
    }
  }

  async function saveAllConfig() {
    if (saveAllConfigDisabled || typeof onSaveConfig !== 'function') return;

    const patch = buildConfigPatch({ includeAll: false });

    if (Object.keys(patch).length === 0) return;

    savingAll = true;
    try {
      const successMessage = dirtySectionCount === 1
        ? `${dirtySectionLabels[0]} saved`
        : `Saved ${dirtySectionCount} configuration sections`;
      const nextConfig = await onSaveConfig(patch, { successMessage });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(nextConfig);
      }
      if (robotsPreviewOpen && (robotsDirty || aiPolicyDirty)) {
        await refreshRobotsPreview();
      }
    } finally {
      savingAll = false;
    }
  }

  async function refreshRobotsPreview() {
    if (typeof onFetchRobotsPreview !== 'function') return;
    robotsPreviewLoading = true;
    try {
      const payload = await onFetchRobotsPreview();
      robotsPreviewContent = payload && typeof payload.content === 'string'
        ? payload.content
        : '# No preview available';
    } catch (error) {
      robotsPreviewContent = `# Error loading preview: ${error && error.message ? error.message : 'Unknown error'}`;
    } finally {
      robotsPreviewLoading = false;
    }
  }

  async function toggleRobotsPreview() {
    if (robotsPreviewOpen) {
      robotsPreviewOpen = false;
      return;
    }
    robotsPreviewOpen = true;
    await refreshRobotsPreview();
  }

  const readBool = (value) => value === true;

  $: testModeToggleText = testMode ? 'Test Mode On' : 'Test Mode Off';
  $: testModeStatusText = !hasConfigSnapshot
    ? 'LOADING...'
    : (testMode ? '(LOGGING ONLY)' : '(BLOCKING ACTIVE)');
  $: testModeStatusClass = `text-muted status-value ${
    !hasConfigSnapshot
      ? ''
      : (testMode ? 'test-mode-status--enabled' : 'test-mode-status--disabled')
  }`.trim();

  $: testModeToggleDisabled = !writable || savingTestMode;

  $: jsRequiredDirty = readBool(jsRequiredEnforced) !== baseline.jsRequired.enforced;

  $: powValid = true;
  $: powDirty = readBool(powEnabled) !== baseline.pow.enabled;

  $: challengePuzzleValid = inRange(challengePuzzleTransformCount, 4, 8);
  $: challengePuzzleDirty = (
    readBool(challengePuzzleEnabled) !== baseline.challenge.enabled ||
    Number(challengePuzzleTransformCount) !== baseline.challenge.count
  );

  $: notABotValid = (
    inRange(notABotScorePassMin, 1, 10) &&
    inRange(notABotScoreEscalateMin, 1, 10) &&
    Number(notABotScoreEscalateMin) <= Number(notABotScorePassMin) &&
    inRange(notABotNonceTtl, 30, 300) &&
    inRange(notABotMarkerTtl, 60, 3600) &&
    inRange(notABotAttemptLimit, 1, 100) &&
    inRange(notABotAttemptWindow, 30, 3600)
  );
  $: notABotDirty = (
    readBool(notABotEnabled) !== baseline.notABot.enabled ||
    Number(notABotScorePassMin) !== baseline.notABot.scorePassMin ||
    Number(notABotScoreEscalateMin) !== baseline.notABot.scoreEscalateMin ||
    Number(notABotNonceTtl) !== baseline.notABot.nonceTtl ||
    Number(notABotMarkerTtl) !== baseline.notABot.markerTtl ||
    Number(notABotAttemptLimit) !== baseline.notABot.attemptLimit ||
    Number(notABotAttemptWindow) !== baseline.notABot.attemptWindow
  );

  $: ipRangeEmergencyAllowlistNormalized = normalizeListTextareaForCompare(ipRangeEmergencyAllowlist);
  $: ipRangeCustomRulesNormalized = normalizeJsonArrayForCompare(ipRangeCustomRulesJson);
  $: ipRangeManagedPoliciesNormalized = normalizeJsonArrayForCompare(ipRangeManagedPoliciesJson);
  $: ipRangeModeNormalized = normalizeIpRangePolicyMode(ipRangePolicyMode);
  $: ipRangeValid = (
    IP_RANGE_POLICY_MODES.has(ipRangeModeNormalized) &&
    ipRangeCustomRulesNormalized !== null &&
    ipRangeManagedPoliciesNormalized !== null &&
    inRange(
      ipRangeManagedMaxStalenessHours,
      IP_RANGE_MANAGED_STALENESS_MIN,
      IP_RANGE_MANAGED_STALENESS_MAX
    )
  );
  $: ipRangeDirty = (
    ipRangeModeNormalized !== baseline.ipRange.mode ||
    ipRangeEmergencyAllowlistNormalized !== baseline.ipRange.emergencyAllowlist ||
    ipRangeCustomRulesNormalized !== baseline.ipRange.customRulesJson ||
    ipRangeManagedPoliciesNormalized !== baseline.ipRange.managedPoliciesJson ||
    Number(ipRangeManagedMaxStalenessHours) !== baseline.ipRange.managedMaxStalenessHours ||
    (ipRangeAllowStaleManagedEnforce === true) !== baseline.ipRange.allowStaleManagedEnforce
  );
  $: ipRangeManagedSetRows = Array.isArray(ipRangeManagedSets) ? ipRangeManagedSets : [];
  $: ipRangeManagedSetStaleCount = ipRangeManagedSetRows.filter((set) => set?.stale === true).length;
  $: ipRangeCatalogStale = (
    (Number.isFinite(ipRangeCatalogAgeHours)
      ? Number(ipRangeCatalogAgeHours) > Number(ipRangeManagedMaxStalenessHours)
      : false) ||
    ipRangeManagedSetStaleCount > 0
  );

  $: rateLimitValid = inRange(rateLimitThreshold, 1, 1000000);
  $: rateLimitDirty = Number(rateLimitThreshold) !== baseline.rateLimit.value;

  $: honeypotNormalized = normalizeListTextareaForCompare(honeypotPaths);
  $: honeypotValid = (() => {
    try {
      parseHoneypotPathsTextarea(honeypotPaths);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: honeypotDirty = (
    readBool(honeypotEnabled) !== baseline.honeypot.enabled ||
    honeypotNormalized !== baseline.honeypot.values
  );

  $: browserBlockNormalized = normalizeBrowserRulesForCompare(browserBlockRules);
  $: browserWhitelistNormalized = normalizeBrowserRulesForCompare(browserWhitelistRules);
  $: browserPolicyValid = browserBlockNormalized !== '__invalid__' && browserWhitelistNormalized !== '__invalid__';
  $: browserPolicyDirty = (
    browserBlockNormalized !== baseline.browserPolicy.block ||
    browserWhitelistNormalized !== baseline.browserPolicy.whitelist
  );

  $: whitelistNetworkNormalized = normalizeListTextareaForCompare(networkWhitelist);
  $: whitelistPathNormalized = normalizeListTextareaForCompare(pathWhitelist);
  $: whitelistDirty = (
    whitelistNetworkNormalized !== baseline.whitelist.network ||
    whitelistPathNormalized !== baseline.whitelist.path
  );

  $: mazeValid = inRange(mazeThreshold, 5, 500);
  $: mazeDirty = (
    readBool(mazeEnabled) !== baseline.maze.enabled ||
    readBool(mazeAutoBan) !== baseline.maze.autoBan ||
    Number(mazeThreshold) !== baseline.maze.threshold
  );

  $: cdpValid = inRange(cdpThreshold, 0.3, 1.0);
  $: cdpDirty = (
    readBool(cdpEnabled) !== baseline.cdp.enabled ||
    readBool(cdpAutoBan) !== baseline.cdp.autoBan ||
    Number(cdpThreshold) !== baseline.cdp.threshold
  );

  $: edgeModeDirty = normalizeEdgeMode(edgeIntegrationMode) !== baseline.edgeMode.mode;

  $: geoRiskNormalized = normalizeCountryCodesForCompare(geoRiskList);
  $: geoAllowNormalized = normalizeCountryCodesForCompare(geoAllowList);
  $: geoChallengeNormalized = normalizeCountryCodesForCompare(geoChallengeList);
  $: geoMazeNormalized = normalizeCountryCodesForCompare(geoMazeList);
  $: geoBlockNormalized = normalizeCountryCodesForCompare(geoBlockList);

  $: geoScoringValid = (() => {
    try {
      parseCountryCodesStrict(geoRiskList);
      return true;
    } catch (_error) {
      return false;
    }
  })();

  $: geoRoutingValid = (() => {
    try {
      parseCountryCodesStrict(geoAllowList);
      parseCountryCodesStrict(geoChallengeList);
      parseCountryCodesStrict(geoMazeList);
      parseCountryCodesStrict(geoBlockList);
      return true;
    } catch (_error) {
      return false;
    }
  })();

  $: geoScoringDirty = geoRiskNormalized !== baseline.geo.risk;
  $: geoRoutingDirty = (
    geoAllowNormalized !== baseline.geo.allow ||
    geoChallengeNormalized !== baseline.geo.challenge ||
    geoMazeNormalized !== baseline.geo.maze ||
    geoBlockNormalized !== baseline.geo.block
  );

  $: honeypotDurationSeconds = durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes);
  $: rateDurationSeconds = durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes);
  $: browserDurationSeconds = durationSeconds(durBrowserDays, durBrowserHours, durBrowserMinutes);
  $: cdpDurationSeconds = durationSeconds(durCdpDays, durCdpHours, durCdpMinutes);
  $: adminDurationSeconds = durationSeconds(durAdminDays, durAdminHours, durAdminMinutes);

  $: durationsValid = (
    isDurationTupleValid(durHoneypotDays, durHoneypotHours, durHoneypotMinutes) &&
    isDurationTupleValid(durRateLimitDays, durRateLimitHours, durRateLimitMinutes) &&
    isDurationTupleValid(durBrowserDays, durBrowserHours, durBrowserMinutes) &&
    isDurationTupleValid(durCdpDays, durCdpHours, durCdpMinutes) &&
    isDurationTupleValid(durAdminDays, durAdminHours, durAdminMinutes)
  );

  $: durationsDirty = (
    honeypotDurationSeconds !== baseline.durations.honeypot ||
    rateDurationSeconds !== baseline.durations.rateLimit ||
    browserDurationSeconds !== baseline.durations.browser ||
    cdpDurationSeconds !== baseline.durations.cdp ||
    adminDurationSeconds !== baseline.durations.admin
  );

  $: robotsValid = inRange(robotsCrawlDelay, 0, 60);
  $: robotsDirty = (
    readBool(robotsEnabled) !== baseline.robots.enabled ||
    Number(robotsCrawlDelay) !== baseline.robots.crawlDelay
  );

  $: aiPolicyDirty = (
    readBool(robotsBlockTraining) !== baseline.aiPolicy.blockTraining ||
    readBool(robotsBlockSearch) !== baseline.aiPolicy.blockSearch ||
    readBool(restrictSearchEngines) !== baseline.aiPolicy.restrictSearchEngines
  );

  $: advancedNormalized = normalizeJsonObjectForCompare(advancedConfigJson);
  $: advancedShapeValid = advancedNormalized !== null;
  $: advancedDirty = advancedShapeValid && advancedNormalized !== baseline.advanced.normalized;
  $: advancedValid = advancedShapeValid
    && !advancedValidationPending
    && advancedValidationError === ''
    && advancedValidationIssues.length === 0;
  $: advancedInvalidMessage = !advancedShapeValid
    ? 'Advanced JSON must be a valid JSON object.'
    : (advancedValidationError
      ? `Advanced JSON validation failed: ${advancedValidationError}`
      : (advancedValidationIssues.length > 0 ? 'Advanced JSON has invalid values.' : ''));

  $: {
    clearAdvancedValidationTimer();
    advancedValidationRequestId += 1;
    const requestId = advancedValidationRequestId;

    if (!writable || typeof onValidateConfig !== 'function' || !advancedDirty) {
      advancedValidationPending = false;
      advancedValidationError = '';
      advancedValidationIssues = [];
    } else if (!advancedShapeValid) {
      advancedValidationPending = false;
      advancedValidationError = '';
      advancedValidationIssues = [];
    } else {
      let advancedPatch = null;
      try {
        advancedPatch = parseAdvancedPatchObject();
      } catch (_error) {
        advancedValidationPending = false;
        advancedValidationError = '';
        advancedValidationIssues = [];
      }

      if (advancedPatch && typeof advancedPatch === 'object') {
        advancedValidationPending = true;
        advancedValidationError = '';
        advancedValidationIssues = [];
        advancedValidationTimer = setTimeout(() => {
          void runAdvancedServerValidation(advancedPatch, requestId);
        }, ADVANCED_VALIDATE_DEBOUNCE_MS);
      }
    }
  }

  $: dirtySections = [
    { label: 'JavaScript required', dirty: jsRequiredDirty, valid: true },
    { label: 'Proof of Work', dirty: powDirty, valid: powValid },
    { label: 'Challenge puzzle', dirty: challengePuzzleDirty, valid: challengePuzzleValid },
    { label: 'Not-a-Bot', dirty: notABotDirty, valid: notABotValid },
    { label: 'Internet Protocol range policy', dirty: ipRangeDirty, valid: ipRangeValid },
    { label: 'Rate limit', dirty: rateLimitDirty, valid: rateLimitValid },
    { label: 'Honeypots', dirty: honeypotDirty, valid: honeypotValid },
    { label: 'Browser policy', dirty: browserPolicyDirty, valid: browserPolicyValid },
    { label: 'Bypass allowlists', dirty: whitelistDirty, valid: true },
    { label: 'Maze', dirty: mazeDirty, valid: mazeValid },
    { label: 'Chrome DevTools Protocol', dirty: cdpDirty, valid: cdpValid },
    { label: 'Edge mode', dirty: edgeModeDirty, valid: true },
    { label: 'Geolocation scoring', dirty: geoScoringDirty, valid: geoScoringValid },
    { label: 'Geolocation routing', dirty: geoRoutingDirty, valid: geoRoutingValid },
    { label: 'Ban durations', dirty: durationsDirty, valid: durationsValid },
    { label: 'Robots serving', dirty: robotsDirty, valid: robotsValid },
    { label: 'Artificial Intelligence bot policy', dirty: aiPolicyDirty, valid: true },
    { label: 'Advanced config', dirty: advancedDirty, valid: advancedValid }
  ];
  $: dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
  $: invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
  $: dirtySectionLabels = dirtySectionEntries.map((section) => section.label);
  $: invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
  $: dirtySectionCount = dirtySectionEntries.length;
  $: hasUnsavedChanges = dirtySectionCount > 0;
  $: hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
  $: saveAllConfigDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingAll;
  $: saveAllConfigLabel = savingAll ? 'Saving...' : 'Save all changes';
  $: saveAllSummaryText = hasUnsavedChanges
    ? `${dirtySectionCount} section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
    : 'No unsaved changes';
  $: saveAllInvalidText = hasInvalidUnsavedChanges
    ? `Fix invalid values in: ${invalidDirtySectionLabels.join(', ')}`
    : '';
  $: exportConfigDisabled = !hasConfigSnapshot;
  $: warnOnUnload = writable && hasUnsavedChanges;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (skipNextConfigVersionApply) {
        skipNextConfigVersionApply = false;
      } else {
        applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
      }
    }
  }
</script>

<section
  id="dashboard-panel-config"
  class="admin-group"
  data-dashboard-tab-panel="config"
  aria-labelledby="dashboard-tab-config"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="config" status={tabStatus} />
  <p id="config-mode-subtitle" class="admin-group-subtitle text-muted">
    {#if !configLoaded}
      Admin page configuration state is LOADING.
    {:else if hasConfigSnapshot}
      {#if writable}
        Admin page configuration enabled. Saved changes persist across builds.
        Set <code class="env-var">SHUMA_ADMIN_CONFIG_WRITE_ENABLED</code> to false in deployment env to disable.
      {:else}
        Admin page configuration disabled.
        Set <code class="env-var">SHUMA_ADMIN_CONFIG_WRITE_ENABLED</code> to true to enable.
      {/if}
    {:else}
      Admin page configuration loaded, but the snapshot is empty.
    {/if}
  </p>
  <div class="controls-grid controls-grid--config">
    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
    >
      <h3>Test Mode</h3>
      <p class="control-desc text-muted">Use for safe tuning. Enabled logs all detections without blocking; disable to enforce defenses.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="test-mode-toggle">{testModeToggleText}</label>
          <label class="toggle-switch" for="test-mode-toggle">
            <input
              type="checkbox"
              id="test-mode-toggle"
              aria-label="Test mode active"
              bind:checked={testMode}
              disabled={testModeToggleDisabled}
              on:change={onTestModeToggleChange}
            >
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
      <span id="test-mode-status" class={testModeStatusClass}>{testModeStatusText}</span>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={jsRequiredDirty}
    >
      <h3><abbr title="JavaScript">JS</abbr> Required</h3>
      <p class="control-desc text-muted">Require non-allowlisted requests to present a valid <code>js_verified</code> cookie. The presence of this cookie is verification that <abbr title="JavaScript">JS</abbr> is enabled. With Shuma-Gorath&rsquo;s <abbr title="Proof of Work">PoW</abbr> requirement also enabled, the cookie is set by the server after <code>/pow/verify</code>; with <abbr title="Proof of Work">PoW</abbr> disabled, it is set directly by the interstitial script. Disable only for non-<abbr title="JavaScript">JS</abbr> clients. WARNING: disabling weakens bot defence and bypasses <abbr title="Proof of Work">PoW</abbr> on this path.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="js-required-enforced-toggle">Enforce <abbr title="JavaScript">JS</abbr> Required</label>
          <label class="toggle-switch" for="js-required-enforced-toggle">
            <input type="checkbox" id="js-required-enforced-toggle" aria-label="Enforce JavaScript required" bind:checked={jsRequiredEnforced}>
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={powDirty}
    >
      <h3>Proof-of-Work (<abbr title="Proof of Work">PoW</abbr>)</h3>
      <p class="control-desc text-muted"><abbr title="Proof of Work">PoW</abbr> is a security mechanism used to help differentiate bots from humans by requiring the requesting client's device to solve a small, moderately complex computational puzzle before being granted access. It will be invisible to human users and incurrs only extremely low energy and request performance costs. <abbr title="Proof of Work">PoW</abbr> depends on <abbr title="JavaScript">JS</abbr> Required being enabled.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="pow-enabled-toggle">Enable <abbr title="Proof of Work">PoW</abbr></label>
          <label class="toggle-switch" for="pow-enabled-toggle">
            <input type="checkbox" id="pow-enabled-toggle" aria-label="Enable Proof of Work challenge verification" bind:checked={powEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={rateLimitDirty}
    >
      <h3>Rate Limiting</h3>
      <p class="control-desc text-muted">Set allowed requests per minute. Default is <code>80</code>; lower values are stricter but can affect legitimate burst traffic.</p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label control-label--wide" for="rate-limit-threshold">Requests Per Minute</label>
          <input class="input-field" type="number" id="rate-limit-threshold" min="1" max="1000000" step="1" inputmode="numeric" aria-label="Rate limit requests per minute" bind:value={rateLimitThreshold}>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={challengePuzzleDirty}
    >
      <h3>Challenge: Puzzle</h3>
      <p class="control-desc text-muted">
        Set how many transform options are shown in puzzle challenges (higher values can increase solve time).
        <a id="preview-challenge-puzzle-link" href="/challenge/puzzle" target="_blank" rel="noopener noreferrer">Preview Puzzle</a>
        (test mode must be enabled).
      </p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="challenge-puzzle-enabled-toggle">Enable Challenge Puzzle</label>
          <label class="toggle-switch" for="challenge-puzzle-enabled-toggle">
            <input type="checkbox" id="challenge-puzzle-enabled-toggle" aria-label="Enable challenge puzzle routing" bind:checked={challengePuzzleEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label" for="challenge-puzzle-transform-count">Transform Options</label>
          <input class="input-field" type="number" id="challenge-puzzle-transform-count" min="4" max="8" step="1" inputmode="numeric" aria-label="Challenge transform option count" bind:value={challengePuzzleTransformCount}>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={notABotDirty}
    >
      <h3>Challenge: Not-a-Bot</h3>
      <p class="control-desc text-muted">This simple "I am not a bot" checkbox is the lightes weight bot verification that when enabled some visitors may encounter when the signals of them being a bot are below that required for more severe measures to be auto deployed but sufficient to warrant a challenge that will be extremely low friction for a human while imposing a cost on bots. To <a id="preview-not-a-bot-link" href="/challenge/not-a-bot-checkbox" target="_blank" rel="noopener noreferrer">preview Not-a-Bot</a>, test mode must be enabled.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="not-a-bot-enabled-toggle">Enable Not-a-Bot</label>
          <label class="toggle-switch" for="not-a-bot-enabled-toggle">
            <input type="checkbox" id="not-a-bot-enabled-toggle" aria-label="Enable not-a-bot challenge routing" bind:checked={notABotEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label" for="not-a-bot-score-pass-min">Pass Score (1-10)</label>
          <input class="input-field" type="number" id="not-a-bot-score-pass-min" min="1" max="10" step="1" inputmode="numeric" aria-label="Not-a-Bot pass score threshold" bind:value={notABotScorePassMin}>
        </div>
        <div class="input-row">
          <label class="control-label" for="not-a-bot-score-escalate-min">Escalate Score (1-10)</label>
          <input class="input-field" type="number" id="not-a-bot-score-escalate-min" min="1" max="10" step="1" inputmode="numeric" aria-label="Not-a-Bot escalation score threshold" bind:value={notABotScoreEscalateMin}>
        </div>
        <div class="input-row">
          <label class="control-label" for="not-a-bot-nonce-ttl"><abbr title="Number Used Once">Nonce</abbr> <abbr title="Time To Live">TTL</abbr> (seconds)</label>
          <input class="input-field" type="number" id="not-a-bot-nonce-ttl" min="30" max="300" step="1" inputmode="numeric" aria-label="Not-a-Bot seed number used once time to live in seconds" bind:value={notABotNonceTtl}>
        </div>
        <div class="input-row">
          <label class="control-label" for="not-a-bot-marker-ttl">Marker <abbr title="Time To Live">TTL</abbr> (seconds)</label>
          <input class="input-field" type="number" id="not-a-bot-marker-ttl" min="60" max="3600" step="1" inputmode="numeric" aria-label="Not-a-Bot marker time to live in seconds" bind:value={notABotMarkerTtl}>
        </div>
        <div class="input-row">
          <label class="control-label" for="not-a-bot-attempt-limit">Attempt Limit / Window</label>
          <input class="input-field" type="number" id="not-a-bot-attempt-limit" min="1" max="100" step="1" inputmode="numeric" aria-label="Not-a-Bot attempt limit per window" bind:value={notABotAttemptLimit}>
        </div>
        <div class="input-row">
          <label class="control-label" for="not-a-bot-attempt-window">Attempt Window (seconds)</label>
          <input class="input-field" type="number" id="not-a-bot-attempt-window" min="30" max="3600" step="1" inputmode="numeric" aria-label="Not-a-Bot attempt window in seconds" bind:value={notABotAttemptWindow}>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={ipRangeDirty}
    >
      <h3><abbr title="Internet Protocol">IP</abbr> Range Policy</h3>
      <p class="control-desc text-muted">
        Configure <abbr title="Classless Inter-Domain Routing">CIDR</abbr> policy mode, emergency allowlist, custom rules, managed set policies, and managed-catalog staleness safeguards.
      </p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label control-label--wide" for="ip-range-policy-mode">Policy Mode</label>
          <select class="input-field" id="ip-range-policy-mode" aria-label="Internet Protocol range policy mode" bind:value={ipRangePolicyMode}>
            <option value="off">off</option>
            <option value="advisory">advisory</option>
            <option value="enforce">enforce</option>
          </select>
        </div>
        <div class="geo-field">
          <label class="control-label" for="ip-range-emergency-allowlist">Emergency Allowlist <abbr title="Classless Inter-Domain Routing">CIDRs</abbr></label>
          <textarea
            class="input-field geo-textarea"
            id="ip-range-emergency-allowlist"
            rows="3"
            aria-label="Internet Protocol range emergency allowlist"
            spellcheck="false"
            bind:value={ipRangeEmergencyAllowlist}
          ></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="ip-range-custom-rules-json">Custom Rules <abbr title="JavaScript Object Notation">JSON</abbr></label>
          <textarea
            class="input-field geo-textarea input-field--mono"
            id="ip-range-custom-rules-json"
            rows="8"
            aria-label="Internet Protocol range custom rules JavaScript Object Notation"
            spellcheck="false"
            bind:value={ipRangeCustomRulesJson}
          ></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="ip-range-managed-policies-json">Managed Policies <abbr title="JavaScript Object Notation">JSON</abbr></label>
          <textarea
            class="input-field geo-textarea input-field--mono"
            id="ip-range-managed-policies-json"
            rows="6"
            aria-label="Internet Protocol range managed policies JavaScript Object Notation"
            spellcheck="false"
            bind:value={ipRangeManagedPoliciesJson}
          ></textarea>
        </div>
        <div class="input-row">
          <label class="control-label control-label--wide" for="ip-range-managed-max-staleness">
            Managed Max Staleness (hours)
          </label>
          <input
            class="input-field"
            type="number"
            id="ip-range-managed-max-staleness"
            min={IP_RANGE_MANAGED_STALENESS_MIN}
            max={IP_RANGE_MANAGED_STALENESS_MAX}
            step="1"
            inputmode="numeric"
            aria-label="Internet Protocol range managed max staleness hours"
            bind:value={ipRangeManagedMaxStalenessHours}
          >
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="ip-range-allow-stale-enforce">
            Allow stale managed enforce
          </label>
          <label class="toggle-switch" for="ip-range-allow-stale-enforce">
            <input
              type="checkbox"
              id="ip-range-allow-stale-enforce"
              aria-label="Allow stale managed enforce"
              bind:checked={ipRangeAllowStaleManagedEnforce}
            >
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
      <div class="info-panel panel-soft pad-sm">
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
      {#if ipRangeCatalogStale}
        <p class="message warning">Managed catalog is stale under current max staleness policy.</p>
      {/if}
      {#if ipRangeManagedSetRows.length > 0}
        <div class="table-wrapper">
          <table id="ip-range-config-managed-sets-table">
            <thead>
              <tr>
                <th>Set</th>
                <th>Provider</th>
                <th>Version</th>
                <th>Entries</th>
                <th>Stale</th>
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
        </div>
      {/if}
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={honeypotDirty}
    >
      <h3>Honeypot Paths</h3>
      <p class="control-desc text-muted">One path per line. Requests that hit these paths are treated as high-confidence bot behavior. Paths must start with <code>/</code>.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="honeypot-enabled-toggle">Enable Honeypot</label>
          <label class="toggle-switch" for="honeypot-enabled-toggle">
            <input type="checkbox" id="honeypot-enabled-toggle" aria-label="Enable honeypot" bind:checked={honeypotEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="geo-field">
          <label class="control-label" for="honeypot-paths">Paths</label>
          <textarea class="input-field geo-textarea" id="honeypot-paths" rows="3" aria-label="Honeypot paths" spellcheck="false" bind:value={honeypotPaths}></textarea>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={browserPolicyDirty}
    >
      <h3>Browser Policy</h3>
      <p class="control-desc text-muted">Use one rule per line in <code>BrowserName,min_major</code> format (for example <code>Chrome,120</code>).</p>
      <div class="admin-controls">
        <div class="geo-field">
          <label class="control-label" for="browser-block-rules">Minimum Versions (Block)</label>
          <textarea class="input-field geo-textarea" id="browser-block-rules" rows="3" aria-label="Browser block minimum versions" spellcheck="false" bind:value={browserBlockRules}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="browser-whitelist-rules">Allowlist Exceptions</label>
          <textarea class="input-field geo-textarea" id="browser-whitelist-rules" rows="2" aria-label="Browser allowlist exceptions" spellcheck="false" bind:value={browserWhitelistRules}></textarea>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={whitelistDirty}
    >
      <h3>Bypass Allowlists</h3>
      <p class="control-desc text-muted">Define trusted bypass entries. Use one entry per line.</p>
      <div class="admin-controls">
        <div class="geo-field">
          <label class="control-label" for="network-whitelist"><abbr title="Internet Protocol">IP</abbr>/<abbr title="Classless Inter-Domain Routing">CIDR</abbr> Allowlist</label>
          <textarea class="input-field geo-textarea" id="network-whitelist" rows="3" aria-label="Internet Protocol and Classless Inter-Domain Routing allowlist" spellcheck="false" bind:value={networkWhitelist}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="path-whitelist">Path Allowlist</label>
          <textarea class="input-field geo-textarea" id="path-whitelist" rows="3" aria-label="Path allowlist" spellcheck="false" bind:value={pathWhitelist}></textarea>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={mazeDirty}
    >
      <h3>Maze</h3>
      <p class="control-desc text-muted">
        Control trap-page routing and optional auto-ban. Lower thresholds ban faster but may increase false positives.
        <a id="preview-maze-link" href="/admin/maze/preview" target="_blank" rel="noopener noreferrer">Preview Maze</a>
        in a non-operational view (admin session required).
      </p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label" for="maze-enabled-toggle">Maze Enabled</label>
          <label class="toggle-switch">
            <input type="checkbox" id="maze-enabled-toggle" aria-label="Enable maze" bind:checked={mazeEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="toggle-row">
          <label class="control-label" for="maze-auto-ban-toggle">Ban on entry</label>
          <label class="toggle-switch">
            <input type="checkbox" id="maze-auto-ban-toggle" aria-label="Enable maze ban on entry" bind:checked={mazeAutoBan}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label" for="maze-threshold">Ban Threshold (pages)</label>
          <input class="input-field" type="number" id="maze-threshold" min="5" max="500" step="1" inputmode="numeric" aria-label="Maze ban threshold in pages" bind:value={mazeThreshold}>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={cdpDirty}
    >
      <h3><abbr title="Chrome DevTools Protocol">CDP</abbr> (Detect Browser Automation)</h3>
      <p class="control-desc text-muted">Control automation-signal detection and optional auto-ban. Stricter thresholds catch more bots but may increase false positives.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="cdp-enabled-toggle">Enable Detection</label>
          <label class="toggle-switch">
            <input type="checkbox" id="cdp-enabled-toggle" aria-label="Enable Chrome DevTools Protocol detection" bind:checked={cdpEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="cdp-auto-ban-toggle">Auto-ban on Detection</label>
          <label class="toggle-switch">
            <input type="checkbox" id="cdp-auto-ban-toggle" aria-label="Enable Chrome DevTools Protocol auto-ban" bind:checked={cdpAutoBan}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="slider-control">
          <div class="slider-header">
            <label class="control-label control-label--wide" for="cdp-threshold-slider">Detection Threshold</label>
            <span id="cdp-threshold-value" class="slider-badge">{Number(cdpThreshold).toFixed(1)}</span>
          </div>
          <input type="range" id="cdp-threshold-slider" min="0.3" max="1.0" step="0.1" aria-label="Chrome DevTools Protocol detection threshold" bind:value={cdpThreshold}>
          <div class="slider-labels">
            <span>Strict</span>
            <span>Permissive</span>
          </div>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={edgeModeDirty}
    >
      <h3>Edge Integration Mode</h3>
      <p class="control-desc text-muted">Control how external edge bot outcomes influence local policy: off ignores edge outcomes, advisory records them without direct enforcement, authoritative allows strong edge outcomes to short-circuit.</p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label control-label--wide" for="edge-integration-mode-select">Mode</label>
          <select class="input-field" id="edge-integration-mode-select" aria-label="Edge integration mode" bind:value={edgeIntegrationMode}>
            <option value="off">off</option>
            <option value="advisory">advisory</option>
            <option value="authoritative">authoritative</option>
          </select>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={geoScoringDirty}
    >
      <h3><abbr title="Geolocation">GEO</abbr> Risk Based Scoring</h3>
      <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be receive added botness risk to contribute to the combined score.</p>
      <div class="admin-controls geo-controls">
        <div class="geo-field">
          <label class="control-label" for="geo-risk-list">Scoring Countries</label>
          <textarea class="input-field geo-textarea" id="geo-risk-list" rows="1" aria-label="Geolocation scoring countries" spellcheck="false" bind:value={geoRiskList}></textarea>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={geoRoutingDirty}
    >
      <h3><abbr title="Geolocation">GEO</abbr> Risk Based Routing</h3>
      <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be automatically routed. Precedence is Block &gt; Maze &gt; Challenge &gt; Allow.</p>
      <div class="admin-controls geo-controls">
        <div class="geo-field">
          <label class="control-label" for="geo-block-list">Block Countries</label>
          <textarea class="input-field geo-textarea" id="geo-block-list" rows="1" aria-label="Geolocation block countries" spellcheck="false" bind:value={geoBlockList}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="geo-maze-list">Maze Countries</label>
          <textarea class="input-field geo-textarea" id="geo-maze-list" rows="1" aria-label="Geolocation maze countries" spellcheck="false" bind:value={geoMazeList}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="geo-challenge-list">Challenge Countries</label>
          <textarea class="input-field geo-textarea" id="geo-challenge-list" rows="1" aria-label="Geolocation challenge countries" spellcheck="false" bind:value={geoChallengeList}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="geo-allow-list">Allow Countries</label>
          <textarea class="input-field geo-textarea" id="geo-allow-list" rows="1" aria-label="Geolocation allow countries" spellcheck="false" bind:value={geoAllowList}></textarea>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={durationsDirty}
    >
      <h3>Ban Durations</h3>
      <p class="control-desc text-muted">Set ban length in days, hours, and minutes per trigger type. Longer bans increase deterrence but slow recovery from false positives.</p>
      <div class="duration-grid">
        <div class="duration-row">
          <label class="control-label" for="dur-honeypot-days">Maze Threshold Exceeded</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-honeypot-days">
              <input id="dur-honeypot-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durHoneypotDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-honeypot-hours">
              <input id="dur-honeypot-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durHoneypotHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-honeypot-minutes">
              <input id="dur-honeypot-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durHoneypotMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-rate-limit-days">Rate Limit Exceeded</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-rate-limit-days">
              <input id="dur-rate-limit-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durRateLimitDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-rate-limit-hours">
              <input id="dur-rate-limit-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durRateLimitHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-rate-limit-minutes">
              <input id="dur-rate-limit-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durRateLimitMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-browser-days">Browser Automation Detected</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-browser-days">
              <input id="dur-browser-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durBrowserDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-browser-hours">
              <input id="dur-browser-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durBrowserHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-browser-minutes">
              <input id="dur-browser-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durBrowserMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-cdp-days"><abbr title="Chrome DevTools Protocol">CDP</abbr> Automation Detected</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-cdp-days">
              <input id="dur-cdp-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durCdpDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-cdp-hours">
              <input id="dur-cdp-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durCdpHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-cdp-minutes">
              <input id="dur-cdp-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durCdpMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-admin-days">Admin Manual Ban Default</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-admin-days">
              <input id="dur-admin-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durAdminDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-admin-hours">
              <input id="dur-admin-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durAdminHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-admin-minutes">
              <input id="dur-admin-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durAdminMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={robotsDirty || aiPolicyDirty}
    >
      <h3>Robots and <abbr title="Artificial Intelligence">AI</abbr> Bot Policy</h3>
      <p class="control-desc text-muted">Keep robots.txt serving controls separate from <abbr title="Artificial Intelligence">AI</abbr> bot policy controls.</p>
      <div class="admin-controls">
        <h4 class="control-subtitle">robots.txt Serving</h4>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-enabled-toggle">Serve robots.txt</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-enabled-toggle" aria-label="Serve robots.txt" bind:checked={robotsEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label control-label--wide" for="robots-crawl-delay">Crawl Delay (seconds)</label>
          <input class="input-field" type="number" id="robots-crawl-delay" min="0" max="60" step="1" inputmode="numeric" aria-label="Robots crawl delay in seconds" bind:value={robotsCrawlDelay}>
        </div>
        <h4 class="control-subtitle"><abbr title="Artificial Intelligence">AI</abbr> Bot Policy</h4>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-block-training-toggle">Opt-out <abbr title="Artificial Intelligence">AI</abbr> Training</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-block-training-toggle" aria-label="Opt-out artificial intelligence training" bind:checked={robotsBlockTraining}>
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-hint">GPTBot, CCBot, ClaudeBot</span>
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-block-search-toggle">Opt-out <abbr title="Artificial Intelligence">AI</abbr> Search</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-block-search-toggle" aria-label="Opt-out artificial intelligence search" bind:checked={robotsBlockSearch}>
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-hint">PerplexityBot, etc.</span>
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-allow-search-toggle">Restrict Search Engines</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-allow-search-toggle" aria-label="Restrict search engines" bind:checked={restrictSearchEngines}>
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-hint">Google, Bing, etc.</span>
        </div>
      </div>
      <button id="preview-robots" class="btn btn-subtle" on:click={toggleRobotsPreview}>{robotsPreviewOpen ? 'Hide robots.txt' : 'Show robots.txt'}</button>
      <div id="robots-preview" class="robots-preview panel pad-sm" class:hidden={!robotsPreviewOpen}>
        <h4>robots.txt Preview</h4>
        <pre id="robots-preview-content">{robotsPreviewLoading ? 'Loading...' : robotsPreviewContent}</pre>
      </div>
    </div>

    <div
      class="control-group panel-soft pad-md config-export-pane"
      class:hidden={!writable}
    >
      <button
        id="export-current-config-json"
        class="btn btn-subtle"
        disabled={exportConfigDisabled}
        on:click={exportCurrentConfigJson}
      >Export the current configuration as JSON</button>
      {#if exportConfigStatus}
        <p id="export-current-config-status" class={`message ${exportConfigStatusKind}`}>{exportConfigStatus}</p>
      {/if}
    </div>

    <div
      class="control-group panel-soft pad-md config-edit-pane"
      class:hidden={!writable}
      class:config-edit-pane--dirty={advancedDirty}
    >
      <h3>Advanced Config <abbr title="JavaScript Object Notation">JSON</abbr></h3>
      <p class="control-desc text-muted">Directly edit writable config keys as a <abbr title="JavaScript Object Notation">JSON</abbr> object. This editor reflects the last loaded snapshot and does not auto-sync while you change controls above.</p>
      <div class="admin-controls">
        <div class="geo-field">
          <label class="control-label" for="advanced-config-json"><abbr title="JavaScript Object Notation">JSON</abbr> Patch</label>
          <textarea
            class="input-field geo-textarea"
            id="advanced-config-json"
            rows="8"
            aria-label="Advanced config JavaScript Object Notation patch"
            aria-invalid={advancedValid ? 'false' : 'true'}
            spellcheck="false"
            bind:value={advancedConfigJson}
          ></textarea>
        </div>
        {#if advancedValidationPending}
          <p id="advanced-config-json-validating" class="text-muted">Validating Advanced <abbr title="JavaScript Object Notation">JSON</abbr>...</p>
        {/if}
        {#if advancedInvalidMessage}
          <div id="advanced-config-json-error" class="message error">
            <p>{advancedInvalidMessage}</p>
            {#if advancedValidationIssues.length > 0}
              <ul id="advanced-config-json-issue-list" class="validation-issue-list">
                {#each advancedValidationIssues as issue, issueIndex}
                  <li id={`advanced-config-json-issue-${issueIndex}`}>
                    {#if issue.field}
                      <code>{issue.field}</code>:&nbsp;
                    {/if}
                    {issue.message}
                    {#if issue.expected}
                      <span class="validation-issue-expected">Expected: {issue.expected}</span>
                    {/if}
                    {#if issue.received !== undefined}
                      <span class="validation-issue-received">Received: <code>{formatIssueReceived(issue.received)}</code></span>
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}
            <a
              id="advanced-config-json-docs-link"
              href="https://github.com/atomless/Shuma-Gorath/blob/main/docs/configuration.md"
              target="_blank"
              rel="noopener noreferrer"
            >Configuration docs</a>
          </div>
        {/if}
      </div>
    </div>

    <div
      id="config-save-all-bar"
      class="config-save-bar panel panel-border"
      class:hidden={!writable || !hasUnsavedChanges}
    >
      <div class="config-save-bar__meta">
        <span id="config-unsaved-summary" class="text-unsaved-changes">{saveAllSummaryText}</span>
        {#if saveAllInvalidText}
          <span id="config-invalid-summary" class="config-save-bar__warning">{saveAllInvalidText}</span>
        {/if}
        <button id="save-config-all" class="btn btn-submit" disabled={saveAllConfigDisabled} on:click={saveAllConfig}>
          {saveAllConfigLabel}
        </button>
      </div>
    </div>
  </div>
</section>
