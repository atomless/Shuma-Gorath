const { test, expect } = require("@playwright/test");
const { seedDashboardData } = require("./seed-dashboard-data");

const BASE_URL = process.env.SHUMA_BASE_URL || "http://127.0.0.1:3000";
const API_KEY = (process.env.SHUMA_API_KEY || "").trim();
const FORWARDED_IP_SECRET = (process.env.SHUMA_FORWARDED_IP_SECRET || "").trim();
const DASHBOARD_TABS = Object.freeze(["monitoring", "ip-bans", "status", "verification", "traps", "rate-limiting", "geo", "fingerprinting", "robots", "tuning", "advanced"]);
const ADMIN_TABS = Object.freeze(["ip-bans", "status", "verification", "traps", "rate-limiting", "geo", "fingerprinting", "robots", "tuning", "advanced"]);
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

async function assertActiveTabPanelVisibility(page, activeTab) {
  for (const tab of DASHBOARD_TABS) {
    await expect(page.locator(`#dashboard-tab-${tab}`)).toHaveAttribute(
      "aria-selected",
      tab === activeTab ? "true" : "false"
    );
  }

  if (activeTab === "monitoring") {
    await expect(page.locator("#dashboard-panel-monitoring")).toBeVisible();
    await expect(page.locator("#dashboard-admin-section")).toBeHidden();
    for (const tab of ADMIN_TABS) {
      await expect(page.locator(`#dashboard-panel-${tab}`)).toBeHidden();
    }
    return;
  }

  await expect(page.locator("#dashboard-panel-monitoring")).toBeHidden();
  await expect(page.locator("#dashboard-admin-section")).toBeVisible();
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
      hasAdversarySim: bodyClasses.includes("adversary-sim"),
      bodyConnectedClassPresent: bodyClasses.includes("connected"),
      bodyDisconnectedClassPresent: bodyClasses.includes("disconnected")
    };
  });
}

async function fetchRuntimeEnvironment(request, ip = "127.0.0.1") {
  const response = await request.get(`${BASE_URL}/admin/config`, {
    headers: buildAdminAuthHeaders(ip)
  });
  if (!response.ok()) {
    const body = await response.text();
    throw new Error(`admin config read should succeed: ${response.status()} ${body}`);
  }
  const payload = await response.json();
  const runtimeEnvironment = String(payload?.runtime_environment || "").trim();
  if (runtimeEnvironment !== "runtime-dev" && runtimeEnvironment !== "runtime-prod") {
    throw new Error(`unexpected runtime_environment from /admin/config: ${runtimeEnvironment || "missing"}`);
  }
  return runtimeEnvironment;
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
    await page.fill("#login-apikey", API_KEY);
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
      if (!page.url().includes("/dashboard/login.html")) {
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
  const initialTab = typeof options.initialTab === "string" ? options.initialTab : "monitoring";
  const targetUrl = `${BASE_URL}/dashboard/index.html#${initialTab}`;
  ensureRuntimeGuard(page);
  await page.goto(targetUrl);
  await page.waitForTimeout(250);
  if (page.url().includes("/dashboard/login.html")) {
    await bootstrapDashboardSession(page, targetUrl);
    await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  }
  if (!page.url().endsWith(`#${initialTab}`)) {
    await page.evaluate((tab) => {
      window.location.hash = tab;
    }, initialTab);
    await expect(page).toHaveURL(new RegExp(`#${initialTab}$`));
  }
  await page.waitForSelector("#logout-btn", { timeout: 15000 });
  await expect(page.locator("#logout-btn")).toBeEnabled();
  if (initialTab === "monitoring") {
    await page.waitForFunction(() => {
      const total = document.getElementById("total-events")?.textContent?.trim();
      return Boolean(total && total !== "-" && total !== "...");
    }, { timeout: 15000 });
  }
  await assertActiveTabPanelVisibility(page, initialTab);
  assertNoRuntimeFailures(page);
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
      "shuma_dashboard_cache_monitoring_v1",
      "shuma_dashboard_cache_ip_bans_v1",
      "shuma_dashboard_auto_refresh_v1"
    ];
    try {
      keys.forEach((key) => window.localStorage.removeItem(key));
    } catch (_error) {}
  });
}

async function clickAdversaryToggleWithRetry(page, desiredEnabled, timeoutMs = 60000) {
  const toggle = page.locator("#global-adversary-sim-toggle");
  const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
  await expect(toggle).toBeEnabled({ timeout: timeoutMs });
  const desired = desiredEnabled === true;
  if ((await toggle.isChecked()) === desired) {
    return null;
  }

  const deadline = Date.now() + Math.max(2000, Number(timeoutMs || 0));
  let lastStatus = 0;
  while (Date.now() < deadline) {
    if ((await toggle.isChecked()) === desired) {
      return null;
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
      return response;
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
  const response = await request.get(`${BASE_URL}/admin/config`, {
    headers: buildAdminAuthHeaders(ip)
  });
  if (!response.ok()) {
    const body = await response.text();
    throw new Error(`admin config read should succeed: ${response.status()} ${body}`);
  }
  const payload = await response.json();
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
  await page.reload();
  await expect(page.locator("#login-form")).toBeVisible();
  await page.fill("#login-apikey", API_KEY);
  await page.click("#login-submit");
  await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  assertNoRuntimeFailures(page);
});

test("not-a-bot browser lifecycle captures telemetry and rejects replayed submit", async ({ page, request }) => {
  const configHeaders = buildAdminAuthHeaders("127.0.0.1");
  const currentConfigResponse = await request.get(`${BASE_URL}/admin/config`, {
    headers: configHeaders
  });
  expect(currentConfigResponse.ok()).toBe(true);
  const currentConfig = await currentConfigResponse.json();
  const originalTestMode = currentConfig && currentConfig.test_mode === true;
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
    test_mode: true,
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
      test_mode: originalTestMode,
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
  await openTab(page, "monitoring");

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
      rate_limit: 3600,
      cdp: 43200,
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
    test_mode: false,
    kv_store_fail_open: true,
    edge_integration_mode: "off",
    geo_risk: [],
    geo_allow: [],
    geo_challenge: [],
    geo_maze: [],
    geo_block: []
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
          analytics: { ban_count: 0, test_mode: false, fail_mode: "open" },
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
      body: JSON.stringify(emptyConfig)
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
  await expect(page.locator("#total-events")).toHaveText("0");
  await expect(page.locator("#monitoring-events tbody")).toContainText(
    /No (recent events|events loaded while freshness is degraded\/stale)/i
  );
  await expect(page.locator("#cdp-events tbody")).toContainText(
    "No CDP detections or auto-bans in the selected window"
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

test("monitoring summary sections render data and cap oversized result lists", async ({ page }) => {
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
          analytics: { ban_count: 2, test_mode: false, fail_mode: "open" },
          events: {
            recent_events: [
              {
                ts: Math.floor(Date.now() / 1000),
                event: "Challenge",
                ip: "198.51.100.44",
                reason: "challenge_reason_1",
                outcome: "served",
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

  await openDashboard(page);
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
  await assertChartsFillPanels(page);

  await expect(page.locator("h1")).toHaveText("Shuma-Gorath");
  await expect(page.locator("h3", { hasText: "API Access" })).toHaveCount(0);

  await expect(page.locator("#last-updated")).toContainText("updated:");

  await expect(page.locator("#total-events")).not.toHaveText("-");
  await expect(page.locator("#monitoring-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#monitoring-events tbody")).not.toContainText("undefined");

  await expect(page.locator("#cdp-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#cdp-total-detections")).not.toHaveText("-");
  await expect(page.locator('label[for="global-test-mode-toggle"]')).toBeVisible();
});

test("dashboard monitoring totals stay in parity with /metrics monitoring families", async ({ page, request }) => {
  await openDashboard(page);

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
  await expect(page.locator("#status-items .status-item h3", { hasText: "Retention and Freshness Health" })).toHaveCount(1);
  await expect(page.locator("#status-items .status-item h3", { hasText: "Test Mode" })).toHaveCount(0);
  await expect(page.locator("#status-items .status-item h3", { hasText: /^Challenge$/ })).toHaveCount(0);

  await expect(page.locator("#runtime-fetch-latency-last")).toContainText("ms");
  await expect(page.locator("#runtime-render-timing-last")).toContainText("ms");
  await expect(page.locator("#runtime-polling-resume-count")).toContainText("resumes:");
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
    .filter({ has: page.locator("code", { hasText: "test_mode" }) });
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

  await openTab(page, "monitoring");
  await page.reload();
  await expect(page).toHaveURL(/\/dashboard\/index\.html#monitoring/);
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

test("dashboard class contract tracks runtime on html and adversary-sim on body", async ({ page, request }) => {
  await updateAdminConfig(request, { adversary_sim_enabled: false, adversary_sim_duration_seconds: 180 });
  const runtimeEnvironment = await fetchRuntimeEnvironment(request);
  await openDashboard(page);
  const toggle = page.locator("#global-adversary-sim-toggle");

  let bodyState = await dashboardDomClassState(page);
  expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
  expect(bodyState.hasAdversarySim).toBeFalsy();
  expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
  expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

  await openTab(page, "status");
  bodyState = await dashboardDomClassState(page);
  expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
  expect(bodyState.hasAdversarySim).toBeFalsy();
  expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
  expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

  if (await toggle.isChecked()) {
    await clickAdversaryToggleWithRetry(page, false, 60000);
  }
  await clickAdversaryToggleWithRetry(page, true, 60000);
  await expect(toggle).toBeChecked();
  bodyState = await dashboardDomClassState(page);
  expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
  expect(bodyState.hasAdversarySim).toBeTruthy();
  expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
  expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

  await openTab(page, "verification");
  bodyState = await dashboardDomClassState(page);
  expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
  expect(bodyState.hasAdversarySim).toBeTruthy();
  expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
  expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();

  await clickAdversaryToggleWithRetry(page, false, 60000);
  await expect(toggle).not.toBeChecked();
  bodyState = await dashboardDomClassState(page);
  expectDashboardRuntimeClass(bodyState, runtimeEnvironment);
  expect(bodyState.hasAdversarySim).toBeFalsy();
  expect(bodyState.bodyConnectedClassPresent).toBeFalsy();
  expect(bodyState.bodyDisconnectedClassPresent).toBeFalsy();
});

test("adversary sim global toggle drives orchestration control lifecycle state", async ({ page, request }) => {
  test.setTimeout(180_000);
  await updateAdminConfig(request, { adversary_sim_enabled: false, adversary_sim_duration_seconds: 180 });
  await openDashboard(page);

  const toggle = page.locator("#global-adversary-sim-toggle");
  const lifecycleCopy = page.locator("#adversary-sim-lifecycle-copy");
  await expect(toggle).toBeEnabled();
  if (await toggle.isChecked()) {
    await clickAdversaryToggleWithRetry(page, false);
  }
  await expect(toggle).not.toBeChecked();
  await expect(lifecycleCopy).toContainText("Generation inactive");

  const onResponse = await clickAdversaryToggleWithRetry(page, true);
  const onBody = await onResponse.json();
  expect(onBody?.requested_enabled).toBe(true);
  await expect(toggle).toBeChecked();
  await expect(lifecycleCopy).toContainText("Generation active");
  let bodyState = await dashboardDomClassState(page);
  expect(bodyState.hasAdversarySim).toBeTruthy();

  const offResponse = await clickAdversaryToggleWithRetry(page, false);
  const offBody = await offResponse.json();
  expect(offBody?.requested_enabled).toBe(false);
  await expect(toggle).not.toBeChecked();
  await expect(lifecycleCopy).toContainText("Generation inactive");
  await expect(lifecycleCopy).toContainText("Retained telemetry remains visible");
  bodyState = await dashboardDomClassState(page);
  expect(bodyState.hasAdversarySim).toBeFalsy();
});

test("adversary sim toggle emits fresh telemetry visible in monitoring raw feed", async ({ page, request }) => {
  test.setTimeout(180_000);
  await updateAdminConfig(request, { adversary_sim_enabled: false, adversary_sim_duration_seconds: 180 });
  await openDashboard(page);
  await openTab(page, "monitoring");
  await setAutoRefresh(page, true);

  const toggle = page.locator("#global-adversary-sim-toggle");
  await expect(toggle).toBeEnabled({ timeout: 15000 });
  if (await toggle.isChecked()) {
    await clickAdversaryToggleWithRetry(page, false, 90000);
  }
  await expect(toggle).not.toBeChecked();

  const baselineMonitoring = await fetchMonitoringSnapshot(request, 24, 200);
  const baselineTs = maxSimulationEventTs(baselineMonitoring);

  try {
    await clickAdversaryToggleWithRetry(page, true);
    await expect(toggle).toBeChecked();

    const advancedTs = await waitForSimulationEventAdvance(request, baselineTs, 20000);
    await expect(page.locator("#monitoring-raw-feed tbody")).toContainText(`"ts":${advancedTs}`);
  } finally {
    if (await toggle.isChecked()) {
      await clickAdversaryToggleWithRetry(page, false, 90000);
    }
    await updateAdminConfig(request, { adversary_sim_enabled: false, adversary_sim_duration_seconds: 180 });
  }
  await page.reload();
  await expect(toggle).not.toBeChecked();
});

test("adversary sim toggle cancel path avoids orchestration request when frontier keys are missing", async ({ page, request }) => {
  await updateAdminConfig(request, { adversary_sim_enabled: false, adversary_sim_duration_seconds: 180 });
  const frontierProviderCount = await fetchFrontierProviderCount(request);
  test.skip(
    frontierProviderCount > 0,
    "requires frontier provider keys to be absent in runtime env"
  );
  await openDashboard(page);

  const toggle = page.locator("#global-adversary-sim-toggle");
  const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
  if (!(await toggle.isEnabled())) {
    // If the monitoring bootstrap hit transient read throttling, force a config-backed
    // tab refresh to repopulate control availability before asserting toggle readiness.
    await openTab(page, "status", { waitForReady: true });
    await openTab(page, "monitoring");
  }
  await expect(toggle).toBeEnabled({ timeout: 15000 });
  if (await toggle.isChecked()) {
    await clickAdversaryToggleWithRetry(page, false, 60000);
  }
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
  await expect(page.locator("#admin-msg")).toContainText("Add SHUMA_FRONTIER_*_API_KEY");
  await page.waitForTimeout(250);
  expect(controlRequestCount).toBe(0);
});

test("auto refresh defaults off and is only available on monitoring/ip-bans tabs", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "monitoring");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#auto-refresh-toggle")).not.toBeChecked();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await expect(page.locator("#refresh-mode")).toContainText("OFF");
  await setAutoRefresh(page, true);
  await expect(page.locator("#refresh-now-btn")).toBeHidden();
  await expect(page.locator("#refresh-mode")).toContainText("ON");

  await openTab(page, "status");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeHidden();
  await expect(page.locator("#refresh-now-btn")).toBeHidden();

  await openTab(page, "ip-bans");
  await expect(page.locator('label[for="auto-refresh-toggle"]')).toBeVisible();
  await expect(page.locator("#auto-refresh-toggle")).toBeChecked();
  await setAutoRefresh(page, false);
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
});

test("monitoring initial load hydrates from full snapshot even when first delta page is empty", async ({ page }) => {
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
      analytics: { ban_count: 0, test_mode: false, fail_mode: "open" },
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
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
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
  });

  await openDashboard(page);
  await expect(page.locator("#monitoring-events tbody")).toContainText("historical-baseline-visible");
});

test("manual refresh button appends new monitoring delta events when auto-refresh is off", async ({ page }) => {
  const now = Math.floor(Date.now() / 1000);
  const buildMonitoringPayload = () => ({
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
      analytics: { ban_count: 0, test_mode: false, fail_mode: "open" },
      events: {
        recent_events: [{
          ts: now,
          event: "Challenge",
          ip: "198.51.100.77",
          reason: "historical-baseline",
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
  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(buildMonitoringPayload())
    });
  });
  await page.route("**/admin/monitoring/delta?hours=*&limit=*", async (route) => {
    deltaRequestCount += 1;
    const url = new URL(route.request().url());
    const limit = Number.parseInt(url.searchParams.get("limit") || "0", 10);
    const afterCursor = (url.searchParams.get("after_cursor") || "").trim();
    if (limit === 1 || !afterCursor) {
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
  await expect(page.locator("#auto-refresh-toggle")).not.toBeChecked();
  await expect(page.locator("#refresh-now-btn")).toBeVisible();
  await expect(page.locator("#monitoring-events tbody")).not.toContainText("manual-refresh-delta-event");

  const beforeRefreshDeltaCalls = deltaRequestCount;
  await page.click("#refresh-now-btn");
  await expect(page.locator("#monitoring-events tbody")).toContainText("manual-refresh-delta-event");
  expect(deltaRequestCount).toBeGreaterThan(beforeRefreshDeltaCalls);
});

test("monitoring recent-event filters use canonical shared control classes", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "monitoring");

  await expect(page.locator("#monitoring-event-filters .input-row")).toHaveCount(5);
  await expect(page.locator("#monitoring-event-filters .field-row")).toHaveCount(0);
  await expect(page.locator("#monitoring-event-filters .control-label.control-label--wide")).toHaveCount(5);
  await expect(page.locator("#monitoring-event-filters select.input-field")).toHaveCount(5);
});

test("route remount preserves keyboard navigation, ban/unban, verification save, and polling", async ({ page }) => {
  let monitoringRefreshRequests = 0;
  page.on("request", (request) => {
    if (request.method() !== "GET") {
      return;
    }
    const url = request.url();
    if (
      url.includes("/admin/monitoring?hours=24") ||
      url.includes("/admin/monitoring/delta?hours=24")
    ) {
      monitoringRefreshRequests += 1;
    }
  });

  await openDashboard(page);
  await page.goto("about:blank");
  await openDashboard(page);

  const monitoringTab = page.locator("#dashboard-tab-monitoring");
  await monitoringTab.focus();
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
  await expect(page.locator("#admin-msg")).toContainText(`Banned ${ip}`);

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
  await expect(page.locator("#admin-msg")).toContainText(`Unbanned ${ip}`);

  await openTab(page, "verification");
  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  const configSave = page.locator("#save-verification-all");
  if (await jsRequiredToggle.isVisible() && await jsRequiredToggle.isEnabled()) {
    const initial = await jsRequiredToggle.isChecked();
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

    if (initial !== await jsRequiredToggle.isChecked()) {
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
  }

  await openTab(page, "monitoring");
  await setAutoRefresh(page, true);
  await page.waitForTimeout(150);
  const beforePollWait = monitoringRefreshRequests;
  await page.waitForTimeout(1300);
  expect(monitoringRefreshRequests).toBeGreaterThan(beforePollWait);
});

test("monitoring auto-refresh avoids placeholder flicker and bounds table churn", async ({ page }) => {
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
          analytics: { ban_count: 1, test_mode: false, fail_mode: "open" },
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
  await openTab(page, "monitoring");
  await setAutoRefresh(page, true);
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

  await page.waitForTimeout(150);
  const beforePollWindow = monitoringSnapshotRequests + monitoringDeltaRequests;
  await page.waitForTimeout(1300);
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
  const remountObservationWindowMs = 1300;
  const maxExpectedRequestsInWindow = 6;

  let monitoringRequests = 0;
  page.on("request", (request) => {
    if (
      request.method() === "GET" &&
      (
        request.url().includes("/admin/monitoring?hours=24") ||
        request.url().includes("/admin/monitoring/delta?hours=")
      )
    ) {
      monitoringRequests += 1;
    }
  });

  const remountRequestDeltas = [];
  const remountCycles = 4;
  for (let cycle = 0; cycle < remountCycles; cycle += 1) {
    await openDashboard(page);
    await openTab(page, "monitoring");
    await setAutoRefresh(page, true);
    const beforeWindow = monitoringRequests;
    await page.waitForTimeout(remountObservationWindowMs);
    let delta = monitoringRequests - beforeWindow;
    let maxRequestsForObservedWindow = maxExpectedRequestsInWindow;
    if (delta === 0) {
      // Polling is serialized behind in-flight refresh work; allow one extra
      // cadence window before failing this cycle.
      const extraWindowMs = 1200;
      await page.waitForTimeout(extraWindowMs);
      delta = monitoringRequests - beforeWindow;
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
  const soakWindowMs = 1300;
  const maxExpectedRequestsInWindow = 4;
  const maxFetchP95Ms = 2500;
  const maxRenderP95Ms = 80;

  let monitoringRequests = 0;
  const delayedPassThrough = async (route) => {
    monitoringRequests += 1;
    await page.waitForTimeout(18);
    await route.continue();
  };
  await page.route("**/admin/monitoring?hours=*&limit=*", delayedPassThrough);
  await page.route("**/admin/monitoring/delta?hours=*", delayedPassThrough);

  const cadenceDeltas = [];
  const fetchP95Samples = [];
  const renderP95Samples = [];
  const remountCycles = 5;

  for (let cycle = 0; cycle < remountCycles; cycle += 1) {
    await openDashboard(page);
    await openTab(page, "monitoring");
    await setAutoRefresh(page, true);

    const before = monitoringRequests;
    await page.waitForTimeout(soakWindowMs);
    let delta = monitoringRequests - before;
    if (delta === 0) {
      // Give the polling loop one more cadence window to tick before failing cadence.
      await page.waitForTimeout(1200);
      delta = monitoringRequests - before;
    }
    cadenceDeltas.push(delta);
    expect(delta).toBeLessThanOrEqual(maxExpectedRequestsInWindow);

    await openTab(page, "status");
    const telemetry = await page.evaluate(() => {
      const parseP95 = (id) => {
        const text = document.getElementById(id)?.textContent || "";
        const match = /p95:\s*([0-9]+(?:\.[0-9]+)?)\s*ms/i.exec(text);
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

test("dashboard tables keep sticky headers", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "monitoring");

  const eventsHeaderPosition = await page
    .locator("#monitoring-events thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);
  const cdpHeaderPosition = await page
    .locator("#cdp-events thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);

  await openTab(page, "ip-bans");
  const bansHeaderPosition = await page
    .locator("#bans-table thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);

  expect(eventsHeaderPosition).toBe("sticky");
  expect(cdpHeaderPosition).toBe("sticky");
  expect(bansHeaderPosition).toBe("sticky");
});

test("tab hash route persists selected panel across reload", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "verification");
  await expect(page.locator("#dashboard-panel-verification")).toBeVisible();
  await expect(page.locator("#dashboard-panel-monitoring")).toBeHidden();

  await page.reload();
  await expect(page).toHaveURL(/\/dashboard\/index\.html#verification/);
  await expect(page.locator("#dashboard-panel-verification")).toBeVisible();
  await assertActiveTabPanelVisibility(page, "verification");
});

test("tab keyboard navigation updates hash and selected state", async ({ page }) => {
  await openDashboard(page);
  const monitoringTab = page.locator("#dashboard-tab-monitoring");
  await monitoringTab.focus();
  await expect(monitoringTab).toHaveAttribute("aria-selected", "true");

  await page.keyboard.press("ArrowRight");
  await expect(page).toHaveURL(/#ip-bans$/);
  await expect(page.locator("#dashboard-tab-ip-bans")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#dashboard-panel-ip-bans")).toBeVisible();
  await assertActiveTabPanelVisibility(page, "ip-bans");

  await page.locator("#dashboard-tab-ip-bans").focus();
  await page.keyboard.press("End");
  await expect(page).toHaveURL(/#advanced$/);
  await expect(page.locator("#dashboard-tab-advanced")).toHaveAttribute("aria-selected", "true");
  await assertActiveTabPanelVisibility(page, "advanced");

  await page.locator("#dashboard-tab-advanced").focus();
  await page.keyboard.press("Home");
  await expect(page).toHaveURL(/#monitoring$/);
  await expect(page.locator("#dashboard-tab-monitoring")).toHaveAttribute("aria-selected", "true");
  await assertActiveTabPanelVisibility(page, "monitoring");
});

test("tab states surface loading and data-ready transitions across all tabs", async ({ page }) => {
  await openDashboard(page);
  await setAutoRefresh(page, false);

  await openTab(page, "status");
  await expect(page.locator('[data-tab-state="status"]')).toBeHidden();

  await openTab(page, "verification");
  await expect(page.locator('[data-tab-state="verification"]')).toBeHidden();

  await openTab(page, "rate-limiting");
  await expect(page.locator('[data-tab-state="rate-limiting"]')).toBeHidden();

  await openTab(page, "geo");
  await expect(page.locator('[data-tab-state="geo"]')).toBeHidden();

  await openTab(page, "fingerprinting");
  await expect(page.locator('[data-tab-state="fingerprinting"]')).toBeHidden();

  await openTab(page, "tuning");
  await expect(page.locator('[data-tab-state="tuning"]')).toBeHidden();

  await clearDashboardClientCache(page);
  let releaseBanFetch = null;
  const banFetchObserved = new Promise((resolve) => {
    releaseBanFetch = resolve;
  });
  let releaseBanResponse = null;
  const banResponseGate = new Promise((resolve) => {
    releaseBanResponse = resolve;
  });
  await page.route("**/admin/ban", async (route) => {
    if (route.request().method() !== "GET") {
      await route.continue();
      return;
    }
    releaseBanFetch();
    await banResponseGate;
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        bans: [
          {
            ip: "198.51.100.250",
            reason: "manual_ban",
            banned_at: Math.floor(Date.now() / 1000) - 60,
            expires: Math.floor(Date.now() / 1000) + 3600,
            fingerprint: { signals: ["ua_transport_mismatch"], score: 4, summary: "seeded row" }
          }
        ]
      })
    });
  }, { times: 1 });

  await openTab(page, "ip-bans");
  await banFetchObserved;
  await expect(page.locator('[data-tab-state="ip-bans"]')).toContainText("Loading ban list...");
  releaseBanResponse();
  await expect(page.locator("#bans-table tbody")).toContainText("198.51.100.250");
  await expect(page.locator('[data-tab-state="ip-bans"]')).toBeHidden();

  await openTab(page, "monitoring");
  await expect(page.locator('[data-tab-state="monitoring"]')).toBeHidden();
});

test("verification save roundtrip clears dirty state after successful write", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "verification");

  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  const configSave = page.locator("#save-verification-all");
  if (!(await jsRequiredToggle.isVisible()) || !(await jsRequiredToggle.isEnabled())) {
    await expect(configSave).toBeHidden();
    return;
  }

  const initial = await jsRequiredToggle.isChecked();
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
  await expect(page.locator("#admin-msg")).toContainText(/saved/i);
  await expect(configSave).toBeHidden();

  if (initial !== await jsRequiredToggle.isChecked()) {
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

test("geo and tuning save flows cover GEO lists, botness controls, and browser policy controls", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "geo");

  const geoSave = page.locator("#save-geo-config");
  await expect(geoSave).toBeHidden();

  const geoSignalToggle = page.locator("#geo-akamai-enabled-toggle");
  const geoSignalSwitch = page.locator("label.toggle-switch[for='geo-akamai-enabled-toggle']");
  if (await geoSignalSwitch.isVisible() && await geoSignalToggle.isEnabled()) {
    const initialGeoSignalEnabled = await geoSignalToggle.isChecked();
    await geoSignalSwitch.click();
    await submitConfigSave(page, geoSave);
    if (initialGeoSignalEnabled !== await geoSignalToggle.isChecked()) {
      await geoSignalSwitch.click();
      await submitConfigSave(page, geoSave);
    }
  }

  const geoScoringToggle = page.locator("#geo-scoring-toggle");
  const geoScoringSwitch = page.locator("label.toggle-switch[for='geo-scoring-toggle']");
  if (await geoScoringSwitch.isVisible() && await geoScoringToggle.isEnabled()) {
    const initialGeoScoringEnabled = await geoScoringToggle.isChecked();
    await geoScoringSwitch.click();
    await submitConfigSave(page, geoSave);
    if (initialGeoScoringEnabled !== await geoScoringToggle.isChecked()) {
      await geoScoringSwitch.click();
      await submitConfigSave(page, geoSave);
    }
  }

  const geoRoutingToggle = page.locator("#geo-routing-toggle");
  const geoRoutingSwitch = page.locator("label.toggle-switch[for='geo-routing-toggle']");
  if (await geoRoutingSwitch.isVisible() && await geoRoutingToggle.isEnabled()) {
    const initialGeoRoutingEnabled = await geoRoutingToggle.isChecked();
    await geoRoutingSwitch.click();
    await submitConfigSave(page, geoSave);
    if (initialGeoRoutingEnabled !== await geoRoutingToggle.isChecked()) {
      await geoRoutingSwitch.click();
      await submitConfigSave(page, geoSave);
    }
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
    await geoRiskList.fill(geoRiskInitial);
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
    await geoAllowList.fill(geoAllowInitial);
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
    await botnessWeight.fill(botnessInitial);
    await botnessWeight.dispatchEvent("input");
    await submitConfigSave(page, tuningSave);
  }

  const browserPolicyToggle = page.locator("#browser-policy-toggle");
  const browserPolicySwitch = page.locator("label.toggle-switch[for='browser-policy-toggle']");
  if (await browserPolicySwitch.isVisible() && await browserPolicyToggle.isEnabled()) {
    const initialBrowserPolicyEnabled = await browserPolicyToggle.isChecked();
    await browserPolicySwitch.click();
    await submitConfigSave(page, tuningSave);
    if (initialBrowserPolicyEnabled !== await browserPolicyToggle.isChecked()) {
      await browserPolicySwitch.click();
      await submitConfigSave(page, tuningSave);
    }
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
    await submitConfigSave(page, tuningSave);
    await browserBlockRules.fill(initialBrowserRules);
    await browserBlockRules.dispatchEvent("input");
    await submitConfigSave(page, tuningSave);
  }

  const pathAllowlist = page.locator("#path-allowlist");
  const pathAllowlistToggle = page.locator("#path-allowlist-enabled-toggle");
  const pathAllowlistSwitch = page.locator("label.toggle-switch[for='path-allowlist-enabled-toggle']");
  if (await pathAllowlistSwitch.isVisible() && await pathAllowlistToggle.isEnabled()) {
    const initialPathAllowlistEnabled = await pathAllowlistToggle.isChecked();
    await pathAllowlistSwitch.click();
    await submitConfigSave(page, tuningSave);
    if (initialPathAllowlistEnabled !== await pathAllowlistToggle.isChecked()) {
      await pathAllowlistSwitch.click();
      await submitConfigSave(page, tuningSave);
    }
  }
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
    await submitConfigSave(page, tuningSave);
    await pathAllowlist.fill(initialPathAllowlist);
    await pathAllowlist.dispatchEvent("input");
    await submitConfigSave(page, tuningSave);
  }
});

test("rate-limiting tab save flows cover local controls and Akamai backend toggle", async ({ page }) => {
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
    await rateThreshold.fill(initialRateThreshold);
    await rateThreshold.dispatchEvent("input");
    await submitConfigSave(page, saveButton);
  }

  const rateEnabledToggle = page.locator("#rate-limiting-enabled-toggle");
  const rateEnabledSwitch = page.locator("label.toggle-switch[for='rate-limiting-enabled-toggle']");
  if (await rateEnabledSwitch.isVisible() && await rateEnabledToggle.isEnabled()) {
    const initialEnabled = await rateEnabledToggle.isChecked();
    await rateEnabledSwitch.click();
    await submitConfigSave(page, saveButton);
    if (initialEnabled !== await rateEnabledToggle.isChecked()) {
      await rateEnabledSwitch.click();
      await submitConfigSave(page, saveButton);
    }
  }

  const akamaiToggle = page.locator("#rate-akamai-enabled-toggle");
  const akamaiSwitch = page.locator("label.toggle-switch[for='rate-akamai-enabled-toggle']");
  if (await akamaiSwitch.isVisible() && await akamaiToggle.isEnabled()) {
    const initialAkamaiEnabled = await akamaiToggle.isChecked();
    await akamaiSwitch.click();
    await submitConfigSave(page, saveButton);
    if (initialAkamaiEnabled !== await akamaiToggle.isChecked()) {
      await akamaiSwitch.click();
      await submitConfigSave(page, saveButton);
    }
  }
});

test("fingerprinting tab save flows cover Akamai toggle and additive/authoritative mode controls", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "fingerprinting");

  const saveButton = page.locator("#save-fingerprinting-config");
  await expect(saveButton).toBeHidden();

  const akamaiToggle = page.locator("#fingerprinting-akamai-enabled-toggle");
  const akamaiToggleSwitch = page.locator("label.toggle-switch[for='fingerprinting-akamai-enabled-toggle']");
  const edgeModeSelect = page.locator("#fingerprinting-edge-mode-select");

  if (await edgeModeSelect.isVisible() && await edgeModeSelect.isEnabled()) {
    const edgeModeInitial = await edgeModeSelect.inputValue();
    const edgeModeNext = edgeModeInitial === "additive" ? "authoritative" : "additive";
    await edgeModeSelect.selectOption(edgeModeNext);
    await submitConfigSave(page, saveButton);
    await edgeModeSelect.selectOption(edgeModeInitial);
    await submitConfigSave(page, saveButton);
  }

  if (await akamaiToggleSwitch.isVisible() && await akamaiToggle.isEnabled()) {
    const initialEnabled = await akamaiToggle.isChecked();
    await akamaiToggleSwitch.click();
    await submitConfigSave(page, saveButton);
    if (initialEnabled !== await akamaiToggle.isChecked()) {
      await akamaiToggleSwitch.click();
    }
    await submitConfigSave(page, saveButton);
  }
});

test("robots tab save flows cover robots serving and AI policy controls", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "robots");

  const saveButton = page.locator("#save-robots-config");
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
  await robotsCrawlDelay.fill(robotsDelayInitial);
  await robotsCrawlDelay.dispatchEvent("input");
  await submitConfigSave(page, saveButton);

  const aiToggle = page.locator("#robots-block-training-toggle");
  const aiToggleSwitch = page.locator("label.toggle-switch[for='robots-block-training-toggle']");
  if (await aiToggleSwitch.isVisible() && await aiToggle.isEnabled()) {
    const aiInitial = await aiToggle.isChecked();
    await aiToggleSwitch.click();
    await submitConfigSave(page, saveButton);
    if (aiInitial !== await aiToggle.isChecked()) {
      await aiToggleSwitch.click();
      await submitConfigSave(page, saveButton);
    }
  }
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

test("monitoring tab surfaces tab-scoped error when consolidated monitoring fetch fails", async ({ page }) => {
  await openDashboard(page, { initialTab: "status" });

  await page.route("**/admin/monitoring?hours=*&limit=*", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "monitoring pipeline unavailable" })
    });
  }, { times: 1 });

  await openTab(page, "monitoring");
  await expect(page.locator('[data-tab-state="monitoring"]')).toContainText(
    "monitoring pipeline unavailable"
  );
});

test("shared config endpoint failures surface per-tab errors for status/verification/advanced/fingerprinting/robots/tuning", async ({ page }) => {
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
  await assertSharedConfigErrorOnInitialTab("robots", "robots endpoint outage");
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
