import { chromium, expect } from '@playwright/test';

const BASE_URL = String(process.env.SHUMA_BASE_URL || '').trim().replace(/\/$/, '');
const API_KEY = String(process.env.SHUMA_API_KEY || '').trim();
const READY_TIMEOUT_MS = 20_000;
const CONTROL_TIMEOUT_MS = 15_000;
const MONITORING_TIMEOUT_MS = 60_000;

function newDashboardIdempotencyKey() {
  return `dash-live-${Date.now().toString(16)}-${Math.floor(Math.random() * 0x1_0000_0000).toString(16).padStart(8, '0')}`;
}

function requireEnv(name, value) {
  if (!value) {
    throw new Error(`${name} must be set for live dashboard smoke.`);
  }
}

function adminHeaders(extra = {}) {
  return {
    Authorization: `Bearer ${API_KEY}`,
    Origin: BASE_URL,
    'Content-Type': 'application/json',
    ...extra
  };
}

async function requestJson(path, init = {}) {
  const response = await fetch(`${BASE_URL}${path}`, init);
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`${init.method || 'GET'} ${path} failed: ${response.status} ${text.slice(0, 200)}`);
  }
  return JSON.parse(text || '{}');
}

async function controlAdversarySimViaApi(desiredEnabled) {
  return requestJson('/admin/adversary-sim/control', {
    method: 'POST',
    headers: adminHeaders({
      'Idempotency-Key': newDashboardIdempotencyKey()
    }),
    body: JSON.stringify({ enabled: desiredEnabled === true })
  });
}

async function forceAdversarySimDisabled() {
  const deadline = Date.now() + 95_000;
  let lastError = '';
  while (Date.now() < deadline) {
    const state = adversarySimStatusState(await fetchAdversarySimStatus());
    if (state.enabled !== true && state.generationActive !== true && state.phase === 'off') {
      return;
    }
    try {
      await controlAdversarySimViaApi(false);
      lastError = '';
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await new Promise((resolve) => setTimeout(resolve, 2_000));
  }
  throw new Error(`failed to force adversary sim disabled within cleanup window (${lastError})`);
}

async function fetchAdversarySimStatus() {
  const cacheBuster = `${Date.now().toString(16)}-${Math.floor(Math.random() * 0x1_0000_0000).toString(16)}`;
  return requestJson(`/admin/adversary-sim/status?cache_bust=${cacheBuster}`, {
    method: 'GET',
    headers: adminHeaders({
      'Cache-Control': 'no-store',
      Pragma: 'no-cache'
    })
  });
}

function adversarySimStatusState(payload) {
  const source = payload && typeof payload === 'object' ? payload : {};
  return {
    enabled: source.adversary_sim_enabled === true || source.enabled === true,
    generationActive: source.generation_active === true || source.generationActive === true,
    phase: String(source.phase || 'off').trim().toLowerCase()
  };
}

function adversarySimStateMatchesDesired(state, desiredEnabled) {
  const desired = desiredEnabled === true;
  if (desired) {
    return state.enabled === true;
  }
  return state.enabled !== true && state.generationActive !== true && state.phase === 'off';
}

function controlRetryDelayMs(retryAfterHeader, fallbackMs = 1100) {
  const fallback = Math.max(250, Number(fallbackMs || 0));
  const retryAfterSeconds = Number.parseInt(String(retryAfterHeader || '').trim(), 10);
  if (Number.isFinite(retryAfterSeconds) && retryAfterSeconds > 0) {
    return retryAfterSeconds * 1000 + 250;
  }
  return fallback;
}

async function waitForAdversarySimEnabledState(desiredEnabled, timeoutMs = 30_000) {
  const desired = desiredEnabled === true;
  const deadline = Date.now() + Math.max(1_000, Number(timeoutMs || 0));
  let lastState = adversarySimStatusState({});
  let consecutiveSettledPolls = 0;
  while (Date.now() < deadline) {
    lastState = adversarySimStatusState(await fetchAdversarySimStatus());
    if (desired) {
      if (lastState.enabled === true) {
        return lastState;
      }
      consecutiveSettledPolls = 0;
    } else if (lastState.enabled !== true && lastState.generationActive !== true && lastState.phase === 'off') {
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

async function waitForDashboardAdversarySimUiConvergence(page, desiredEnabled, timeoutMs = 30_000) {
  const toggle = page.locator('#global-adversary-sim-toggle');
  if (desiredEnabled === true) {
    await expect(toggle).toBeChecked({ timeout: timeoutMs });
  } else {
    await expect(toggle).not.toBeChecked({ timeout: timeoutMs });
  }

  await expect
    .poll(async () => {
      const bodyClasses = await page.evaluate(() => Array.from(document?.body?.classList || []));
      return bodyClasses.includes('adversary-sim');
    }, { timeout: timeoutMs })
    .toBe(desiredEnabled === true);
}

async function waitForDashboardAdversarySimUiState(page, desiredEnabled, timeoutMs = 30_000) {
  await waitForAdversarySimEnabledState(desiredEnabled, timeoutMs);
  await waitForDashboardAdversarySimUiConvergence(page, desiredEnabled, timeoutMs);
}

async function clickAdversaryToggleWithRetry(page, desiredEnabled, timeoutMs = 60_000) {
  const toggle = page.locator('#global-adversary-sim-toggle');
  const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
  await expect(toggle).toBeEnabled({ timeout: timeoutMs });
  const desired = desiredEnabled === true;
  if ((await toggle.isChecked()) === desired) {
    await waitForDashboardAdversarySimUiState(page, desired, timeoutMs);
    return null;
  }

  const deadline = Date.now() + Math.max(2_000, Number(timeoutMs || 0));
  let lastStatus = 0;
  while (Date.now() < deadline) {
    if ((await toggle.isChecked()) === desired) {
      return null;
    }

    const responsePromise = page.waitForResponse((response) => (
      response.url().includes('/admin/adversary-sim/control') &&
      response.request().method() === 'POST'
    ), { timeout: 6_000 }).catch(() => null);

    await toggleSwitch.click({ timeout: CONTROL_TIMEOUT_MS });

    const response = await responsePromise;
    if (!response) {
      try {
        await waitForDashboardAdversarySimUiState(page, desired, 5_000);
        return null;
      } catch (_error) {
        await page.waitForTimeout(750);
        continue;
      }
    }
    lastStatus = response.status();
    if (lastStatus === 200) {
      const payload = await response.json();
      const backendState = adversarySimStatusState(
        payload && typeof payload === 'object' ? payload.status : {}
      );
      if (!adversarySimStateMatchesDesired(backendState, desired)) {
        await waitForDashboardAdversarySimUiState(page, desired, timeoutMs);
      } else {
        await waitForDashboardAdversarySimUiConvergence(page, desired, timeoutMs);
      }
      return payload;
    }
    if (lastStatus === 409 || lastStatus === 429) {
      await page.waitForTimeout(controlRetryDelayMs(response.headers()['retry-after']));
      continue;
    }
    const body = await response.text().catch(() => '');
    throw new Error(`adversary toggle control request failed with ${lastStatus}: ${body}`);
  }

  throw new Error(
    `adversary toggle did not reach desired state=${desired} before timeout; last_status=${lastStatus}`
  );
}

async function setAutoRefresh(page, enabled) {
  const toggle = page.locator('#auto-refresh-toggle');
  const toggleSwitch = page.locator('label[for="auto-refresh-toggle"]');
  await expect(toggleSwitch).toBeVisible({ timeout: READY_TIMEOUT_MS });
  if ((await toggle.isChecked()) !== (enabled === true)) {
    await toggleSwitch.click({ timeout: CONTROL_TIMEOUT_MS });
  }
  if (enabled === true) {
    await expect(toggle).toBeChecked();
  } else {
    await expect(toggle).not.toBeChecked();
  }
}

async function fetchMonitoringBootstrap() {
  return requestJson('/admin/monitoring?hours=24&limit=50&bootstrap=1', {
    method: 'GET',
    headers: adminHeaders({ 'Content-Type': 'application/json' })
  });
}

async function fetchMonitoringDelta(afterCursor = '') {
  const suffix = afterCursor
    ? `&after_cursor=${encodeURIComponent(String(afterCursor || '').trim())}`
    : '';
  return requestJson(`/admin/monitoring/delta?hours=24&limit=40${suffix}`, {
    method: 'GET',
    headers: adminHeaders({ 'Content-Type': 'application/json' })
  });
}

function maxSimulationEventTs(payload) {
  const recentEvents = Array.isArray(payload?.details?.events?.recent_events)
    ? payload.details.events.recent_events
    : [];
  const deltaEvents = Array.isArray(payload?.events) ? payload.events : [];
  return [...recentEvents, ...deltaEvents].reduce((maxTs, entry) => {
    const ts = Number(entry?.ts || 0);
    if (entry?.is_simulation === true && Number.isFinite(ts) && ts > maxTs) {
      return ts;
    }
    return maxTs;
  }, 0);
}

async function waitForSimulationEventAdvance(baselineTs, baselineCursor = '') {
  const deadline = Date.now() + MONITORING_TIMEOUT_MS;
  let currentCursor = String(baselineCursor || '').trim();
  let lastObserved = Number(baselineTs || 0);
  while (Date.now() < deadline) {
    const payload = await fetchMonitoringDelta(currentCursor);
    const nextCursor = String(payload?.next_cursor || payload?.window_end_cursor || '').trim();
    if (nextCursor) {
      currentCursor = nextCursor;
    }
    const nextTs = maxSimulationEventTs(payload);
    if (nextTs > lastObserved) {
      lastObserved = nextTs;
    }
    if (nextTs > baselineTs) {
      return nextTs;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }
  throw new Error(
    `Simulation telemetry did not advance beyond baseline_ts=${baselineTs} within ${MONITORING_TIMEOUT_MS}ms (last_observed=${lastObserved}).`
  );
}

async function main() {
  requireEnv('SHUMA_BASE_URL', BASE_URL);
  requireEnv('SHUMA_API_KEY', API_KEY);

  const baselineMonitoring = await fetchMonitoringBootstrap();
  const baselineTs = maxSimulationEventTs(baselineMonitoring);
  const baselineCursor = String(
    baselineMonitoring?.window_end_cursor ||
    baselineMonitoring?.next_cursor ||
    ''
  ).trim();

  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  try {
    await page.goto(`${BASE_URL}/dashboard/login.html?next=%2Fdashboard%2Findex.html`, {
      waitUntil: 'domcontentloaded'
    });
    await page.fill('#current-password', API_KEY);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard\/index\.html/, { timeout: READY_TIMEOUT_MS });
    await setAutoRefresh(page, true);

    const readyStart = Date.now();
    try {
      await page.waitForFunction(() => {
        const testModeToggle = document.querySelector('#global-test-mode-toggle');
        const adversaryToggle = document.querySelector('#global-adversary-sim-toggle');
        return Boolean(
          testModeToggle &&
          adversaryToggle &&
          !testModeToggle.disabled &&
          !adversaryToggle.disabled
        );
      }, null, { timeout: READY_TIMEOUT_MS });
      await page.waitForFunction(() => {
        const feedRows = Array.from(
          document.querySelectorAll('#monitoring-raw-feed tbody tr')
        ).filter((row) => row.querySelector('code'));
        return feedRows.length > 0;
      }, null, { timeout: READY_TIMEOUT_MS });
    } catch (error) {
      throw new Error(
        `Dashboard monitoring readiness did not converge within ${READY_TIMEOUT_MS}ms: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
    const readyElapsedMs = Date.now() - readyStart;

    const toggle = page.locator('#global-adversary-sim-toggle');
    await expect(toggle).toBeEnabled({ timeout: READY_TIMEOUT_MS });
    await expect(toggle).not.toBeChecked();

    const controlStart = Date.now();
    await clickAdversaryToggleWithRetry(page, true, 60_000);
    const controlElapsedMs = Date.now() - controlStart;

    const simulationTs = await waitForSimulationEventAdvance(baselineTs, baselineCursor);
    try {
      await page.waitForFunction((minimumTs) => {
        const rows = Array.from(document.querySelectorAll('#monitoring-raw-feed tbody tr code'));
        return rows.some((node) => {
          const text = String(node.textContent || '');
          if (!text.includes('"is_simulation":true')) return false;
          const match = /"ts":(\d+)/.exec(text);
          if (!match) return false;
          return Number(match[1]) > Number(minimumTs || 0);
        });
      }, baselineTs, { timeout: MONITORING_TIMEOUT_MS });
    } catch (error) {
      const toggleChecked = await page.locator('#global-adversary-sim-toggle').isChecked().catch(() => false);
      const bodyClasses = await page.evaluate(() => Array.from(document?.body?.classList || []));
      const firstRow = await page.evaluate(
        () => document.querySelector('#monitoring-raw-feed tbody tr code')?.textContent?.slice(0, 220) || ''
      );
      throw new Error(
        `Monitoring raw feed did not surface a simulation event newer than baseline_ts=${baselineTs} ` +
          `(backend_advanced_to=${simulationTs}) within ${MONITORING_TIMEOUT_MS}ms ` +
          `(toggle_checked=${toggleChecked}, body_classes=${JSON.stringify(bodyClasses)}, first_row=${JSON.stringify(firstRow)}): ` +
          `${error instanceof Error ? error.message : String(error)}`
      );
    }
    await waitForDashboardAdversarySimUiState(page, true, MONITORING_TIMEOUT_MS);

    console.log(JSON.stringify({
      base_url: BASE_URL,
      ready_elapsed_ms: readyElapsedMs,
      control_elapsed_ms: controlElapsedMs,
      simulation_event_ts: simulationTs
    }, null, 2));
  } finally {
    try {
      await forceAdversarySimDisabled();
    } catch (error) {
      console.error(`cleanup_warning=${error instanceof Error ? error.message : String(error)}`);
    }
    await browser.close();
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
