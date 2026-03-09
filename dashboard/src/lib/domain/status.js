// @ts-check

import { writableStatusVarPaths } from './config-schema.js';

const INITIAL_STATE = Object.freeze({
    failMode: 'unknown',
    httpsEnforced: false,
    forwardedHeaderTrustConfigured: false,
    runtimeEnvironment: '',
    gatewayDeploymentProfile: '',
    localProdDirectMode: false,
    adminConfigWriteEnabled: false,
    testMode: false,
    powEnabled: false,
    mazeEnabled: false,
    tarpitEnabled: false,
    mazeAutoBan: false,
    cdpEnabled: false,
    cdpAutoBan: false,
    jsRequiredEnforced: true,
    challengeEnabled: true,
    notABotEnabled: true,
    notABotThreshold: 2,
    challengeThreshold: 3,
    mazeThreshold: 6,
    ipRangePolicyMode: 'off',
    ipRangeEmergencyAllowCount: 0,
    ipRangeCustomRuleCount: 0,
    ipRangeCustomRuleEnabledCount: 0,
    rateLimit: 80,
    geoRiskCount: 0,
    geoAllowCount: 0,
    geoChallengeCount: 0,
    geoMazeCount: 0,
    geoBlockCount: 0,
    botnessWeights: {
      js_required: 1,
      geo_risk: 2,
      rate_medium: 1,
      rate_high: 2
    },
    configSnapshot: {}
  });

const createInitialState = () => ({
    ...INITIAL_STATE,
    botnessWeights: { ...INITIAL_STATE.botnessWeights },
    configSnapshot: cloneConfigSnapshot(INITIAL_STATE.configSnapshot)
  });

const WRITABLE_VAR_PATHS = new Set(writableStatusVarPaths || []);

const ABBR_TITLE_MAP = Object.freeze({
    AI: 'Artificial Intelligence',
    API: 'Application Programming Interface',
    CDP: 'Chrome DevTools Protocol',
    CIDR: 'Classless Inter-Domain Routing',
    CORS: 'Cross-Origin Resource Sharing',
    CPU: 'Central Processing Unit',
    CSRF: 'Cross-Site Request Forgery',
    GEO: 'Geolocation',
    GDPR: 'General Data Protection Regulation',
    HMAC: 'Hash-based Message Authentication Code',
    HTTP: 'Hypertext Transfer Protocol',
    HTTPS: 'Hypertext Transfer Protocol Secure',
    ID: 'Identifier',
    IP: 'Internet Protocol',
    JS: 'JavaScript',
    JSON: 'JavaScript Object Notation',
    KV: 'Key-Value',
    PoW: 'Proof of Work',
    TTL: 'Time To Live',
    UA: 'User Agent',
    UI: 'User Interface',
    URL: 'Uniform Resource Locator',
    UX: 'User Experience',
    VPN: 'Virtual Private Network',
    WASM: 'WebAssembly',
    WAF: 'Web Application Firewall'
  });

const ABBR_PATTERN = new RegExp(
    `\\b(${Object.keys(ABBR_TITLE_MAP).sort((a, b) => b.length - a.length).join('|')})\\b`,
    'g'
  );

function escapeHtml(value) {
    return String(value || '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

function withAbbrMarkup(text) {
    return String(text || '').replace(ABBR_PATTERN, (token) => {
      const title = ABBR_TITLE_MAP[token];
      if (!title) return token;
      return `<abbr title="${title}">${token}</abbr>`;
    });
  }

const EMPTY_VAR_MEANINGS = Object.freeze({});
const QUICK_REFERENCE_RUNTIME_MATRIX_URL =
  'https://github.com/atomless/Shuma-Gorath/blob/main/docs/quick-reference.md#runtime-and-deployment-posture-matrix';

const VAR_GROUP_DEFINITIONS = Object.freeze([
    {
      key: 'policy_runtime',
      title: 'Policy and Runtime Controls',
      matches: path => (
        path === 'test_mode' ||
        path === 'ban_duration' ||
        path.startsWith('ban_durations.') ||
        path === 'rate_limit' ||
        path === 'admin_config_write_enabled' ||
        path === 'kv_store_fail_open' ||
        path === 'https_enforced' ||
        path === 'forwarded_header_trust_configured'
      )
    },
    {
      key: 'ip_range_policy',
      title: 'IP Range Policy',
      matches: path => path.startsWith('ip_range_')
    },
    {
      key: 'risk_challenge',
      title: 'Risk Scoring and Challenge',
      matches: path => (
        path === 'js_required_enforced' ||
        path.startsWith('not_a_bot_') ||
        path.startsWith('pow_') ||
        path.startsWith('challenge_') ||
        path.startsWith('botness_')
      )
    },
    {
      key: 'signals_bypass',
      title: 'Signals and Bypass Lists',
      matches: path => (
        path === 'honeypots' ||
        path.startsWith('browser_') ||
        path === 'allowlist' ||
        path === 'path_allowlist' ||
        path.startsWith('geo_') ||
        path.startsWith('cdp_') ||
        path.startsWith('fingerprint_')
      )
    },
    {
      key: 'maze_runtime',
      title: 'Maze Runtime',
      matches: path => path.startsWith('maze_')
    },
    {
      key: 'tarpit_runtime',
      title: 'Tarpit Runtime',
      matches: path => path.startsWith('tarpit_')
    },
    {
      key: 'crawler_policy',
      title: 'Crawler and AI Policy',
      matches: path => path.startsWith('robots_') || path.startsWith('ai_policy_')
    },
    {
      key: 'provider_edge',
      title: 'Provider and Edge Integration',
      matches: path => (
        path.startsWith('defence_modes') ||
        path.startsWith('provider_backends') ||
        path === 'edge_integration_mode' ||
        path === 'defence_mode_warnings'
      )
    },
    {
      key: 'enterprise_guardrails',
      title: 'Enterprise Guardrails',
      matches: path => path.startsWith('enterprise_')
    },
    {
      key: 'signal_taxonomy',
      title: 'Signal Taxonomy',
      matches: path => path.startsWith('botness_signal_definitions')
    }
  ]);

function envVar(name) {
    return `<code class="env-var">${name}</code>`;
  }

export function normalizeFailMode(value) {
    const mode = (value || 'unknown').toString().toLowerCase();
    if (mode === 'open' || mode === 'closed') return mode;
    return 'unknown';
  }

function boolStatus(enabled) {
    return enabled ? 'ENABLED' : 'DISABLED';
  }

function cloneConfigSnapshot(configSnapshot) {
    if (!configSnapshot || typeof configSnapshot !== 'object') return {};
    try {
      return JSON.parse(JSON.stringify(configSnapshot));
    } catch (_e) {
      return {};
    }
  }

function parseBoolLike(value, fallback) {
    if (typeof value === 'boolean') return value;
    const normalized = String(value || '').trim().toLowerCase();
    if (!normalized) return fallback;
    if (normalized === 'true' || normalized === '1' || normalized === 'yes' || normalized === 'on') return true;
    if (normalized === 'false' || normalized === '0' || normalized === 'no' || normalized === 'off') return false;
    return fallback;
  }

function parseIntegerLike(value, fallback) {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? parsed : fallback;
  }

function listCount(value) {
    return Array.isArray(value) ? value.length : 0;
  }

function countEnabledEntries(value) {
    if (!Array.isArray(value)) return 0;
    return value.filter((entry) => entry && typeof entry === 'object' && entry.enabled === true).length;
  }

function normalizeIpRangeMode(value) {
    const mode = String(value || '').trim().toLowerCase();
    if (mode === 'advisory' || mode === 'enforce' || mode === 'off') return mode;
    return 'off';
  }

function normalizeRuntimeEnvironment(value) {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'runtime-dev' || normalized === 'runtime-prod') return normalized;
    return '';
  }

function normalizeGatewayDeploymentProfile(value) {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'shared-server' || normalized === 'edge-fermyon') return normalized;
    return '';
  }

function normalizeFreshnessState(value) {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'fresh' || normalized === 'degraded' || normalized === 'stale') return normalized;
    return 'unknown';
  }

function normalizeRetentionHealthState(value) {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'healthy' || normalized === 'degraded' || normalized === 'stalled') return normalized;
    return 'unknown';
  }

function formatDeploymentPosture(snapshot) {
    const runtime = snapshot.runtimeEnvironment ? snapshot.runtimeEnvironment.toUpperCase() : 'UNKNOWN';
    if (snapshot.localProdDirectMode === true) {
      return `${runtime} / LOCAL-DIRECT`;
    }
    if (snapshot.gatewayDeploymentProfile) {
      return `${runtime} / ${String(snapshot.gatewayDeploymentProfile || '').toUpperCase()}`;
    }
    return runtime;
  }

function formatRetentionFreshnessStatus(statusOperationalSnapshot = {}) {
    const source = statusOperationalSnapshot && typeof statusOperationalSnapshot === 'object'
      ? statusOperationalSnapshot
      : {};
    const freshness = source.freshness && typeof source.freshness === 'object' ? source.freshness : {};
    const retentionHealth =
      source.retention_health && typeof source.retention_health === 'object'
        ? source.retention_health
        : {};
    const freshnessState = normalizeFreshnessState(freshness.state);
    const retentionState = normalizeRetentionHealthState(retentionHealth.state);

    if (retentionState === 'stalled' || freshnessState === 'stale') return 'STALLED';
    if (retentionState === 'degraded' || freshnessState === 'degraded') return 'DEGRADED';
    if (retentionState === 'healthy' && freshnessState === 'fresh') return 'HEALTHY';
    return 'UNKNOWN';
  }

function formatLagHours(value) {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric < 0) return 'n/a';
    return `${numeric.toFixed(1)}h`;
  }

function formatLagMs(value) {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric < 0) return 'n/a';
    return `${Math.round(numeric)} ms`;
  }

function formatUnixTimestamp(value) {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric <= 0) return 'n/a';
    return new Date(numeric * 1000).toISOString();
  }

function formatIpRangeModeLabel(mode) {
    const normalized = normalizeIpRangeMode(mode);
    if (normalized === 'advisory') return 'LOGGING-ONLY';
    return normalized.toUpperCase();
  }

export function deriveStatusSnapshot(configSnapshot = {}) {
    const config = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};
    const base = createInitialState();
    const botnessWeights = config.botness_weights && typeof config.botness_weights === 'object'
      ? config.botness_weights
      : {};
    return {
      ...base,
      failMode: parseBoolLike(config.kv_store_fail_open, true) ? 'open' : 'closed',
      httpsEnforced: parseBoolLike(config.https_enforced, false),
      forwardedHeaderTrustConfigured: parseBoolLike(config.forwarded_header_trust_configured, false),
      runtimeEnvironment: normalizeRuntimeEnvironment(config.runtime_environment),
      gatewayDeploymentProfile: normalizeGatewayDeploymentProfile(config.gateway_deployment_profile),
      localProdDirectMode: parseBoolLike(config.local_prod_direct_mode, false),
      adminConfigWriteEnabled: parseBoolLike(config.admin_config_write_enabled, false),
      testMode: parseBoolLike(config.test_mode, false),
      powEnabled: parseBoolLike(config.pow_enabled, true),
      mazeEnabled: parseBoolLike(config.maze_enabled, true),
      tarpitEnabled: parseBoolLike(config.tarpit_enabled, true),
      mazeAutoBan: parseBoolLike(config.maze_auto_ban, true),
      cdpEnabled: parseBoolLike(config.cdp_detection_enabled, true),
      cdpAutoBan: parseBoolLike(config.cdp_auto_ban, true),
      jsRequiredEnforced: parseBoolLike(config.js_required_enforced, true),
      challengeEnabled: parseBoolLike(config.challenge_puzzle_enabled, true),
      notABotEnabled: parseBoolLike(config.not_a_bot_enabled, true),
      notABotThreshold: parseIntegerLike(config.not_a_bot_risk_threshold, base.notABotThreshold),
      challengeThreshold: parseIntegerLike(
        config.challenge_puzzle_risk_threshold,
        base.challengeThreshold
      ),
      mazeThreshold: parseIntegerLike(config.botness_maze_threshold, base.mazeThreshold),
      ipRangePolicyMode: normalizeIpRangeMode(config.ip_range_policy_mode),
      ipRangeEmergencyAllowCount: listCount(config.ip_range_emergency_allowlist),
      ipRangeCustomRuleCount: listCount(config.ip_range_custom_rules),
      ipRangeCustomRuleEnabledCount: countEnabledEntries(config.ip_range_custom_rules),
      rateLimit: parseIntegerLike(config.rate_limit, base.rateLimit),
      geoRiskCount: listCount(config.geo_risk),
      geoAllowCount: listCount(config.geo_allow),
      geoChallengeCount: listCount(config.geo_challenge),
      geoMazeCount: listCount(config.geo_maze),
      geoBlockCount: listCount(config.geo_block),
      botnessWeights: {
        js_required: parseIntegerLike(botnessWeights.js_required, base.botnessWeights.js_required),
        geo_risk: parseIntegerLike(botnessWeights.geo_risk, base.botnessWeights.geo_risk),
        rate_medium: parseIntegerLike(botnessWeights.rate_medium, base.botnessWeights.rate_medium),
        rate_high: parseIntegerLike(botnessWeights.rate_high, base.botnessWeights.rate_high)
      },
      configSnapshot: cloneConfigSnapshot(config)
    };
  }

function flattenConfigEntries(value, prefix = '') {
    if (value === null || value === undefined) {
      return [{ path: prefix, value: value === undefined ? null : value }];
    }
    if (Array.isArray(value)) {
      return [{ path: prefix, value }];
    }
    if (typeof value !== 'object') {
      return [{ path: prefix, value }];
    }
    const keys = Object.keys(value).sort();
    if (keys.length === 0) {
      return prefix ? [{ path: prefix, value: {} }] : [];
    }
    const entries = [];
    keys.forEach((key) => {
      const nextPath = prefix ? `${prefix}.${key}` : key;
      const child = value[key];
      if (child && typeof child === 'object' && !Array.isArray(child)) {
        entries.push(...flattenConfigEntries(child, nextPath));
        return;
      }
      entries.push({ path: nextPath, value: child });
    });
    return entries;
  }

function classifyVarPath(path) {
    return WRITABLE_VAR_PATHS.has(path) ? 'ADMIN_WRITE' : 'READ_ONLY';
  }

function classifyVarGroup(path) {
    const matched = VAR_GROUP_DEFINITIONS.find(group => group.matches(path));
    if (matched) return matched;
    return {
      key: 'other',
      title: 'Other Runtime Variables'
    };
  }

function formatVarValue(value) {
    if (value === null) return 'null';
    if (Array.isArray(value)) return JSON.stringify(value);
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  }

function humanizeVarPath(path) {
    return path
      .replace(/\./g, ' ')
      .replace(/_/g, ' ')
      .replace(/\b[a-z]/g, char => char.toUpperCase());
  }

function normalizeVarMeanings(varMeanings) {
    if (!varMeanings || typeof varMeanings !== 'object') return EMPTY_VAR_MEANINGS;
    return varMeanings;
  }

function meaningForVarPath(path, varMeanings) {
    const meanings = normalizeVarMeanings(varMeanings);
    if (Object.prototype.hasOwnProperty.call(meanings, path)) {
      return meanings[path];
    }
    return `${humanizeVarPath(path)} runtime value. See docs/configuration.md for canonical definition.`;
  }

function cumulativeBotnessRoutingText(snapshot) {
    return (
      `This contributes to the cumulative <strong>botness</strong> score used for defense routing decisions ` +
      `(challenge at <strong>${snapshot.challengeThreshold}</strong>, maze at <strong>${snapshot.mazeThreshold}</strong>, ` +
      `and higher-severity controls such as tar pit or immediate IP ban where configured).`
    );
  }

const STATUS_DEFINITIONS = [
    {
      title: 'Fail Mode Policy',
      description: () => (
        `Controls request handling when the KV store is unavailable. ${envVar('SHUMA_KV_STORE_FAIL_OPEN')}=<strong>true</strong> allows requests to continue (fail-open); ` +
        `${envVar('SHUMA_KV_STORE_FAIL_OPEN')}=<strong>false</strong> blocks requests that require KV-backed decisions (fail-closed). Can only be set in ENV. Set to false for a Security-first stance, and true for an Availability-first stance.`
      ),
      status: snapshot => normalizeFailMode(snapshot.failMode).toUpperCase()
    },
    {
      title: 'HTTPS Enforcement',
      description: snapshot => (
        `When ${envVar('SHUMA_ENFORCE_HTTPS')} is true, the app rejects non-HTTPS requests with <strong>403 HTTPS required</strong>. ` +
        `Forwarded proto headers are trusted only when ${envVar('SHUMA_FORWARDED_IP_SECRET')} validation succeeds. ` +
        `Current forwarded-header trust configuration is <strong>${boolStatus(snapshot.forwardedHeaderTrustConfigured)}</strong>.`
      ),
      status: snapshot => boolStatus(snapshot.httpsEnforced)
    },
    {
      title: 'Proof-of-Work (PoW)',
      description: snapshot => (
        `PoW is applied in the JS verification flow and increases bot cost before <code>js_verified</code> is issued. ` +
        `Primary controls are ${envVar('SHUMA_POW_ENABLED')}, ${envVar('SHUMA_POW_DIFFICULTY')}, and ${envVar('SHUMA_POW_TTL_SECONDS')}. ` +
        `Runtime updates are available only when ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} is enabled. ` +
        `If ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} is disabled, normal visitor requests bypass this flow.`
      ),
      status: snapshot => boolStatus(snapshot.powEnabled)
    },
    {
      title: 'Challenge Puzzle',
      description: snapshot => (
        `Step-up routing sends suspicious traffic to the puzzle challenge when ${envVar('SHUMA_CHALLENGE_PUZZLE_ENABLED')} is true and cumulative botness reaches ${envVar('SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD')} ` +
        `(enabled: <strong>${boolStatus(snapshot.challengeEnabled)}</strong>, current: <strong>${snapshot.challengeThreshold}</strong>). ` +
        `Puzzle complexity is controlled by ${envVar('SHUMA_CHALLENGE_PUZZLE_TRANSFORM_COUNT')}. ` +
        `Runtime updates are available only when ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} is enabled.`
      ),
      status: snapshot => boolStatus(snapshot.challengeEnabled)
    },
    {
      title: 'Challenge Not-A-Bot',
      description: snapshot => (
        `Lower-certainty step-up routing can serve the Not-a-Bot checkbox when ${envVar('SHUMA_NOT_A_BOT_ENABLED')} is true and cumulative botness reaches ${envVar('SHUMA_NOT_A_BOT_RISK_THRESHOLD')} ` +
        `(enabled: <strong>${boolStatus(snapshot.notABotEnabled)}</strong>, current: <strong>${snapshot.notABotThreshold}</strong>). ` +
        `Submit scoring uses ${envVar('SHUMA_NOT_A_BOT_PASS_SCORE')} and ${envVar('SHUMA_NOT_A_BOT_FAIL_SCORE')}. ` +
        `Token and attempt controls are governed by ${envVar('SHUMA_NOT_A_BOT_NONCE_TTL_SECONDS')}, ${envVar('SHUMA_NOT_A_BOT_MARKER_TTL_SECONDS')}, ${envVar('SHUMA_NOT_A_BOT_ATTEMPT_LIMIT_PER_WINDOW')}, and ${envVar('SHUMA_NOT_A_BOT_ATTEMPT_WINDOW_SECONDS')}.`
      ),
      status: snapshot => boolStatus(snapshot.notABotEnabled)
    },
    {
      title: 'CDP Detection',
      description: () => (
        `Detects browser automation from client CDP reports. Primary controls are ${envVar('SHUMA_CDP_DETECTION_ENABLED')}, ${envVar('SHUMA_CDP_AUTO_BAN')}, and ${envVar('SHUMA_CDP_DETECTION_THRESHOLD')}. ` +
        `Hard checks (for example <code>webdriver</code> or <code>automation_props</code>) are treated as <strong>strong</strong>. ` +
        `Without hard checks, detections are tiered by score and soft signals using ${envVar('SHUMA_CDP_DETECTION_THRESHOLD')}. ` +
        `If ${envVar('SHUMA_CDP_AUTO_BAN')} is enabled, only final <strong>strong</strong> CDP detections trigger automatic IP bans.`
      ),
      status: snapshot => boolStatus(snapshot.cdpEnabled)
    },
    {
      title: 'Maze',
      description: () => (
        `Maze routes suspicious traffic into trap pages when ${envVar('SHUMA_MAZE_ENABLED')} is enabled. ` +
        `If ${envVar('SHUMA_MAZE_AUTO_BAN')} is enabled, automatic bans trigger when maze hits exceed ${envVar('SHUMA_MAZE_AUTO_BAN_THRESHOLD')}.`
      ),
      status: snapshot => boolStatus(snapshot.mazeEnabled)
    },
    {
      title: 'Tarpit',
      description: () => (
        `Tarpit applies progression-gated attack defence for confirmed challenge attacks when ${envVar('SHUMA_TARPIT_ENABLED')} is enabled. ` +
        `Tarpit is served only when maze routing is available and uses bounded concurrency, egress, and flow-duration budgets.`
      ),
      status: snapshot => boolStatus(snapshot.tarpitEnabled)
    },
    {
      title: 'JS Required',
      description: snapshot => (
        `When ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} is true, requests without a valid <code>js_verified</code> cookie are sent to the JS verification page. ` +
        `That flow writes <code>js_verified</code>, reloads the original path, and re-evaluates access. ` +
        `If ${envVar('SHUMA_POW_ENABLED')} is true, this step includes PoW before the cookie is issued. ` +
        `Disabling ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} allows non-JS clients but removes PoW from the normal request path. ` +
        `Its botness contribution is weighted separately by ${envVar('SHUMA_BOTNESS_WEIGHT_JS_REQUIRED')} ` +
        `(current weight: <strong>${snapshot.botnessWeights.js_required || 0}</strong>). ` +
        cumulativeBotnessRoutingText(snapshot)
      ),
      status: snapshot => boolStatus(snapshot.jsRequiredEnforced)
    },
    {
      title: 'GEO Fencing',
      description: snapshot => (
        `Uses trusted upstream GEO headers only (headers are trusted when ${envVar('SHUMA_FORWARDED_IP_SECRET')} validation succeeds). ` +
        `Scoring countries are configured by ${envVar('SHUMA_GEO_RISK_COUNTRIES')} ` +
        `(current count: <strong>${snapshot.geoRiskCount}</strong>). ` +
        `Routing precedence uses ${envVar('SHUMA_GEO_BLOCK_COUNTRIES')} (<strong>${snapshot.geoBlockCount}</strong>), ` +
        `${envVar('SHUMA_GEO_MAZE_COUNTRIES')} (<strong>${snapshot.geoMazeCount}</strong>), ` +
        `${envVar('SHUMA_GEO_CHALLENGE_COUNTRIES')} (<strong>${snapshot.geoChallengeCount}</strong>), ` +
        `and ${envVar('SHUMA_GEO_ALLOW_COUNTRIES')} (<strong>${snapshot.geoAllowCount}</strong>). ` +
        `Scoring matches contribute via ${envVar('SHUMA_BOTNESS_WEIGHT_GEO_RISK')} ` +
        `(current weight: <strong>${snapshot.botnessWeights.geo_risk || 0}</strong>). ` +
        cumulativeBotnessRoutingText(snapshot)
      ),
      status: snapshot => boolStatus((snapshot.botnessWeights.geo_risk || 0) > 0)
    },
    {
      title: 'IP Range Policy',
      description: snapshot => (
        `${envVar('SHUMA_IP_RANGE_POLICY_MODE')} controls CIDR policy execution mode ` +
        `(<strong>${formatIpRangeModeLabel(snapshot.ipRangePolicyMode)}</strong>). ` +
        `Emergency bypass CIDRs use ${envVar('SHUMA_IP_RANGE_EMERGENCY_ALLOWLIST')} ` +
        `(<strong>${snapshot.ipRangeEmergencyAllowCount}</strong>). ` +
        `Custom rule count from ${envVar('SHUMA_IP_RANGE_CUSTOM_RULES')}: ` +
        `<strong>${snapshot.ipRangeCustomRuleCount}</strong> (enabled: <strong>${snapshot.ipRangeCustomRuleEnabledCount}</strong>).`
      ),
      status: snapshot => formatIpRangeModeLabel(snapshot.ipRangePolicyMode)
    },
    {
      title: 'Rate Limiting',
      description: snapshot => (
        `Rate pressure is measured against ${envVar('SHUMA_RATE_LIMIT')} (current limit: <strong>${snapshot.rateLimit}</strong> requests/min). ` +
        `Crossing the hard limit triggers immediate rate-limit enforcement. ` +
        `Medium pressure (>=50%) contributes ${envVar('SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM')} and high pressure (>=80%) ` +
        `contributes ${envVar('SHUMA_BOTNESS_WEIGHT_RATE_HIGH')} (current weights: <strong>${snapshot.botnessWeights.rate_medium || 0}</strong> / ` +
        `<strong>${snapshot.botnessWeights.rate_high || 0}</strong>). ` +
        cumulativeBotnessRoutingText(snapshot)
      ),
      status: snapshot => boolStatus(
        (snapshot.botnessWeights.rate_medium || 0) > 0 || (snapshot.botnessWeights.rate_high || 0) > 0
      )
    },
    {
      title: 'Runtime and Deployment Posture',
      description: snapshot => (
        `Current runtime posture is derived from ${envVar('SHUMA_RUNTIME_ENV')} ` +
        `(<strong>${escapeHtml(snapshot.runtimeEnvironment || 'unknown')}</strong>). ` +
        `Deployment profile comes from ${envVar('SHUMA_GATEWAY_DEPLOYMENT_PROFILE')} ` +
        `(<strong>${escapeHtml(snapshot.gatewayDeploymentProfile || 'unknown')}</strong>). ` +
        `When ${envVar('SHUMA_LOCAL_PROD_DIRECT_MODE')} is true, localhost prod-like runs are treated as <strong>local-direct</strong> and do not require ${envVar('SHUMA_GATEWAY_UPSTREAM_ORIGIN')}. ` +
        `See the <a href="${QUICK_REFERENCE_RUNTIME_MATRIX_URL}" target="_blank" rel="noopener noreferrer">runtime and deployment posture matrix</a> for the exact dev versus local prod-like versus deployed production differences.`
      ),
      status: snapshot => formatDeploymentPosture(snapshot)
    },
    {
      title: 'Admin Config Write Posture',
      description: snapshot => (
        `${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} controls whether dashboard config mutations are accepted at runtime ` +
        `(<strong>${boolStatus(snapshot.adminConfigWriteEnabled)}</strong>). ` +
        `When disabled, write-dependent controls across config tabs must stay hidden and operators must treat the dashboard as observation-only for live defense tuning.`
      ),
      status: snapshot => boolStatus(snapshot.adminConfigWriteEnabled)
    },
    {
      title: 'Retention and Freshness Health',
      description: (_snapshot, options = {}) => {
        const source = options.statusOperationalSnapshot && typeof options.statusOperationalSnapshot === 'object'
          ? options.statusOperationalSnapshot
          : {};
        const freshness = source.freshness && typeof source.freshness === 'object' ? source.freshness : {};
        const retentionHealth =
          source.retention_health && typeof source.retention_health === 'object'
            ? source.retention_health
            : {};
        const freshnessState = normalizeFreshnessState(freshness.state);
        const retentionState = normalizeRetentionHealthState(retentionHealth.state);
        const retentionHours = Number(retentionHealth.retention_hours);
        const pendingExpiredBuckets = Number(retentionHealth.pending_expired_buckets || 0);
        const lastPurgeError = String(retentionHealth.last_purge_error || '').trim();
        return (
          `Telemetry retention is governed by ${envVar('SHUMA_EVENT_LOG_RETENTION_HOURS')} ` +
          `(configured window: <strong>${Number.isFinite(retentionHours) && retentionHours > 0 ? `${Math.floor(retentionHours)}h` : 'n/a'}</strong>). ` +
          `Current monitoring freshness is <strong>${escapeHtml(freshnessState)}</strong> ` +
          `(lag: <strong>${escapeHtml(formatLagMs(freshness.lag_ms))}</strong>, last event: <strong>${escapeHtml(formatUnixTimestamp(freshness.last_event_ts))}</strong>). ` +
          `Retention worker health is <strong>${escapeHtml(retentionState)}</strong> ` +
          `(purge lag: <strong>${escapeHtml(formatLagHours(retentionHealth.purge_lag_hours))}</strong>, pending expired buckets: <strong>${Number.isFinite(pendingExpiredBuckets) ? pendingExpiredBuckets : 0}</strong>, oldest retained: <strong>${escapeHtml(formatUnixTimestamp(retentionHealth.oldest_retained_ts))}</strong>). ` +
          `Last purge success: <strong>${escapeHtml(formatUnixTimestamp(retentionHealth.last_purge_success_ts))}</strong>. ` +
          `${lastPurgeError ? `Last purge error: <code>${escapeHtml(lastPurgeError)}</code>.` : ''}`
        );
      },
      status: (_snapshot, options = {}) => formatRetentionFreshnessStatus(options.statusOperationalSnapshot)
    }
  ];

export function buildFeatureStatusItems(snapshot, options = {}) {
    return STATUS_DEFINITIONS.map((definition) => ({
      title: withAbbrMarkup(escapeHtml(definition.title)),
      description: withAbbrMarkup(definition.description(snapshot, options)),
      status: definition.status(snapshot, options)
    }));
  }

export function buildVariableInventoryGroups(snapshot, options = {}) {
    const varMeanings = normalizeVarMeanings(options.varMeanings);
    const flattened = flattenConfigEntries(snapshot.configSnapshot || {})
      .filter((entry) => entry.path && entry.path.length > 0)
      .map((entry) => {
        const valueClass = classifyVarPath(entry.path);
        return {
          path: entry.path,
          valueClass,
          group: classifyVarGroup(entry.path),
          valueText: formatVarValue(entry.value),
          meaning: withAbbrMarkup(escapeHtml(meaningForVarPath(entry.path, varMeanings))),
          isAdminWrite: valueClass === 'ADMIN_WRITE'
        };
      })
      .sort((a, b) => {
        if (a.group.key !== b.group.key) {
          const groupOrder = VAR_GROUP_DEFINITIONS.map((group) => group.key).concat(['other']);
          return groupOrder.indexOf(a.group.key) - groupOrder.indexOf(b.group.key);
        }
        if (a.valueClass !== b.valueClass) {
          return a.valueClass === 'ADMIN_WRITE' ? -1 : 1;
        }
        return a.path.localeCompare(b.path);
      });

    if (flattened.length === 0) {
      return [];
    }

    const grouped = new Map();
    flattened.forEach((entry) => {
      if (!grouped.has(entry.group.key)) {
        grouped.set(entry.group.key, {
          key: entry.group.key,
          title: withAbbrMarkup(escapeHtml(entry.group.title)),
          entries: []
        });
      }
      grouped.get(entry.group.key).entries.push(entry);
    });

    const orderedGroupKeys = VAR_GROUP_DEFINITIONS
      .map((group) => group.key)
      .concat(['other'])
      .filter((key) => grouped.has(key));
    return orderedGroupKeys.map((key) => grouped.get(key));
  }
