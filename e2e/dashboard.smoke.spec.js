const { test, expect } = require("@playwright/test");
const { seedDashboardData } = require("./seed-dashboard-data");

const BASE_URL = process.env.SHUMA_BASE_URL || "http://127.0.0.1:3000";
const API_KEY = process.env.SHUMA_API_KEY || "changeme-dev-only-api-key";

async function openDashboard(page) {
  await page.goto(`${BASE_URL}/dashboard/index.html`);
  await page.waitForTimeout(250);
  if (page.url().includes("/dashboard/login.html")) {
    await page.fill("#login-apikey", API_KEY);
    await page.click("#login-submit");
    await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  }
  await page.waitForSelector("#logout-btn", { timeout: 15000 });
  await expect(page.locator("#logout-btn")).toBeEnabled();
  await page.waitForFunction(() => {
    const total = document.getElementById("total-events")?.textContent?.trim();
    return Boolean(total && total !== "-" && total !== "...");
  }, { timeout: 15000 });
}

async function openTab(page, tab) {
  await page.click(`#dashboard-tab-${tab}`);
  await expect(page).toHaveURL(new RegExp(`#${tab}$`));
}

test.beforeAll(async () => {
  await seedDashboardData();
});

test("dashboard loads and shows seeded operational data", async ({ page }) => {
  await openDashboard(page);

  await expect(page.locator("h1")).toHaveText("Shuma-Gorath");
  await expect(page.locator("h3", { hasText: "API Access" })).toHaveCount(0);

  await expect(page.locator("#last-updated")).toContainText("updated:");
  await expect(page.locator("#config-mode-subtitle")).toContainText("Admin page configuration");

  await expect(page.locator("#total-events")).not.toHaveText("-");
  await expect(page.locator("#events tbody tr").first()).toBeVisible();
  await expect(page.locator("#events tbody")).not.toContainText("undefined");

  await expect(page.locator("#cdp-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#cdp-total-detections")).not.toHaveText("-");
});

test("ban form enforces IP validity and submit state", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "ip-bans");

  const banButton = page.locator("#ban-btn");
  await expect(banButton).toBeDisabled();

  await page.fill("#ban-ip", "not-an-ip");
  await page.dispatchEvent("#ban-ip", "input");
  await expect(banButton).toBeDisabled();

  await page.fill("#ban-ip", "198.51.100.42");
  await page.dispatchEvent("#ban-ip", "input");
  await expect(banButton).toBeEnabled();
});

test("maze and duration save buttons use shared dirty-state behavior", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "config");

  const mazeSave = page.locator("#save-maze-config");
  const durationsSave = page.locator("#save-durations-btn");
  const rateLimitSave = page.locator("#save-rate-limit-config");
  const jsRequiredSave = page.locator("#save-js-required-config");
  const edgeModeSave = page.locator("#save-edge-integration-mode-config");
  const edgeModeSelect = page.locator("#edge-integration-mode-select");

  await expect(mazeSave).toBeDisabled();
  await expect(durationsSave).toBeDisabled();
  await expect(rateLimitSave).toBeDisabled();
  await expect(jsRequiredSave).toBeDisabled();
  await expect(edgeModeSave).toBeDisabled();

  const mazeThreshold = page.locator("#maze-threshold");
  if (!(await mazeThreshold.isVisible())) {
    await expect(page.locator("#config-mode-subtitle")).toContainText(/disabled|read-only|Admin page configuration/i);
    return;
  }
  const initialMazeThreshold = await mazeThreshold.inputValue();
  const nextMazeThreshold = String(Math.min(500, Number(initialMazeThreshold || "50") + 1));
  await mazeThreshold.fill(nextMazeThreshold);
  await mazeThreshold.dispatchEvent("input");
  await expect(mazeSave).toBeEnabled();
  await mazeThreshold.fill(initialMazeThreshold);
  await mazeThreshold.dispatchEvent("input");
  await expect(mazeSave).toBeDisabled();

  const durationField = page.locator("#dur-admin-minutes");
  const initialDuration = await durationField.inputValue();
  const nextDuration = String((Number(initialDuration || "0") + 1) % 60);
  await durationField.fill(nextDuration);
  await durationField.dispatchEvent("input");
  await expect(durationsSave).toBeEnabled();
  await durationField.fill(initialDuration);
  await durationField.dispatchEvent("input");
  await expect(durationsSave).toBeDisabled();

  const rateLimitField = page.locator("#rate-limit-threshold");
  const initialRateLimit = await rateLimitField.inputValue();
  const nextRateLimit = String(Math.max(1, Number(initialRateLimit || "80") + 1));
  await rateLimitField.fill(nextRateLimit);
  await rateLimitField.dispatchEvent("input");
  await expect(rateLimitSave).toBeEnabled();
  await rateLimitField.fill(initialRateLimit);
  await rateLimitField.dispatchEvent("input");
  await expect(rateLimitSave).toBeDisabled();

  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  if (await jsRequiredToggle.isVisible()) {
    const jsRequiredInitial = await jsRequiredToggle.isChecked();
    await jsRequiredToggle.click();
    await expect(jsRequiredSave).toBeEnabled();
    if (jsRequiredInitial !== await jsRequiredToggle.isChecked()) {
      await jsRequiredToggle.click();
    }
    await expect(jsRequiredSave).toBeDisabled();
  }

  const initialEdgeMode = await edgeModeSelect.inputValue();
  const nextEdgeMode = initialEdgeMode === "off" ? "advisory" : "off";
  await edgeModeSelect.selectOption(nextEdgeMode);
  await expect(edgeModeSave).toBeEnabled();
  await edgeModeSelect.selectOption(initialEdgeMode);
  await expect(edgeModeSave).toBeDisabled();
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

test("dashboard tables keep sticky headers", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "monitoring");

  const eventsHeaderPosition = await page
    .locator("#events thead th")
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
  await openTab(page, "config");
  await expect(page.locator("#dashboard-panel-config")).toBeVisible();
  await expect(page.locator("#dashboard-panel-monitoring")).toBeHidden();

  await page.reload();
  await expect(page).toHaveURL(/\/dashboard\/index\.html#config/);
  await expect(page.locator("#dashboard-panel-config")).toBeVisible();
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

  await page.locator("#dashboard-tab-ip-bans").focus();
  await page.keyboard.press("End");
  await expect(page).toHaveURL(/#tuning$/);
  await expect(page.locator("#dashboard-tab-tuning")).toHaveAttribute("aria-selected", "true");

  await page.locator("#dashboard-tab-tuning").focus();
  await page.keyboard.press("Home");
  await expect(page).toHaveURL(/#monitoring$/);
  await expect(page.locator("#dashboard-tab-monitoring")).toHaveAttribute("aria-selected", "true");
});

test("tab error state is surfaced when tab-scoped fetch fails", async ({ page }) => {
  await openDashboard(page);

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

test("logout redirects back to login page", async ({ page }) => {
  await openDashboard(page);
  await page.click("#logout-btn");
  await expect(page).toHaveURL(/\/dashboard\/login\.html\?next=/);
});
