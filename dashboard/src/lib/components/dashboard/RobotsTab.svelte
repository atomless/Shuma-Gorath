<script>
  import { onDestroy, onMount } from 'svelte';
  import {
    formatBrowserRulesTextarea,
    formatListTextarea,
    normalizeListTextareaForCompare,
    parseListTextarea,
    normalizeBrowserRulesForCompare,
    parseBrowserRulesTextarea
  } from '../../domain/config-form-utils.js';
  import { durationPartsFromSeconds, durationSeconds } from '../../domain/core/date-time.js';
  import { parseInteger } from '../../domain/core/math.js';
  import { inRange, isDurationTupleValid } from '../../domain/core/validation.js';
  import { isAdminConfigWritable } from '../../domain/config-runtime.js';
  import ConfigDurationsSection from './config/ConfigDurationsSection.svelte';
  import ConfigNetworkSection from './config/ConfigNetworkSection.svelte';
  import ConfigPathAllowlistSection from './config/ConfigPathAllowlistSection.svelte';
  import ConfigRobotsSection from './config/ConfigRobotsSection.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let onFetchRobotsPreview = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  const ROBOTS_PREVIEW_AUTO_REFRESH_DEBOUNCE_MS = 120;
  const MAX_DURATION_SECONDS = 31536000;
  const MIN_DURATION_SECONDS = 60;
  const DURATION_VALIDATION_BOUNDS = Object.freeze({
    minSeconds: MIN_DURATION_SECONDS,
    maxSeconds: MAX_DURATION_SECONDS
  });

  let writable = false;
  let savingPolicy = false;
  let warnOnUnload = false;
  let lastAppliedConfigVersion = -1;

  let robotsEnabled = true;
  let robotsCrawlDelay = 2;
  let robotsBlockTraining = true;
  let robotsBlockSearch = false;
  let restrictSearchEngines = false;

  let durHoneypotDays = 1;
  let durHoneypotHours = 0;
  let durHoneypotMinutes = 0;
  let durIpRangeHoneypotDays = 1;
  let durIpRangeHoneypotHours = 0;
  let durIpRangeHoneypotMinutes = 0;
  let durMazeCrawlerDays = 1;
  let durMazeCrawlerHours = 0;
  let durMazeCrawlerMinutes = 0;
  let durRateLimitDays = 0;
  let durRateLimitHours = 1;
  let durRateLimitMinutes = 0;
  let durCdpDays = 0;
  let durCdpHours = 12;
  let durCdpMinutes = 0;
  let durEdgeFingerprintDays = 0;
  let durEdgeFingerprintHours = 12;
  let durEdgeFingerprintMinutes = 0;
  let durTarpitPersistenceDays = 0;
  let durTarpitPersistenceHours = 0;
  let durTarpitPersistenceMinutes = 10;
  let durNotABotAbuseDays = 0;
  let durNotABotAbuseHours = 0;
  let durNotABotAbuseMinutes = 10;
  let durChallengePuzzleAbuseDays = 0;
  let durChallengePuzzleAbuseHours = 0;
  let durChallengePuzzleAbuseMinutes = 10;
  let durAdminDays = 0;
  let durAdminHours = 6;
  let durAdminMinutes = 0;

  let browserPolicyEnabled = true;
  let browserBlockRules = '';
  let pathAllowlistEnabled = true;
  let pathAllowlist = '';

  let robotsPreviewOpen = false;
  let robotsPreviewLoading = false;
  let robotsPreviewContent = '';
  let robotsPreviewPatchKey = '';
  let robotsPreviewRequestId = 0;
  let robotsPreviewAutoRefreshTimer = null;
  let robotsCrawlDelayValid = true;
  let robotsValid = true;
  let robotsDirty = false;
  let aiPolicyDirty = false;
  let honeypotDurationSeconds = 86400;
  let ipRangeHoneypotDurationSeconds = 86400;
  let mazeCrawlerDurationSeconds = 86400;
  let rateDurationSeconds = 3600;
  let cdpDurationSeconds = 43200;
  let edgeFingerprintDurationSeconds = 43200;
  let tarpitPersistenceDurationSeconds = 600;
  let notABotAbuseDurationSeconds = 600;
  let challengePuzzleAbuseDurationSeconds = 600;
  let adminDurationSeconds = 21600;
  let durHoneypotValid = true;
  let durIpRangeHoneypotValid = true;
  let durMazeCrawlerValid = true;
  let durRateLimitValid = true;
  let durCdpValid = true;
  let durEdgeFingerprintValid = true;
  let durTarpitPersistenceValid = true;
  let durNotABotAbuseValid = true;
  let durChallengePuzzleAbuseValid = true;
  let durAdminValid = true;
  let durationsValid = true;
  let durationsDirty = false;
  let browserBlockNormalized = '';
  let browserBlockRulesValid = true;
  let browserPolicyValid = true;
  let browserPolicyDirty = false;
  let pathAllowlistNormalized = '';
  let pathAllowlistDirty = false;
  let dirtySections = [];
  let dirtySectionEntries = [];
  let invalidDirtySectionEntries = [];
  let invalidDirtySectionLabels = [];
  let dirtySectionCount = 0;
  let hasUnsavedChanges = false;
  let hasInvalidUnsavedChanges = false;
  let savePolicyDisabled = true;
  let savePolicyLabel = 'Save policy changes';
  let savePolicySummary = 'No unsaved changes';
  let savePolicyInvalidText = '';

  let baseline = {
    robotsPolicy: {
      enabled: true,
      crawlDelay: 2,
      blockTraining: true,
      blockSearch: false,
      restrictSearchEngines: false
    },
    durations: {
      honeypot: 86400,
      ipRangeHoneypot: 86400,
      mazeCrawler: 86400,
      rateLimit: 3600,
      cdp: 43200,
      edgeFingerprint: 43200,
      tarpitPersistence: 600,
      notABotAbuse: 600,
      challengePuzzleAbuse: 600,
      admin: 21600
    },
    browserPolicy: {
      enabled: true,
      block: ''
    },
    pathAllowlist: {
      enabled: true,
      entries: ''
    }
  };

  const readBool = (value) => value === true;

  const toDurationBaseline = (config = {}) => {
    const durations = config && typeof config.ban_durations === 'object'
      ? config.ban_durations
      : {};
    const readDuration = (key, fallbackSeconds) => parseInteger(durations[key], fallbackSeconds);
    return {
      honeypot: readDuration('honeypot', 86400),
      ipRangeHoneypot: readDuration('ip_range_honeypot', 86400),
      mazeCrawler: readDuration('maze_crawler', 86400),
      rateLimit: readDuration('rate_limit', 3600),
      cdp: readDuration('cdp', 43200),
      edgeFingerprint: readDuration('edge_fingerprint', 43200),
      tarpitPersistence: readDuration('tarpit_persistence', 600),
      notABotAbuse: readDuration('not_a_bot_abuse', 600),
      challengePuzzleAbuse: readDuration('challenge_puzzle_abuse', 600),
      admin: readDuration('admin', 21600)
    };
  };

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  const clearRobotsPreviewAutoRefreshTimer = () => {
    if (robotsPreviewAutoRefreshTimer) {
      clearTimeout(robotsPreviewAutoRefreshTimer);
      robotsPreviewAutoRefreshTimer = null;
    }
  };

  const buildRobotsPreviewPatch = () => {
    const crawlDelayValue = Number(robotsCrawlDelay);
    return {
      robots_enabled: robotsEnabled === true,
      robots_crawl_delay: Number.isFinite(crawlDelayValue) ? Math.max(0, Math.floor(crawlDelayValue)) : 0,
      ai_policy_block_training: robotsBlockTraining === true,
      ai_policy_block_search: robotsBlockSearch === true,
      ai_policy_allow_search_engines: restrictSearchEngines !== true
    };
  };

  const applyPolicyFieldChange = (field, value) => {
    switch (field) {
      case 'robotsEnabled':
        robotsEnabled = value === true;
        break;
      case 'robotsCrawlDelay':
        robotsCrawlDelay = value;
        break;
      case 'robotsBlockTraining':
        robotsBlockTraining = value === true;
        break;
      case 'robotsBlockSearch':
        robotsBlockSearch = value === true;
        break;
      case 'restrictSearchEngines':
        restrictSearchEngines = value === true;
        break;
      case 'durHoneypotDays':
        durHoneypotDays = value;
        break;
      case 'durHoneypotHours':
        durHoneypotHours = value;
        break;
      case 'durHoneypotMinutes':
        durHoneypotMinutes = value;
        break;
      case 'durIpRangeHoneypotDays':
        durIpRangeHoneypotDays = value;
        break;
      case 'durIpRangeHoneypotHours':
        durIpRangeHoneypotHours = value;
        break;
      case 'durIpRangeHoneypotMinutes':
        durIpRangeHoneypotMinutes = value;
        break;
      case 'durMazeCrawlerDays':
        durMazeCrawlerDays = value;
        break;
      case 'durMazeCrawlerHours':
        durMazeCrawlerHours = value;
        break;
      case 'durMazeCrawlerMinutes':
        durMazeCrawlerMinutes = value;
        break;
      case 'durRateLimitDays':
        durRateLimitDays = value;
        break;
      case 'durRateLimitHours':
        durRateLimitHours = value;
        break;
      case 'durRateLimitMinutes':
        durRateLimitMinutes = value;
        break;
      case 'durCdpDays':
        durCdpDays = value;
        break;
      case 'durCdpHours':
        durCdpHours = value;
        break;
      case 'durCdpMinutes':
        durCdpMinutes = value;
        break;
      case 'durEdgeFingerprintDays':
        durEdgeFingerprintDays = value;
        break;
      case 'durEdgeFingerprintHours':
        durEdgeFingerprintHours = value;
        break;
      case 'durEdgeFingerprintMinutes':
        durEdgeFingerprintMinutes = value;
        break;
      case 'durTarpitPersistenceDays':
        durTarpitPersistenceDays = value;
        break;
      case 'durTarpitPersistenceHours':
        durTarpitPersistenceHours = value;
        break;
      case 'durTarpitPersistenceMinutes':
        durTarpitPersistenceMinutes = value;
        break;
      case 'durNotABotAbuseDays':
        durNotABotAbuseDays = value;
        break;
      case 'durNotABotAbuseHours':
        durNotABotAbuseHours = value;
        break;
      case 'durNotABotAbuseMinutes':
        durNotABotAbuseMinutes = value;
        break;
      case 'durChallengePuzzleAbuseDays':
        durChallengePuzzleAbuseDays = value;
        break;
      case 'durChallengePuzzleAbuseHours':
        durChallengePuzzleAbuseHours = value;
        break;
      case 'durChallengePuzzleAbuseMinutes':
        durChallengePuzzleAbuseMinutes = value;
        break;
      case 'durAdminDays':
        durAdminDays = value;
        break;
      case 'durAdminHours':
        durAdminHours = value;
        break;
      case 'durAdminMinutes':
        durAdminMinutes = value;
        break;
      case 'browserPolicyEnabled':
        browserPolicyEnabled = value === true;
        break;
      case 'browserBlockRules':
        browserBlockRules = String(value || '');
        break;
      case 'pathAllowlistEnabled':
        pathAllowlistEnabled = value === true;
        break;
      case 'pathAllowlist':
        pathAllowlist = String(value || '');
        break;
      default:
        break;
    }
  };

  const applyConfig = (config = {}) => {
    robotsEnabled = config.robots_enabled !== false;
    robotsCrawlDelay = parseInteger(config.robots_crawl_delay, 2);
    robotsBlockTraining = config.ai_policy_block_training !== false;
    robotsBlockSearch = config.ai_policy_block_search === true;
    const aiAllowSearchEngines = config.ai_policy_allow_search_engines;
    restrictSearchEngines = aiAllowSearchEngines === undefined ? false : aiAllowSearchEngines !== true;

    const durations = toDurationBaseline(config);
    const honeypotParts = durationPartsFromSeconds(durations.honeypot, 86400);
    durHoneypotDays = honeypotParts.days;
    durHoneypotHours = honeypotParts.hours;
    durHoneypotMinutes = honeypotParts.minutes;

    const ipRangeHoneypotParts = durationPartsFromSeconds(durations.ipRangeHoneypot, 86400);
    durIpRangeHoneypotDays = ipRangeHoneypotParts.days;
    durIpRangeHoneypotHours = ipRangeHoneypotParts.hours;
    durIpRangeHoneypotMinutes = ipRangeHoneypotParts.minutes;

    const mazeCrawlerParts = durationPartsFromSeconds(durations.mazeCrawler, 86400);
    durMazeCrawlerDays = mazeCrawlerParts.days;
    durMazeCrawlerHours = mazeCrawlerParts.hours;
    durMazeCrawlerMinutes = mazeCrawlerParts.minutes;

    const rateLimitParts = durationPartsFromSeconds(durations.rateLimit, 3600);
    durRateLimitDays = rateLimitParts.days;
    durRateLimitHours = rateLimitParts.hours;
    durRateLimitMinutes = rateLimitParts.minutes;

    const cdpParts = durationPartsFromSeconds(durations.cdp, 43200);
    durCdpDays = cdpParts.days;
    durCdpHours = cdpParts.hours;
    durCdpMinutes = cdpParts.minutes;

    const edgeFingerprintParts = durationPartsFromSeconds(durations.edgeFingerprint, 43200);
    durEdgeFingerprintDays = edgeFingerprintParts.days;
    durEdgeFingerprintHours = edgeFingerprintParts.hours;
    durEdgeFingerprintMinutes = edgeFingerprintParts.minutes;

    const tarpitPersistenceParts = durationPartsFromSeconds(durations.tarpitPersistence, 600);
    durTarpitPersistenceDays = tarpitPersistenceParts.days;
    durTarpitPersistenceHours = tarpitPersistenceParts.hours;
    durTarpitPersistenceMinutes = tarpitPersistenceParts.minutes;

    const notABotAbuseParts = durationPartsFromSeconds(durations.notABotAbuse, 600);
    durNotABotAbuseDays = notABotAbuseParts.days;
    durNotABotAbuseHours = notABotAbuseParts.hours;
    durNotABotAbuseMinutes = notABotAbuseParts.minutes;

    const challengePuzzleAbuseParts = durationPartsFromSeconds(durations.challengePuzzleAbuse, 600);
    durChallengePuzzleAbuseDays = challengePuzzleAbuseParts.days;
    durChallengePuzzleAbuseHours = challengePuzzleAbuseParts.hours;
    durChallengePuzzleAbuseMinutes = challengePuzzleAbuseParts.minutes;

    const adminParts = durationPartsFromSeconds(durations.admin, 21600);
    durAdminDays = adminParts.days;
    durAdminHours = adminParts.hours;
    durAdminMinutes = adminParts.minutes;

    browserPolicyEnabled = config.browser_policy_enabled !== false;
    browserBlockRules = formatBrowserRulesTextarea(config.browser_block);
    pathAllowlistEnabled = config.path_allowlist_enabled !== false;
    pathAllowlist = formatListTextarea(config.path_allowlist);

    baseline = {
      robotsPolicy: {
        enabled: robotsEnabled === true,
        crawlDelay: Number(robotsCrawlDelay),
        blockTraining: robotsBlockTraining === true,
        blockSearch: robotsBlockSearch === true,
        restrictSearchEngines: restrictSearchEngines === true
      },
      durations,
      browserPolicy: {
        enabled: browserPolicyEnabled === true,
        block: normalizeBrowserRulesForCompare(formatBrowserRulesTextarea(config.browser_block))
      },
      pathAllowlist: {
        enabled: pathAllowlistEnabled === true,
        entries: normalizeListTextareaForCompare(formatListTextarea(config.path_allowlist))
      }
    };
  };

  async function savePolicyConfig() {
    if (savePolicyDisabled || typeof onSaveConfig !== 'function') return;
    savingPolicy = true;
    const payload = {};

    if (robotsDirty || aiPolicyDirty) {
      payload.robots_enabled = robotsEnabled === true;
      payload.robots_crawl_delay = Number(robotsCrawlDelay);
      payload.ai_policy_block_training = robotsBlockTraining === true;
      payload.ai_policy_block_search = robotsBlockSearch === true;
      payload.ai_policy_allow_search_engines = !restrictSearchEngines;
    }
    if (durationsDirty) {
      payload.ban_durations = {
        honeypot: honeypotDurationSeconds,
        ip_range_honeypot: ipRangeHoneypotDurationSeconds,
        maze_crawler: mazeCrawlerDurationSeconds,
        rate_limit: rateDurationSeconds,
        cdp: cdpDurationSeconds,
        edge_fingerprint: edgeFingerprintDurationSeconds,
        tarpit_persistence: tarpitPersistenceDurationSeconds,
        not_a_bot_abuse: notABotAbuseDurationSeconds,
        challenge_puzzle_abuse: challengePuzzleAbuseDurationSeconds,
        admin: adminDurationSeconds
      };
    }
    if (browserPolicyDirty) {
      payload.browser_policy_enabled = browserPolicyEnabled;
      payload.browser_block = parseBrowserRulesTextarea(browserBlockRules);
    }
    if (pathAllowlistDirty) {
      payload.path_allowlist_enabled = pathAllowlistEnabled === true;
      payload.path_allowlist = parseListTextarea(pathAllowlist);
    }

    try {
      const nextConfig = await onSaveConfig(payload, { successMessage: 'Policy settings saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      } else {
        baseline = {
          robotsPolicy: {
            enabled: robotsEnabled === true,
            crawlDelay: Number(robotsCrawlDelay),
            blockTraining: robotsBlockTraining === true,
            blockSearch: robotsBlockSearch === true,
            restrictSearchEngines: restrictSearchEngines === true
          },
          durations: {
            honeypot: honeypotDurationSeconds,
            ipRangeHoneypot: ipRangeHoneypotDurationSeconds,
            mazeCrawler: mazeCrawlerDurationSeconds,
            rateLimit: rateDurationSeconds,
            cdp: cdpDurationSeconds,
            edgeFingerprint: edgeFingerprintDurationSeconds,
            tarpitPersistence: tarpitPersistenceDurationSeconds,
            notABotAbuse: notABotAbuseDurationSeconds,
            challengePuzzleAbuse: challengePuzzleAbuseDurationSeconds,
            admin: adminDurationSeconds
          },
          browserPolicy: {
            enabled: browserPolicyEnabled === true,
            block: normalizeBrowserRulesForCompare(browserBlockRules)
          },
          pathAllowlist: {
            enabled: pathAllowlistEnabled === true,
            entries: normalizeListTextareaForCompare(pathAllowlist)
          }
        };
      }
    } finally {
      savingPolicy = false;
    }
  }

  async function refreshRobotsPreview(previewPatch = null) {
    if (typeof onFetchRobotsPreview !== 'function') return;
    const patch = previewPatch && typeof previewPatch === 'object'
      ? previewPatch
      : buildRobotsPreviewPatch();
    const requestId = ++robotsPreviewRequestId;
    robotsPreviewLoading = true;
    try {
      const payload = await onFetchRobotsPreview(patch);
      if (requestId !== robotsPreviewRequestId) return;
      robotsPreviewContent = payload && typeof payload.content === 'string'
        ? payload.content
        : '# No preview available';
    } catch (error) {
      if (requestId !== robotsPreviewRequestId) return;
      robotsPreviewContent = `# Error loading preview: ${error && error.message ? error.message : 'Unknown error'}`;
    } finally {
      if (requestId === robotsPreviewRequestId) {
        robotsPreviewLoading = false;
      }
    }
  }

  function scheduleRobotsPreviewRefresh() {
    clearRobotsPreviewAutoRefreshTimer();
    robotsPreviewAutoRefreshTimer = setTimeout(() => {
      void refreshRobotsPreview();
    }, ROBOTS_PREVIEW_AUTO_REFRESH_DEBOUNCE_MS);
  }

  function onRobotsPreviewControlChanged() {
    if (!robotsPreviewOpen) return;
    scheduleRobotsPreviewRefresh();
  }

  function toggleRobotsPreview() {
    if (robotsPreviewOpen) {
      clearRobotsPreviewAutoRefreshTimer();
      robotsPreviewRequestId += 1;
      robotsPreviewOpen = false;
      robotsPreviewLoading = false;
      return;
    }
    robotsPreviewOpen = true;
    scheduleRobotsPreviewRefresh();
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  onDestroy(() => {
    clearRobotsPreviewAutoRefreshTimer();
  });

  $: {
    writable = isAdminConfigWritable(configRuntimeSnapshot);
    robotsCrawlDelayValid = inRange(robotsCrawlDelay, 0, 60);
    robotsValid = robotsCrawlDelayValid;
    robotsDirty = (
      readBool(robotsEnabled) !== baseline.robotsPolicy.enabled ||
      Number(robotsCrawlDelay) !== baseline.robotsPolicy.crawlDelay
    );
    aiPolicyDirty = (
      readBool(robotsBlockTraining) !== baseline.robotsPolicy.blockTraining ||
      readBool(robotsBlockSearch) !== baseline.robotsPolicy.blockSearch ||
      readBool(restrictSearchEngines) !== baseline.robotsPolicy.restrictSearchEngines
    );

    honeypotDurationSeconds = durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes);
    ipRangeHoneypotDurationSeconds = durationSeconds(
      durIpRangeHoneypotDays,
      durIpRangeHoneypotHours,
      durIpRangeHoneypotMinutes
    );
    mazeCrawlerDurationSeconds = durationSeconds(
      durMazeCrawlerDays,
      durMazeCrawlerHours,
      durMazeCrawlerMinutes
    );
    rateDurationSeconds = durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes);
    cdpDurationSeconds = durationSeconds(durCdpDays, durCdpHours, durCdpMinutes);
    edgeFingerprintDurationSeconds = durationSeconds(
      durEdgeFingerprintDays,
      durEdgeFingerprintHours,
      durEdgeFingerprintMinutes
    );
    tarpitPersistenceDurationSeconds = durationSeconds(
      durTarpitPersistenceDays,
      durTarpitPersistenceHours,
      durTarpitPersistenceMinutes
    );
    notABotAbuseDurationSeconds = durationSeconds(
      durNotABotAbuseDays,
      durNotABotAbuseHours,
      durNotABotAbuseMinutes
    );
    challengePuzzleAbuseDurationSeconds = durationSeconds(
      durChallengePuzzleAbuseDays,
      durChallengePuzzleAbuseHours,
      durChallengePuzzleAbuseMinutes
    );
    adminDurationSeconds = durationSeconds(durAdminDays, durAdminHours, durAdminMinutes);

    durHoneypotValid = isDurationTupleValid(
      durHoneypotDays,
      durHoneypotHours,
      durHoneypotMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durIpRangeHoneypotValid = isDurationTupleValid(
      durIpRangeHoneypotDays,
      durIpRangeHoneypotHours,
      durIpRangeHoneypotMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durMazeCrawlerValid = isDurationTupleValid(
      durMazeCrawlerDays,
      durMazeCrawlerHours,
      durMazeCrawlerMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durRateLimitValid = isDurationTupleValid(
      durRateLimitDays,
      durRateLimitHours,
      durRateLimitMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durCdpValid = isDurationTupleValid(
      durCdpDays,
      durCdpHours,
      durCdpMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durEdgeFingerprintValid = isDurationTupleValid(
      durEdgeFingerprintDays,
      durEdgeFingerprintHours,
      durEdgeFingerprintMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durTarpitPersistenceValid = isDurationTupleValid(
      durTarpitPersistenceDays,
      durTarpitPersistenceHours,
      durTarpitPersistenceMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durNotABotAbuseValid = isDurationTupleValid(
      durNotABotAbuseDays,
      durNotABotAbuseHours,
      durNotABotAbuseMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durChallengePuzzleAbuseValid = isDurationTupleValid(
      durChallengePuzzleAbuseDays,
      durChallengePuzzleAbuseHours,
      durChallengePuzzleAbuseMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durAdminValid = isDurationTupleValid(
      durAdminDays,
      durAdminHours,
      durAdminMinutes,
      DURATION_VALIDATION_BOUNDS
    );
    durationsValid = (
      durHoneypotValid &&
      durIpRangeHoneypotValid &&
      durMazeCrawlerValid &&
      durRateLimitValid &&
      durCdpValid &&
      durEdgeFingerprintValid &&
      durTarpitPersistenceValid &&
      durNotABotAbuseValid &&
      durChallengePuzzleAbuseValid &&
      durAdminValid
    );
    durationsDirty = (
      honeypotDurationSeconds !== baseline.durations.honeypot ||
      ipRangeHoneypotDurationSeconds !== baseline.durations.ipRangeHoneypot ||
      mazeCrawlerDurationSeconds !== baseline.durations.mazeCrawler ||
      rateDurationSeconds !== baseline.durations.rateLimit ||
      cdpDurationSeconds !== baseline.durations.cdp ||
      edgeFingerprintDurationSeconds !== baseline.durations.edgeFingerprint ||
      tarpitPersistenceDurationSeconds !== baseline.durations.tarpitPersistence ||
      notABotAbuseDurationSeconds !== baseline.durations.notABotAbuse ||
      challengePuzzleAbuseDurationSeconds !== baseline.durations.challengePuzzleAbuse ||
      adminDurationSeconds !== baseline.durations.admin
    );

    browserBlockNormalized = normalizeBrowserRulesForCompare(browserBlockRules);
    browserBlockRulesValid = browserBlockNormalized !== '__invalid__';
    browserPolicyValid = browserBlockRulesValid;
    browserPolicyDirty = (
      (browserPolicyEnabled === true) !== baseline.browserPolicy.enabled ||
      browserBlockNormalized !== baseline.browserPolicy.block
    );

    pathAllowlistNormalized = normalizeListTextareaForCompare(pathAllowlist);
    pathAllowlistDirty = (
      (pathAllowlistEnabled === true) !== baseline.pathAllowlist.enabled ||
      pathAllowlistNormalized !== baseline.pathAllowlist.entries
    );

    dirtySections = [
      { label: 'Robots and AI policy', dirty: robotsDirty || aiPolicyDirty, valid: robotsValid },
      { label: 'Ban durations', dirty: durationsDirty, valid: durationsValid },
      { label: 'Browser policy', dirty: browserPolicyDirty, valid: browserPolicyValid },
      { label: 'Path allowlist', dirty: pathAllowlistDirty, valid: true }
    ];
    dirtySectionEntries = dirtySections.filter((section) => section.dirty === true);
    invalidDirtySectionEntries = dirtySectionEntries.filter((section) => section.valid !== true);
    invalidDirtySectionLabels = invalidDirtySectionEntries.map((section) => section.label);
    dirtySectionCount = dirtySectionEntries.length;
    hasUnsavedChanges = dirtySectionCount > 0;
    hasInvalidUnsavedChanges = invalidDirtySectionEntries.length > 0;
    savePolicyDisabled = !writable || !hasUnsavedChanges || hasInvalidUnsavedChanges || savingPolicy;
    savePolicyLabel = savingPolicy ? 'Saving...' : 'Save policy changes';
    savePolicySummary = hasUnsavedChanges
      ? `${dirtySectionCount} policy section${dirtySectionCount === 1 ? '' : 's'} with unsaved changes`
      : 'No unsaved changes';
    savePolicyInvalidText = hasInvalidUnsavedChanges
      ? `Fix invalid values in: ${invalidDirtySectionLabels.join(', ')}`
      : '';
    warnOnUnload = writable && hasUnsavedChanges;
    robotsPreviewPatchKey = JSON.stringify(buildRobotsPreviewPatch());
  }

  $: if (robotsPreviewOpen) {
    void robotsPreviewPatchKey;
    scheduleRobotsPreviewRefresh();
  }

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (!hasUnsavedChanges && !savingPolicy) {
        applyConfig(
          configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    }
  }

</script>

<section
  id="dashboard-panel-policy"
  class="admin-group config-edit-pane"
  class:config-edit-pane--dirty={hasUnsavedChanges}
  data-dashboard-tab-panel="policy"
  aria-labelledby="dashboard-tab-policy"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="policy" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <ConfigRobotsSection
      writable={writable}
      robotsDirty={robotsDirty}
      aiPolicyDirty={aiPolicyDirty}
      robotsEnabled={robotsEnabled}
      robotsCrawlDelay={robotsCrawlDelay}
      robotsCrawlDelayValid={robotsCrawlDelayValid}
      robotsBlockTraining={robotsBlockTraining}
      robotsBlockSearch={robotsBlockSearch}
      restrictSearchEngines={restrictSearchEngines}
      robotsPreviewOpen={robotsPreviewOpen}
      robotsPreviewLoading={robotsPreviewLoading}
      robotsPreviewContent={robotsPreviewContent}
      onRobotsPreviewControlChanged={onRobotsPreviewControlChanged}
      onToggleRobotsPreview={toggleRobotsPreview}
      onFieldChange={applyPolicyFieldChange}
    />

    <ConfigDurationsSection
      writable={writable}
      durationsDirty={durationsDirty}
      durHoneypotDays={durHoneypotDays}
      durHoneypotHours={durHoneypotHours}
      durHoneypotMinutes={durHoneypotMinutes}
      durIpRangeHoneypotDays={durIpRangeHoneypotDays}
      durIpRangeHoneypotHours={durIpRangeHoneypotHours}
      durIpRangeHoneypotMinutes={durIpRangeHoneypotMinutes}
      durMazeCrawlerDays={durMazeCrawlerDays}
      durMazeCrawlerHours={durMazeCrawlerHours}
      durMazeCrawlerMinutes={durMazeCrawlerMinutes}
      durRateLimitDays={durRateLimitDays}
      durRateLimitHours={durRateLimitHours}
      durRateLimitMinutes={durRateLimitMinutes}
      durCdpDays={durCdpDays}
      durCdpHours={durCdpHours}
      durCdpMinutes={durCdpMinutes}
      durEdgeFingerprintDays={durEdgeFingerprintDays}
      durEdgeFingerprintHours={durEdgeFingerprintHours}
      durEdgeFingerprintMinutes={durEdgeFingerprintMinutes}
      durTarpitPersistenceDays={durTarpitPersistenceDays}
      durTarpitPersistenceHours={durTarpitPersistenceHours}
      durTarpitPersistenceMinutes={durTarpitPersistenceMinutes}
      durNotABotAbuseDays={durNotABotAbuseDays}
      durNotABotAbuseHours={durNotABotAbuseHours}
      durNotABotAbuseMinutes={durNotABotAbuseMinutes}
      durChallengePuzzleAbuseDays={durChallengePuzzleAbuseDays}
      durChallengePuzzleAbuseHours={durChallengePuzzleAbuseHours}
      durChallengePuzzleAbuseMinutes={durChallengePuzzleAbuseMinutes}
      durAdminDays={durAdminDays}
      durAdminHours={durAdminHours}
      durAdminMinutes={durAdminMinutes}
      durHoneypotValid={durHoneypotValid}
      durIpRangeHoneypotValid={durIpRangeHoneypotValid}
      durMazeCrawlerValid={durMazeCrawlerValid}
      durRateLimitValid={durRateLimitValid}
      durCdpValid={durCdpValid}
      durEdgeFingerprintValid={durEdgeFingerprintValid}
      durTarpitPersistenceValid={durTarpitPersistenceValid}
      durNotABotAbuseValid={durNotABotAbuseValid}
      durChallengePuzzleAbuseValid={durChallengePuzzleAbuseValid}
      durAdminValid={durAdminValid}
      onFieldChange={applyPolicyFieldChange}
    />

    <ConfigNetworkSection
      writable={writable}
      showHoneypot={false}
      showBrowserPolicy={true}
      browserPolicyDirty={browserPolicyDirty}
      browserPolicyEnabled={browserPolicyEnabled}
      browserBlockRules={browserBlockRules}
      browserBlockRulesValid={browserBlockRulesValid}
      onFieldChange={applyPolicyFieldChange}
    />

    <ConfigPathAllowlistSection
      writable={writable}
      pathAllowlistDirty={pathAllowlistDirty}
      pathAllowlistEnabled={pathAllowlistEnabled}
      pathAllowlist={pathAllowlist}
      onFieldChange={applyPolicyFieldChange}
    />

    <SaveChangesBar
      containerId="policy-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="policy-unsaved-summary"
      summaryText={savePolicySummary}
      summaryClass="text-unsaved-changes"
      invalidId="policy-invalid-summary"
      invalidText={savePolicyInvalidText}
      buttonId="save-policy-config"
      buttonLabel={savePolicyLabel}
      buttonDisabled={savePolicyDisabled}
      onSave={savePolicyConfig}
    />
  </div>
</section>
