import { chromium, expect } from '@playwright/test';

const BASE_URL = String(process.env.SHUMA_BASE_URL || '').trim().replace(/\/$/, '');
const API_KEY = String(process.env.SHUMA_API_KEY || '').trim();
const READY_TIMEOUT_MS = 20_000;
const READY_BUDGET_MS = Math.max(
  1_000,
  Number.parseInt(String(process.env.SHUMA_DASHBOARD_READY_BUDGET_MS || '8000').trim(), 10) || 8_000
);
const ACTIVE_SIM_CONTROL_READY_BUDGET_MS = Math.max(
  1_000,
  Number.parseInt(String(process.env.SHUMA_DASHBOARD_ACTIVE_SIM_CONTROL_BUDGET_MS || '3500').trim(), 10) || 3_500
);
const ACTIVE_SIM_FEED_READY_BUDGET_MS = Math.max(
  1_000,
  Number.parseInt(String(process.env.SHUMA_DASHBOARD_ACTIVE_SIM_FEED_BUDGET_MS || '8000').trim(), 10) || 8_000
);
const MONITORING_SNAPSHOT_BUDGET_MS = Math.max(
  1_000,
  Number.parseInt(String(process.env.SHUMA_DASHBOARD_MONITORING_SNAPSHOT_BUDGET_MS || '12000').trim(), 10) || 12_000
);
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

async function controlAdversarySimViaApi(desiredEnabled, desiredLane = '') {
  const normalizedDesiredLane = String(desiredLane || '').trim().toLowerCase();
  const payload = { enabled: desiredEnabled === true };
  if (normalizedDesiredLane) {
    payload.lane = normalizedDesiredLane;
  }
  return requestJson('/shuma/admin/adversary-sim/control', {
    method: 'POST',
    headers: adminHeaders({
      'Idempotency-Key': newDashboardIdempotencyKey()
    }),
    body: JSON.stringify(payload)
  });
}

async function controlAdversarySimViaSessionApi(page, desiredEnabled) {
  const result = await page.evaluate(async ({ desiredEnabled }) => {
    const sessionResponse = await fetch('/shuma/admin/session', {
      method: 'GET',
      credentials: 'same-origin'
    });
    const sessionText = await sessionResponse.text();
    let sessionPayload = {};
    try {
      sessionPayload = JSON.parse(sessionText || '{}');
    } catch (_error) {}
    const csrfToken = typeof sessionPayload?.csrf_token === 'string'
      ? sessionPayload.csrf_token.trim()
      : '';
    const headers = {
      Accept: 'application/json',
      'Content-Type': 'application/json',
      'Idempotency-Key': `dash-live-session-${Date.now().toString(16)}-${Math.floor(Math.random() * 0x1_0000_0000).toString(16).padStart(8, '0')}`
    };
    if (csrfToken) {
      headers['X-Shuma-CSRF'] = csrfToken;
    }
    const response = await fetch('/shuma/admin/adversary-sim/control', {
      method: 'POST',
      credentials: 'same-origin',
      headers,
      body: JSON.stringify({ enabled: desiredEnabled === true })
    });
    const text = await response.text();
    return {
      ok: response.ok,
      status: response.status,
      text
    };
  }, { desiredEnabled: desiredEnabled === true });

  if (!result?.ok) {
    throw new Error(
      `session POST /shuma/admin/adversary-sim/control failed: ${result?.status || 0} ${String(result?.text || '').slice(0, 200)}`
    );
  }
  return JSON.parse(String(result?.text || '{}') || '{}');
}

async function forceAdversarySimDisabled(page) {
  const deadline = Date.now() + 95_000;
  let lastError = '';
  while (Date.now() < deadline) {
    const state = adversarySimStatusState(await fetchAdversarySimStatus());
    if (state.enabled !== true && state.generationActive !== true && state.phase === 'off') {
      return;
    }
    try {
      await controlAdversarySimViaSessionApi(page, false);
      lastError = '';
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await new Promise((resolve) => setTimeout(resolve, 2_000));
  }
  throw new Error(`failed to force adversary sim disabled within cleanup window (${lastError})`);
}

async function disableAdversarySimAndWaitOff(page) {
  await forceAdversarySimDisabled(page);
  try {
    await waitForDashboardAdversarySimUiState(page, false, 30_000);
  } catch (_error) {
    // Cleanup truth is backend state; the browser can be behind during teardown.
  }
}

async function fetchAdversarySimStatus() {
  const cacheBuster = `${Date.now().toString(16)}-${Math.floor(Math.random() * 0x1_0000_0000).toString(16)}`;
  return requestJson(`/shuma/admin/adversary-sim/status?cache_bust=${cacheBuster}`, {
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

async function waitForAdversarySimDesiredLane(desiredLane, timeoutMs = 30_000) {
  const normalizedDesiredLane = String(desiredLane || '').trim().toLowerCase();
  if (!normalizedDesiredLane) {
    throw new Error('desired adversary-sim lane must be a non-empty string');
  }
  const deadline = Date.now() + Math.max(1_000, Number(timeoutMs || 0));
  let lastObservedLane = '';
  while (Date.now() < deadline) {
    const payload = await fetchAdversarySimStatus();
    lastObservedLane = String(payload?.desired_lane || '').trim().toLowerCase();
    if (lastObservedLane === normalizedDesiredLane) {
      return lastObservedLane;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(
    `adversary sim desired lane did not settle to ${normalizedDesiredLane} within ${timeoutMs}ms (last_observed=${lastObservedLane || 'missing'})`
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

async function waitForDashboardAdversarySimUiConsistency(page, timeoutMs = 30_000) {
  const status = adversarySimStatusState(await fetchAdversarySimStatus());
  await waitForDashboardAdversarySimUiConvergence(page, status.enabled === true, timeoutMs);
  return status;
}

async function waitForDashboardMonitoringFeed(page, timeoutMs = READY_TIMEOUT_MS) {
  await page.waitForFunction(() => {
    const feedRows = Array.from(
      document.querySelectorAll('#monitoring-raw-feed tbody tr')
    ).filter((row) => row.querySelector('code'));
    return feedRows.length > 0;
  }, null, { timeout: timeoutMs });
}

async function assertDashboardReadyWithActiveSim(page) {
  const toggle = page.locator('#global-adversary-sim-toggle');
  const controlStart = Date.now();
  await expect(toggle).toBeEnabled({ timeout: ACTIVE_SIM_CONTROL_READY_BUDGET_MS });
  await expect(toggle).toBeChecked({ timeout: ACTIVE_SIM_CONTROL_READY_BUDGET_MS });
  const controlElapsedMs = Date.now() - controlStart;

  const feedStart = Date.now();
  await waitForDashboardMonitoringFeed(page, ACTIVE_SIM_FEED_READY_BUDGET_MS);
  const feedElapsedMs = Date.now() - feedStart;

  return {
    controlElapsedMs,
    feedElapsedMs
  };
}

async function setAdversaryToggleViaUi(page, desiredEnabled, timeoutMs = 60_000) {
  const toggle = page.locator('#global-adversary-sim-toggle');
  const toggleSwitch = page.locator("label.toggle-switch[for='global-adversary-sim-toggle']");
  await expect(toggle).toBeEnabled({ timeout: timeoutMs });
  const desired = desiredEnabled === true;
  if ((await toggle.isChecked()) === desired) {
    await waitForDashboardAdversarySimUiState(page, desired, timeoutMs);
    return null;
  }
  const controlResponses = [];
  const onResponse = (response) => {
    if (!response.url().includes('/shuma/admin/adversary-sim/control')) return;
    if (response.request().method() !== 'POST') return;
    controlResponses.push({
      status: response.status(),
      retryAfter: response.headers()['retry-after'] || ''
    });
  };
  page.on('response', onResponse);
  try {
    await toggleSwitch.click({ timeout: CONTROL_TIMEOUT_MS });
    await waitForDashboardAdversarySimUiState(page, desired, timeoutMs);
  } catch (error) {
    const controlSummary = controlResponses.length > 0
      ? controlResponses.map((entry) => (
        entry.retryAfter
          ? `${entry.status} (retry-after=${entry.retryAfter})`
          : `${entry.status}`
      )).join(', ')
      : 'none observed';
    throw new Error(
      `adversary toggle did not converge to desired state=${desired} within ${timeoutMs}ms ` +
      `(control_responses=${controlSummary}): ${error instanceof Error ? error.message : String(error)}`
    );
  } finally {
    page.off('response', onResponse);
  }
  return controlResponses;
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

async function setShadowModeViaUi(page, desiredEnabled, timeoutMs = 20_000) {
  const toggle = page.locator('#global-shadow-mode-toggle');
  const toggleSwitch = page.locator("label.toggle-switch[for='global-shadow-mode-toggle']");
  await expect(toggle).toBeEnabled({ timeout: timeoutMs });
  const desired = desiredEnabled === true;
  if ((await toggle.isChecked()) !== desired) {
    await toggleSwitch.click({ timeout: CONTROL_TIMEOUT_MS });
  }
  if (desired) {
    await expect(toggle).toBeChecked({ timeout: timeoutMs });
  } else {
    await expect(toggle).not.toBeChecked({ timeout: timeoutMs });
  }
}

async function fetchMonitoringBootstrap() {
  return requestJson('/shuma/admin/monitoring?hours=24&limit=50&bootstrap=1', {
    method: 'GET',
    headers: adminHeaders({ 'Content-Type': 'application/json' })
  });
}

async function fetchMonitoringSnapshot(limit = 50) {
  return requestJson(`/shuma/admin/monitoring?hours=24&limit=${encodeURIComponent(String(limit))}`, {
    method: 'GET',
    headers: adminHeaders({ 'Content-Type': 'application/json' })
  });
}

async function fetchMonitoringDelta(afterCursor = '') {
  const suffix = afterCursor
    ? `&after_cursor=${encodeURIComponent(String(afterCursor || '').trim())}`
    : '';
  return requestJson(`/shuma/admin/monitoring/delta?hours=24&limit=40${suffix}`, {
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

  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  try {
    await page.goto(`${BASE_URL}/shuma/dashboard/login.html?next=%2Fshuma%2Fdashboard%2Findex.html`, {
      waitUntil: 'domcontentloaded'
    });
    await page.fill('#current-password', API_KEY);
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/shuma/dashboard\/index\.html/, { timeout: READY_TIMEOUT_MS });
    await setAutoRefresh(page, true);

    const readyStart = Date.now();
    try {
      await page.waitForFunction(() => {
        const shadowModeToggle = document.querySelector('#global-shadow-mode-toggle');
        const adversaryToggle = document.querySelector('#global-adversary-sim-toggle');
        return Boolean(
          shadowModeToggle &&
          adversaryToggle &&
          !shadowModeToggle.disabled &&
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
    if (readyElapsedMs > READY_BUDGET_MS) {
      throw new Error(
        `Dashboard monitoring readiness exceeded budget (${readyElapsedMs}ms > ${READY_BUDGET_MS}ms).`
      );
    }

    await waitForDashboardAdversarySimUiConsistency(page, 30_000);
    await disableAdversarySimAndWaitOff(page);
    await controlAdversarySimViaApi(false, 'synthetic_traffic');
    await waitForAdversarySimDesiredLane('synthetic_traffic', 30_000);
    await setShadowModeViaUi(page, true, 20_000);
    await setShadowModeViaUi(page, false, 20_000);

    const baselineMonitoring = await fetchMonitoringBootstrap();
    const baselineTs = maxSimulationEventTs(baselineMonitoring);
    const baselineCursor = String(
      baselineMonitoring?.window_end_cursor ||
      baselineMonitoring?.next_cursor ||
      ''
    ).trim();

    const toggle = page.locator('#global-adversary-sim-toggle');
    await expect(toggle).toBeEnabled({ timeout: READY_TIMEOUT_MS });
    await expect(toggle).not.toBeChecked();

    const controlStart = Date.now();
    await setAdversaryToggleViaUi(page, true, 60_000);
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

    const monitoringSnapshotStart = Date.now();
    const monitoringSnapshot = await fetchMonitoringSnapshot(200);
    const monitoringSnapshotElapsedMs = Date.now() - monitoringSnapshotStart;
    if (monitoringSnapshotElapsedMs > MONITORING_SNAPSHOT_BUDGET_MS) {
      throw new Error(
        `Monitoring snapshot exceeded budget (${monitoringSnapshotElapsedMs}ms > ${MONITORING_SNAPSHOT_BUDGET_MS}ms).`
      );
    }
    if (
      monitoringSnapshot?.details?.events?.recent_events_window?.response_shaping_reason !==
      'edge_profile_bounded_details'
    ) {
      throw new Error(
        `Expected edge bounded monitoring details, received ${JSON.stringify(
          monitoringSnapshot?.details?.events?.recent_events_window?.response_shaping_reason || ''
        )}.`
      );
    }

    await page.reload({ waitUntil: 'domcontentloaded' });
    await page.waitForURL(/\/shuma/dashboard\/index\.html/, { timeout: READY_TIMEOUT_MS });
    const postReload = await assertDashboardReadyWithActiveSim(page);

    console.log(JSON.stringify({
      base_url: BASE_URL,
      ready_elapsed_ms: readyElapsedMs,
      control_elapsed_ms: controlElapsedMs,
      simulation_event_ts: simulationTs,
      monitoring_snapshot_elapsed_ms: monitoringSnapshotElapsedMs,
      active_sim_reload_control_elapsed_ms: postReload.controlElapsedMs,
      active_sim_reload_feed_elapsed_ms: postReload.feedElapsedMs
    }, null, 2));
  } finally {
    try {
      await forceAdversarySimDisabled(page);
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
