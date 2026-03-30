const { test, expect } = require("@playwright/test");
const { seedDashboardData } = require("./seed-dashboard-data");

const BASE_URL = process.env.SHUMA_BASE_URL || "http://127.0.0.1:3000";
const API_KEY = (process.env.SHUMA_API_KEY || "").trim();
const FORWARDED_IP_SECRET = (process.env.SHUMA_FORWARDED_IP_SECRET || "").trim();
const DASHBOARD_TABS = Object.freeze(["traffic", "ip-bans", "red-team", "game-loop", "tuning", "verification", "traps", "rate-limiting", "geo", "fingerprinting", "policy", "status", "advanced", "diagnostics"]);
const ADMIN_TABS = Object.freeze(["traffic", "ip-bans", "red-team", "game-loop", "tuning", "verification", "traps", "rate-limiting", "geo", "fingerprinting", "policy", "status", "advanced", "diagnostics"]);
const VERIFICATION_RESTORE_PATHS = Object.freeze([
  "js_required_enforced"
]);
const VERIFIED_IDENTITY_RESTORE_PATHS = Object.freeze([
  "verified_identity.enabled",
  "verified_identity.native_web_bot_auth_enabled",
  "verified_identity.provider_assertions_enabled",
  "verified_identity.replay_window_seconds",
  "verified_identity.clock_skew_seconds",
  "verified_identity.directory_cache_ttl_seconds",
  "verified_identity.directory_freshness_requirement_seconds"
]);
const SHADOW_MODE_RESTORE_PATHS = Object.freeze([
  "shadow_mode"
]);
const GEO_AND_TUNING_RESTORE_PATHS = Object.freeze([
  "defence_modes.geo",
  "geo_edge_headers_enabled",
  "geo_risk",
  "geo_allow",
  "geo_challenge",
  "geo_maze",
  "geo_block",
  "not_a_bot_risk_threshold",
  "challenge_puzzle_risk_threshold",
  "botness_maze_threshold",
  "botness_weights.js_required",
  "botness_weights.geo_risk",
  "botness_weights.rate_medium",
  "botness_weights.rate_high",
  "browser_policy_enabled",
  "browser_block",
  "path_allowlist_enabled",
  "path_allowlist"
]);
const RATE_LIMITING_RESTORE_PATHS = Object.freeze([
  "rate_limit",
  "defence_modes.rate",
  "provider_backends.rate_limiter"
]);
const POLICY_RESTORE_PATHS = Object.freeze([
  "robots_enabled",
  "robots_crawl_delay",
  "ai_policy_block_training",
  "ai_policy_block_search",
  "ai_policy_allow_search_engines",
  "ban_durations.rate_limit",
  "ban_durations.tarpit_persistence",
  "browser_policy_enabled",
  "browser_block",
  "path_allowlist_enabled",
  "path_allowlist"
]);
const TRAPS_RESTORE_PATHS = Object.freeze([
  "honeypot_enabled",
  "honeypots",
  "maze_enabled",
  "maze_auto_ban",
  "maze_auto_ban_threshold",
  "tarpit_enabled"
]);
const ADVERSARY_SIM_RESTORE_PATHS = Object.freeze([
  "adversary_sim_duration_seconds"
]);
const runtimeGuards = new WeakMap();

function ensureRequiredEnv() {
  if (!API_KEY) {
    throw new Error("Missing SHUMA_API_KEY for dashboard smoke tests.");
  }
  if (/^changeme/i.test(API_KEY)) {
    throw new Error("SHUMA_API_KEY is a placeholder value; run make setup or make api-key-generate.");
  }
}

function buildTrustedForwardingHeaders(ip = "127.0.0.1") {
  const headers = {
    "X-Forwarded-For": ip
  };
  if (FORWARDED_IP_SECRET) {
    headers["X-Shuma-Forwarded-Secret"] = FORWARDED_IP_SECRET;
  }
  return headers;
}

function isStaticRuntimeRequest(request) {
  const resourceType = request.resourceType();
  return resourceType === "script" || resourceType === "stylesheet";
}

function ensureRuntimeGuard(page) {
  if (runtimeGuards.has(page)) {
    return runtimeGuards.get(page);
  }

  const guard = {
    failures: []
  };

  page.on("pageerror", (error) => {
    guard.failures.push(`pageerror: ${error.message}`);
  });

  page.on("requestfailed", (request) => {
    if (!isStaticRuntimeRequest(request)) {
      return;
    }
    const failure = request.failure();
    const errorText = String(failure?.errorText || "");
    // Navigation transitions can intentionally abort in-flight static fetches.
    if (errorText.includes("ERR_ABORTED") || errorText.includes("NS_BINDING_ABORTED")) {
      return;
    }
    guard.failures.push(
      `requestfailed: ${request.method()} ${request.url()} (${failure ? failure.errorText : "unknown"})`
    );
  });

  page.on("response", (response) => {
    const request = response.request();
    if (!isStaticRuntimeRequest(request)) {
      return;
    }
    if (response.status() >= 400) {
      guard.failures.push(
        `asset-response: ${request.method()} ${response.url()} -> ${response.status()}`
      );
    }
  });

  runtimeGuards.set(page, guard);
  return guard;
}

function assertNoRuntimeFailures(page) {
  const guard = runtimeGuards.get(page);
  if (!guard || guard.failures.length === 0) {
    return;
  }
  throw new Error(`Unexpected dashboard runtime failures:\n${guard.failures.join("\n")}`);
}

function runtimeFailures(page) {
  const guard = runtimeGuards.get(page);
  if (!guard || !Array.isArray(guard.failures)) {
    return [];
  }
  return [...guard.failures];
}

function clearRuntimeFailures(page) {
  const guard = runtimeGuards.get(page);
  if (!guard || !Array.isArray(guard.failures)) {
    return;
  }
  guard.failures.length = 0;
}

function hasOnlyTransientStaticDisconnectFailures(page) {
  const failures = runtimeFailures(page);
  if (failures.length === 0) {
    return false;
  }
  return failures.every((failure) =>
    /requestfailed:\s+GET\s+.*\/dashboard\/_app\/immutable\/.*ERR_CONNECTION_REFUSED/i.test(failure)
  );
}

function parsePrometheusLabeledCounters(text, metricName, labelName) {
  const counters = {};
  const escapedMetric = metricName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const escapedLabel = labelName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pattern = new RegExp(
    `^${escapedMetric}\\{${escapedLabel}="([^"]+)"\\}\\s+([0-9]+(?:\\.[0-9]+)?)$`
  );
  for (const line of String(text || "").split("\n")) {
    const match = pattern.exec(line.trim());
    if (!match) continue;
    counters[match[1]] = Number.parseInt(match[2], 10);
  }
  return counters;
}

function parsePrometheusScalar(text, metricName) {
  const escapedMetric = metricName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pattern = new RegExp(`^${escapedMetric}\\s+([0-9]+(?:\\.[0-9]+)?)$`, "m");
  const match = pattern.exec(String(text || ""));
  if (!match) return 0;
  return Number.parseInt(match[1], 10);
}

function sumCounterValues(counters) {
  return Object.values(counters || {}).reduce((total, value) => total + Number(value || 0), 0);
}

function expectCounterNear(actual, expected, tolerance = 0) {
  const delta = Math.abs(Number(actual || 0) - Number(expected || 0));
  expect(delta).toBeLessThanOrEqual(Math.max(0, Number(tolerance || 0)));
}

function parseDashboardCounterText(text) {
  const raw = String(text || "").trim().toLowerCase();
  const compactMatch = raw.match(/(-?\d+(?:[.,]\d+)?)\s*([kmb])\b/);
  if (compactMatch) {
    const base = Number.parseFloat(compactMatch[1].replace(/,/g, ""));
    if (Number.isFinite(base)) {
      const suffix = compactMatch[2];
      const multiplier = suffix === "k" ? 1_000 : suffix === "m" ? 1_000_000 : 1_000_000_000;
      return Math.round(base * multiplier);
    }
  }
  const digits = raw.replace(/,/g, "").replace(/[^0-9.-]/g, "");
  if (!digits) return 0;
  const parsed = Number.parseFloat(digits);
  return Number.isFinite(parsed) ? parsed : 0;
}

function formatTextareaList(values) {
  return Array.isArray(values)
    ? values.map((value) => String(value || "").trim()).filter((value) => value.length > 0).join("\n")
    : "";
}

function formatBrowserRules(rules) {
  if (!Array.isArray(rules)) {
    return "";
  }
  return rules
    .map((rule) => {
      if (!Array.isArray(rule) || rule.length < 2) {
        return "";
      }
      const name = String(rule[0] || "").trim();
      const version = Number.parseInt(rule[1], 10);
      if (!name || !Number.isFinite(version)) {
        return "";
      }
      return `${name},${version}`;
    })
    .filter((rule) => rule.length > 0)
    .join("\n");
}

function durationParts(totalSeconds) {
  const safeSeconds = Math.max(0, Number.parseInt(totalSeconds, 10) || 0);
  const totalMinutes = Math.floor(safeSeconds / 60);
  const days = Math.floor(totalMinutes / (24 * 60));
  const hours = Math.floor((totalMinutes % (24 * 60)) / 60);
  const minutes = totalMinutes % 60;
  return { days, hours, minutes };
}

async function assertActiveTabPanelVisibility(page, activeTab) {
  for (const tab of DASHBOARD_TABS) {
    await expect(page.locator(`#dashboard-tab-${tab}`)).toHaveAttribute(
      "aria-selected",
      tab === activeTab ? "true" : "false"
    );
  }

  if (activeTab === "game-loop") {
    await expect(page.locator("#dashboard-panel-game-loop")).toBeVisible();
    await expect(page.locator("#dashboard-admin-section")).toBeHidden();
    for (const tab of ADMIN_TABS) {
      if (tab === "game-loop") {
        continue;
      }
      await expect(page.locator(`#dashboard-panel-${tab}`)).toBeHidden();
    }
    return;
  }

  await expect(page.locator("#dashboard-panel-game-loop")).toBeHidden();
  await expect(page.locator("#dashboard-admin-section")).toHaveJSProperty("hidden", false);
  await expect(page.locator("#dashboard-admin-section")).toHaveAttribute("aria-hidden", "false");
  for (const tab of ADMIN_TABS) {
    const panel = page.locator(`#dashboard-panel-${tab}`);
    if (tab === activeTab) {
      const forcedHidden = await panel.evaluate((element) => element.classList.contains("hidden"));
      if (forcedHidden) {
        await expect(panel).toBeHidden();
      } else {
        await expect(panel).toHaveJSProperty("hidden", false);
        await expect(panel).toHaveAttribute("aria-hidden", "false");
      }
    } else {
      await expect(panel).toBeHidden();
    }
  }
}

function dashboardRelativePath(url) {
  try {
    const parsed = new URL(url);
    return `${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch (_error) {
    return "/dashboard/index.html";
  }
}

function isDashboardLoginUrl(url) {
  return String(url || "").includes("/dashboard/login.html");
}

function newDashboardIdempotencyKey() {
  return `dash-e2e-${Date.now().toString(16)}-${Math.floor(Math.random() * 0x1_0000_0000).toString(16).padStart(8, "0")}`;
}

async function dashboardDomClassState(page) {
  return page.evaluate(() => {
    const rootClasses = Array.from(document?.documentElement?.classList || []);
    const bodyClasses = Array.from(document?.body?.classList || []);
    const hasRuntimeDev = rootClasses.includes("runtime-dev");
    const hasRuntimeProd = rootClasses.includes("runtime-prod");
    return {
      rootClasses,
      bodyClasses,
      hasRuntimeDev,
      hasRuntimeProd,
      runtimeClassCount: (hasRuntimeDev ? 1 : 0) + (hasRuntimeProd ? 1 : 0),
      hasAdversarySim: rootClasses.includes("adversary-sim"),
      hasShadowMode: rootClasses.includes("shadow-mode"),
      bodyHasAdversarySim: bodyClasses.includes("adversary-sim"),
      bodyHasShadowMode: bodyClasses.includes("shadow-mode"),
      bodyConnectedClassPresent: bodyClasses.includes("connected"),
      bodyDisconnectedClassPresent: bodyClasses.includes("disconnected")
    };
  });
}

function cloneJsonValue(value) {
  if (value === undefined) return undefined;
  return JSON.parse(JSON.stringify(value));
}

function getConfigPathValue(config, path) {
  return String(path || "")
    .split(".")
    .reduce((cursor, segment) => {
      if (!cursor || typeof cursor !== "object") return undefined;
      return cursor[segment];
    }, config);
}

function setConfigPathValue(target, path, value) {
  const segments = String(path || "").split(".").filter(Boolean);
  if (segments.length === 0) return;
  let cursor = target;
  for (let index = 0; index < segments.length - 1; index += 1) {
    const segment = segments[index];
    if (!cursor[segment] || typeof cursor[segment] !== "object" || Array.isArray(cursor[segment])) {
      cursor[segment] = {};
    }
    cursor = cursor[segment];
  }
  cursor[segments[segments.length - 1]] = cloneJsonValue(value);
}

function extractConfigPatchFromPaths(config, paths = []) {
  const patch = {};
  for (const path of paths) {
    const value = getConfigPathValue(config, path);
    if (value === undefined) continue;
    setConfigPathValue(patch, path, value);
  }
  return patch;
}

async function fetchAdminConfig(request, ip = "127.0.0.1") {
  const response = await request.get(`${BASE_URL}/admin/config`, {
    headers: buildAdminAuthHeaders(ip)
  });
  if (!response.ok()) {
    const body = await response.text();
    throw new Error(`admin config read should succeed: ${response.status()} ${body}`);
  }
  const payload = await response.json();
  return payload && typeof payload === "object" && payload.config && typeof payload.config === "object"
    ? payload.config
    : {};
}

async function fetchAdminConfigRuntime(request, ip = "127.0.0.1") {
  const response = await request.get(`${BASE_URL}/admin/config`, {
    headers: buildAdminAuthHeaders(ip)
  });
  if (!response.ok()) {
    const body = await response.text();
    throw new Error(`admin config read should succeed: ${response.status()} ${body}`);
  }
  const payload = await response.json();
  return payload && typeof payload === "object" && payload.runtime && typeof payload.runtime === "object"
    ? payload.runtime
    : {};
}

async function withRestoredAdminConfig(request, paths, callback, ip = "127.0.0.1") {
  const restorePatch = extractConfigPatchFromPaths(await fetchAdminConfig(request, ip), paths);
  try {
    return await callback(restorePatch);
  } finally {
    if (Object.keys(restorePatch).length === 0) return;
    await updateAdminConfig(request, restorePatch, ip);
    const restoredPatch = extractConfigPatchFromPaths(await fetchAdminConfig(request, ip), paths);
    expect(restoredPatch).toEqual(restorePatch);
  }
}

async function fetchRuntimeEnvironment(request, ip = "127.0.0.1") {
  const payload = await fetchAdminConfigRuntime(request, ip);
  const runtimeEnvironment = String(payload?.runtime_environment || "").trim();
  if (runtimeEnvironment !== "runtime-dev" && runtimeEnvironment !== "runtime-prod") {
    throw new Error(`unexpected runtime_environment from /admin/config: ${runtimeEnvironment || "missing"}`);
  }
  return runtimeEnvironment;
}

async function fetchAdversarySimStatus(_request, ip = "127.0.0.1") {
  const cacheBuster = `${Date.now().toString(16)}-${Math.floor(Math.random() * 0x1_0000_0000).toString(16)}`;
  const response = await fetch(`${BASE_URL}/admin/adversary-sim/status?cache_bust=${cacheBuster}`, {
    method: "GET",
    headers: {
      ...buildAdminAuthHeaders(ip),
      "Cache-Control": "no-store",
      Pragma: "no-cache"
    },
    cache: "no-store"
  });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`adversary sim status read should succeed: ${response.status} ${body}`);
  }
  return response.json();
}

function adversarySimStatusState(payload) {
  const source = payload && typeof payload === "object" ? payload : {};
  return {
    enabled:
      source.adversary_sim_enabled === true ||
      source.enabled === true,
    generationActive:
      source.generation_active === true ||
      source.generationActive === true,
    phase: String(source.phase || "off").trim().toLowerCase(),
    desiredLane: String(source.desired_lane || source.desiredLane || "scrapling_traffic")
      .trim()
      .toLowerCase() || "scrapling_traffic"
  };
}

async function waitForAdversarySimControllerLeaseExpiry(request, timeoutMs = 30000, ip = "127.0.0.1") {
  const deadline = Date.now() + Math.max(1000, Number(timeoutMs || 0));
  while (Date.now() < deadline) {
    const payload = await fetchAdversarySimStatus(request, ip);
    const lease = payload?.controller_lease && typeof payload.controller_lease === "object"
      ? payload.controller_lease
      : null;
    const expiresAtSeconds = Number(lease?.expires_at || 0);
    const nowSeconds = Math.floor(Date.now() / 1000);
    if (!lease || !Number.isFinite(expiresAtSeconds) || expiresAtSeconds <= nowSeconds) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`adversary sim controller lease did not expire within ${timeoutMs}ms`);
}

async function waitForAdversarySimEnabledState(request, desiredEnabled, timeoutMs = 30000, ip = "127.0.0.1") {
  const desired = desiredEnabled === true;
  const deadline = Date.now() + Math.max(1000, Number(timeoutMs || 0));
  let lastState = adversarySimStatusState({});
  let consecutiveSettledPolls = 0;
  while (Date.now() < deadline) {
    lastState = adversarySimStatusState(await fetchAdversarySimStatus(request, ip));
    if (desired) {
      if (lastState.enabled === true) {
        return lastState;
      }
      consecutiveSettledPolls = 0;
    } else if (lastState.enabled !== true && lastState.generationActive !== true && lastState.phase === "off") {
      consecutiveSettledPolls += 1;
      if (consecutiveSettledPolls >= 3) {
        return lastState;
      }
    } else {
      consecutiveSettledPolls = 0;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(
    `adversary sim did not settle to enabled=${desired} within ${timeoutMs}ms (last_state=${JSON.stringify(lastState)})`
  );
}

async function waitForDashboardAdversarySimUiState(
  page,
  request,
  desiredEnabled,
  timeoutMs = 30000,
  ip = "127.0.0.1"
) {
  const desired = desiredEnabled === true;
  await waitForAdversarySimEnabledState(request, desired, timeoutMs, ip);

  await waitForDashboardAdversarySimUiConvergence(page, desired, timeoutMs);
}

async function waitForDashboardAdversarySimUiConvergence(page, desiredEnabled, timeoutMs = 30000) {
  const desired = desiredEnabled === true;

  const toggle = page.locator("#global-adversary-sim-toggle");
  if (desired) {
    await expect(toggle).toBeChecked({ timeout: timeoutMs });
  } else {
    await expect(toggle).not.toBeChecked({ timeout: timeoutMs });
  }

  await expect
    .poll(async () => {
      const state = await dashboardDomClassState(page);
      return state.hasAdversarySim === true;
    }, { timeout: timeoutMs })
    .toBe(desired);
}

function adversarySimStateMatchesDesired(state, desiredEnabled) {
  const desired = desiredEnabled === true;
  if (desired) {
    return state.enabled === true;
  }
  return state.enabled !== true && state.generationActive !== true && state.phase === "off";
}

async function controlAdversarySimViaAdmin(
  request,
  desiredEnabled,
  ip = "127.0.0.1",
  timeoutMs = 95_000,
  controlOptions = {}
) {
  const deadline = Date.now() + Math.max(5_000, Number(timeoutMs || 0));
  let lastStatus = 0;
  let lastBody = "";
  while (Date.now() < deadline) {
    const payload = { enabled: desiredEnabled === true };
    const desiredLane = String(controlOptions?.lane || "").trim().toLowerCase();
    if (desiredLane) {
      payload.lane = desiredLane;
    }
    const response = await request.post(`${BASE_URL}/admin/adversary-sim/control`, {
      headers: {
        ...buildAdminAuthHeaders(ip),
        "Content-Type": "application/json",
        "Idempotency-Key": newDashboardIdempotencyKey(),
        Origin: BASE_URL,
        "Sec-Fetch-Site": "same-origin"
      },
      data: payload
    });
    if (response.ok()) {
      return response.json();
    }
    lastStatus = response.status();
    lastBody = await response.text();
    if (lastStatus === 409 || lastStatus === 429) {
      const fallbackDelayMs = lastStatus === 409 ? 2_000 : 1_100;
      await new Promise((resolve) => setTimeout(resolve, controlRetryDelayMs(response, fallbackDelayMs)));
      continue;
    }
    throw new Error(`adversary sim control should succeed: ${lastStatus} ${lastBody}`);
  }
  throw new Error(
    `adversary sim control should succeed before timeout: last_status=${lastStatus} body=${lastBody}`
  );
}

async function forceAdversarySimDisabled(
  request,
  ip = "127.0.0.1",
  desiredLane = "scrapling_traffic"
) {
  const timeoutMs = 95_000;
  const deadline = Date.now() + timeoutMs;
  let lastState = adversarySimStatusState({});
  let lastControlError = "";
  let consecutiveSettledPolls = 0;
  let nextControlAttemptAt = 0;
  const normalizedDesiredLane = String(desiredLane || "").trim().toLowerCase();
  const shouldNormalizeLane = normalizedDesiredLane.length > 0;

  await updateAdminConfig(request, {
    adversary_sim_duration_seconds: 180
  }, ip);

  while (Date.now() < deadline) {
    lastState = adversarySimStatusState(await fetchAdversarySimStatus(request, ip));
    const desiredLaneSettled =
      shouldNormalizeLane !== true || lastState.desiredLane === normalizedDesiredLane;
    if (
      lastState.enabled !== true &&
      lastState.generationActive !== true &&
      lastState.phase === "off" &&
      desiredLaneSettled
    ) {
      consecutiveSettledPolls += 1;
      if (consecutiveSettledPolls >= 3) {
        return lastState;
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
      continue;
    }

    consecutiveSettledPolls = 0;
    if (Date.now() >= nextControlAttemptAt) {
      try {
        await controlAdversarySimViaAdmin(
          request,
          false,
          ip,
          Math.max(5_000, Math.min(20_000, deadline - Date.now())),
          shouldNormalizeLane ? { lane: normalizedDesiredLane } : {}
        );
        lastControlError = "";
      } catch (error) {
        lastControlError = error && typeof error.message === "string"
          ? error.message.trim()
          : String(error || "").trim();
      }
      nextControlAttemptAt = Date.now() + 2_000;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  throw new Error(
    `adversary sim disable cleanup did not settle within ${timeoutMs}ms ` +
    `(last_state=${JSON.stringify(lastState)}${lastControlError ? ` last_control_error=${lastControlError}` : ""})`
  );
}

async function withRestoredAdversarySimConfig(request, callback, ip = "127.0.0.1") {
  const restoreStatus = adversarySimStatusState(await fetchAdversarySimStatus(request, ip));
  const restoreDesiredEnabled = restoreStatus.enabled === true;
  const restoreDesiredLane = restoreStatus.desiredLane;
  const restorePatch = extractConfigPatchFromPaths(
    await fetchAdminConfig(request, ip),
    ADVERSARY_SIM_RESTORE_PATHS
  );
  try {
    return await callback(restorePatch);
  } finally {
    if (Object.keys(restorePatch).length === 0) return;
    await updateAdminConfig(request, restorePatch, ip);
    await controlAdversarySimViaAdmin(
      request,
      restoreDesiredEnabled,
      ip,
      95_000,
      { lane: restoreDesiredLane }
    );
    await waitForAdversarySimEnabledState(request, restoreDesiredEnabled, 30000, ip);
    const restoredPatch = extractConfigPatchFromPaths(await fetchAdminConfig(request, ip), ADVERSARY_SIM_RESTORE_PATHS);
    expect(restoredPatch).toEqual(restorePatch);
  }
}

async function waitForDashboardSessionAuthenticated(page, timeoutMs = 10000) {
  await page.waitForFunction(async () => {
    try {
      const response = await fetch("/admin/session", {
        method: "GET",
        credentials: "same-origin",
        cache: "no-store"
      });
      if (!response.ok) return false;
      const payload = await response.json();
      const runtimeEnvironment = String(payload?.runtime_environment || "").trim().toLowerCase();
      return payload?.authenticated === true &&
        (runtimeEnvironment === "runtime-dev" || runtimeEnvironment === "runtime-prod");
    } catch (_error) {
      return false;
    }
  }, { timeout: timeoutMs });
}

function expectDashboardRuntimeClass(bodyState, runtimeEnvironment) {
  expect(bodyState.runtimeClassCount).toBe(1);
  expect(bodyState.hasRuntimeDev).toBe(runtimeEnvironment === "runtime-dev");
  expect(bodyState.hasRuntimeProd).toBe(runtimeEnvironment === "runtime-prod");
}

async function bootstrapDashboardSession(page, targetUrl) {
  const nextPath = dashboardRelativePath(targetUrl);
  const loginUrl = `${BASE_URL}/dashboard/login.html?next=${encodeURIComponent(nextPath)}`;
  const maxAttempts = 4;
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    await page.goto(loginUrl);
    await page.waitForSelector("#login-form", { timeout: 10000 });
    await page.fill("#current-password", API_KEY);
    await page.click("#login-submit");

    await page.waitForFunction(() => {
      const path = window.location.pathname;
      if (path.endsWith("/dashboard/index.html")) {
        return true;
      }
      const message = (document.getElementById("login-msg")?.textContent || "").trim();
      return message.length > 0;
    }, { timeout: 8000 }).catch(() => {});

    let failureMessage = "";
    if (page.url().includes("/dashboard/login.html")) {
      failureMessage = (await page.locator("#login-msg").textContent().catch(() => "") || "").trim();
    } else {
      await page.goto(targetUrl);
      await page.waitForTimeout(200);
      if (!isDashboardLoginUrl(page.url())) {
        await waitForDashboardSessionAuthenticated(page);
        return;
      }
      failureMessage = "redirected back to login after successful submit";
    }

    const retryMatch = /retry in\s+(\d+)s/i.exec(failureMessage);
    const retryAfterSeconds = retryMatch ? Number.parseInt(retryMatch[1], 10) : Number.NaN;
    const backoffMs = Number.isFinite(retryAfterSeconds) && retryAfterSeconds > 0
      ? retryAfterSeconds * 1000
      : 600 * (attempt + 1);

    if (attempt < (maxAttempts - 1)) {
      await page.waitForTimeout(backoffMs);
      continue;
    }

    throw new Error(
      `Dashboard login bootstrap failed after ${maxAttempts} attempts: ${failureMessage || "no login error message"}`
    );
  }
}

async function openDashboard(page, options = {}) {
  const initialTab = typeof options.initialTab === "string" ? options.initialTab : "game-loop";
  const targetUrl = `${BASE_URL}/dashboard/index.html#${initialTab}`;
  ensureRuntimeGuard(page);
  let lastError = null;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    clearRuntimeFailures(page);
    try {
      await page.goto(targetUrl);
      await page.waitForTimeout(250);
      if (isDashboardLoginUrl(page.url())) {
        await bootstrapDashboardSession(page, targetUrl);
        await expect(page).toHaveURL(/\/dashboard\/index\.html/);
      }
      if (!page.url().endsWith(`#${initialTab}`)) {
        await page.evaluate((tab) => {
          window.location.hash = tab;
        }, initialTab);
        await expect(page).toHaveURL(new RegExp(`#${initialTab}$`));
      }
      await waitForDashboardSessionAuthenticated(page, 15000);
      await page.waitForSelector("#logout-btn", { timeout: 15000 });
      await expect(page.locator("#logout-btn")).toBeEnabled();
      if (initialTab === "game-loop") {
        await page.waitForFunction(() => {
          const total = document.getElementById("total-events")?.textContent?.trim();
          return Boolean(total && total !== "-" && total !== "...");
        }, { timeout: 15000 });
      }
      await assertActiveTabPanelVisibility(page, initialTab);
      assertNoRuntimeFailures(page);
      return;
    } catch (error) {
      lastError = error;
      if (attempt === 0 && isDashboardLoginUrl(page.url())) {
        clearRuntimeFailures(page);
        await bootstrapDashboardSession(page, targetUrl);
        await page.waitForTimeout(250);
        continue;
      }
      if (attempt === 0 && hasOnlyTransientStaticDisconnectFailures(page)) {
        clearRuntimeFailures(page);
        await page.waitForTimeout(1000);
        continue;
      }
      throw error;
    }
  }
  throw lastError;
}

async function openTab(page, tab, options = {}) {
  const waitForReady = options.waitForReady === true;
  await page.click(`#dashboard-tab-${tab}`);
  await expect(page).toHaveURL(new RegExp(`#${tab}$`));
  await assertActiveTabPanelVisibility(page, tab);
  if (waitForReady && ADMIN_TABS.includes(tab)) {
    await page.waitForFunction((tabName) => {
      const state = document.querySelector(`[data-tab-state="${tabName}"]`);
      if (!state) return true;
      const text = (state.textContent || "").trim();
      return !/^loading/i.test(text);
    }, tab, { timeout: 15000 });
  }
  assertNoRuntimeFailures(page);
}

async function setAutoRefresh(page, enabled) {
  const toggle = page.locator("#auto-refresh-toggle");
  const toggleSwitch = page.locator('label[for="auto-refresh-toggle"]');
  await expect(toggleSwitch).toBeVisible();
  if ((await toggle.isChecked()) !== enabled) {
    await toggleSwitch.click();
  }
  if (enabled) {
    await expect(toggle).toBeChecked();
  } else {
    await expect(toggle).not.toBeChecked();
  }
}

async function clearDashboardClientCache(page) {
  await page.evaluate(() => {
    const keys = [
      "shuma_dashboard_cache_monitoring_v2",
      "shuma_dashboard_cache_ip_bans_v1",
      "shuma_dashboard_auto_refresh_v1"
    ];
    try {
      keys.forEach((key) => window.localStorage.removeItem(key));
    } catch (_error) {}
  });
}

async function clickAdversaryToggleWithRetry(page, desiredEnabled, timeoutMs = 60000, request = null) {
  const toggle = page.locator("#global-adversary-sim-toggle");
  const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
  await expect(toggle).toBeEnabled({ timeout: timeoutMs });
  const desired = desiredEnabled === true;
  const settleDesiredUiState = async () => {
    if (request) {
      await waitForDashboardAdversarySimUiState(page, request, desired, timeoutMs);
    } else {
      await waitForDashboardAdversarySimUiConvergence(page, desired, timeoutMs);
    }
    return { requested_enabled: desired };
  };
  if ((await toggle.isChecked()) === desired) {
    return settleDesiredUiState();
  }

  const deadline = Date.now() + Math.max(2000, Number(timeoutMs || 0));
  let lastStatus = 0;
  while (Date.now() < deadline) {
    if ((await toggle.isChecked()) === desired) {
      return settleDesiredUiState();
    }

    const maybeDialog = page.waitForEvent("dialog", { timeout: 2500 }).then(async (dialog) => {
      await dialog.accept();
      return true;
    }).catch(() => false);

    const responsePromise = page.waitForResponse((resp) => (
      resp.url().includes("/admin/adversary-sim/control") &&
      resp.request().method() === "POST"
    ), { timeout: 6000 });

    await Promise.all([
      maybeDialog,
      toggleSwitch.click()
    ]);

    const response = await responsePromise;
    lastStatus = response.status();
    if (lastStatus === 200) {
      const payload = await response.json();
      const backendState = adversarySimStatusState(payload && typeof payload === "object" ? payload.status : {});
      if (request && !adversarySimStateMatchesDesired(backendState, desired)) {
        await waitForDashboardAdversarySimUiState(page, request, desired, timeoutMs);
      } else {
        await waitForDashboardAdversarySimUiConvergence(page, desired, timeoutMs);
      }
      return payload && typeof payload === "object"
        ? payload
        : { requested_enabled: desired };
    }
    if (lastStatus === 429 || lastStatus === 409) {
      await page.waitForTimeout(controlRetryDelayMs(response));
      continue;
    }
    const body = await response.text().catch(() => "");
    throw new Error(`adversary toggle control request failed with ${lastStatus}: ${body}`);
  }

  throw new Error(
    `adversary toggle did not reach desired state=${desired} before timeout; last_status=${lastStatus}`
  );
}

function controlRetryDelayMs(response, fallbackMs = 1100) {
  const fallback = Math.max(250, Number(fallbackMs || 0));
  try {
    const retryAfterRaw = response?.headers?.()["retry-after"];
    const retryAfterSeconds = Number.parseInt(String(retryAfterRaw || "").trim(), 10);
    if (Number.isFinite(retryAfterSeconds) && retryAfterSeconds > 0) {
      return retryAfterSeconds * 1000 + 250;
    }
  } catch (_error) {}
  return fallback;
}

function buildAdminAuthHeaders(ip = "127.0.0.1") {
  return {
    Authorization: `Bearer ${API_KEY}`,
    ...buildTrustedForwardingHeaders(ip)
  };
}

async function updateAdminConfig(request, patch, ip = "127.0.0.1") {
  const response = await request.post(`${BASE_URL}/admin/config`, {
    headers: {
      ...buildAdminAuthHeaders(ip),
      "Content-Type": "application/json"
    },
    data: patch
  });
  if (!response.ok()) {
    const errorBody = await response.text();
    throw new Error(`admin config update should succeed: ${response.status()} ${errorBody}`);
  }
  const payload = await response.json();
  return payload && payload.config ? payload.config : {};
}

async function fetchFrontierProviderCount(request, ip = "127.0.0.1") {
  const payload = await fetchAdminConfigRuntime(request, ip);
  const count = Number(payload?.frontier_provider_count || 0);
  return Number.isFinite(count) ? count : 0;
}

async function fetchMonitoringSnapshot(request, hours = 24, limit = 200, ip = "127.0.0.1") {
  const response = await request.get(`${BASE_URL}/admin/monitoring?hours=${hours}&limit=${limit}`, {
    headers: buildAdminAuthHeaders(ip)
  });
  if (!response.ok()) {
    const body = await response.text();
    throw new Error(`monitoring read should succeed: ${response.status()} ${body}`);
  }
  return response.json();
}

function maxSimulationEventTs(monitoringPayload) {
  const rows = Array.isArray(monitoringPayload?.details?.events?.recent_events)
    ? monitoringPayload.details.events.recent_events
    : [];
  return rows.reduce((maxTs, row) => {
    if (!row || row.is_simulation !== true) return maxTs;
    const ts = Number(row.ts || 0);
    return Number.isFinite(ts) ? Math.max(maxTs, ts) : maxTs;
  }, 0);
}

async function waitForSimulationEventAdvance(request, baselineTs, timeoutMs = 20000) {
  const deadline = Date.now() + Math.max(1000, Number(timeoutMs || 0));
  let lastObserved = Number(baselineTs || 0);
  while (Date.now() < deadline) {
    const monitoring = await fetchMonitoringSnapshot(request, 24, 200);
    const observed = maxSimulationEventTs(monitoring);
    lastObserved = Math.max(lastObserved, observed);
    if (observed > baselineTs) {
      return observed;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(
    `simulation telemetry did not advance within ${timeoutMs}ms (baseline=${baselineTs}, last_observed=${lastObserved})`
  );
}

async function submitConfigSave(page, button = page.locator("#save-verification-all")) {
  await expect(button).toBeEnabled();
  const [response] = await Promise.all([
    page.waitForResponse((resp) => (
      resp.url().includes("/admin/config") &&
      resp.request().method() === "POST"
    )),
    button.click()
  ]);
  if (!response.ok()) {
    const body = await response.text();
    throw new Error(`Config save failed with ${response.status()}: ${body}`);
  }
}

async function assertChartsFillPanels(page) {
  const metrics = await page.evaluate(() => {
    const ids = ["eventTypesChart", "topIpsChart", "timeSeriesChart"];
    return ids.map((id) => {
      const canvas = document.getElementById(id);
      const panel = canvas ? canvas.closest(".chart-container") : null;
      if (!canvas || !panel) {
        return { id, missing: true };
      }
      const canvasRect = canvas.getBoundingClientRect();
      const panelRect = panel.getBoundingClientRect();
      return {
        id,
        missing: false,
        canvasWidth: canvasRect.width,
        canvasHeight: canvasRect.height,
        panelWidth: panelRect.width
      };
    });
  });

  for (const metric of metrics) {
    expect(metric.missing, `${metric.id} should exist in a chart panel`).toBe(false);
    expect(metric.canvasWidth, `${metric.id} should fill most of panel width`).toBeGreaterThan(
      metric.panelWidth * 0.8
    );
    expect(metric.canvasHeight, `${metric.id} should have non-squashed height`).toBeGreaterThan(170);
  }
}

test.beforeAll(async () => {
  ensureRequiredEnv();
  await seedDashboardData();
});

test.beforeEach(async ({ page }) => {
  await page.context().setExtraHTTPHeaders(buildTrustedForwardingHeaders());
});

test.afterEach(async ({ page }) => {
  assertNoRuntimeFailures(page);
});

test("dashboard bare path redirects to canonical index route", async ({ request }) => {
  const response = await request.get(`${BASE_URL}/dashboard`, { maxRedirects: 0 });
  expect(response.status()).toBe(308);
  expect(response.headers().location).toBe("/dashboard/index.html");
});

test("sveltekit assets resolve under /dashboard/_app and /dashboard/assets base paths", async ({ page }) => {
  await openDashboard(page);
  const assets = await page.evaluate(() => {
    const modulePreloads = Array.from(document.querySelectorAll("link[rel='modulepreload'][href]"))
      .map((node) => new URL(node.getAttribute("href"), window.location.href).pathname);
    const scripts = Array.from(document.querySelectorAll("script[src]"))
      .map((node) => new URL(node.getAttribute("src"), window.location.href).pathname);
    const styles = Array.from(document.querySelectorAll("link[rel='stylesheet'][href]"))
      .map((node) => new URL(node.getAttribute("href"), window.location.href).pathname);
    const shumaImage = document.querySelector("img.shuma-gorath-img");
    const shumaImagePath = shumaImage
      ? new URL(shumaImage.getAttribute("src") || "", window.location.href).pathname
      : "";
    const shumaImageComplete = Boolean(shumaImage && shumaImage.complete);
    const shumaImageNaturalWidth = Number(shumaImage?.naturalWidth || 0);
    return {
      modulePreloads,
      scripts,
      styles,
      shumaImagePath,
      shumaImageComplete,
      shumaImageNaturalWidth
    };
  });

  expect(assets.modulePreloads.some((path) => path.startsWith("/dashboard/_app/"))).toBe(true);
  expect(assets.styles.some((path) => path.startsWith("/dashboard/_app/"))).toBe(true);
  expect(
    assets.scripts.some((path) => path.includes("/assets/vendor/chart-lite"))
  ).toBe(false);
  expect(assets.shumaImagePath).toBe("/dashboard/assets/shuma-gorath-pencil.png");
  expect(assets.shumaImageComplete).toBe(true);
  expect(assets.shumaImageNaturalWidth).toBeGreaterThan(0);
});

test("dashboard login route remains functional after direct navigation and refresh", async ({ page }) => {
  ensureRuntimeGuard(page);
  await page.goto(`${BASE_URL}/dashboard/login.html?next=%2Fdashboard%2Findex.html`);
  await expect(page.locator("#login-form")).toBeVisible();
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute(
    "href",
    /\/dashboard\/assets\/shuma-gorath-pencil-closed\.png$/
  );
  await expect(page.locator("#login-form")).toHaveAttribute("method", "POST");
  await expect(page.locator("#login-form")).toHaveAttribute("action", "/admin/login");
  await expect(page.locator('label.control-label[for="username"]')).toHaveText("Account");
  await expect(page.locator('input#username[name="username"]')).toHaveValue("admin");
  await expect(page.locator('#username')).toHaveAttribute("readonly", "");
  await expect(page.locator('input[type="hidden"][name="next"]')).toHaveValue("/dashboard/index.html");
  await expect(page.locator("#current-password")).toHaveAttribute("name", "password");
  await expect(page.locator("#current-password")).toHaveAttribute("autocomplete", "current-password");
  await expect(page.locator("#current-password")).toBeFocused();
  await page.reload();
  await expect(page.locator("#login-form")).toBeVisible();
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute(
    "href",
    /\/dashboard\/assets\/shuma-gorath-pencil-closed\.png$/
  );
  await expect(page.locator("#current-password")).toBeFocused();
  await page.fill("#current-password", API_KEY);
  await page.click("#login-submit");
  await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  assertNoRuntimeFailures(page);
});

test("dashboard trailing-slash root preserves operator login flow", async ({ page }) => {
  ensureRuntimeGuard(page);
  await page.goto(`${BASE_URL}/dashboard/`);
  await expect(page).toHaveURL(/\/dashboard\/login\.html\?next=/);
  await expect(page.locator("#login-form")).toBeVisible();
  await page.fill("#current-password", API_KEY);
  await page.click("#login-submit");
  await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  await waitForDashboardSessionAuthenticated(page, 15000);
  assertNoRuntimeFailures(page);
});

test("logged-out dashboard navigation keeps the auth gate visible until redirect", async ({ page }) => {
  ensureRuntimeGuard(page);

  let releaseSessionResponse;
  const sessionResponseReleased = new Promise((resolve) => {
    releaseSessionResponse = resolve;
  });

  await page.route("**/admin/session", async (route) => {
    await sessionResponseReleased;
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      headers: {
        "Cache-Control": "no-store"
      },
      body: JSON.stringify({
        authenticated: false,
        method: "none",
        csrf_token: null,
        access: "none",
        expires_at: null,
        runtime_environment: "runtime-dev"
      })
    });
  });

  await page.goto(`${BASE_URL}/dashboard/index.html`);

  await expect(page.locator("#dashboard-auth-gate")).toBeAttached();
  await expect(page.locator("nav.dashboard-tabs")).toHaveCount(0);
  await expect(page.locator("#dashboard-panel-game-loop")).toHaveCount(0);
  await expect(page.locator("#dashboard-auth-gate")).toHaveText("");
  await expect
    .poll(async () => {
      const rootClassList = await page.locator("html").evaluate((node) => Array.from(node.classList));
      return rootClassList.includes("disconnected");
    })
    .toBe(true);

  releaseSessionResponse();

  await expect(page).toHaveURL(/\/dashboard\/login\.html\?next=/);
  await expect(page.locator("#login-form")).toBeVisible();
  assertNoRuntimeFailures(page);
});

test("not-a-bot browser lifecycle captures telemetry and rejects replayed submit", async ({ page, request }) => {
  const configHeaders = buildAdminAuthHeaders("127.0.0.1");
  const currentConfigResponse = await request.get(`${BASE_URL}/admin/config`, {
    headers: configHeaders
  });
  expect(currentConfigResponse.ok()).toBe(true);
  const currentConfigEnvelope = await currentConfigResponse.json();
  const currentConfig =
    currentConfigEnvelope && typeof currentConfigEnvelope === "object" && currentConfigEnvelope.config
      ? currentConfigEnvelope.config
      : {};
  const originalTestMode = currentConfig && currentConfig.shadow_mode === true;
  const originalNotABotEnabled = currentConfig && currentConfig.not_a_bot_enabled !== false;
  const originalNotABotPassMin = Number.isFinite(currentConfig?.not_a_bot_pass_score)
    ? currentConfig.not_a_bot_pass_score
    : 7;
  const originalNotABotFailScore = Number.isFinite(currentConfig?.not_a_bot_fail_score)
    ? currentConfig.not_a_bot_fail_score
    : 4;
  const originalNotABotAttemptLimit = Number.isFinite(currentConfig?.not_a_bot_attempt_limit_per_window)
    ? currentConfig.not_a_bot_attempt_limit_per_window
    : 6;
  const originalNotABotAttemptWindow = Number.isFinite(currentConfig?.not_a_bot_attempt_window_seconds)
    ? currentConfig.not_a_bot_attempt_window_seconds
    : 300;

  await updateAdminConfig(request, {
    shadow_mode: true,
    not_a_bot_enabled: true,
    not_a_bot_pass_score: 2,
    not_a_bot_fail_score: 1,
    not_a_bot_attempt_limit_per_window: 100,
    not_a_bot_attempt_window_seconds: 300
  });

  try {
    ensureRuntimeGuard(page);
    let submitOutcome = null;
    for (let attempt = 0; attempt < 2; attempt += 1) {
      await page.goto(`${BASE_URL}/challenge/not-a-bot-checkbox`);
      const notABotCheckbox = page.locator("#not-a-bot-checkbox");
      await expect(notABotCheckbox).toBeVisible();
      await expect(notABotCheckbox).toHaveAttribute("role", "checkbox");
      // Keep dwell above the signed operation envelope minimum latency.
      await page.waitForTimeout(1200 + (attempt * 300));

      const submitRequestPromise = page.waitForRequest((req) =>
        req.method() === "POST" && req.url().includes("/challenge/not-a-bot-checkbox")
      );
      const submitResponsePromise = page.waitForResponse((resp) =>
        resp.request().method() === "POST" && resp.url().includes("/challenge/not-a-bot-checkbox")
      );

      await notABotCheckbox.click();

      const submitRequest = await submitRequestPromise;
      const submitResponse = await submitResponsePromise;
      const submitStatus = submitResponse.status();
      submitOutcome = {
        status: submitStatus,
        location: submitResponse.headers()["location"] || "",
        formBody: submitRequest.postData() || "",
        responseBody: submitStatus === 200 ? await submitResponse.text() : ""
      };
      if (submitOutcome.status === 303 || submitOutcome.status === 200) {
        break;
      }
    }

    if (!submitOutcome || (submitOutcome.status !== 303 && submitOutcome.status !== 200)) {
      throw new Error(
        `not-a-bot submit did not produce expected outcome (status=${submitOutcome ? submitOutcome.status : "none"}) body=${submitOutcome ? submitOutcome.responseBody : ""}`
      );
    }
    if (submitOutcome.status === 200) {
      expect(
        submitOutcome.responseBody.includes("maze-nav-grid")
          || submitOutcome.responseBody.includes("data-link-kind=\"maze\"")
      ).toBe(true);
    } else {
      expect(submitOutcome.location.length > 0).toBe(true);
    }

    const formBody = submitOutcome.formBody;
    expect(formBody.includes("checked=1")).toBe(true);
    expect(formBody.includes("telemetry=")).toBe(true);

    const replayResponse = await request.fetch(`${BASE_URL}/challenge/not-a-bot-checkbox`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded"
      },
      data: formBody,
      maxRedirects: 0
    });
    const replayResult = {
      status: replayResponse.status(),
      location: replayResponse.headers()["location"] || ""
    };
    expect(replayResult.status).not.toBe(303);
    expect(replayResult.location).not.toBe("/");
    assertNoRuntimeFailures(page);
  } finally {
    await updateAdminConfig(request, {
      shadow_mode: originalTestMode,
      not_a_bot_enabled: originalNotABotEnabled,
      not_a_bot_pass_score: originalNotABotPassMin,
      not_a_bot_fail_score: originalNotABotFailScore,
      not_a_bot_attempt_limit_per_window: originalNotABotAttemptLimit,
      not_a_bot_attempt_window_seconds: originalNotABotAttemptWindow
    });
  }
});

test("dashboard generated runtime has no missing script or stylesheet requests", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "status");
  await openTab(page, "verification");
  await openTab(page, "fingerprinting");
  await openTab(page, "tuning");
  await openTab(page, "ip-bans");
  await openTab(page, "diagnostics");

  const failures = runtimeFailures(page).filter((entry) =>
    entry.includes("requestfailed") || entry.includes("asset-response")
  );
  expect(failures).toEqual([]);
});

test("dashboard clean-state renders explicit empty placeholders", async ({ page }) => {
  const emptyConfig = {
    admin_config_write_enabled: true,
    pow_enabled: true,
    challenge_puzzle_enabled: true,
    challenge_puzzle_transform_count: 6,
    challenge_puzzle_risk_threshold: 3,
    challenge_puzzle_risk_threshold_default: 3,
    botness_maze_threshold: 6,
    botness_maze_threshold_default: 6,
    botness_weights: {
      js_required: 1,
      geo_risk: 2,
      rate_medium: 1,
      rate_high: 2
    },
    ban_durations: {
      honeypot: 86400,
      ip_range_honeypot: 86400,
      maze_crawler: 86400,
      rate_limit: 3600,
      cdp: 43200,
      edge_fingerprint: 43200,
      tarpit_persistence: 600,
      not_a_bot_abuse: 600,
      challenge_puzzle_abuse: 600,
      admin: 21600
    },
    honeypot_enabled: true,
    honeypots: ["/instaban"],
    browser_block: [["Chrome", 120], ["Firefox", 115], ["Safari", 15]],
    browser_allowlist: [],
    allowlist: [],
    path_allowlist: [],
    maze_enabled: true,
    maze_threshold: 50,
    maze_auto_ban: false,
    robots_enabled: true,
    ai_robots_block: true,
    ai_robots_aggressive: false,
    ai_robots_content_signal: true,
    robots_crawl_delay: 2,
    cdp_enabled: true,
    cdp_mode: "report-only",
    cdp_score_threshold: 0.8,
    cdp_auto_ban: false,
    cdp_auto_ban_threshold: 0.9,
    rate_limit: 80,
    js_required_enforced: true,
    shadow_mode: false,
    kv_store_fail_open: true,
    edge_integration_mode: "off",
    geo_risk: [],
    geo_allow: [],
    geo_challenge: [],
    geo_maze: [],
    geo_block: []
  };
  const emptyConfigRuntime = {
    admin_config_write_enabled: true,
    kv_store_fail_open: true,
    runtime_environment: "runtime-dev",
    gateway_deployment_profile: "shared-server",
    akamai_edge_available: false,
    local_prod_direct_mode: false,
    adversary_sim_available: true,
    adversary_sim_enabled: false,
    frontier_mode: "disabled",
    frontier_provider_count: 0,
    frontier_diversity_confidence: "none",
    frontier_reduced_diversity_warning: false,
    frontier_providers: [],
    challenge_puzzle_risk_threshold_default: 3,
    not_a_bot_risk_threshold_default: 2,
    botness_maze_threshold_default: 6,
    defence_modes_effective: {},
    defence_mode_warnings: [],
    enterprise_multi_instance: false,
    enterprise_unsynced_state_exception_confirmed: false,
    enterprise_state_guardrail_warnings: [],
    enterprise_state_guardrail_error: null,
    botness_signal_definitions: {
      scored_signals: [],
      terminal_signals: []
    }
  };

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        summary: {
          honeypot: { total_hits: 0, unique_crawlers: 0, top_crawlers: [], top_paths: [] },
          challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
          pow: {
            total_failures: 0,
            total_successes: 0,
            total_attempts: 0,
            success_ratio: 0,
            unique_offenders: 0,
            top_offenders: [],
            reasons: {},
            outcomes: {},
            trend: []
          },
          rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
          geo: { total_violations: 0, actions: { block: 0, challenge: 0, maze: 0 }, top_countries: [] }
        },
        prometheus: { endpoint: "/metrics", notes: [] },
        details: {
          analytics: { ban_count: 0, shadow_mode: false, fail_mode: "open" },
          events: { recent_events: [], event_counts: {}, top_ips: [], unique_ips: 0 },
          bans: { bans: [] },
          maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
          cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
          cdp_events: { events: [] }
        }
      })
    });
  });
  await page.route("**/admin/config", async (route) => {
    if (route.request().method() !== "GET") {
      await route.continue();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ config: emptyConfig, runtime: emptyConfigRuntime })
    });
  });
  await page.route("**/admin/ban", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ bans: [] })
    });
  });
  await page.route("**/admin/ip-range/suggestions?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        generated_at: 0,
        hours: 24,
        summary: {
          suggestions_total: 0,
          low_risk: 0,
          medium_risk: 0,
          high_risk: 0
        },
        suggestions: []
      })
    });
  });
  await page.route("**/admin/ip-bans/delta?*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        events: [],
        active_bans: [],
        next_cursor: "",
        window_end_cursor: "",
        has_more: false,
        overflow: "none"
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "traffic");
  await expect(page.locator("#total-events")).toHaveText("0");
  await expect(page.locator("#monitoring-events tbody")).toContainText(
    /No (recent events|events loaded while freshness is degraded\/stale)/i
  );

  await openTab(page, "diagnostics");
  await expect(page.locator("#cdp-events tbody")).toContainText(
    "No CDP detections or detection-triggered bans in the selected window"
  );
  await expect(page.locator("#maze-top-offender")).toHaveText("None");
  await expect(page.locator("#honeypot-top-paths")).toContainText("No honeypot path data yet");
  await expect(page.locator("#challenge-failure-reasons")).toContainText("No failures in window");
  await expect(page.locator("#pow-failure-reasons")).toContainText("No failures in window");
  await expect(page.locator("#pow-outcomes-list")).toContainText("No verify outcomes yet");
  await expect(page.locator("#pow-total-attempts")).toHaveText("0");
  await expect(page.locator("#rate-outcomes-list")).toContainText("No outcomes yet");
  await expect(page.locator("#geo-top-countries")).toContainText("No GEO violations yet");

  await openTab(page, "ip-bans");
  await expect(page.locator("#bans-table tbody")).toContainText("No active bans");
  await expect(page.locator('[data-tab-state="ip-bans"]')).toHaveText("");
});

test("game loop, traffic, and diagnostics tabs expose their ownership split", async ({ page }) => {
  await openDashboard(page);

  await expect(page.locator("#dashboard-tab-game-loop")).toHaveText("Game Loop");
  await expect(page.locator(".dashboard-tabs .dashboard-tab-link")).toHaveText([
    "Traffic",
    "IP Bans",
    "Red Team",
    "Game Loop",
    "Tuning",
    "Verification",
    "Traps",
    "Rate Limiting",
    "GEO",
    "Fingerprinting",
    "Policy",
    "Status",
    "Advanced",
    "Diagnostics"
  ]);
  await expect(page.locator("#dashboard-panel-game-loop")).not.toHaveClass(/admin-group/);
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-intro]')).toHaveCount(0);
  await expect(page.locator("#dashboard-panel-game-loop")).not.toContainText("Closed-Loop Accountability");
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="recent-rounds"]')
  ).toContainText("Recent Rounds");
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="adversary-cast"]')
  ).toContainText("Adversaries In This Round");
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="defence-cast"]')
  ).toContainText("Defences In This Round");
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="current-status"]')
  ).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="recent-loop-progress"]')
  ).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="outcome-frontier"]')
  ).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="change-judgment"]')
  ).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="pressure-sits"]')
  ).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="trust-and-blockers"]')
  ).toHaveCount(0);

  await openTab(page, "traffic");
  await expect(page.locator("#dashboard-panel-traffic")).not.toHaveClass(/admin-group/);
  await expect(page.locator('#dashboard-panel-traffic [data-traffic-intro]')).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-traffic [data-traffic-section="telemetry-health"]')
  ).not.toContainText("Traffic Telemetry Health");
  await expect(
    page.locator('#dashboard-panel-traffic [data-traffic-section="traffic-overview"]')
  ).not.toContainText("Traffic Overview");
  await expect(
    page.locator('#dashboard-panel-traffic [data-traffic-section="traffic-overview"]')
  ).not.toContainText("Inspect the bounded traffic summary");
  await expect(
    page.locator('#dashboard-panel-traffic [data-traffic-section="recent-events"]')
  ).toContainText("Recent Events");
  await expect(
    page.locator('#dashboard-panel-traffic [data-traffic-section="telemetry-health"]')
  ).toContainText("Freshness:");
  await expect(
    page.locator('#dashboard-panel-traffic [data-traffic-section="telemetry-health"]')
  ).toContainText("Read path:");
  await expect(page.locator('#dashboard-panel-traffic .section .section')).toHaveCount(0);

  await openTab(page, "diagnostics");
  await expect(page.locator("#dashboard-panel-diagnostics")).not.toHaveClass(/admin-group/);
  await expect(page.locator('#dashboard-panel-diagnostics [data-diagnostics-intro]')).toHaveCount(0);
  await expect(
    page.locator('#dashboard-panel-diagnostics [data-diagnostics-section="defense-breakdown"]')
  ).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-traffic .section-copy-block')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop .section-copy-block')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-diagnostics .section-copy-block')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-diagnostics .section .section')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop .section .section')).toHaveCount(0);
});

test("diagnostics tab keeps subsystem inspection surfaces after overview-rollup retirement", async ({ page }) => {
  await page.route("**/admin/monitoring?**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        summary: {
          honeypot: {
            total_hits: 4,
            unique_crawlers: 2,
            top_crawlers: [{ label: "crawler-1", count: 4 }],
            top_paths: [{ path: "/trap", count: 4 }]
          },
          challenge: {
            total_failures: 9,
            unique_offenders: 3,
            top_offenders: [{ label: "solver-1", count: 9 }],
            reasons: { incorrect: 6 },
            trend: []
          },
          not_a_bot: {
            served: 10,
            submitted: 8,
            pass: 6,
            escalate: 2,
            fail: 1,
            replay: 1,
            abandonments_estimated: 1,
            solve_latency_buckets: {},
            outcomes: {}
          },
          pow: {
            total_failures: 2,
            total_successes: 6,
            total_attempts: 8,
            success_ratio: 0.75,
            unique_offenders: 2,
            top_offenders: [{ label: "pow-1", count: 6 }],
            reasons: {},
            outcomes: {},
            trend: []
          },
          rate: {
            total_violations: 5,
            unique_offenders: 2,
            top_offenders: [{ label: "ratelimit-1", count: 5 }],
            outcomes: { limited: 5 }
          },
          geo: {
            total_violations: 3,
            actions: { block: 1, challenge: 1, maze: 1 },
            top_countries: [["GB", 2]]
          }
        },
        details: {
          analytics: { ban_count: 0, shadow_mode: false, fail_mode: "open" },
          events: {
            recent_events: [
              {
                ts: Math.floor(Date.now() / 1000),
                event: "Challenge",
                ip: "198.51.100.44",
                reason: "challenge_reason_1",
                outcome: "served",
                execution_mode: "enforced"
              },
              {
                ts: Math.floor(Date.now() / 1000) - 1,
                event: "CDP Detection",
                ip: "198.51.100.45",
                reason: "cdp_detected",
                outcome: "ban",
                execution_mode: "enforced"
              }
            ],
            event_counts: { Challenge: 1, "CDP Detection": 1 },
            top_ips: [["198.51.100.44", 1], ["198.51.100.45", 1]],
            unique_ips: 2
          },
          bans: { bans: [] },
          maze: {
            total_hits: 12,
            unique_crawlers: 5,
            maze_auto_bans: 2,
            top_crawlers: [{ ip: "198.51.100.55", hits: 7 }]
          },
          cdp: {
            stats: { total_detections: 13, auto_bans: 2 },
            config: {},
            fingerprint_stats: { flow_violation: 4 }
          },
          cdp_events: { events: [] }
        },
        prometheus: { endpoint: "/metrics", notes: [] }
      })
    });
  });
  await page.route("**/admin/events?**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        recent_events: [
          {
            ts: Math.floor(Date.now() / 1000),
            event: "Challenge",
            ip: "198.51.100.44",
            reason: "challenge_reason_1",
            outcome: "served",
            execution_mode: "enforced"
          },
          {
            ts: Math.floor(Date.now() / 1000) - 1,
            event: "CDP Detection",
            ip: "198.51.100.45",
            reason: "cdp_detected",
            outcome: "ban",
            execution_mode: "enforced"
          }
        ],
        event_counts: { Challenge: 1, "CDP Detection": 1 },
        top_ips: [["198.51.100.44", 1], ["198.51.100.45", 1]],
        unique_ips: 2
      })
    });
  });
  await page.route("**/admin/cdp", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        stats: { total_detections: 13, auto_bans: 2 },
        config: {},
        fingerprint_stats: {
          ua_client_hint_mismatch: 3,
          ua_transport_mismatch: 2,
          flow_violation: 4
        }
      })
    });
  });
  await page.route("**/admin/cdp/events?**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ events: [] })
    });
  });
  await page.route("**/admin/config", async (route) => {
    if (route.request().method() !== "GET") {
      await route.continue();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        config: {
          ip_range_policy_mode: "enforce",
          ip_range_custom_rules: [{ enabled: true }],
          ip_range_emergency_allowlist: ["203.0.113.0/24"]
        },
        runtime: {}
      })
    });
  });
  await page.route("**/admin/ban", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ bans: [] })
    });
  });
  await page.route("**/admin/ip-range/suggestions?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        generated_at: 0,
        hours: 24,
        summary: {
          suggestions_total: 0,
          low_risk: 0,
          medium_risk: 0,
          high_risk: 0
        },
        suggestions: []
      })
    });
  });
  await page.route("**/admin/ip-bans/delta?*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        events: [],
        active_bans: [],
        next_cursor: "",
        window_end_cursor: "",
        has_more: false,
        overflow: "none"
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "diagnostics");
  await page.click("#refresh-now-btn");
  await expect(page.locator('#dashboard-panel-diagnostics [data-diagnostics-section="defense-breakdown"]')).toHaveCount(0);
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("CDP");
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("Maze");
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("Tarpit");
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("Rate");
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("IP Range");
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("Telemetry Diagnostics");
  await expect(page.locator("#dashboard-panel-diagnostics")).toContainText("External Monitoring");
});

test("game loop projects observer-facing round, adversary, and defence accountability from machine-first contracts", async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 1200 });
  await page.route("**/admin/operator-snapshot", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "operator_snapshot_v1",
        generated_at: 1774306800,
        objectives: {
          profile_id: "site_default_v1",
          revision: "objective-1774306800",
          window_hours: 24,
          category_postures: [
            {
              category_id: "indexing_bot",
              posture: "cost_reduced"
            },
            {
              category_id: "ai_scraper_bot",
              posture: "blocked"
            }
          ]
        },
        runtime_posture: {
          shadow_mode: false,
          fail_mode: "closed",
          runtime_environment: "runtime-prod",
          gateway_deployment_profile: "shared_server",
          adversary_sim_available: true
        },
        live_traffic: {
          traffic_origin: "live",
          execution_mode: "enforced",
          total_requests: 1200,
          forwarded_requests: 860,
          short_circuited_requests: 340,
          human_friction: {
            friction_rate: 0.018
          }
        },
        shadow_mode: {
          enabled: false,
          total_actions: 0,
          pass_through_total: 0
        },
        adversary_sim: {
          traffic_origin: "adversary_sim",
          execution_mode: "enforced",
          total_requests: 180,
          forwarded_requests: 9,
          short_circuited_requests: 171,
          recent_runs: [
            {
              run_id: "sim-42",
              lane: "scrapling_traffic",
              profile: "reference",
              observed_fulfillment_modes: ["bulk_scraper", "http_agent"],
              observed_category_ids: ["ai_scraper_bot"],
              first_ts: 1774306700,
              last_ts: 1774306750,
              monitoring_event_count: 64,
              defense_delta_count: 6,
              ban_outcome_count: 1,
              owned_surface_coverage: {
                overall_status: "partial",
                canonical_surface_ids: ["challenge_routing", "maze_navigation"],
                surface_labels: {
                  challenge_routing: "Challenge Routing",
                  maze_navigation: "Maze Navigation"
                },
                required_surface_ids: ["challenge_routing", "maze_navigation"],
                satisfied_surface_ids: ["challenge_routing"],
                blocking_surface_ids: ["maze_navigation"],
                receipts: [
                  {
                    surface_id: "challenge_routing",
                    success_contract: "mixed_outcomes",
                    dependency_kind: "independent",
                    dependency_surface_ids: [],
                    coverage_status: "pass_observed",
                    surface_state: "satisfied",
                    satisfied: true,
                    blocked_by_surface_ids: [],
                    attempt_count: 3,
                    sample_request_method: "GET",
                    sample_request_path: "/catalog?page=1",
                    sample_response_status: 200
                  },
                  {
                    surface_id: "maze_navigation",
                    success_contract: "should_pass_some",
                    dependency_kind: "independent",
                    dependency_surface_ids: [],
                    coverage_status: "unavailable",
                    surface_state: "unreached",
                    satisfied: false,
                    blocked_by_surface_ids: [],
                    attempt_count: 0,
                    sample_request_method: "",
                    sample_request_path: "",
                    sample_response_status: null
                  }
                ]
              }
            },
            {
              run_id: "sim-44",
              lane: "scrapling_traffic",
              profile: "reference",
              observed_fulfillment_modes: ["http_agent"],
              observed_category_ids: ["http_agent"],
              first_ts: 1774306755,
              last_ts: 1774306795,
              monitoring_event_count: 22,
              defense_delta_count: 3,
              ban_outcome_count: 0
            },
            {
              run_id: "sim-43",
              lane: "bot_red_team",
              profile: "reference",
              observed_fulfillment_modes: ["request_mode"],
              observed_category_ids: ["browser_agent"],
              first_ts: 1774306760,
              last_ts: 1774306790,
              monitoring_event_count: 28,
              defense_delta_count: 4,
              ban_outcome_count: 0,
              llm_runtime_summary: {
                category_targets: ["browser_agent", "agent_on_behalf_of_human"],
                generation_source: "provider_response",
                provider: "openai",
                model_id: "gpt-5-mini",
                generated_action_count: 3,
                executed_action_count: 2,
                failed_tick_count: 0,
                latest_action_receipts: [
                  {
                    action_type: "http_get",
                    path: "/pow/check",
                    status: 403
                  }
                ]
              }
            }
          ]
        },
        recent_changes: {
          rows: [
            {
              changed_at_ts: 1774303200,
              summary: "Enabled stricter fingerprint evidence for suspicious automation."
            }
          ]
        },
        non_human_traffic: {
          availability: "taxonomy_seeded",
          restriction_readiness: {
            status: "ready",
            blockers: [],
            live_receipt_count: 6,
            adversary_sim_receipt_count: 3
          },
          recognition_evaluation: {
            comparison_status: "mixed",
            current_exact_match_count: 1,
            degraded_match_count: 0,
            collapsed_to_unknown_count: 1,
            not_materialized_count: 0,
            simulator_ground_truth: {
              status: "observed_recent_runs",
              recent_sim_run_count: 3,
              categories: [
                {
                  category_id: "ai_scraper_bot",
                  category_label: "AI Scraper Bot",
                  recent_run_count: 1,
                  evidence_references: ["recent_sim_runs:sim-42:reference:ai_scraper_bot"]
                },
                {
                  category_id: "browser_agent",
                  category_label: "Browser Agent",
                  recent_run_count: 1,
                  evidence_references: ["recent_sim_runs:sim-43:reference:browser_agent"]
                }
              ]
            },
            readiness: {
              status: "partial",
              blockers: ["exact_category_inference_not_ready"],
              live_receipt_count: 6,
              adversary_sim_receipt_count: 3
            },
            coverage: {
              overall_status: "partial",
              blocking_reasons: ["mapped_categories_have_partial_coverage"],
              blocking_category_ids: ["browser_agent"]
            },
            comparison_rows: [
              {
                category_id: "ai_scraper_bot",
                category_label: "AI Scraper Bot",
                inference_capability_status: "candidate",
                comparison_status: "current_exact_match",
                inferred_category_id: "ai_scraper_bot",
                inferred_category_label: "AI Scraper Bot",
                exactness: "derived",
                basis: "observed",
                note: "Shared-path inference matched this category."
              },
              {
                category_id: "browser_agent",
                category_label: "Browser Agent",
                inference_capability_status: "candidate",
                comparison_status: "collapsed_to_unknown_non_human",
                inferred_category_id: "unknown_non_human",
                inferred_category_label: "Unknown Non Human",
                exactness: "derived",
                basis: "observed",
                note: "Recognition collapsed to unknown."
              }
            ]
          }
        },
        verified_identity: {
          availability: "supported",
          enabled: true,
          native_web_bot_auth_enabled: true,
          provider_assertions_enabled: true,
          effective_non_human_policy: {
            profile_id: "site_default_v1",
            objective_revision: "objective-1774306800",
            verified_identity_override_mode: "explicit_overrides_eligible",
            rows: []
          },
          taxonomy_alignment: {
            status: "degraded"
          }
        }
      })
    });
  });

  await page.route("**/admin/oversight/history", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_history_v1",
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-2",
              completed_at_ts: 1774306800,
              proposal_status: "accepted",
              watch_window_result: "improved",
              retain_or_rollback: "retained",
              judged_lane_ids: ["scrapling_traffic", "bot_red_team"],
              judged_run_ids: ["sim-42", "sim-43"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Retained after the mixed round improved."
              },
              cycle_judgment: "continue"
            },
            {
              episode_id: "episode-1",
              completed_at_ts: 1774220400,
              proposal_status: "accepted",
              watch_window_result: "regressed",
              retain_or_rollback: "rolled_back",
              judged_lane_ids: ["scrapling_traffic"],
              proposal: {
                patch_family: "maze_core",
                expected_impact: "Tighten maze traversal restrictions.",
                confidence: "medium",
                note: "Rolled back after regressions."
              },
              cycle_judgment: "rollback_and_continue"
            }
          ]
        },
        observer_round_archive: {
          schema_version: "oversight_observer_round_archive_v1",
          rows: [
            {
              episode_id: "episode-2",
              completed_at_ts: 1774306800,
              basis_status: "exact_judged_run_receipts",
              missing_run_ids: [],
              run_rows: [
                {
                  run_id: "sim-42",
                  lane: "scrapling_traffic",
                  profile: "reference",
                  observed_fulfillment_modes: ["bulk_scraper", "http_agent"],
                  observed_category_ids: ["ai_scraper_bot"],
                  monitoring_event_count: 64,
                  defense_delta_count: 6,
                  ban_outcome_count: 1
                },
                {
                  run_id: "sim-43",
                  lane: "bot_red_team",
                  profile: "reference",
                  observed_fulfillment_modes: ["request_mode"],
                  observed_category_ids: ["browser_agent"],
                  monitoring_event_count: 28,
                  defense_delta_count: 4,
                  ban_outcome_count: 0
                }
              ],
              scrapling_surface_rows: [
                {
                  run_id: "sim-42",
                  surface_id: "challenge_routing",
                  surface_state: "satisfied",
                  coverage_status: "pass_observed",
                  success_contract: "mixed_outcomes",
                  dependency_kind: "independent",
                  dependency_surface_ids: [],
                  attempt_count: 3,
                  sample_request_method: "GET",
                  sample_request_path: "/catalog?page=1",
                  sample_response_status: 200
                },
                {
                  run_id: "sim-42",
                  surface_id: "maze_navigation",
                  surface_state: "required_but_unreached",
                  coverage_status: "unavailable",
                  success_contract: "should_pass_some",
                  dependency_kind: "independent",
                  dependency_surface_ids: [],
                  attempt_count: 0,
                  sample_request_method: "",
                  sample_request_path: "",
                  sample_response_status: null
                }
              ],
              llm_surface_rows: [
                {
                  run_id: "sim-43",
                  surface_id: "pow_verify_abuse",
                  surface_state: "satisfied",
                  coverage_status: "pass_observed",
                  success_contract: "mixed_outcomes",
                  dependency_kind: "independent",
                  dependency_surface_ids: [],
                  attempt_count: 1,
                  sample_request_method: "GET",
                  sample_request_path: "/pow/check",
                  sample_response_status: 403
                }
              ]
            }
          ]
        },
        rows: [
          {
            decision_id: "ovr-2",
            recorded_at_ts: 1774306800,
            trigger_source: "periodic_supervisor",
            outcome: "canary_applied",
            summary: "Applied a bounded fingerprint tightening patch.",
            benchmark_overall_status: "outside_budget",
            improvement_status: "improved",
            replay_promotion_availability: "materialized",
            trigger_family_ids: ["suspicious_origin_cost"],
            candidate_action_families: ["fingerprint_signal"],
            refusal_reasons: [],
            validation_status: "valid",
            validation_issues: [],
            apply: {
              stage: "canary_applied",
              summary: "Canary opened for bounded fingerprint tuning.",
              patch_family: "fingerprint_signal",
              watch_window_seconds: 86400,
              comparison_status: "improved"
            }
          },
          {
            decision_id: "ovr-1",
            recorded_at_ts: 1774220400,
            trigger_source: "post_adversary_sim",
            outcome: "observe_longer",
            summary: "Held position because protected evidence was incomplete.",
            benchmark_overall_status: "near_limit",
            improvement_status: "not_available",
            replay_promotion_availability: "advisory_only",
            trigger_family_ids: ["suspicious_origin_cost"],
            candidate_action_families: [],
            refusal_reasons: ["protected_tuning_evidence_not_ready"],
            validation_status: "skipped",
            validation_issues: [],
            apply: {
              stage: "refused",
              summary: "Current oversight cycle is not allowed to mutate config automatically.",
              refusal_reasons: ["protected_tuning_evidence_not_ready"]
            }
          }
        ]
      })
    });
  });

  await page.route("**/admin/oversight/agent/status", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_agent_status_v1",
        execution_boundary: "shared_host_only",
        periodic_trigger: {
          surface: "host_supervisor_wrapper",
          wrapper_command: "scripts/run_with_oversight_supervisor.sh",
          default_interval_seconds: 300
        },
        post_sim_trigger: {
          surface: "internal_adversary_sim_completion_hook",
          qualifying_completion: "transition_to_off_with_completed_run_id_and_generated_traffic",
          dedupe_key: "sim_run_id"
        },
        candidate_window: {
          status: "running",
          canary_id: "canary-2",
          patch_family: "fingerprint_signal",
          requested_lane: "bot_red_team",
          requested_duration_seconds: 120,
          requested_at_ts: 1774306810,
          watch_window_end_at: 1774306920,
          follow_on_run_id: "simrun-002",
          follow_on_started_at: 1774306812,
          required_runs: [
            {
              lane: "scrapling_traffic",
              status: "materialized",
              requested_at_ts: 1774306800,
              requested_duration_seconds: 30,
              follow_on_run_id: "simrun-001",
              follow_on_started_at: 1774306801,
              materialized_at_ts: 1774306805
            },
            {
              lane: "bot_red_team",
              status: "running",
              requested_at_ts: 1774306810,
              requested_duration_seconds: 120,
              follow_on_run_id: "simrun-002",
              follow_on_started_at: 1774306812
            }
          ]
        },
        continuation_run: {
          status: "pending",
          requested_lane: "scrapling_traffic",
          requested_duration_seconds: 30,
          requested_at_ts: 1774307000,
          source_decision_id: "ovr-2",
          source_decision_outcome: "retained",
          continue_reason: "outside_budget",
          required_runs: [
            {
              lane: "scrapling_traffic",
              status: "pending",
              requested_at_ts: 1774307000,
              requested_duration_seconds: 30
            },
            {
              lane: "bot_red_team",
              status: "pending",
              requested_at_ts: 1774307000,
              requested_duration_seconds: 120
            }
          ]
        },
        latest_decision: {
          decision_id: "ovr-2",
          recorded_at_ts: 1774306800,
          trigger_source: "periodic_supervisor",
          outcome: "canary_applied",
          summary: "Applied a bounded fingerprint tightening patch."
        },
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-2",
              completed_at_ts: 1774306800,
              proposal_status: "accepted",
              watch_window_result: "improved",
              retain_or_rollback: "retained",
              judged_lane_ids: ["scrapling_traffic", "bot_red_team"],
              judged_run_ids: ["sim-42", "sim-43"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Retained after the mixed round improved."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        recent_runs: [
          {
            run_id: "run-1",
            trigger_kind: "periodic_supervisor",
            requested_at_ts: 1774306800,
            started_at_ts: 1774306800,
            completed_at_ts: 1774306805,
            execution: {
              apply: {
                stage: "canary_applied",
                summary: "Canary opened."
              }
            }
          }
        ]
      })
    });
  });

  await openDashboard(page);

  await expect(
    page.locator('#dashboard-panel-game-loop [data-game-loop-section="recent-rounds"]')
  ).toContainText("Recent Rounds");
  await expect(page.locator("#game-loop-round-history")).toContainText("Scrapling Traffic, Agentic Traffic");
  await expect(page.locator("#game-loop-round-history")).toContainText("Fingerprint Signal");
  await expect(page.locator("#game-loop-round-history")).toContainText("Retained");
  await expect(page.locator("#game-loop-round-history")).toContainText("Continue");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("Adversaries In This Round");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("AI Scraper Bot");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("Browser Agent via Agentic Traffic");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Recent recognition evaluation inferred AI Scraper Bot"
  );
  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Recent recognition evaluation inferred Unknown Non Human"
  );
  await expect(page.locator("#game-loop-adversary-cast")).not.toContainText("HTTP Agent");
  await expect(page.locator("#game-loop-adversary-cast")).not.toContainText("Automated Browser");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Defences In This Round");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Challenge Routing");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Maze Navigation");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("/catalog?page=1");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("PoW Verify Abuse");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Saw GET /pow/check");
  await expect(page.locator("#game-loop-defence-cast")).not.toContainText("scrapling-");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("required but unreached");
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-section="current-status"]')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-section="recent-loop-progress"]')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-section="outcome-frontier"]')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-section="change-judgment"]')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-section="pressure-sits"]')).toHaveCount(0);
  await expect(page.locator('#dashboard-panel-game-loop [data-game-loop-section="trust-and-blockers"]')).toHaveCount(0);
});

test("game loop distinguishes recognition evaluation from scrapling surface contract truth", async ({ page }) => {
  await page.route("**/admin/operator-snapshot", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "operator_snapshot_v1",
        generated_at: 1774306900,
        objectives: {
          profile_id: "human_only_private",
          revision: "objective-1774306900",
          window_hours: 24,
          category_postures: [
            {
              category_id: "ai_scraper_bot",
              posture: "blocked"
            }
          ]
        },
        runtime_posture: {
          shadow_mode: false,
          fail_mode: "closed",
          runtime_environment: "runtime-prod",
          gateway_deployment_profile: "shared_server",
          adversary_sim_available: true
        },
        live_traffic: {
          traffic_origin: "live",
          execution_mode: "enforced",
          total_requests: 1200,
          forwarded_requests: 860,
          short_circuited_requests: 340
        },
        shadow_mode: {
          enabled: false,
          total_actions: 0,
          pass_through_total: 0
        },
        adversary_sim: {
          traffic_origin: "adversary_sim",
          execution_mode: "enforced",
          total_requests: 180,
          forwarded_requests: 9,
          short_circuited_requests: 171,
          recent_runs: []
        },
        recent_changes: {
          rows: []
        },
        non_human_traffic: {
          availability: "taxonomy_seeded",
          restriction_readiness: {
            status: "ready",
            blockers: [],
            live_receipt_count: 6,
            adversary_sim_receipt_count: 3
          },
          recognition_evaluation: {
            comparison_status: "mixed",
            current_exact_match_count: 1,
            degraded_match_count: 0,
            collapsed_to_unknown_count: 1,
            not_materialized_count: 0,
            simulator_ground_truth: {
              status: "observed_recent_runs",
              recent_sim_run_count: 1,
              categories: [
                {
                  category_id: "ai_scraper_bot",
                  category_label: "AI Scraper Bot",
                  recent_run_count: 1,
                  evidence_references: [
                    "recent_sim_runs:sim-attack-partial:scrapling_runtime_lane:ai_scraper_bot"
                  ]
                },
                {
                  category_id: "automated_browser",
                  category_label: "Automated Browser",
                  recent_run_count: 1,
                  evidence_references: [
                    "recent_sim_runs:sim-attack-partial:scrapling_runtime_lane:automated_browser"
                  ]
                }
              ]
            },
            readiness: {
              status: "partial",
              blockers: ["exact_category_inference_not_ready"],
              live_receipt_count: 6,
              adversary_sim_receipt_count: 3
            },
            coverage: {
              overall_status: "partial",
              blocking_reasons: ["mapped_categories_have_partial_coverage"],
              blocking_category_ids: ["automated_browser"]
            },
            comparison_rows: [
              {
                category_id: "ai_scraper_bot",
                category_label: "AI Scraper Bot",
                inference_capability_status: "candidate",
                comparison_status: "current_exact_match",
                inferred_category_id: "ai_scraper_bot",
                inferred_category_label: "AI Scraper Bot",
                exactness: "derived",
                basis: "observed",
                note: "Shared-path inference matched this category."
              },
              {
                category_id: "automated_browser",
                category_label: "Automated Browser",
                inference_capability_status: "candidate",
                comparison_status: "collapsed_to_unknown_non_human",
                inferred_category_id: "unknown_non_human",
                inferred_category_label: "Unknown Non Human",
                exactness: "derived",
                basis: "observed",
                note: "Recognition collapsed to unknown."
              }
            ]
          }
        },
        verified_identity: {
          availability: "supported",
          enabled: true,
          native_web_bot_auth_enabled: true,
          provider_assertions_enabled: true,
          effective_non_human_policy: {
            profile_id: "human_only_private",
            objective_revision: "objective-1774306900",
            verified_identity_override_mode: "strict_human_only",
            rows: []
          },
          taxonomy_alignment: {
            status: "aligned"
          }
        }
      })
    });
  });

  await page.route("**/admin/oversight/history", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_history_v1",
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-sim-attack-partial",
              completed_at_ts: 1774306900,
              proposal_status: "accepted",
              watch_window_result: "improved",
              retain_or_rollback: "retained",
              judged_lane_ids: ["scrapling_traffic"],
              judged_run_ids: ["sim-attack-partial"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Retained after the judged round."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        observer_round_archive: {
          schema_version: "oversight_observer_round_archive_v1",
          rows: [
            {
              episode_id: "episode-sim-attack-partial",
              completed_at_ts: 1774306900,
              basis_status: "exact_judged_run_receipts",
              missing_run_ids: [],
              run_rows: [
                {
                  run_id: "sim-attack-partial",
                  lane: "scrapling_traffic",
                  profile: "scrapling_runtime_lane",
                  observed_fulfillment_modes: ["bulk_scraper", "browser_automation"],
                  observed_category_ids: ["ai_scraper_bot", "automated_browser"],
                  monitoring_event_count: 64,
                  defense_delta_count: 3,
                  ban_outcome_count: 1
                }
              ],
              scrapling_surface_rows: [
                {
                  run_id: "sim-attack-partial",
                  surface_id: "challenge_routing",
                  surface_state: "satisfied",
                  coverage_status: "pass_observed",
                  success_contract: "mixed_outcomes",
                  dependency_kind: "independent",
                  dependency_surface_ids: [],
                  attempt_count: 3,
                  sample_request_method: "GET",
                  sample_request_path: "/catalog?page=1",
                  sample_response_status: 200
                },
                {
                  run_id: "sim-attack-partial",
                  surface_id: "maze_navigation",
                  surface_state: "required_but_unreached",
                  coverage_status: "unavailable",
                  success_contract: "should_pass_some",
                  dependency_kind: "independent",
                  dependency_surface_ids: [],
                  attempt_count: 0,
                  sample_request_method: "",
                  sample_request_path: "",
                  sample_response_status: null
                }
              ]
            }
          ]
        },
        rows: []
      })
    });
  });

  await page.route("**/admin/oversight/agent/status", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_agent_status_v1",
        execution_boundary: "shared_host_only",
        periodic_trigger: {
          surface: "host_supervisor_wrapper",
          wrapper_command: "scripts/run_with_oversight_supervisor.sh",
          default_interval_seconds: 300
        },
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-sim-attack-partial",
              completed_at_ts: 1774306900,
              proposal_status: "accepted",
              watch_window_result: "improved",
              retain_or_rollback: "retained",
              judged_lane_ids: ["scrapling_traffic"],
              judged_run_ids: ["sim-attack-partial"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Retained after the judged round."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        latest_decision: {},
        recent_runs: []
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "game-loop");

  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Adversaries In This Round"
  );
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("AI Scraper Bot");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("Automated Browser");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Recent recognition evaluation inferred AI Scraper Bot"
  );
  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Recent recognition evaluation inferred Unknown Non Human"
  );
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Defences In This Round");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Challenge Routing");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("satisfied");
  await expect(page.locator("#game-loop-defence-cast")).toContainText("Maze Navigation");
  await expect(page.locator("#game-loop-defence-cast")).toContainText(
    "required but unreached"
  );
  await expect(page.locator("#game-loop-defence-cast")).toContainText("independent surface");
});

test("game loop observer archive marks missing judged runs without guessing adversary or defence casts", async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 1200 });
  await page.route("**/admin/operator-snapshot", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "operator_snapshot_v1",
        generated_at: 1774307000,
        objectives: {
          profile_id: "human_only_private",
          revision: "objective-1774307000",
          window_hours: 24,
          category_postures: []
        },
        runtime_posture: {
          shadow_mode: false,
          fail_mode: "closed",
          runtime_environment: "runtime-prod",
          gateway_deployment_profile: "shared_server",
          adversary_sim_available: true
        },
        live_traffic: {
          traffic_origin: "live",
          execution_mode: "enforced",
          total_requests: 1200,
          forwarded_requests: 860,
          short_circuited_requests: 340,
          human_friction: {
            friction_rate: 0.018
          }
        },
        shadow_mode: {
          enabled: false,
          total_actions: 0,
          pass_through_total: 0
        },
        adversary_sim: {
          traffic_origin: "adversary_sim",
          execution_mode: "enforced",
          total_requests: 180,
          forwarded_requests: 9,
          short_circuited_requests: 171,
          recent_runs: []
        },
        recent_changes: {
          rows: []
        }
      })
    });
  });

  await page.route("**/admin/oversight/history", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_history_v1",
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-missing-round",
              completed_at_ts: 1774306990,
              proposal_status: "accepted",
              watch_window_result: "not_improved",
              retain_or_rollback: "rolled_back",
              judged_lane_ids: ["scrapling_traffic", "bot_red_team"],
              judged_run_ids: ["sim-missing-scrapling", "sim-bot-red-team"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Rolled back after the judged round."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        observer_round_archive: {
          schema_version: "oversight_observer_round_archive_v1",
          rows: [
            {
              episode_id: "episode-missing-round",
              completed_at_ts: 1774306990,
              basis_status: "partial_missing_run_receipts",
              missing_run_ids: ["sim-missing-scrapling"],
              run_rows: [
                {
                  run_id: "sim-bot-red-team",
                  lane: "bot_red_team",
                  profile: "browser_mode",
                  observed_fulfillment_modes: ["browser_mode"],
                  observed_category_ids: ["browser_agent"],
                  monitoring_event_count: 4,
                  defense_delta_count: 1,
                  ban_outcome_count: 0
                }
              ],
              scrapling_surface_rows: []
            }
          ]
        },
        rows: []
      })
    });
  });

  await page.route("**/admin/oversight/agent/status", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_agent_status_v1",
        execution_boundary: "shared_host_only",
        periodic_trigger: {
          surface: "host_supervisor_wrapper",
          wrapper_command: "scripts/run_with_oversight_supervisor.sh",
          default_interval_seconds: 300
        },
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-missing-round",
              completed_at_ts: 1774306990,
              proposal_status: "accepted",
              watch_window_result: "not_improved",
              retain_or_rollback: "rolled_back",
              judged_lane_ids: ["scrapling_traffic", "bot_red_team"],
              judged_run_ids: ["sim-missing-scrapling", "sim-bot-red-team"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Rolled back after the judged round."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        current_cycle: null
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "game-loop");

  await expect(page.locator("#game-loop-adversary-cast")).toContainText("Browser Agent via Agentic Traffic");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Recent recognition evaluation not materialized"
  );
  await expect(page.locator("#game-loop-adversary-cast")).not.toContainText("AI Scraper Bot");
  await expect(page.locator("#game-loop-adversary-cast")).not.toContainText("via Scrapling Traffic");
  await expect(page.locator("#game-loop-defence-cast")).toContainText(
    "Missing exact run receipts: sim-missing-scrapling."
  );
  await expect(page.locator("#game-loop-defence-cast")).toContainText(
    "The current observer evidence is only partially materialized, so the page will not guess the missing defence cast."
  );
  await expect(page.locator("#game-loop-defence-cast")).not.toContainText("Challenge Routing");
});

test("game loop top casts prefer the freshest exact recent sim run over stale judged history", async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 1200 });
  await page.route("**/admin/operator-snapshot", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "operator_snapshot_v1",
        generated_at: 1774307100,
        objectives: {
          profile_id: "human_only_private",
          revision: "objective-1774307100",
          window_hours: 24,
          category_postures: []
        },
        runtime_posture: {
          shadow_mode: false,
          fail_mode: "closed",
          runtime_environment: "runtime-prod",
          gateway_deployment_profile: "shared_server",
          adversary_sim_available: true
        },
        live_traffic: {
          traffic_origin: "live",
          execution_mode: "enforced",
          total_requests: 1200,
          forwarded_requests: 860,
          short_circuited_requests: 340,
          human_friction: {
            friction_rate: 0.018
          }
        },
        shadow_mode: {
          enabled: false,
          total_actions: 0,
          pass_through_total: 0
        },
        adversary_sim: {
          traffic_origin: "adversary_sim",
          execution_mode: "enforced",
          total_requests: 180,
          forwarded_requests: 9,
          short_circuited_requests: 171,
          recent_runs: [
            {
              run_id: "sim-fresh-agentic",
              lane: "bot_red_team",
              profile: "llm_runtime_lane",
              observed_fulfillment_modes: ["request_mode"],
              observed_category_ids: ["ai_scraper_bot", "http_agent"],
              first_ts: 1774307060,
              last_ts: 1774307095,
              monitoring_event_count: 18,
              defense_delta_count: 5,
              ban_outcome_count: 0,
              llm_runtime_summary: {
                category_targets: ["http_agent", "ai_scraper_bot"],
                generation_source: "provider_response",
                provider: "openai",
                model_id: "gpt-5-mini",
                generated_action_count: 3,
                executed_action_count: 3,
                failed_tick_count: 0,
                latest_action_receipts: [
                  {
                    action_type: "http_get",
                    path: "/pow/check",
                    status: 403
                  },
                  {
                    action_type: "http_get",
                    path: "/",
                    status: 200
                  }
                ]
              }
            },
            {
              run_id: "sim-old-judged",
              lane: "scrapling_traffic",
              profile: "scrapling_runtime_lane",
              observed_fulfillment_modes: ["crawler"],
              observed_category_ids: ["indexing_bot"],
              first_ts: 1774306900,
              last_ts: 1774306940,
              monitoring_event_count: 7,
              defense_delta_count: 1,
              ban_outcome_count: 0
            }
          ]
        },
        recent_changes: {
          rows: []
        },
        non_human_traffic: {
          availability: "taxonomy_seeded",
          restriction_readiness: {
            status: "ready",
            blockers: [],
            note: "Ready"
          },
          recognition_evaluation: {
            comparison_status: "partial",
            current_exact_match_count: 2,
            degraded_match_count: 1,
            collapsed_to_unknown_count: 1,
            not_materialized_count: 0,
            readiness: {
              status: "ready",
              blockers: [],
              note: "Ready"
            },
            coverage: {
              overall_status: "covered",
              blocking_reasons: [],
              blocking_category_ids: []
            },
            simulator_ground_truth: {
              categories: [
                {
                  category_id: "ai_scraper_bot",
                  category_label: "AI Scraper Bot",
                  recent_run_count: 1,
                  evidence_references: ["sim-fresh-agentic"]
                },
                {
                  category_id: "http_agent",
                  category_label: "HTTP Agent",
                  recent_run_count: 1,
                  evidence_references: ["sim-fresh-agentic"]
                }
              ]
            },
            comparison_rows: [
              {
                category_id: "ai_scraper_bot",
                category_label: "AI Scraper Bot",
                inferred_category_label: "AI Scraper Bot",
                comparison_status: "current_exact_match",
                note: "Exact recent match."
              },
              {
                category_id: "http_agent",
                category_label: "HTTP Agent",
                inferred_category_label: "HTTP Agent",
                comparison_status: "current_exact_match",
                note: "Exact recent match."
              }
            ]
          }
        },
        verified_identity: {
          availability: "not_configured",
          enabled: false,
          native_web_bot_auth_enabled: false,
          provider_assertions_enabled: false,
          effective_non_human_policy: {
            schema_version: "verified_identity_effective_policy_v1",
            profile_id: "human_only_private",
            objective_revision: "objective-1774307100",
            verified_identity_override_mode: "strict",
            rows: []
          }
        }
      })
    });
  });

  await page.route("**/admin/oversight/history", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_history_v1",
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-old-judged-round",
              completed_at_ts: 1774306950,
              proposal_status: "accepted",
              watch_window_result: "improved",
              retain_or_rollback: "retained",
              judged_lane_ids: ["scrapling_traffic"],
              judged_run_ids: ["sim-old-judged"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Retained after the judged round."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        observer_round_archive: {
          schema_version: "oversight_observer_round_archive_v1",
          rows: [
            {
              episode_id: "episode-old-judged-round",
              completed_at_ts: 1774306950,
              basis_status: "exact_judged_run_receipts",
              missing_run_ids: [],
              run_rows: [
                {
                  run_id: "sim-old-judged",
                  lane: "scrapling_traffic",
                  profile: "scrapling_runtime_lane",
                  observed_fulfillment_modes: ["crawler"],
                  observed_category_ids: ["indexing_bot"],
                  monitoring_event_count: 7,
                  defense_delta_count: 1,
                  ban_outcome_count: 0
                }
              ],
              scrapling_surface_rows: [
                {
                  run_id: "sim-old-judged",
                  surface_id: "challenge_routing",
                  surface_state: "satisfied",
                  coverage_status: "pass_observed",
                  success_contract: "mixed_outcomes",
                  dependency_kind: "independent",
                  dependency_surface_ids: [],
                  attempt_count: 1,
                  sample_request_method: "GET",
                  sample_request_path: "/catalog?page=1",
                  sample_response_status: 200
                }
              ]
            }
          ]
        },
        rows: []
      })
    });
  });

  await page.route("**/admin/oversight/agent/status", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "oversight_agent_status_v1",
        execution_boundary: "shared_host_only",
        periodic_trigger: {
          surface: "host_supervisor_wrapper",
          wrapper_command: "scripts/run_with_oversight_supervisor.sh",
          default_interval_seconds: 300
        },
        candidate_window: {
          status: "not_requested",
          required_runs: []
        },
        continuation_run: {
          status: "not_requested",
          required_runs: []
        },
        episode_archive: {
          schema_version: "oversight_episode_archive_v1",
          homeostasis: {
            status: "not_enough_completed_cycles"
          },
          rows: [
            {
              episode_id: "episode-old-judged-round",
              completed_at_ts: 1774306950,
              proposal_status: "accepted",
              watch_window_result: "improved",
              retain_or_rollback: "retained",
              judged_lane_ids: ["scrapling_traffic"],
              judged_run_ids: ["sim-old-judged"],
              proposal: {
                patch_family: "fingerprint_signal",
                expected_impact: "Tighten suspicious automation detection.",
                confidence: "medium",
                note: "Retained after the judged round."
              },
              cycle_judgment: "continue"
            }
          ]
        },
        latest_decision: {},
        recent_runs: []
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "game-loop");

  await expect(page.locator("#game-loop-round-history")).toContainText("Recent Rounds");
  await expect(page.locator("#game-loop-round-history")).toContainText("Fingerprint Signal");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("AI Scraper Bot");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("HTTP Agent");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText("Agentic Traffic");
  await expect(page.locator("#game-loop-adversary-cast")).toContainText(
    "Showing the latest exact recent sim run"
  );
  await expect(page.locator("#game-loop-defence-cast")).toContainText("PoW Verify Abuse");
  await expect(page.locator("#game-loop-defence-cast")).toContainText(
    "Saw GET /pow/check"
  );
});

test("traffic manual refresh renders bounded traffic sections and preserves furniture diagnostics separately", async ({ page }) => {
  const buildCountEntries = (prefix, count, start = 1) =>
    Array.from({ length: count }, (_, index) => ({
      label: `${prefix}-${index + start}`,
      count: count - index
    }));

  const buildReasonMap = (prefix, count) =>
    Array.from({ length: count }, (_, index) => [`${prefix}_${index + 1}`, count - index]).reduce(
      (accumulator, [key, value]) => ({ ...accumulator, [key]: value }),
      {}
    );

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        summary: {
          honeypot: {
            total_hits: 125,
            unique_crawlers: 17,
            top_crawlers: buildCountEntries("crawler", 14),
            top_paths: buildCountEntries("/trap", 15).map((entry) => ({
              label: entry.label,
              count: entry.count
            }))
          },
          challenge: {
            total_failures: 41,
            unique_offenders: 12,
            top_offenders: buildCountEntries("challenge-offender", 12),
            reasons: buildReasonMap("challenge_reason", 12),
            trend: []
          },
          pow: {
            total_failures: 20,
            total_successes: 80,
            total_attempts: 100,
            success_ratio: 0.8,
            unique_offenders: 9,
            top_offenders: buildCountEntries("pow-offender", 12),
            reasons: buildReasonMap("pow_reason", 12),
            outcomes: { success: 80, failure: 20 },
            trend: []
          },
          rate: {
            total_violations: 31,
            unique_offenders: 7,
            top_offenders: buildCountEntries("rate-offender", 11),
            outcomes: { limited: 20, banned: 10, fallback_allow: 1, fallback_deny: 0 }
          },
          geo: {
            total_violations: 29,
            actions: { block: 9, challenge: 11, maze: 9 },
            top_countries: buildCountEntries("CC", 12)
          }
        },
        prometheus: { endpoint: "/metrics", notes: [] },
        details: {
          analytics: { ban_count: 2, shadow_mode: false, fail_mode: "open" },
          events: {
            recent_events: [
              {
                ts: Math.floor(Date.now() / 1000),
                event: "Challenge",
                ip: "198.51.100.44",
                reason: "challenge_reason_1",
                outcome: "served",
                execution_mode: "shadow",
                intended_action: "challenge",
                enforcement_applied: false,
                admin: "ops"
              }
            ],
            event_counts: { Challenge: 1 },
            top_ips: [["198.51.100.44", 1]],
            unique_ips: 1
          },
          bans: { bans: [] },
          maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
          cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
          cdp_events: { events: [] }
        }
      })
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    const url = new URL(route.request().url());
    const limit = Number.parseInt(url.searchParams.get("limit") || "0", 10);
    const afterCursor = String(url.searchParams.get("after_cursor") || "").trim();
    if (limit === 1) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          after_cursor: "",
          window_end_cursor: "cursor-summary-1",
          next_cursor: "cursor-summary-1",
          has_more: false,
          overflow: "none",
          events: [],
          freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
        })
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        after_cursor: afterCursor,
        window_end_cursor: "cursor-summary-2",
        next_cursor: "cursor-summary-2",
        has_more: false,
        overflow: "limit_exceeded",
        events: [],
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "diagnostics");
  await page.click("#refresh-now-btn");
  await expect(page.locator("#honeypot-total-hits")).toHaveText("125");
  await expect(page.locator("#challenge-failures-total")).toHaveText("41");
  await expect(page.locator("#pow-total-attempts")).toHaveText("100");
  await expect(page.locator("#pow-failures-total")).toHaveText("20");
  await expect(page.locator("#pow-outcomes-list")).toContainText("Success: 80");
  await expect(page.locator("#pow-outcomes-list")).toContainText("Failure: 20");
  await expect(page.locator("#rate-violations-total")).toHaveText("31");
  await expect(page.locator("#geo-violations-total")).toHaveText("29");

  await expect(page.locator("#honeypot-top-paths .crawler-item")).toHaveCount(10);
  await expect(page.locator("#challenge-failure-reasons tr")).toHaveCount(10);
  await expect(page.locator("#pow-failure-reasons tr")).toHaveCount(10);
  await expect(page.locator("#geo-top-countries .crawler-item")).toHaveCount(10);

  await openTab(page, "traffic");
  await expect(page.locator("#monitoring-events tbody")).toContainText("Shadow");
  await expect(page.locator("#monitoring-events tbody")).toContainText("Would Challenge");
});

test("status/verification/rate-limiting/geo/fingerprinting/tuning show empty state when config snapshot is empty", async ({ page }) => {
  await page.route("**/admin/config", async (route) => {
    if (route.request().method() !== "GET") {
      await route.continue();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({})
    });
  });

  await openDashboard(page, { initialTab: "status" });
  await expect(page.locator('[data-tab-state="status"]')).toContainText("No status config snapshot available yet.");

  await openTab(page, "verification");
  await expect(page.locator('[data-tab-state="verification"]')).toContainText("No verification config snapshot available yet.");

  await openTab(page, "rate-limiting");
  await expect(page.locator('[data-tab-state="rate-limiting"]')).toContainText("No rate limiting config snapshot available yet.");

  await openTab(page, "geo");
  await expect(page.locator('[data-tab-state="geo"]')).toContainText("No GEO config snapshot available yet.");

  await openTab(page, "fingerprinting");
  await expect(page.locator('[data-tab-state="fingerprinting"]')).toContainText("No fingerprinting config snapshot available yet.");

  await openTab(page, "tuning");
  await expect(page.locator('[data-tab-state="tuning"]')).toContainText("No tuning config snapshot available yet.");
});

test("dashboard loads and shows seeded operational data", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "traffic");
  await assertChartsFillPanels(page);

  await expect(page.locator("h1")).toHaveText("Shuma-Gorath");
  await expect(page.locator("h3", { hasText: "API Access" })).toHaveCount(0);

  await expect(page.locator("#last-updated")).toContainText("updated:");

  await expect(page.locator("#total-events")).not.toHaveText("-");
  await expect(page.locator("#monitoring-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#monitoring-events tbody")).not.toContainText("undefined");

  await openTab(page, "diagnostics");
  await expect(page.locator("#cdp-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#cdp-total-detections")).not.toHaveText("-");
  await expect(page.locator('label[for="global-shadow-mode-toggle"]')).toBeVisible();
  await expect(page.locator(".dashboard-shadow-mode-eye")).toHaveCount(1);
  await expect(page.locator(".dashboard-shadow-mode-eye")).toBeHidden();
});

test("dashboard header overlays the eye only while shadow mode is enabled", async ({ page, request }) => {
  await withRestoredAdminConfig(request, SHADOW_MODE_RESTORE_PATHS, async () => {
    await openDashboard(page);

    const eyeOverlay = page.locator(".dashboard-shadow-mode-eye");

    await updateAdminConfig(request, { shadow_mode: false });
    await page.reload();
    await expect(eyeOverlay).toHaveCount(1);
    await expect(eyeOverlay).toBeHidden();
    let classState = await dashboardDomClassState(page);
    expect(classState.hasShadowMode).toBeFalsy();
    expect(classState.bodyHasShadowMode).toBeFalsy();

    await updateAdminConfig(request, { shadow_mode: true });
    await page.reload();
    await expect(eyeOverlay).toBeVisible();
    classState = await dashboardDomClassState(page);
    expect(classState.hasShadowMode).toBeTruthy();
    expect(classState.bodyHasShadowMode).toBeFalsy();

    await updateAdminConfig(request, { shadow_mode: false });
    await page.reload();
    await expect(eyeOverlay).toHaveCount(1);
    await expect(eyeOverlay).toBeHidden();
    classState = await dashboardDomClassState(page);
    expect(classState.hasShadowMode).toBeFalsy();
    expect(classState.bodyHasShadowMode).toBeFalsy();
  });
});

test("dashboard diagnostics totals stay in parity with /metrics monitoring families", async ({ page, request }) => {
  await openDashboard(page);
  await openTab(page, "diagnostics");

  const adminHeaders = {
    ...buildTrustedForwardingHeaders(),
    Authorization: `Bearer ${API_KEY}`
  };

  const [monitoringResponse, metricsResponse] = await Promise.all([
    request.get(`${BASE_URL}/admin/monitoring?hours=24&limit=10`, { headers: adminHeaders }),
    request.get(`${BASE_URL}/metrics`, { headers: buildTrustedForwardingHeaders() })
  ]);

  expect(monitoringResponse.ok()).toBe(true);
  expect(metricsResponse.ok()).toBe(true);

  const monitoring = await monitoringResponse.json();
  const metricsText = await metricsResponse.text();

  const challengeReasons = parsePrometheusLabeledCounters(
    metricsText,
    "bot_defence_monitoring_challenge_failures_total",
    "reason"
  );
  const powOutcomes = parsePrometheusLabeledCounters(
    metricsText,
    "bot_defence_monitoring_pow_verifications_total",
    "outcome"
  );
  const powReasons = parsePrometheusLabeledCounters(
    metricsText,
    "bot_defence_monitoring_pow_failures_total",
    "reason"
  );
  const rateOutcomes = parsePrometheusLabeledCounters(
    metricsText,
    "bot_defence_monitoring_rate_violations_total",
    "outcome"
  );
  const geoActions = parsePrometheusLabeledCounters(
    metricsText,
    "bot_defence_monitoring_geo_violations_total",
    "action"
  );
  const cdpDetections = parsePrometheusScalar(metricsText, "bot_defence_cdp_detections_total");

  expect(Object.keys(challengeReasons).sort()).toEqual([
    "expired_replay",
    "forbidden",
    "incorrect",
    "invalid_output",
    "sequence_violation"
  ]);
  expect(Object.keys(powOutcomes).sort()).toEqual(["failure", "success"]);
  expect(Object.keys(powReasons).sort()).toEqual([
    "binding_timing_mismatch",
    "expired_replay",
    "invalid_proof",
    "missing_seed_nonce",
    "sequence_violation"
  ]);
  expect(Object.keys(rateOutcomes).sort()).toEqual([
    "banned",
    "fallback_allow",
    "fallback_deny",
    "limited"
  ]);
  expect(Object.keys(geoActions).sort()).toEqual(["block", "challenge", "maze"]);

  const summary = monitoring.summary || {};
  expect(challengeReasons).toEqual(summary.challenge?.reasons || {});
  expect(powOutcomes).toEqual(summary.pow?.outcomes || {});
  expect(powReasons).toEqual(summary.pow?.reasons || {});
  expect(rateOutcomes).toEqual(summary.rate?.outcomes || {});
  expect(geoActions).toEqual(summary.geo?.actions || {});
  expect(cdpDetections).toBe(Number(monitoring.details?.cdp?.stats?.total_detections || 0));

  const uiChallengeFailures = parseDashboardCounterText(
    await page.locator("#challenge-failures-total").textContent()
  );
  const uiPowAttempts = parseDashboardCounterText(
    await page.locator("#pow-total-attempts").textContent()
  );
  const uiRateViolations = parseDashboardCounterText(
    await page.locator("#rate-violations-total").textContent()
  );
  const uiGeoViolations = parseDashboardCounterText(
    await page.locator("#geo-violations-total").textContent()
  );

  // UI counters can lag backend sampling by one poll window; keep parity checks tight but non-flaky.
  expectCounterNear(uiChallengeFailures, sumCounterValues(challengeReasons), 50);
  expectCounterNear(uiPowAttempts, sumCounterValues(powOutcomes), 50);
  expectCounterNear(uiRateViolations, Number(summary.rate?.total_violations || 0), 50);
  expectCounterNear(uiGeoViolations, sumCounterValues(geoActions), 50);
});

test("status tab resolves fail mode without requiring monitoring bootstrap", async ({ page }) => {
  await openDashboard(page, { initialTab: "status" });
  const failModeCard = page
    .locator("#status-items .status-item")
    .filter({ has: page.locator("h3", { hasText: "Fail Mode Policy" }) });

  await expect(failModeCard).toHaveCount(1);
  await expect(failModeCard.locator(".status-value")).toHaveText(/OPEN|CLOSED/);
  await expect(failModeCard.locator(".status-value")).not.toHaveText("UNKNOWN");
  await expect(page.locator("#status-items .status-item h3", { hasText: "Challenge Puzzle" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Challenge Not-A-Bot" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Tarpit" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Runtime and Deployment Posture" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Admin Config Write Posture" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Shadow Mode" })).toHaveCount(0);
  await expect(page.locator("#status-items .status-item h3", { hasText: /^Challenge$/ })).toHaveCount(0);
  await expect(page.locator(".status-item h3", { hasText: "Dashboard Connectivity" })).toHaveCount(1);
  await expect(page.locator(".status-item h3", { hasText: "Telemetry Delivery Health" })).toHaveCount(1);
  await expect(page.locator(".status-item h3", { hasText: "Retention Health" })).toHaveCount(1);
  await expect(page.locator(".status-item h3", { hasText: "Runtime Performance Telemetry" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Retention and Freshness Health" })).toHaveCount(0);

  await expect(page.locator("#runtime-fetch-latency-last")).toContainText("ms");
  await expect(page.locator("#runtime-render-timing-last")).toContainText("ms");
  await expect(page.locator("#runtime-polling-skip-count")).toContainText("/");
});

test("tab-local monitoring failures do not flip the global dashboard connection state", async ({ page }) => {
  await openDashboard(page, { initialTab: "status" });
  await openTab(page, "diagnostics", { waitForReady: true });

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "monitoring pipeline unavailable" })
    });
  }, { times: 1 });

  await page.click("#refresh-now-btn");
  await expect(page.locator('[data-tab-state="diagnostics"]')).toContainText(
    "monitoring pipeline unavailable"
  );

  await openTab(page, "status", { waitForReady: true });
  await expect(page.locator("#status-connection-state")).toHaveText("Connected");
  await expect.poll(async () => {
    const value = await page.locator("#status-connection-ignored-non-heartbeat").textContent();
    return Number.parseInt(String(value || "0").trim(), 10) || 0;
  }).toBeGreaterThan(0);
  await expect(page.locator("#status-connection-last-failure-class")).toHaveText("-");

  await expect.poll(async () => {
    const domState = await dashboardDomClassState(page);
    return {
      degraded: domState.rootClasses.includes("degraded"),
      disconnected: domState.rootClasses.includes("disconnected")
    };
  }).toEqual({ degraded: false, disconnected: false });
});

test("advanced tab shows runtime variable inventory groups", async ({ page }) => {
  await openDashboard(page, { initialTab: "advanced" });

  const statusVarTables = page.locator("#status-vars-groups .status-vars-table");
  expect(await statusVarTables.count()).toBeGreaterThan(1);
  await expect(page.locator("#status-vars-groups .status-var-group-title", { hasText: "Tarpit Runtime" })).toHaveCount(1);

  const statusVarRows = page.locator("#status-vars-groups .status-vars-table tbody tr");
  expect(await statusVarRows.count()).toBeGreaterThan(20);
  const testModeRow = page
    .locator("#status-vars-groups .status-vars-table tbody tr")
    .filter({ has: page.locator("code", { hasText: "shadow_mode" }) });
  await expect(testModeRow).toHaveCount(1);
  await expect(testModeRow).toHaveClass(/status-var-row--admin-write/);
  await expect(testModeRow.locator("td").nth(2)).not.toHaveText("");
});

test("ban form enforces IP validity and submit state", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "ip-bans");

  const durationState = await page.evaluate(() => {
    const readNumber = (id) => {
      const raw = document.getElementById(id)?.value || "0";
      const parsed = Number.parseInt(String(raw), 10);
      return Number.isFinite(parsed) ? parsed : 0;
    };
    const days = readNumber("ban-duration-days");
    const hours = readNumber("ban-duration-hours");
    const minutes = readNumber("ban-duration-minutes");
    return {
      days,
      hours,
      minutes,
      totalSeconds: (days * 86400) + (hours * 3600) + (minutes * 60)
    };
  });
  expect(durationState.totalSeconds).toBeGreaterThan(0);

  const banButton = page.locator("#ban-btn");
  await expect(banButton).toBeDisabled();

  await page.fill("#ban-ip", "not-an-ip");
  await page.dispatchEvent("#ban-ip", "input");
  await expect(banButton).toBeDisabled();

  await page.fill("#ban-ip", "198.51.100.42");
  await page.dispatchEvent("#ban-ip", "input");
  await expect(banButton).toBeEnabled();
});

test("ip bans bypass allowlists pane reflects dirty-state changes", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "ip-bans");

  const saveBar = page.locator("#bypass-allowlists-save-bar");
  const networkAllowlistField = page.locator("#network-allowlist");
  const bypassAllowlistsEnabledToggle = page.locator("#bypass-allowlists-toggle");
  const bypassAllowlistsEnabledSwitch = page.locator("label.toggle-switch[for='bypass-allowlists-toggle']");

  await expect(saveBar).toBeHidden();

  if (!(await networkAllowlistField.isEditable())) {
    await expect(networkAllowlistField).toBeDisabled();
    return;
  }

  const initialNetworkAllowlist = await networkAllowlistField.inputValue();
  await networkAllowlistField.fill(`${initialNetworkAllowlist}\n198.51.100.0/24`);
  await networkAllowlistField.dispatchEvent("input");
  await expect(saveBar).toBeVisible();
  await networkAllowlistField.fill(initialNetworkAllowlist);
  await networkAllowlistField.dispatchEvent("input");
  await expect(saveBar).toBeHidden();

  if (await bypassAllowlistsEnabledSwitch.isVisible() && await bypassAllowlistsEnabledToggle.isEnabled()) {
    const initialBypassEnabled = await bypassAllowlistsEnabledToggle.isChecked();
    await bypassAllowlistsEnabledSwitch.click();
    await expect(saveBar).toBeVisible();
    if (initialBypassEnabled !== await bypassAllowlistsEnabledToggle.isChecked()) {
      await bypassAllowlistsEnabledSwitch.click();
    }
    await expect(saveBar).toBeHidden();
  }
});

test("verification save-all button reflects shared dirty-state behavior", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "verification", { waitForReady: true });

  const configSave = page.locator("#save-verification-all");
  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  const powJsRequiredWarning = page.locator("#verification-pow-js-required-warning");
  await expect(configSave).toBeHidden();

  if (!(await jsRequiredToggle.isVisible())) {
    await expect(page.locator("#dashboard-panel-verification")).toBeVisible();
    return;
  }

  if (!(await jsRequiredToggle.isEnabled())) {
    await expect(configSave).toBeHidden();
    return;
  }

  const jsRequiredInitial = await jsRequiredToggle.isChecked();
  await jsRequiredToggle.click();
  if (jsRequiredInitial) {
    await expect(powJsRequiredWarning).toBeVisible();
  } else {
    await expect(powJsRequiredWarning).toBeHidden();
  }
  await expect(configSave).toBeEnabled();
  if (jsRequiredInitial !== await jsRequiredToggle.isChecked()) {
    await jsRequiredToggle.click();
    if (jsRequiredInitial) {
      await expect(powJsRequiredWarning).toBeHidden();
    } else {
      await expect(powJsRequiredWarning).toBeVisible();
    }
  }
  await expect(configSave).toBeHidden();

  const powToggle = page.locator("#pow-enabled-toggle");
  const powToggleSwitch = page.locator("label.toggle-switch[for='pow-enabled-toggle']");
  if (await powToggleSwitch.isVisible() && await powToggle.isEnabled()) {
    const powInitial = await powToggle.isChecked();
    await powToggleSwitch.click();
    await expect(configSave).toBeEnabled();
    if (powInitial !== await powToggle.isChecked()) {
      await powToggleSwitch.click();
    }
    await expect(configSave).toBeHidden();
  }

  const cdpThresholdSlider = page.locator("#verification-cdp-threshold-slider");
  if (await cdpThresholdSlider.isVisible() && await cdpThresholdSlider.isEnabled()) {
    const cdpThresholdInitial = await cdpThresholdSlider.inputValue();
    const cdpThresholdNext = Number(cdpThresholdInitial || "0.6") >= 0.9
      ? "0.8"
      : (Number(cdpThresholdInitial || "0.6") + 0.1).toFixed(1);
    await cdpThresholdSlider.fill(cdpThresholdNext);
    await cdpThresholdSlider.dispatchEvent("input");
    await expect(configSave).toBeEnabled();
    await cdpThresholdSlider.fill(cdpThresholdInitial);
    await cdpThresholdSlider.dispatchEvent("input");
    await expect(configSave).toBeHidden();
  }

  const passScore = page.locator("#not-a-bot-score-pass-min");
  if (await passScore.isVisible() && await passScore.isEnabled()) {
    const initialPassScore = await passScore.inputValue();
    const nextPassScore = Number(initialPassScore || "7") >= 10
      ? "9"
      : String(Number(initialPassScore || "7") + 1);
    await passScore.fill(nextPassScore);
    await passScore.dispatchEvent("input");
    await expect(configSave).toBeEnabled();
    await passScore.fill(initialPassScore);
    await passScore.dispatchEvent("input");
    await expect(configSave).toBeHidden();
  }

  const challengeEnabledToggle = page.locator("#challenge-puzzle-enabled-toggle");
  const challengeEnabledSwitch = page.locator("label.toggle-switch[for='challenge-puzzle-enabled-toggle']");
  if (await challengeEnabledSwitch.isVisible() && await challengeEnabledToggle.isEnabled()) {
    const initialEnabled = await challengeEnabledToggle.isChecked();
    await challengeEnabledSwitch.click();
    await expect(configSave).toBeEnabled();
    if (initialEnabled !== await challengeEnabledToggle.isChecked()) {
      await challengeEnabledSwitch.click();
    }
    await expect(configSave).toBeHidden();
  }
});

test("verification tab surfaces verified identity controls and health summary", async ({ page, request }) => {
  await page.route("**/admin/operator-snapshot", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        schema_version: "operator_snapshot_v1",
        generated_at: 1774224000,
        verified_identity: {
          availability: "supported",
          enabled: true,
          native_web_bot_auth_enabled: true,
          provider_assertions_enabled: true,
          effective_non_human_policy: {
            profile_id: "humans_plus_verified_only",
            objective_revision: "objective-1774224000",
            verified_identity_override_mode: "explicit_overrides_eligible",
            rows: []
          },
          named_policy_count: 2,
          service_profile_count: 4,
          attempts: 12,
          verified: 9,
          failed: 3,
          unique_verified_identities: 5,
          top_failure_reasons: [{ label: "directory_stale", count: 2 }],
          top_schemes: [{ label: "http_message_signatures", count: 9 }],
          top_categories: [{ label: "search", count: 6 }]
        }
      })
    });
  });

  await withRestoredAdminConfig(
    request,
    [...VERIFICATION_RESTORE_PATHS, ...VERIFIED_IDENTITY_RESTORE_PATHS],
    async () => {
      await openDashboard(page);
      await openTab(page, "verification", { waitForReady: true });

      await expect(page.locator("h3")).toContainText(["Verified Identity"]);
      await expect(page.locator("label.toggle-switch[for='verified-identity-enabled-toggle']")).toBeVisible();
      await expect(page.locator("label.toggle-switch[for='verified-identity-native-web-bot-auth-toggle']")).toBeVisible();
      await expect(page.locator("label.toggle-switch[for='verified-identity-provider-assertions-toggle']")).toBeVisible();
      await expect(page.locator("#verified-identity-replay-window")).toBeVisible();
      await expect(page.locator("#verified-identity-attempts")).toHaveText("12");
      await expect(page.locator("#verified-identity-verified")).toHaveText("9");
      await expect(page.locator("#verified-identity-failed")).toHaveText("3");
      await expect(page.locator("#verified-identity-top-failure-reasons")).toContainText("Directory Stale");
      await expect(page.locator("#verified-identity-top-schemes")).toContainText("Http Message Signatures");
      await expect(page.locator("#verified-identity-top-categories")).toContainText("Search");

      const replayWindow = page.locator("#verified-identity-replay-window");
      const configSave = page.locator("#save-verification-all");

      if (!(await replayWindow.isVisible()) || !(await replayWindow.isEnabled())) {
        await expect(configSave).toBeHidden();
        return;
      }

      const initialReplayWindow = await replayWindow.inputValue();
      const nextReplayWindow = String(Number(initialReplayWindow || "120") === 120 ? 180 : 120);
      await replayWindow.fill(nextReplayWindow);
      await replayWindow.dispatchEvent("input");
      await expect(configSave).toBeVisible();
      await expect(configSave).toBeEnabled();

      await Promise.all([
        page.waitForResponse((resp) => (
          resp.url().includes("/admin/config") &&
          resp.request().method() === "POST" &&
          resp.status() >= 200 &&
          resp.status() < 300
        )),
        configSave.click()
      ]);

      await expect(configSave).toBeHidden();
    }
  );
});

test("advanced tab save flow validates and persists advanced JSON edits", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "advanced", { waitForReady: true });

  const saveButton = page.locator("#save-advanced-config");
  const advancedField = page.locator("#advanced-config-json");

  await expect(saveButton).toBeHidden();
  if (!(await advancedField.isVisible()) || !(await advancedField.isEditable())) {
    await expect(advancedField).toBeDisabled();
    return;
  }

  const initialAdvanced = await advancedField.inputValue();
  let parsedAdvanced;
  try {
    parsedAdvanced = JSON.parse(initialAdvanced);
  } catch (_e) {
    parsedAdvanced = {};
  }
  const nextAdvanced = {
    ...parsedAdvanced,
    rate_limit: Number(parsedAdvanced.rate_limit || 80) + 1
  };
  await advancedField.fill(JSON.stringify(nextAdvanced, null, 2));
  await advancedField.dispatchEvent("input");
  await expect(saveButton).toBeVisible();
  await expect(saveButton).toBeEnabled();

  await advancedField.fill("{invalid");
  await advancedField.dispatchEvent("input");
  await expect(saveButton).toBeDisabled();

  await advancedField.fill(initialAdvanced);
  await advancedField.dispatchEvent("input");
  await expect(saveButton).toBeHidden();
});

test("session survives reload and time-range controls refresh chart data", async ({ page }) => {
  await openDashboard(page);

  await openTab(page, "traffic");
  await page.reload();
  await expect(page).toHaveURL(/\/dashboard\/index\.html#traffic/);
  await expect(page.locator("#logout-btn")).toBeEnabled();

  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/admin/events?hours=168") && resp.ok()),
    page.click('.time-btn[data-range="week"]')
  ]);
  await expect(page.locator('.time-btn[data-range="week"]')).toHaveClass(/active/);

  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/admin/events?hours=720") && resp.ok()),
    page.click('.time-btn[data-range="month"]')
  ]);
  await expect(page.locator('.time-btn[data-range="month"]')).toHaveClass(/active/);
});

test("dashboard class contract tracks runtime and adversary-sim on html root only", async ({ page, request }) => {
  test.setTimeout(180_000);
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    const runtimeEnvironment = await fetchRuntimeEnvironment(request);
    await openDashboard(page);
    await waitForDashboardAdversarySimUiState(page, request, false);
    const toggle = page.locator("#global-adversary-sim-toggle");

    let bodyState = await dashboardDomClassState(page);
    expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
    expect(bodyState.hasAdversarySim).toBeFalsy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();
    expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
    expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

    await openTab(page, "status");
    bodyState = await dashboardDomClassState(page);
    expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
    expect(bodyState.hasAdversarySim).toBeFalsy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();
    expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
    expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

    await openTab(page, "red-team");
    await expect(toggle).not.toBeChecked();
    await clickAdversaryToggleWithRetry(page, true, 60000, request);
    await expect(toggle).toBeChecked();
    bodyState = await dashboardDomClassState(page);
    expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
    expect(bodyState.hasAdversarySim).toBeTruthy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();
    expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
    expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

    await openTab(page, "verification");
    bodyState = await dashboardDomClassState(page);
    expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
    expect(bodyState.hasAdversarySim).toBeTruthy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();
    expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
    expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

    await openTab(page, "red-team");
    await clickAdversaryToggleWithRetry(page, false, 60000, request);
    await expect(toggle).not.toBeChecked();
    bodyState = await dashboardDomClassState(page);
    expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
    expect(bodyState.hasAdversarySim).toBeFalsy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();
    expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
    expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();
  });
});

test("adversary sim global toggle drives orchestration control lifecycle state", async ({ page, request }) => {
  test.setTimeout(180_000);
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    await openDashboard(page);
    await waitForDashboardAdversarySimUiState(page, request, false);
    await openTab(page, "red-team");

    const toggle = page.locator("#global-adversary-sim-toggle");
    const lifecycleCopy = page.locator("#adversary-sim-lifecycle-copy");
    await expect(toggle).toBeEnabled();
    await expect(toggle).not.toBeChecked();
    await expect(lifecycleCopy).toContainText("Generation inactive");

    const onBody = await clickAdversaryToggleWithRetry(page, true, 60000, request);
    expect(onBody?.requested_enabled).toBe(true);
    await expect(toggle).toBeChecked();
    await expect(lifecycleCopy).toContainText("Generation active");
    let bodyState = await dashboardDomClassState(page);
    expect(bodyState.hasAdversarySim).toBeTruthy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();

    const offBody = await clickAdversaryToggleWithRetry(page, false, 60000, request);
    expect(offBody?.requested_enabled).toBe(false);
    await expect(toggle).not.toBeChecked();
    await expect(lifecycleCopy).toContainText("Generation inactive");
    await expect(lifecycleCopy).toContainText("Retained telemetry remains visible");
    bodyState = await dashboardDomClassState(page);
    expect(bodyState.hasAdversarySim).toBeFalsy();
    expect(bodyState.bodyHasAdversarySim).toBeFalsy();
  });
});

test("adversary sim toggle emits fresh telemetry visible in monitoring raw feed", async ({ page, request }) => {
  test.setTimeout(180_000);
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    await openDashboard(page);
    await waitForDashboardAdversarySimUiState(page, request, false);
    await openTab(page, "diagnostics");
    await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeHidden();
    await expect(page.locator("#refresh-now-btn")).toBeVisible();

    const toggle = page.locator("#global-adversary-sim-toggle");
    await expect(toggle).toBeEnabled({ timeout: 15000 });
    await expect(toggle).not.toBeChecked();

    const baselineMonitoring = await fetchMonitoringSnapshot(request, 24, 200);
    const baselineTs = maxSimulationEventTs(baselineMonitoring);

    try {
      await openTab(page, "red-team");
      await clickAdversaryToggleWithRetry(page, true, 60000, request);
      await expect(toggle).toBeChecked();

      const advancedTs = await waitForSimulationEventAdvance(request, baselineTs, 20000);
      await openTab(page, "diagnostics");
      await page.click("#refresh-now-btn");
      await expect(page.locator("#monitoring-raw-feed tbody")).toContainText(`"ts":${advancedTs}`);
    } finally {
      await forceAdversarySimDisabled(request);
    }
    await page.reload();
    await waitForDashboardAdversarySimUiState(page, request, false);
  });
});

test("adversary sim lane selector keeps off-state desired truth and allows agentic traffic", async ({ page, request }) => {
  test.setTimeout(180_000);
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    await controlAdversarySimViaAdmin(
      request,
      false,
      "127.0.0.1",
      95_000,
      { lane: "scrapling_traffic" }
    );
    await waitForAdversarySimControllerLeaseExpiry(request, 30000);

    await openDashboard(page);
    await waitForDashboardAdversarySimUiState(page, request, false);
    await openTab(page, "red-team");

    const laneSelect = page.locator("#adversary-sim-lane-select");
    const botRedTeamOption = page.locator('#adversary-sim-lane-select option[value="bot_red_team"]');
    await expect(laneSelect).toHaveValue("scrapling_traffic");
    await expect(botRedTeamOption).toHaveText("Agentic Traffic");
    await expect(botRedTeamOption).not.toBeDisabled();

    await laneSelect.selectOption("bot_red_team");
    await expect.poll(async () => {
      const payload = await fetchAdversarySimStatus(request);
      return String(payload?.desired_lane || "").trim().toLowerCase();
    }, { timeout: 30000 }).toBe("bot_red_team");
    await expect(laneSelect).toHaveValue("bot_red_team");

    await laneSelect.selectOption("synthetic_traffic");
    await expect.poll(async () => {
      const payload = await fetchAdversarySimStatus(request);
      return String(payload?.desired_lane || "").trim().toLowerCase();
    }, { timeout: 30000 }).toBe("synthetic_traffic");
    await expect(laneSelect).toHaveValue("synthetic_traffic");
  });
});

test("red team tab keeps controls and recent run history without machine-diagnostic clutter", async ({ page, request }) => {
  test.setTimeout(180_000);
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    await page.route("**/admin/oversight/history", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          schema_version: "oversight_history_v1",
          episode_archive: {
            schema_version: "oversight_episode_archive_v1",
            homeostasis: {
              status: "not_enough_completed_cycles"
            },
            rows: [
              {
                episode_id: "episode-red-team-1",
                proposal_status: "accepted",
                watch_window_result: "improved",
                retain_or_rollback: "retained",
                judged_lane_ids: ["scrapling_traffic", "bot_red_team"]
              }
            ]
          },
          rows: []
        })
      });
    });
    await page.route("**/admin/oversight/agent/status", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          schema_version: "oversight_agent_status_v1",
          execution_boundary: "shared_host_only",
          periodic_trigger: {
            surface: "host_supervisor_wrapper",
            wrapper_command: "scripts/run_with_oversight_supervisor.sh",
            default_interval_seconds: 300
          },
          post_sim_trigger: {
            surface: "internal_adversary_sim_completion_hook",
            qualifying_completion: "transition_to_off_with_completed_run_id_and_generated_traffic",
            dedupe_key: "sim_run_id"
          },
          candidate_window: {
            status: "running",
            canary_id: "canary-red-team",
            patch_family: "fingerprint_signal",
            requested_lane: "bot_red_team",
            requested_duration_seconds: 120,
            requested_at_ts: 1710000040,
            watch_window_end_at: 1710000160,
            follow_on_run_id: "simrun-red-team-follow-on",
            follow_on_started_at: 1710000042,
            required_runs: [
              {
                lane: "scrapling_traffic",
                status: "materialized",
                requested_at_ts: 1710000030,
                requested_duration_seconds: 30,
                follow_on_run_id: "simrun-red-team-scrapling",
                follow_on_started_at: 1710000031,
                materialized_at_ts: 1710000035
              },
              {
                lane: "bot_red_team",
                status: "running",
                requested_at_ts: 1710000040,
                requested_duration_seconds: 120,
                follow_on_run_id: "simrun-red-team-follow-on",
                follow_on_started_at: 1710000042
              }
            ]
          },
          continuation_run: {
            status: "pending",
            requested_lane: "scrapling_traffic",
            requested_duration_seconds: 30,
            requested_at_ts: 1710000200,
            source_decision_id: "ovr-red-team-1",
            source_decision_outcome: "retained",
            continue_reason: "outside_budget",
            required_runs: [
              {
                lane: "scrapling_traffic",
                status: "pending",
                requested_at_ts: 1710000200,
                requested_duration_seconds: 30
              },
              {
                lane: "bot_red_team",
                status: "pending",
                requested_at_ts: 1710000200,
                requested_duration_seconds: 120
              }
            ]
          },
          episode_archive: {
            schema_version: "oversight_episode_archive_v1",
            homeostasis: {
              status: "not_enough_completed_cycles"
            },
            rows: [
              {
                episode_id: "episode-red-team-1",
                proposal_status: "accepted",
                watch_window_result: "improved",
                retain_or_rollback: "retained",
                judged_lane_ids: ["scrapling_traffic", "bot_red_team"]
              }
            ]
          },
          recent_runs: []
        })
      });
    });
    await page.route("**/admin/adversary-sim/status", async (route) => {
      if (route.request().method() !== "GET") {
        await route.continue();
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          runtime_environment: "runtime-dev",
          adversary_sim_available: true,
          adversary_sim_enabled: false,
          generation_active: false,
          historical_data_visible: true,
          phase: "off",
          run_id: "simrun-red-team-truth",
          started_at: 0,
          ends_at: 0,
          duration_seconds: 180,
          remaining_seconds: 0,
          active_run_count: 0,
          active_lane_count: 0,
          desired_lane: "scrapling_traffic",
          active_lane: null,
          lane_switch_seq: 2,
          last_lane_switch_at: 1710000000,
          last_lane_switch_reason: "beat_boundary_reconciliation",
          queue_policy: "single_flight",
          history_retention: {
            retention_hours: 168,
            cleanup_supported: true,
            cleanup_endpoint: "/admin/adversary-sim/history/cleanup",
            cleanup_command: "make telemetry-clean"
          },
          supervisor: {
            owner: "backend_autonomous_supervisor",
            cadence_seconds: 1,
            max_catchup_ticks_per_invocation: 8,
            heartbeat_active: false,
            worker_active: false,
            last_heartbeat_at: 1710000000,
            idle_seconds: 25,
            off_state_inert: true,
            trigger_surface: "internal_beat_endpoint"
          },
          generation_diagnostics: {
            health: "ok",
            reason: "persisted_events_observed",
            recommended_action: "No action required.",
            generated_tick_count: 1,
            generated_request_count: 235,
            last_generated_at: 1710000030,
            last_generation_error: "",
            truth_basis: "persisted_event_lower_bound"
          },
          lane_diagnostics: {
            schema_version: "v1",
            truth_basis: "persisted_event_lower_bound",
            lanes: {
              synthetic_traffic: {
                beat_attempts: 0,
                beat_successes: 0,
                beat_failures: 0,
                generated_requests: 0,
                blocked_requests: 0,
                offsite_requests: 0,
                response_bytes: 0,
                response_status_count: {},
                last_generated_at: 0,
                last_error: ""
              },
              scrapling_traffic: {
                beat_attempts: 1,
                beat_successes: 1,
                beat_failures: 0,
                generated_requests: 235,
                blocked_requests: 5,
                offsite_requests: 0,
                response_bytes: 4096,
                response_status_count: { "200": 230, "429": 5 },
                last_generated_at: 1710000030,
                last_error: ""
              },
              bot_red_team: {
                beat_attempts: 0,
                beat_successes: 0,
                beat_failures: 0,
                generated_requests: 0,
                blocked_requests: 0,
                offsite_requests: 0,
                response_bytes: 0,
                response_status_count: {},
                last_generated_at: 0,
                last_error: ""
              }
            },
            request_failure_classes: {
              cancelled: { count: 0, last_seen_at: 0 },
              timeout: { count: 0, last_seen_at: 0 },
              transport: { count: 0, last_seen_at: 0 },
              http: { count: 0, last_seen_at: 0 }
            }
          },
          persisted_event_evidence: {
            run_id: "simrun-red-team-truth",
            lane: "scrapling_traffic",
            profile: "baseline",
            monitoring_event_count: 235,
            defense_delta_count: 4,
            ban_outcome_count: 1,
            first_observed_at: 1710000005,
            last_observed_at: 1710000030,
            truth_basis: "persisted_event_lower_bound"
          }
        })
      });
    });

    await openDashboard(page);
    await openTab(page, "red-team");

    await expect(page.locator("#adversary-runs")).toBeVisible();
    await expect(page.locator("#adversary-sim-lifecycle-copy")).toBeVisible();
    await expect(page.locator("#adversary-sim-generation-truth-basis")).toHaveCount(0);
    await expect(page.locator("#adversary-sim-lane-diagnostics-truth-basis")).toHaveCount(0);
    await expect(page.locator("#adversary-sim-truth-state-lower-bound")).toHaveCount(0);
    await expect(page.locator("#adversary-sim-persisted-event-evidence")).toHaveCount(0);
    await expect(page.locator("#red-team-judged-episode-basis")).toHaveCount(0);
    await expect(page.locator("#red-team-scrapling-evidence")).toHaveCount(0);
  });
});

test("adversary sim toggle cancel path avoids orchestration request when frontier keys are missing", async ({ page, request }) => {
  test.setTimeout(180_000);
  const frontierProviderCount = await fetchFrontierProviderCount(request);
  test.skip(
    frontierProviderCount > 0,
    "requires frontier provider keys to be absent in runtime env"
  );
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    await openDashboard(page);
    await waitForDashboardAdversarySimUiState(page, request, false);
    await openTab(page, "red-team");

    const toggle = page.locator("#global-adversary-sim-toggle");
    const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
    if (!(await toggle.isEnabled())) {
      // If the monitoring bootstrap hit transient read throttling, force a config-backed
      // tab refresh to repopulate control availability before asserting toggle readiness.
      await openTab(page, "status", { waitForReady: true });
      await openTab(page, "red-team");
    }
    await expect(toggle).toBeEnabled({ timeout: 15000 });
    await expect(toggle).not.toBeChecked();

    let controlRequestCount = 0;
    page.on("request", (req) => {
      if (
        req.url().includes("/admin/adversary-sim/control") &&
        req.method() === "POST"
      ) {
        controlRequestCount += 1;
      }
    });

    const dialogHandledPromise = page.waitForEvent("dialog").then(async (dialog) => {
      expect(dialog.message()).toContain("No frontier model provider keys are configured");
      await dialog.dismiss();
    });
    await Promise.all([
      dialogHandledPromise,
      toggleSwitch.click()
    ]);

    await expect(toggle).not.toBeChecked();
    await expect(page.locator('[data-tab-notice="red-team"]')).toContainText("Add SHUMA_FRONTIER_*_API_KEY");
    await page.waitForTimeout(250);
    expect(controlRequestCount).toBe(0);
  });
});

test("adversary sim toggle continue path omits the no-frontier warning after confirmation", async ({ page, request }) => {
  test.setTimeout(180_000);
  await withRestoredAdversarySimConfig(request, async () => {
    await forceAdversarySimDisabled(request);
    await page.route("**/admin/config", async (route) => {
      if (route.request().method() !== "GET") {
        await route.continue();
        return;
      }
      const response = await route.fetch();
      const payload = await response.json();
      const runtime = payload && typeof payload.runtime === "object" && payload.runtime
        ? payload.runtime
        : {};
      await route.fulfill({
        response,
        json: {
          ...payload,
          runtime: {
            ...runtime,
            frontier_provider_count: 0,
            frontier_providers: [],
            frontier_reduced_diversity_warning: false
          }
        }
      });
    });
    await openDashboard(page);
    await waitForDashboardAdversarySimUiState(page, request, false);
    await openTab(page, "red-team");

    const toggle = page.locator("#global-adversary-sim-toggle");
    const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
    if (!(await toggle.isEnabled())) {
      await openTab(page, "status", { waitForReady: true });
      await openTab(page, "red-team");
    }
    await expect(toggle).toBeEnabled({ timeout: 15000 });
    await expect(toggle).not.toBeChecked();

    let controlRequestCount = 0;
    page.on("request", (req) => {
      if (
        req.url().includes("/admin/adversary-sim/control") &&
        req.method() === "POST"
      ) {
        controlRequestCount += 1;
      }
    });

    const dialogHandledPromise = page.waitForEvent("dialog").then(async (dialog) => {
      expect(dialog.message()).toContain("No frontier model provider keys are configured");
      await dialog.accept();
    });
    await Promise.all([
      dialogHandledPromise,
      toggleSwitch.click()
    ]);

    await waitForDashboardAdversarySimUiState(page, request, true);
    await expect(page.locator('[data-tab-notice="red-team"]')).toHaveCount(0);
    await expect.poll(() => controlRequestCount, { timeout: 5000 }).toBeGreaterThan(0);
  });
});

test("traffic, game loop, red team, and ip-bans share the refresh bar while diagnostics remains manual-refresh only", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "traffic");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#auto-refresh-toggle")).not.toBeChecked();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await setAutoRefresh(page, true);
  await expect(page.locator("#refresh-now-btn")).toBeHidden();
  await expect(page.locator("#refresh-mode")).toContainText("ON");
  await setAutoRefresh(page, false);
  await expect(page.locator("#refresh-now-btn")).toBeVisible();

  await openTab(page, "game-loop");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#auto-refresh-toggle")).not.toBeChecked();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await setAutoRefresh(page, true);
  await expect(page.locator("#refresh-now-btn")).toBeHidden();
  await expect(page.locator("#refresh-mode")).toContainText("ON");
  await setAutoRefresh(page, false);
  await expect(page.locator("#refresh-now-btn")).toBeVisible();

  await openTab(page, "diagnostics");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeHidden();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await expect(page.locator("#refresh-mode")).toContainText("Manual refresh only");

  await openTab(page, "status");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeHidden();
  await expect(page.locator("#refresh-now-btn")).toBeHidden();

  await openTab(page, "red-team");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#auto-refresh-toggle")).not.toBeChecked();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await setAutoRefresh(page, true);
  await expect(page.locator("#refresh-now-btn")).toBeHidden();
  await expect(page.locator("#refresh-mode")).toContainText("ON");

  await openTab(page, "ip-bans");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#auto-refresh-toggle")).toBeChecked();
  await expect(page.locator("#refresh-now-btn")).toBeHidden();
  await setAutoRefresh(page, false);
  await expect(page.locator("#refresh-now-btn")).toBeVisible();

  await openTab(page, "diagnostics");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeHidden();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await expect(page.locator("#refresh-mode")).toContainText("Manual refresh only");
});

test("traffic manual refresh hydrates full snapshot even when first delta page is empty", async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  const buildMonitoringPayload = (reason) => ({
    summary: {
      honeypot: { total_hits: 2, unique_crawlers: 1, top_crawlers: [], top_paths: [] },
      challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
      pow: {
        total_failures: 0,
        total_successes: 0,
        total_attempts: 0,
        success_ratio: 0,
        unique_offenders: 0,
        top_offenders: [],
        reasons: {},
        outcomes: {},
        trend: []
      },
      rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
      geo: { total_violations: 0, actions: { block: 0, challenge: 0, maze: 0 }, top_countries: [] }
    },
    prometheus: { endpoint: "/metrics", notes: [] },
    details: {
      analytics: { ban_count: 0, shadow_mode: false, fail_mode: "open" },
      events: {
        recent_events: [{
          ts: now,
          event: "Challenge",
          ip: "198.51.100.42",
          reason,
          outcome: "served",
          admin: "ops"
        }],
        event_counts: { Challenge: 1 },
        top_ips: [["198.51.100.42", 1]],
        unique_ips: 1
      },
      bans: { bans: [] },
      maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
      cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
      cdp_events: { events: [] }
    }
  });

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(buildMonitoringPayload("historical-baseline-visible"))
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    const url = new URL(route.request().url());
    const limit = Number.parseInt(url.searchParams.get("limit") || "0", 10);
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        after_cursor: afterCursor,
        window_end_cursor: "cursor-1",
        next_cursor: "cursor-1",
        has_more: false,
        overflow: limit === 1 ? "none" : "limit_exceeded",
        events: [],
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "traffic");
  await page.click("#refresh-now-btn");
  await expect(page.locator("#monitoring-events tbody")).toContainText("historical-baseline-visible");
});

test("red team recent runs table renders compact run-history rows from monitoring payloads", async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  const buildMonitoringPayload = () => ({
    summary: {
      honeypot: { total_hits: 0, unique_crawlers: 0, top_crawlers: [], top_paths: [] },
      challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
      pow: {
        total_failures: 0,
        total_successes: 0,
        total_attempts: 0,
        success_ratio: 0,
        unique_offenders: 0,
        top_offenders: [],
        reasons: {},
        outcomes: {},
        trend: []
      },
      rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
      geo: { total_violations: 0, actions: { block: 0, challenge: 0, maze: 0 }, top_countries: [] }
    },
    prometheus: { endpoint: "/metrics", notes: [] },
    details: {
      analytics: { ban_count: 1, shadow_mode: false, fail_mode: "open" },
      events: {
        recent_events: [],
        recent_sim_runs: [
          {
            run_id: "simrun-ui-2",
            lane: "deterministic_black_box",
            profile: "runtime_toggle",
            first_ts: now - 10,
            last_ts: now,
            monitoring_event_count: 11,
            defense_delta_count: 3,
            ban_outcome_count: 1
          },
          {
            run_id: "simrun-ui-1",
            lane: "deterministic_black_box",
            profile: "runtime_toggle",
            first_ts: now - 30,
            last_ts: now - 20,
            monitoring_event_count: 2,
            defense_delta_count: 1,
            ban_outcome_count: 0
          }
        ],
        event_counts: {},
        top_ips: [],
        unique_ips: 0
      },
      bans: {
        bans: [{
          ip: "198.51.100.55",
          reason: "rate",
          banned_at: now,
          expires: now + 300
        }]
      },
      maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
      cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
      cdp_events: { events: [] }
    }
  });

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(buildMonitoringPayload())
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    const url = new URL(route.request().url());
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        after_cursor: afterCursor,
        window_end_cursor: "cursor-run-history",
        next_cursor: "cursor-run-history",
        has_more: false,
        overflow: "none",
        events: [],
        recent_sim_runs: buildMonitoringPayload().details.events.recent_sim_runs,
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "red-team");
  await expect(page.locator("#adversary-runs tbody")).toContainText("simrun-ui-2");
  await expect(page.locator("#adversary-runs tbody")).toContainText("simrun-ui-1");
});

test("red team tab surfaces llm runtime lineage in recent adversary runs", async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  const buildMonitoringPayload = () => ({
    summary: {
      honeypot: { total_hits: 0, unique_crawlers: 0, top_crawlers: [], top_paths: [] },
      challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
      pow: {
        total_failures: 0,
        total_successes: 0,
        total_attempts: 0,
        success_ratio: 0,
        unique_offenders: 0,
        top_offenders: [],
        reasons: {},
        outcomes: {},
        trend: []
      },
      rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
      geo: { total_violations: 0, actions: { block: 0, challenge: 0, maze: 0 }, top_countries: [] }
    },
    prometheus: { endpoint: "/metrics", notes: [] },
    details: {
      analytics: { ban_count: 0, shadow_mode: false, fail_mode: "open" },
      events: {
        recent_events: [],
        recent_sim_runs: [
          {
            run_id: "simrun-llm-runtime",
            lane: "bot_red_team",
            profile: "llm_runtime_lane",
            observed_fulfillment_modes: ["request_mode"],
            observed_category_ids: ["ai_scraper_bot", "http_agent"],
            first_ts: now - 20,
            last_ts: now,
            monitoring_event_count: 0,
            defense_delta_count: 0,
            ban_outcome_count: 0,
            llm_runtime_summary: {
              receipt_count: 1,
              fulfillment_mode: "request_mode",
              category_targets: ["http_agent", "ai_scraper_bot"],
              backend_kind: "frontier_reference",
              backend_state: "configured",
              generation_source: "provider_response",
              provider: "openai",
              model_id: "gpt-5-mini",
              fallback_reason: null,
              generated_action_count: 2,
              executed_action_count: 2,
              failed_action_count: 0,
              passed_tick_count: 1,
              failed_tick_count: 0,
              last_response_status: 404,
              failure_class: null,
              error: null,
              terminal_failure: null,
              latest_action_receipts: [
                {
                  action_index: 1,
                  action_type: "http_get",
                  path: "/",
                  label: "root",
                  status: 200,
                  error: null
                },
                {
                  action_index: 2,
                  action_type: "http_get",
                  path: "/robots.txt",
                  label: "robots",
                  status: 404,
                  error: null
                }
              ]
            }
          }
        ],
        event_counts: {},
        top_ips: [],
        unique_ips: 0
      },
      bans: { bans: [] },
      maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
      cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
      cdp_events: { events: [] }
    }
  });

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(buildMonitoringPayload())
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    const url = new URL(route.request().url());
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        after_cursor: afterCursor,
        window_end_cursor: "cursor-llm-runtime",
        next_cursor: "cursor-llm-runtime",
        has_more: false,
        overflow: "none",
        events: [],
        recent_sim_runs: buildMonitoringPayload().details.events.recent_sim_runs,
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });
  await page.route("**/admin/ip-bans*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        items: [],
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "red-team");

  const row = page.locator("#adversary-runs tbody tr").first();
  await expect(row).toContainText("simrun-llm-runtime");
  await expect(row).toContainText("Agentic Traffic");
  await expect(row).toContainText("Request Mode");
  await expect(row).toContainText("AI Scraper Bot, HTTP Agent");
  await expect(row).toContainText("Provider Response");
  await expect(row).toContainText("OpenAI");
  await expect(row).toContainText("gpt-5-mini");
  await expect(row).toContainText("2 / 2 actions");
});

test("manual refresh button appends new monitoring delta events when auto-refresh is off", async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  const buildMonitoringPayload = (reason = "historical-baseline") => ({
    summary: {
      honeypot: { total_hits: 2, unique_crawlers: 1, top_crawlers: [], top_paths: [] },
      challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
      pow: {
        total_failures: 0,
        total_successes: 0,
        total_attempts: 0,
        success_ratio: 0,
        unique_offenders: 0,
        top_offenders: [],
        reasons: {},
        outcomes: {},
        trend: []
      },
      rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
      geo: { total_violations: 0, actions: { block: 0, challenge: 0, maze: 0 }, top_countries: [] }
    },
    prometheus: { endpoint: "/metrics", notes: [] },
    details: {
      analytics: { ban_count: 0, shadow_mode: false, fail_mode: "open" },
      events: {
        recent_events: [{
          ts: now,
          event: "Challenge",
          ip: "198.51.100.77",
          reason,
          outcome: "served",
          admin: "ops"
        }],
        event_counts: { Challenge: 1 },
        top_ips: [["198.51.100.77", 1]],
        unique_ips: 1
      },
      bans: { bans: [] },
      maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
      cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
      cdp_events: { events: [] }
    }
  });

  let deltaRequestCount = 0;
  let manualRefreshTriggered = false;
  let nonFreshnessDeltaCount = 0;
  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    const reason = manualRefreshTriggered
      ? "manual-refresh-delta-event"
      : "historical-baseline";
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(buildMonitoringPayload(reason))
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    deltaRequestCount += 1;
    const url = new URL(route.request().url());
    const limit = Number.parseInt(url.searchParams.get("limit") || "0", 10);
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
    if (limit === 1 || !afterCursor) {
      if (limit !== 1) {
        nonFreshnessDeltaCount += 1;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          after_cursor: "",
          window_end_cursor: "cursor-1",
          next_cursor: "cursor-1",
          has_more: false,
          overflow: "none",
          events: [],
          freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
        })
      });
      return;
    }
    nonFreshnessDeltaCount += 1;
    if (nonFreshnessDeltaCount < 2) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          after_cursor: afterCursor,
          window_end_cursor: "cursor-1",
          next_cursor: "cursor-1",
          has_more: false,
          overflow: "none",
          events: [],
          freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
        })
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        after_cursor: afterCursor,
        window_end_cursor: "cursor-2",
        next_cursor: "cursor-2",
        has_more: false,
        overflow: "none",
        events: [{
          cursor: "cursor-2",
          ts: now + 1,
          event: "Challenge",
          ip: "198.51.100.88",
          reason: "manual-refresh-delta-event",
          outcome: "served",
          admin: "ops"
        }],
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "traffic");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await expect(page.locator("#monitoring-events tbody")).not.toContainText("manual-refresh-delta-event");

  const beforeRefreshDeltaCalls = deltaRequestCount;
  manualRefreshTriggered = true;
  await page.click("#refresh-now-btn");
  await expect(page.locator("#monitoring-events tbody")).toContainText("manual-refresh-delta-event");
  expect(deltaRequestCount).toBeGreaterThan(beforeRefreshDeltaCalls);
});

test("traffic recent-event filters use canonical shared control classes", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "traffic");

  await expect(page.locator("#monitoring-event-filters .input-row")).toHaveCount(6);
  await expect(page.locator("#monitoring-event-filters .field-row")).toHaveCount(0);
  await expect(page.locator("#monitoring-event-filters .control-label.control-label--wide")).toHaveCount(6);
  await expect(page.locator("#monitoring-event-filters select.input-field")).toHaveCount(6);
});

test("route remount preserves keyboard navigation, ban/unban, verification save, and polling", async ({ page, request }) => {
  let ipBansRefreshRequests = 0;
  page.on("request", (request) => {
    if (request.method() !== "GET") {
      return;
    }
    const url = request.url();
    if (
      url.includes("/admin/ban") ||
      url.includes("/admin/ip-bans/delta?hours=24")
    ) {
      ipBansRefreshRequests += 1;
    }
  });

  await openDashboard(page);
  await page.goto("about:blank");
  await openDashboard(page);

  const trafficTab = page.locator("#dashboard-tab-traffic");
  await trafficTab.focus();
  await page.keyboard.press("ArrowRight");
  await expect(page).toHaveURL(/#ip-bans$/);
  await assertActiveTabPanelVisibility(page, "ip-bans");

  const ip = "198.51.100.211";
  await page.fill("#ban-ip", ip);
  await page.dispatchEvent("#ban-ip", "input");
  await expect(page.locator("#ban-btn")).toBeEnabled();
  await Promise.all([
    page.waitForResponse((resp) => (
      resp.url().includes("/admin/ban") &&
      resp.request().method() === "POST" &&
      resp.status() >= 200 &&
      resp.status() < 300
    )),
    page.click("#ban-btn")
  ]);
  await page.fill("#unban-ip", ip);
  await page.dispatchEvent("#unban-ip", "input");
  await expect(page.locator("#unban-btn")).toBeEnabled();
  await Promise.all([
    page.waitForResponse((resp) => (
      resp.url().includes("/admin/unban") &&
      resp.request().method() === "POST" &&
      resp.status() >= 200 &&
      resp.status() < 300
    )),
    page.click("#unban-btn")
  ]);

  await withRestoredAdminConfig(request, VERIFICATION_RESTORE_PATHS, async () => {
    await openTab(page, "verification");
    const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
    const configSave = page.locator("#save-verification-all");
    if (await jsRequiredToggle.isVisible() && await jsRequiredToggle.isEnabled()) {
      await jsRequiredToggle.click();
      await expect(configSave).toBeEnabled();
      await Promise.all([
        page.waitForResponse((resp) => (
          resp.url().includes("/admin/config") &&
          resp.request().method() === "POST" &&
          resp.status() >= 200 &&
          resp.status() < 300
        )),
        configSave.click()
      ]);
      await expect(configSave).toBeHidden();
    }
  });

  await openTab(page, "ip-bans");
  await setAutoRefresh(page, true);
  await page.waitForTimeout(150);
  const beforePollWait = ipBansRefreshRequests;
  await page.waitForTimeout(1300);
  expect(ipBansRefreshRequests).toBeGreaterThan(beforePollWait);
});

test("traffic manual refresh avoids placeholder flicker and bounds table churn", async ({ page }) => {
  let monitoringSnapshotRequests = 0;
  let monitoringDeltaRequests = 0;
  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    monitoringSnapshotRequests += 1;
    const sample = monitoringSnapshotRequests;
    await new Promise((resolve) => setTimeout(resolve, 120));
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        summary: {
          honeypot: {
            total_hits: sample * 3,
            unique_crawlers: sample,
            top_crawlers: [{ label: "203.0.113.200", count: sample * 3 }],
            top_paths: [{ label: "/instaban", count: sample * 3 }]
          },
          challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
          pow: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
          rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
          geo: { total_violations: 0, actions: { block: 0, challenge: 0, maze: 0 }, top_countries: [] }
        },
        prometheus: { endpoint: "/metrics", notes: [] },
        details: {
          analytics: { ban_count: 1, shadow_mode: false, fail_mode: "open" },
          events: {
            recent_events: [
              {
                ts: Math.floor(Date.now() / 1000),
                event: "Challenge",
                ip: "198.51.100.1",
                reason: "risk",
                outcome: "served",
                admin: "ops"
              }
            ],
            event_counts: { Challenge: 1 },
            top_ips: [["198.51.100.1", 1]],
            unique_ips: 1
          },
          bans: { bans: [{ ip: "198.51.100.2", reason: "manual_ban", expires: 1999999999 }] },
          maze: { total_hits: sample * 3, unique_crawlers: sample, maze_auto_bans: 0, top_crawlers: [] },
          cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
          cdp_events: { events: [] }
        }
      })
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    monitoringDeltaRequests += 1;
    const url = new URL(route.request().url());
    const limit = Number.parseInt(url.searchParams.get("limit") || "0", 10);
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
    if (limit === 1 || !afterCursor) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          after_cursor: "",
          window_end_cursor: "cursor-baseline",
          next_cursor: "cursor-baseline",
          has_more: false,
          overflow: "none",
          events: [],
          freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
        })
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        after_cursor: afterCursor,
        window_end_cursor: afterCursor,
        next_cursor: afterCursor,
        has_more: false,
        overflow: "none",
        events: [],
        freshness: { state: "fresh", lag_ms: 0, transport: "cursor_delta_poll" }
      })
    });
  });

  await openDashboard(page);
  await openTab(page, "traffic");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await expect(page.locator("#honeypot-total-hits")).not.toHaveText("...");

  await page.evaluate(() => {
    const tbody = document.querySelector("#monitoring-events tbody");
    window.__shumaEventRowMutations = 0;
    if (!tbody) return;
    const observer = new MutationObserver((records) => {
      for (const record of records) {
        if (record.type === "childList") {
          window.__shumaEventRowMutations += record.addedNodes.length + record.removedNodes.length;
        }
      }
    });
    observer.observe(tbody, { childList: true });
    window.__shumaEventObserver = observer;
  });

  const beforePollWindow = monitoringSnapshotRequests + monitoringDeltaRequests;
  await page.click("#refresh-now-btn");
  const afterPollWindow = monitoringSnapshotRequests + monitoringDeltaRequests;
  expect(monitoringSnapshotRequests).toBeGreaterThanOrEqual(1);
  expect(afterPollWindow).toBeGreaterThan(beforePollWindow);

  const honeypotSamples = await page.evaluate(() => {
    const samples = [];
    for (let i = 0; i < 3; i += 1) {
      samples.push(String(document.getElementById("honeypot-total-hits")?.textContent || "").trim());
    }
    return samples;
  });
  honeypotSamples.forEach((sample) => {
    expect(sample).not.toBe("...");
    expect(sample).toMatch(/^\d[\d,]*$/);
  });

  const rowMutations = await page.evaluate(() => {
    if (window.__shumaEventObserver) {
      window.__shumaEventObserver.disconnect();
      window.__shumaEventObserver = null;
    }
    return Number(window.__shumaEventRowMutations || 0);
  });
  expect(rowMutations).toBeLessThanOrEqual(2);
});

test("repeated route remount loops keep polling request fan-out bounded", async ({ page }) => {
  test.setTimeout(90_000);
  const remountObservationWindowMs = 1300;
  const maxExpectedRequestsInWindow = 6;

  let ipBansRequests = 0;
  page.on("request", (request) => {
    if (
      request.method() === "GET" &&
      (
        request.url().includes("/admin/ban") ||
        request.url().includes("/admin/ip-bans/delta?hours=")
      )
    ) {
      ipBansRequests += 1;
    }
  });

  const remountRequestDeltas = [];
  const remountCycles = 4;
  for (let cycle = 0; cycle < remountCycles; cycle += 1) {
    await openDashboard(page);
    await openTab(page, "ip-bans");
    await setAutoRefresh(page, true);
    const beforeWindow = ipBansRequests;
    await page.waitForTimeout(remountObservationWindowMs);
    let delta = ipBansRequests - beforeWindow;
    let maxRequestsForObservedWindow = maxExpectedRequestsInWindow;
    if (delta === 0) {
      // Polling is serialized behind in-flight refresh work; allow one extra
      // cadence window before failing this cycle.
      const extraWindowMs = 1200;
      await page.waitForTimeout(extraWindowMs);
      delta = ipBansRequests - beforeWindow;
      maxRequestsForObservedWindow = 5;
    }
    remountRequestDeltas.push(delta);
    expect(delta).toBeLessThanOrEqual(maxRequestsForObservedWindow);
    await page.goto("about:blank");
  }

  const positiveCycles = remountRequestDeltas.filter((delta) => delta > 0).length;
  expect(positiveCycles).toBeGreaterThan(0);
  const maxDelta = Math.max(...remountRequestDeltas);
  const minDelta = Math.min(...remountRequestDeltas);
  // Repeated remounts can jitter one cadence window; larger spreads indicate
  // duplicate polling loops or stalled scheduling.
  expect(maxDelta - minDelta).toBeLessThanOrEqual(3);
});

test("native remount soak keeps refresh p95 and polling cadence within bounds", async ({ page }) => {
  test.setTimeout(90_000);
  const soakWindowMs = 1300;
  const maxExpectedRequestsInWindow = 4;
  const maxFetchP95Ms = 2500;
  const maxRenderP95Ms = 80;

  let ipBansRequests = 0;
  const delayedPassThrough = async (route) => {
    ipBansRequests += 1;
    await page.waitForTimeout(18);
    await route.continue();
  };
  await page.route("**/admin/ban", delayedPassThrough);
  await page.route("**/admin/ip-bans/delta?*", delayedPassThrough);

  const cadenceDeltas = [];
  const fetchP95Samples = [];
  const renderP95Samples = [];
  const remountCycles = 5;

  for (let cycle = 0; cycle < remountCycles; cycle += 1) {
    await openDashboard(page);
    await openTab(page, "ip-bans");
    await setAutoRefresh(page, true);

    const before = ipBansRequests;
    await page.waitForTimeout(soakWindowMs);
    let delta = ipBansRequests - before;
    if (delta === 0) {
      // Give the polling loop one more cadence window to tick before failing cadence.
      await page.waitForTimeout(1200);
      delta = ipBansRequests - before;
    }
    cadenceDeltas.push(delta);
    expect(delta).toBeLessThanOrEqual(maxExpectedRequestsInWindow);

    await openTab(page, "status");
    const telemetry = await page.evaluate(() => {
      const parseP95 = (id) => {
        const text = document.getElementById(id)?.textContent || "";
        const match = /p95:?\s*([0-9]+(?:\.[0-9]+)?)\s*ms/i.exec(text);
        return match ? Number(match[1]) : NaN;
      };
      return {
        fetchP95: parseP95("runtime-fetch-latency-avg"),
        renderP95: parseP95("runtime-render-timing-avg")
      };
    });
    expect(Number.isFinite(telemetry.fetchP95)).toBe(true);
    expect(Number.isFinite(telemetry.renderP95)).toBe(true);
    fetchP95Samples.push(telemetry.fetchP95);
    renderP95Samples.push(telemetry.renderP95);
    expect(telemetry.fetchP95).toBeLessThanOrEqual(maxFetchP95Ms);
    // Browser scheduling jitter in CI/sandbox can push render p95 above a frame budget
    // even when polling fan-out remains bounded; keep this threshold regression-sensitive
    // without making the soak check flaky.
    expect(telemetry.renderP95).toBeLessThanOrEqual(maxRenderP95Ms);

    await page.goto("about:blank");
  }

  const maxCadence = Math.max(...cadenceDeltas);
  const minCadence = Math.min(...cadenceDeltas);
  const positiveCadenceCycles = cadenceDeltas.filter((delta) => delta > 0).length;
  expect(positiveCadenceCycles).toBeGreaterThan(0);
  expect(maxCadence - minCadence).toBeLessThanOrEqual(4);
  expect(Math.max(...fetchP95Samples)).toBeLessThanOrEqual(maxFetchP95Ms);
  expect(Math.max(...renderP95Samples)).toBeLessThanOrEqual(maxRenderP95Ms);
});

test("verification save roundtrip clears dirty state after successful write", async ({ page, request }) => {
  await withRestoredAdminConfig(request, VERIFICATION_RESTORE_PATHS, async () => {
    await openDashboard(page);
    await openTab(page, "verification");

    const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
    const configSave = page.locator("#save-verification-all");
    if (!(await jsRequiredToggle.isVisible()) || !(await jsRequiredToggle.isEnabled())) {
      await expect(configSave).toBeHidden();
      return;
    }

    await jsRequiredToggle.click();
    await expect(configSave).toBeEnabled();

    await Promise.all([
      page.waitForResponse((resp) => (
        resp.url().includes("/admin/config") &&
        resp.request().method() === "POST" &&
        resp.status() >= 200 &&
        resp.status() < 300
      )),
      configSave.click()
    ]);
    await expect(configSave).toBeHidden();
  });
});

test("geo and tuning save flows cover GEO lists and botness controls", async ({ page, request }) => {
  await withRestoredAdminConfig(request, GEO_AND_TUNING_RESTORE_PATHS, async () => {
    await openDashboard(page);
    await openTab(page, "geo");

    const geoSave = page.locator("#save-geo-config");
    await expect(geoSave).toBeHidden();

    const geoScoringToggle = page.locator("#geo-scoring-toggle");
    const geoScoringSwitch = page.locator("label.toggle-switch[for='geo-scoring-toggle']");
    if (await geoScoringSwitch.isVisible() && await geoScoringToggle.isEnabled()) {
      await geoScoringSwitch.click();
      await submitConfigSave(page, geoSave);
    }

    const geoRoutingToggle = page.locator("#geo-routing-toggle");
    const geoRoutingSwitch = page.locator("label.toggle-switch[for='geo-routing-toggle']");
    if (await geoRoutingSwitch.isVisible() && await geoRoutingToggle.isEnabled()) {
      await geoRoutingSwitch.click();
      await submitConfigSave(page, geoSave);
    }

    const geoRiskList = page.locator("#geo-risk-list");
    if (await geoRiskList.isVisible() && await geoRiskList.isEnabled()) {
      const geoRiskInitial = await geoRiskList.inputValue();
      const geoRiskNext = geoRiskInitial.includes("CA")
        ? geoRiskInitial.replace(/\bCA\b,?/g, "").replace(/(^,|,,|,$)/g, "")
        : (geoRiskInitial ? `${geoRiskInitial},CA` : "CA");
      await geoRiskList.fill(geoRiskNext);
      await geoRiskList.dispatchEvent("input");
      await submitConfigSave(page, geoSave);
    }

    const geoAllowList = page.locator("#geo-allow-list");
    if (await geoAllowList.isVisible() && await geoAllowList.isEnabled()) {
      const geoAllowInitial = await geoAllowList.inputValue();
      const geoAllowNext = geoAllowInitial.includes("GB")
        ? geoAllowInitial.replace(/\bGB\b,?/g, "").replace(/(^,|,,|,$)/g, "")
        : (geoAllowInitial ? `${geoAllowInitial},GB` : "GB");
      await geoAllowList.fill(geoAllowNext);
      await geoAllowList.dispatchEvent("input");
      await submitConfigSave(page, geoSave);
    }

    await openTab(page, "tuning");
    const tuningSave = page.locator("#save-tuning-all");
    const botnessWeight = page.locator("#weight-js-required");
    if (await botnessWeight.isVisible() && await botnessWeight.isEnabled()) {
      const botnessInitial = await botnessWeight.inputValue();
      const nextWeight = Number(botnessInitial || "1") >= 10
        ? "9"
        : String(Number(botnessInitial || "1") + 1);
      await botnessWeight.fill(nextWeight);
      await botnessWeight.dispatchEvent("input");
      await submitConfigSave(page, tuningSave);
    }

  });
});

test("geo tab hides trusted edge header controls outside edge-fermyon posture", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "geo");

  await expect(page.locator("#geo-edge-header-enabled-toggle")).toHaveCount(0);
  await expect(page.locator("#geo-edge-unavailable-message")).toContainText(
    "available only when Shuma-Gorath is deployed on Akamai edge"
  );
  await expect(page.locator("#save-geo-config")).toBeHidden();
});

test("rate-limiting tab save flows cover local controls", async ({ page, request }) => {
  await withRestoredAdminConfig(request, RATE_LIMITING_RESTORE_PATHS, async () => {
    await openDashboard(page);
    await openTab(page, "rate-limiting");

    const saveButton = page.locator("#save-rate-limiting-config");
    await expect(saveButton).toBeHidden();

    const rateThreshold = page.locator("#rate-limit-threshold");
    if (await rateThreshold.isVisible() && await rateThreshold.isEnabled()) {
      const initialRateThreshold = await rateThreshold.inputValue();
      const nextRateThreshold = String(Math.max(1, Number(initialRateThreshold || "80") + 1));
      await rateThreshold.fill(nextRateThreshold);
      await rateThreshold.dispatchEvent("input");
      await submitConfigSave(page, saveButton);
    }

    const rateEnabledToggle = page.locator("#rate-limiting-enabled-toggle");
    const rateEnabledSwitch = page.locator("label.toggle-switch[for='rate-limiting-enabled-toggle']");
    if (await rateEnabledSwitch.isVisible() && await rateEnabledToggle.isEnabled()) {
      await rateEnabledSwitch.click();
      await submitConfigSave(page, saveButton);
    }
  });
});

test("rate-limiting tab hides external backend controls outside edge-fermyon posture", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "rate-limiting");

  await expect(page.locator("#rate-external-backend-enabled-toggle")).toHaveCount(0);
  await expect(page.locator("#rate-edge-unavailable-message")).toContainText(
    "available only when Shuma-Gorath is deployed on Akamai edge"
  );
});

test("fingerprinting tab hides Akamai controls outside edge-fermyon posture", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "fingerprinting");

  await expect(page.locator("#fingerprinting-akamai-enabled-toggle")).toHaveCount(0);
  await expect(page.locator("#fingerprinting-edge-mode-select")).toHaveCount(0);
  await expect(page.locator("#fingerprinting-akamai-unavailable-message")).toContainText(
    "available only when Shuma-Gorath is deployed on Akamai edge"
  );
  await expect(page.locator("#save-fingerprinting-config")).toBeHidden();
});

test("policy tab save flows cover robots serving, durations, browser policy, and path allowlist controls", async ({ page, request }) => {
  await withRestoredAdminConfig(request, POLICY_RESTORE_PATHS, async () => {
    await openDashboard(page);
    await openTab(page, "policy");

    await expect(page.locator("#dur-honeypot-days")).toBeVisible();
    await expect(page.locator("#dur-ip-range-honeypot-days")).toBeVisible();
    await expect(page.locator("#dur-maze-crawler-days")).toBeVisible();
    await expect(page.locator("#dur-rate-limit-days")).toBeVisible();
    await expect(page.locator("#dur-cdp-days")).toBeVisible();
    await expect(page.locator("#dur-edge-fingerprint-days")).toBeVisible();
    await expect(page.locator("#dur-tarpit-persistence-days")).toBeVisible();
    await expect(page.locator("#dur-not-a-bot-abuse-days")).toBeVisible();
    await expect(page.locator("#dur-challenge-puzzle-abuse-days")).toBeVisible();
    await expect(page.locator("#dur-admin-days")).toBeVisible();

    const saveButton = page.locator("#save-policy-config");
    await expect(saveButton).toBeHidden();

    const robotsCrawlDelay = page.locator("#robots-crawl-delay");
    if (!(await robotsCrawlDelay.isVisible()) || !(await robotsCrawlDelay.isEditable())) {
      await expect(robotsCrawlDelay).toBeDisabled();
      return;
    }

    const robotsDelayInitial = await robotsCrawlDelay.inputValue();
    const robotsDelayNext = String(Math.min(60, Number(robotsDelayInitial || "2") + 1));
    await robotsCrawlDelay.fill(robotsDelayNext);
    await robotsCrawlDelay.dispatchEvent("input");
    await submitConfigSave(page, saveButton);

    const aiToggle = page.locator("#robots-block-training-toggle");
    const aiToggleSwitch = page.locator("label.toggle-switch[for='robots-block-training-toggle']");
    if (await aiToggleSwitch.isVisible() && await aiToggle.isEnabled()) {
      await aiToggleSwitch.click();
      await submitConfigSave(page, saveButton);
    }

    const durationHours = page.locator("#dur-rate-limit-hours");
    if (await durationHours.isVisible() && await durationHours.isEnabled()) {
      const initialDurationHours = await durationHours.inputValue();
      const nextDurationHours = String(Math.min(23, Number(initialDurationHours || "1") + 1));
      await durationHours.fill(nextDurationHours);
      await durationHours.dispatchEvent("input");
      await submitConfigSave(page, saveButton);
    }

    const tarpitDurationMinutes = page.locator("#dur-tarpit-persistence-minutes");
    if (await tarpitDurationMinutes.isVisible() && await tarpitDurationMinutes.isEnabled()) {
      const initialTarpitMinutes = await tarpitDurationMinutes.inputValue();
      const nextTarpitMinutes = String((Number(initialTarpitMinutes || "10") + 5) % 60);
      await tarpitDurationMinutes.fill(nextTarpitMinutes);
      await tarpitDurationMinutes.dispatchEvent("input");
      await submitConfigSave(page, saveButton);
    }

    const browserPolicyToggle = page.locator("#browser-policy-toggle");
    const browserPolicySwitch = page.locator("label.toggle-switch[for='browser-policy-toggle']");
    if (await browserPolicySwitch.isVisible() && await browserPolicyToggle.isEnabled()) {
      await browserPolicySwitch.click();
      await submitConfigSave(page, saveButton);
    }

    const browserBlockRules = page.locator("#browser-block-rules");
    if (await browserBlockRules.isVisible() && await browserBlockRules.isEnabled()) {
      const initialBrowserRules = await browserBlockRules.inputValue();
      const candidateRule = "Brave,120";
      const existingRules = initialBrowserRules
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line.length > 0);
      const nextBrowserRules = existingRules.includes(candidateRule)
        ? existingRules.filter((line) => line !== candidateRule).join("\n")
        : [...existingRules, candidateRule].join("\n");
      await browserBlockRules.fill(nextBrowserRules);
      await browserBlockRules.dispatchEvent("input");
      await submitConfigSave(page, saveButton);
    }

    const pathAllowlist = page.locator("#path-allowlist");
    if (await pathAllowlist.isVisible() && await pathAllowlist.isEnabled()) {
      const initialPathAllowlist = await pathAllowlist.inputValue();
      const candidatePath = "/webhook/stripe";
      const existingPaths = initialPathAllowlist
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line.length > 0);
      const nextPathAllowlist = existingPaths.includes(candidatePath)
        ? existingPaths.filter((line) => line !== candidatePath).join("\n")
        : [...existingPaths, candidatePath].join("\n");
      await pathAllowlist.fill(nextPathAllowlist);
      await pathAllowlist.dispatchEvent("input");
      await submitConfigSave(page, saveButton);
    }

    const pathAllowlistToggle = page.locator("#path-allowlist-enabled-toggle");
    const pathAllowlistSwitch = page.locator("label.toggle-switch[for='path-allowlist-enabled-toggle']");
    if (await pathAllowlistSwitch.isVisible() && await pathAllowlistToggle.isEnabled()) {
      await pathAllowlistSwitch.click();
      await submitConfigSave(page, saveButton);
    }
  });
});

test("config-backed tabs rehydrate external config mutations without surfacing unsaved changes", async ({ page, request }) => {
  await withRestoredAdminConfig(
    request,
    [...POLICY_RESTORE_PATHS, ...TRAPS_RESTORE_PATHS],
    async () => {
      const initialConfig = await fetchAdminConfig(request);
      const initialBrowserPolicyEnabled = initialConfig.browser_policy_enabled !== false;
      const initialBrowserRules = formatBrowserRules(initialConfig.browser_block);
      const initialPathAllowlist = formatTextareaList(initialConfig.path_allowlist);
      const initialRateLimitDuration = durationParts(initialConfig?.ban_durations?.rate_limit);
      const initialTarpitPersistenceDuration = durationParts(initialConfig?.ban_durations?.tarpit_persistence);
      const initialMazeAutoBan = initialConfig.maze_auto_ban !== false;
      const initialHoneypotPaths = formatTextareaList(initialConfig.honeypots);

      await openDashboard(page);
      await openTab(page, "policy", { waitForReady: true });
      await expect(page.locator("#dur-rate-limit-days")).toHaveValue(String(initialRateLimitDuration.days));
      await expect(page.locator("#dur-rate-limit-hours")).toHaveValue(String(initialRateLimitDuration.hours));
      await expect(page.locator("#dur-rate-limit-minutes")).toHaveValue(String(initialRateLimitDuration.minutes));
      await expect(page.locator("#dur-tarpit-persistence-days")).toHaveValue(String(initialTarpitPersistenceDuration.days));
      await expect(page.locator("#dur-tarpit-persistence-hours")).toHaveValue(String(initialTarpitPersistenceDuration.hours));
      await expect(page.locator("#dur-tarpit-persistence-minutes")).toHaveValue(String(initialTarpitPersistenceDuration.minutes));
      await expect(page.locator("#browser-block-rules")).toHaveValue(initialBrowserRules);
      await expect(page.locator("#path-allowlist")).toHaveValue(initialPathAllowlist);
      await expect(page.locator("#save-policy-config")).toBeHidden();
      await expect(page.locator("#dashboard-panel-policy")).toContainText("No unsaved changes");

      await updateAdminConfig(request, {
        browser_policy_enabled: !initialBrowserPolicyEnabled
      });

      await openTab(page, "traps", { waitForReady: true });
      await openTab(page, "policy", { waitForReady: true });
      if (initialBrowserPolicyEnabled) {
        await expect(page.locator("#browser-policy-toggle")).not.toBeChecked();
      } else {
        await expect(page.locator("#browser-policy-toggle")).toBeChecked();
      }
      await expect(page.locator("#save-policy-config")).toBeHidden();
      await expect(page.locator("#dashboard-panel-policy")).toContainText("No unsaved changes");

      await updateAdminConfig(request, {
        maze_auto_ban: !initialMazeAutoBan
      });

      await openTab(page, "game-loop");
      await openTab(page, "traps", { waitForReady: true });
      await expect(page.locator("#honeypot-paths")).toHaveValue(initialHoneypotPaths);
      if (initialMazeAutoBan) {
        await expect(page.locator("#maze-auto-ban-toggle")).not.toBeChecked();
      } else {
        await expect(page.locator("#maze-auto-ban-toggle")).toBeChecked();
      }
      await expect(page.locator("#save-traps-config")).toBeHidden();
      await expect(page.locator("#dashboard-panel-traps")).toContainText("No unsaved changes");
    }
  );
});

test("tab error state is surfaced when tab-scoped fetch fails", async ({ page }) => {
  await openDashboard(page);
  await clearDashboardClientCache(page);

  await page.route("**/admin/ban", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "temporary ban endpoint outage" })
    });
  });

  await openTab(page, "ip-bans");
  await expect(page.locator('[data-tab-state="ip-bans"]')).toContainText("temporary ban endpoint outage");
  await page.unroute("**/admin/ban");
});

test("diagnostics tab surfaces tab-scoped error when consolidated monitoring fetch fails", async ({ page }) => {
  await openDashboard(page, { initialTab: "status" });
  await openTab(page, "diagnostics", { waitForReady: true });

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "monitoring pipeline unavailable" })
    });
  }, { times: 1 });

  await page.click("#refresh-now-btn");
  await expect(page.locator('[data-tab-state="diagnostics"]')).toContainText(
    "monitoring pipeline unavailable"
  );
});

test("shared config endpoint failures surface per-tab errors for status/verification/advanced/fingerprinting/policy/tuning", async ({ page }) => {
  const assertSharedConfigErrorOnInitialTab = async (tab, message) => {
    await page.route("**/admin/config", async (route) => {
      if (route.request().method() !== "GET") {
        await route.continue();
        return;
      }
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: JSON.stringify({ error: message })
      });
    }, { times: 1 });
    await openDashboard(page, { initialTab: tab });
    await expect(page.locator(`[data-tab-state="${tab}"]`)).toContainText(message);
    await page.goto("about:blank");
  };

  await assertSharedConfigErrorOnInitialTab("status", "status endpoint outage");
  await assertSharedConfigErrorOnInitialTab("verification", "config endpoint outage");
  await assertSharedConfigErrorOnInitialTab("advanced", "advanced endpoint outage");
  await assertSharedConfigErrorOnInitialTab("fingerprinting", "fingerprinting endpoint outage");
  await assertSharedConfigErrorOnInitialTab("policy", "robots endpoint outage");
  await assertSharedConfigErrorOnInitialTab("tuning", "tuning endpoint outage");
});

test("logout redirects back to login page", async ({ page }) => {
  await openDashboard(page);
  await page.click("#logout-btn");
  await expect(page).toHaveURL(/\/dashboard\/login\.html\?next=/);
});

test("logout with unsaved config changes prompts once and cancel preserves the dashboard session", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "verification");

  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  const configSave = page.locator("#save-verification-all");
  if (!(await jsRequiredToggle.isVisible()) || !(await jsRequiredToggle.isEnabled())) {
    await expect(configSave).toBeHidden();
    return;
  }

  const initialChecked = await jsRequiredToggle.isChecked();
  await jsRequiredToggle.click();
  await expect(configSave).toBeVisible();
  await expect(configSave).toBeEnabled();

  const dialogs = [];
  const handleDialog = async (dialog) => {
    dialogs.push({
      type: dialog.type(),
      message: dialog.message()
    });
    await dialog.dismiss();
  };
  page.on("dialog", handleDialog);

  try {
    await page.click("#logout-btn");
    await page.waitForTimeout(500);
  } finally {
    page.off("dialog", handleDialog);
  }

  expect(dialogs).toHaveLength(1);
  expect(dialogs[0].type).toBe("confirm");
  await expect(page).toHaveURL(/\/dashboard\/index\.html#verification$/);
  await expect(page.locator("#logout-btn")).toBeEnabled();
  await expect(configSave).toBeVisible();
  await expect(configSave).toBeEnabled();

  const domState = await dashboardDomClassState(page);
  expect(domState.bodyDisconnectedClassPresent).toBeFalsy();

  if (initialChecked !== await jsRequiredToggle.isChecked()) {
    await jsRequiredToggle.click();
    await expect(configSave).toBeHidden();
  }
});

test("logout with unsaved config changes prompts once before redirecting to login", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "verification");

  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  const configSave = page.locator("#save-verification-all");
  if (!(await jsRequiredToggle.isVisible()) || !(await jsRequiredToggle.isEnabled())) {
    await expect(configSave).toBeHidden();
    return;
  }

  await jsRequiredToggle.click();
  await expect(configSave).toBeVisible();
  await expect(configSave).toBeEnabled();

  const dialogs = [];
  const handleDialog = async (dialog) => {
    dialogs.push({
      type: dialog.type(),
      message: dialog.message()
    });
    await dialog.accept();
  };
  page.on("dialog", handleDialog);

  try {
    await page.click("#logout-btn");
    await expect(page).toHaveURL(/\/dashboard\/login\.html\?next=/);
    await page.waitForTimeout(500);
  } finally {
    page.off("dialog", handleDialog);
  }

  expect(dialogs).toHaveLength(1);
  expect(dialogs[0].type).toBe("confirm");
});
