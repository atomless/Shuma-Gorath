#!/usr/bin/env node

import { chromium } from "@playwright/test";
import process from "node:process";

const SANDBOX_PATTERNS = [
  /settings\.dat: Operation not permitted/i,
  /crashpad\.child_port_handshake.*Permission denied \(1100\)/i,
  /mach_port_rendezvous.*Permission denied/i,
  /signal=SIGABRT/i,
];

const ALLOWED_ACTIONS = new Set([
  "allow_browser_allowlist",
  "not_a_bot_pass",
  "challenge_puzzle_fail_maze",
  "geo_challenge",
  "geo_maze",
  "geo_block",
  "honeypot_deny_temp",
  "header_spoofing_probe",
]);

const ALLOWED_STORAGE_MODES = new Set([
  "stateful_cookie_jar",
  "stateless",
  "cookie_reset_each_request",
]);

const FORBIDDEN_HEADERS = new Set([
  "authorization",
  "x-shuma-api-key",
  "x-shuma-forwarded-secret",
  "x-shuma-health-secret",
  "x-shuma-js-secret",
  "x-shuma-challenge-secret",
]);

const MAX_LINEAGE_ENTRIES = 64;
const MAX_DOM_PATH_ENTRIES = 24;

function clampInt(value, minimum, maximum, fallback) {
  const parsed = Number.parseInt(String(value ?? ""), 10);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }
  return Math.max(minimum, Math.min(maximum, parsed));
}

function extractErrorMessage(error) {
  return String(error?.message || error || "unknown_error");
}

function classifyError(message) {
  if (SANDBOX_PATTERNS.some((pattern) => pattern.test(message))) {
    return "sandbox_launch_failure";
  }
  if (/timeout/i.test(message)) {
    return "timeout";
  }
  if (/net::/i.test(message)) {
    return "network_failure";
  }
  return "runtime_failure";
}

async function readPayloadFromStdin() {
  let raw = "";
  process.stdin.setEncoding("utf8");
  for await (const chunk of process.stdin) {
    raw += chunk;
  }
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error("browser_driver_input_missing");
  }
  let parsed;
  try {
    parsed = JSON.parse(trimmed);
  } catch (error) {
    throw new Error(`browser_driver_input_invalid_json: ${extractErrorMessage(error)}`);
  }
  if (!parsed || typeof parsed !== "object") {
    throw new Error("browser_driver_input_must_be_object");
  }
  return parsed;
}

function normalizeBaseUrl(rawBaseUrl) {
  const value = String(rawBaseUrl || "").trim();
  if (!value) {
    throw new Error("browser_driver_base_url_missing");
  }
  let parsed;
  try {
    parsed = new URL(value);
  } catch (error) {
    throw new Error(`browser_driver_base_url_invalid: ${extractErrorMessage(error)}`);
  }
  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw new Error(`browser_driver_base_url_protocol_unsupported: ${parsed.protocol}`);
  }
  parsed.hash = "";
  return parsed.toString().replace(/\/+$/, "");
}

function normalizeHeaders(rawHeaders) {
  const incoming = rawHeaders && typeof rawHeaders === "object" ? rawHeaders : {};
  const headers = {};
  for (const [rawKey, rawValue] of Object.entries(incoming)) {
    const key = String(rawKey || "").trim();
    if (!key) {
      continue;
    }
    const lowered = key.toLowerCase();
    if (FORBIDDEN_HEADERS.has(lowered)) {
      throw new Error(`browser_driver_forbidden_header:${lowered}`);
    }
    const value = String(rawValue ?? "").trim();
    if (!value) {
      continue;
    }
    headers[key] = value;
  }
  return headers;
}

function normalizeTrustedForwardedSecret(rawSecret) {
  const value = String(rawSecret || "").trim();
  if (!value) {
    return "";
  }
  if (value.length > 256) {
    throw new Error("browser_driver_forwarded_secret_too_long");
  }
  if (/[\r\n]/.test(value)) {
    throw new Error("browser_driver_forwarded_secret_invalid");
  }
  return value;
}

function mustUseSafePath(path) {
  const normalized = String(path || "").trim();
  if (!normalized.startsWith("/")) {
    throw new Error(`browser_driver_path_must_start_with_slash:${normalized}`);
  }
  if (normalized.startsWith("/admin/") || normalized === "/admin") {
    throw new Error(`browser_driver_forbidden_path:${normalized}`);
  }
  return normalized;
}

function appendDomPath(evidence, action, selector) {
  if (!selector || typeof selector !== "string") {
    return;
  }
  if (evidence.challenge_dom_path.length >= MAX_DOM_PATH_ENTRIES) {
    return;
  }
  evidence.challenge_dom_path.push(`${action}:${selector}`);
}

function maybeRecordLineage(evidence, row) {
  if (evidence.request_lineage.length >= MAX_LINEAGE_ENTRIES) {
    return;
  }
  evidence.request_lineage.push(row);
}

function summarizeRequestHeaders(headers) {
  return {
    sim_run_id: String(headers["x-shuma-sim-run-id"] || ""),
    sim_profile: String(headers["x-shuma-sim-profile"] || ""),
    sim_lane: String(headers["x-shuma-sim-lane"] || ""),
    sim_nonce: String(headers["x-shuma-sim-nonce"] || ""),
    sim_ts: String(headers["x-shuma-sim-ts"] || ""),
  };
}

async function runScenario(payload) {
  const action = String(payload.action || "").trim();
  if (!ALLOWED_ACTIONS.has(action)) {
    throw new Error(`browser_driver_action_unsupported:${action}`);
  }

  const baseUrl = normalizeBaseUrl(payload.base_url);
  const headers = normalizeHeaders(payload.headers);
  const trustedForwardedSecret = normalizeTrustedForwardedSecret(
    payload.trusted_forwarded_secret,
  );
  const userAgent = String(payload.user_agent || "ShumaAdversarial/1.0 browser-driver");
  const timeoutMs = clampInt(payload.timeout_ms, 1000, 60000, 15000);
  const settleMs = clampInt(payload.settle_ms, 0, 5000, 200);
  const storageModeRaw = String(payload.storage_mode || "stateful_cookie_jar");
  const storageMode = ALLOWED_STORAGE_MODES.has(storageModeRaw)
    ? storageModeRaw
    : "stateful_cookie_jar";
  const honeypotPath = mustUseSafePath(String(payload.honeypot_path || "/instaban"));

  const evidence = {
    driver_runtime: "playwright_chromium",
    action,
    js_executed: false,
    dom_events: 0,
    storage_mode: storageMode,
    challenge_dom_path: [],
    request_lineage: [],
    correlation_ids: [],
    response_statuses: [],
    launch_mode: "headless",
    anti_flake_policy: {
      timeout_ms: timeoutMs,
      settle_ms: settleMs,
    },
  };

  const knownCorrelationIds = new Set();
  let browser;
  const scenarioStartedAt = Date.now();
  let actionStartedAt = 0;
  let scriptedDelayMs = 0;

  const normalizePath = (targetPath) => mustUseSafePath(String(targetPath || ""));
  const toUrl = (targetPath) => new URL(normalizePath(targetPath), `${baseUrl}/`).toString();

  try {
    browser = await chromium.launch({
      channel: "chromium",
      headless: true,
      args: ["--disable-crashpad", "--disable-crash-reporter"],
    });
    const contextHeaders = { ...headers };
    if (trustedForwardedSecret) {
      // This header is edge-injected by the harness, not attacker-controlled input.
      contextHeaders["X-Shuma-Forwarded-Secret"] = trustedForwardedSecret;
    }
    const context = await browser.newContext({
      userAgent,
      extraHTTPHeaders: contextHeaders,
      ignoreHTTPSErrors: true,
    });
    const page = await context.newPage();

    page.on("domcontentloaded", () => {
      evidence.dom_events += 1;
    });
    page.on("load", () => {
      evidence.dom_events += 1;
    });
    page.on("request", (request) => {
      if (!request.url().startsWith(baseUrl)) {
        return;
      }
      const requestHeaders = request.headers();
      const simHeaders = summarizeRequestHeaders(requestHeaders);
      const nonce = String(simHeaders.sim_nonce || "");
      if (nonce && !knownCorrelationIds.has(nonce)) {
        knownCorrelationIds.add(nonce);
      }
      const requestPath = new URL(request.url()).pathname;
      maybeRecordLineage(evidence, {
        method: request.method(),
        path: requestPath,
        ...simHeaders,
      });
    });
    page.on("response", (response) => {
      if (!response.url().startsWith(baseUrl)) {
        return;
      }
      const responsePath = new URL(response.url()).pathname;
      evidence.response_statuses.push({
        path: responsePath,
        status: response.status(),
      });
    });

    async function maybeApplyStoragePolicy() {
      if (storageMode === "stateful_cookie_jar") {
        return;
      }
      await context.clearCookies();
      try {
        await page.evaluate(() => {
          window.localStorage.clear();
          window.sessionStorage.clear();
        });
      } catch {
        // Ignore until a real document is available.
      }
    }

    async function navigate(targetPath) {
      await maybeApplyStoragePolicy();
      const response = await page.goto(toUrl(targetPath), {
        waitUntil: "domcontentloaded",
        timeout: timeoutMs,
      });
      if (settleMs > 0) {
        await page.waitForTimeout(settleMs);
        scriptedDelayMs += settleMs;
      }
      const content = await page.content();
      return { response, content };
    }

    async function executeAction() {
      if (action === "allow_browser_allowlist") {
        const { response, content } = await navigate("/");
        const status = Number(response?.status() || 0);
        const bodyLower = String(content || "").toLowerCase();
        const gatewayForwardingUnavailable =
          status === 500 && bodyLower.includes("gateway forwarding unavailable");
        const frictionMarkers = [
          "access blocked",
          "access restricted",
          "rate limit exceeded",
          'data-link-kind="maze"',
          "i am not a bot",
          "puzzle",
        ];
        if (
          (status !== 200 && !gatewayForwardingUnavailable) ||
          frictionMarkers.some((marker) => bodyLower.includes(marker))
        ) {
          throw new Error(`browser_allow_expected_clean_allow status=${status}`);
        }
        appendDomPath(evidence, "read", "body");
        return {
          observed_outcome: "allow",
          detail: gatewayForwardingUnavailable ? "gateway_forwarding_unavailable" : "ok",
        };
      }

      if (action === "not_a_bot_pass") {
        await navigate("/challenge/not-a-bot-checkbox");
        appendDomPath(evidence, "read", "#not-a-bot-form");
        const checkbox = page.locator("#not-a-bot-checkbox");
        if (!(await checkbox.isVisible())) {
          throw new Error("browser_not_a_bot_checkbox_missing");
        }
        await page.waitForTimeout(1200);
        scriptedDelayMs += 1200;

        const submitRequestPromise = page.waitForRequest(
          (request) =>
            request.method() === "POST" &&
            request.url().includes("/challenge/not-a-bot-checkbox"),
          { timeout: timeoutMs },
        );
        const submitResponsePromise = page.waitForResponse(
          (response) =>
            response.request().method() === "POST" &&
            response.url().includes("/challenge/not-a-bot-checkbox"),
          { timeout: timeoutMs },
        );

        await checkbox.click();
        appendDomPath(evidence, "click", "#not-a-bot-checkbox");

        const submitRequest = await submitRequestPromise;
        const submitResponse = await submitResponsePromise;
        const submitStatus = submitResponse.status();
        const submitHeaders = await submitResponse.allHeaders();
        const postData = String(submitRequest.postData() || "");
        if (!postData.includes("checked=1") || !postData.includes("telemetry=")) {
          throw new Error("browser_not_a_bot_submit_missing_checked_or_telemetry");
        }
        const setCookie = String(submitHeaders["set-cookie"] || "");
        if (submitStatus !== 303 || !setCookie.includes("shuma_not_a_bot=")) {
          throw new Error(
            `browser_not_a_bot_submit_unexpected status=${submitStatus} set_cookie=${setCookie.length > 0}`,
          );
        }
        return {
          observed_outcome: "not-a-bot",
          detail: "ok",
        };
      }

      if (action === "challenge_puzzle_fail_maze") {
        await navigate("/challenge/puzzle");
        const heading = page.locator("h2");
        if (!(await heading.isVisible())) {
          throw new Error("browser_puzzle_heading_missing");
        }
        appendDomPath(evidence, "read", "h2");
        const outputField = page.locator("#challenge-output");
        if (!(await outputField.isVisible())) {
          throw new Error("browser_puzzle_output_field_missing");
        }
        let currentOutput = String(await outputField.inputValue());
        if (!currentOutput) {
          currentOutput = "0".repeat(16);
        }
        const first = currentOutput[0] === "0" ? "1" : "0";
        const wrongOutput = `${first}${currentOutput.slice(1)}`;
        await outputField.fill(wrongOutput);
        appendDomPath(evidence, "write", "#challenge-output");

        const submitResponsePromise = page.waitForResponse(
          (response) =>
            response.request().method() === "POST" &&
            response.url().includes("/challenge/puzzle"),
          { timeout: timeoutMs },
        );
        await page.click("button[type='submit']");
        appendDomPath(evidence, "click", "button[type='submit']");
        const submitResponse = await submitResponsePromise;
        const status = submitResponse.status();
        const body = await page.content();
        if (
          status === 200 &&
          (body.includes('data-link-kind="maze"') || body.includes("maze-nav-grid"))
        ) {
          return { observed_outcome: "maze", detail: "ok" };
        }
        throw new Error(`browser_puzzle_expected_maze status=${status}`);
      }

      if (action === "geo_challenge") {
        const { response, content } = await navigate("/");
        const status = Number(response?.status() || 0);
        if (
          status === 200 &&
          (content.includes("Puzzle") || content.includes("I am not a bot"))
        ) {
          appendDomPath(evidence, "read", "body");
          return { observed_outcome: "challenge", detail: "ok" };
        }
        throw new Error(`browser_geo_challenge_expected status=${status}`);
      }

      if (action === "geo_maze") {
        const { response, content } = await navigate("/");
        const status = Number(response?.status() || 0);
        if (status === 200 && content.includes('data-link-kind="maze"')) {
          appendDomPath(evidence, "read", "[data-link-kind='maze']");
          return { observed_outcome: "maze", detail: "ok" };
        }
        throw new Error(`browser_geo_maze_expected status=${status}`);
      }

      if (action === "geo_block") {
        const { response, content } = await navigate("/");
        const status = Number(response?.status() || 0);
        if (
          status === 403 &&
          (content.includes("Access Blocked") || content.includes("Access Restricted"))
        ) {
          appendDomPath(evidence, "read", "body");
          return { observed_outcome: "deny_temp", detail: "ok" };
        }
        throw new Error(`browser_geo_block_expected status=${status}`);
      }

      if (action === "honeypot_deny_temp") {
        await navigate(honeypotPath);
        appendDomPath(evidence, "navigate", honeypotPath);
        const { response, content } = await navigate("/");
        const status = Number(response?.status() || 0);
        if (
          [403, 429].includes(status) &&
          (content.includes("Access Blocked") || content.includes("Access Restricted"))
        ) {
          appendDomPath(evidence, "read", "body");
          return { observed_outcome: "deny_temp", detail: "ok" };
        }
        throw new Error(`browser_honeypot_expected_deny status=${status}`);
      }

      if (action === "header_spoofing_probe") {
        const { response, content } = await navigate("/");
        const status = Number(response?.status() || 0);
        if (
          [403, 429].includes(status) ||
          content.includes("Access Blocked") ||
          content.includes("Access Restricted")
        ) {
          throw new Error("browser_header_spoof_probe_expected_untrusted_monitor");
        }
        appendDomPath(evidence, "read", "body");
        return { observed_outcome: "monitor", detail: "ok" };
      }

      throw new Error(`browser_driver_unhandled_action:${action}`);
    }

    actionStartedAt = Date.now();
    const actionResult = await executeAction();
    const finishedAt = Date.now();
    const rawActionDurationMs = Math.max(0, finishedAt - actionStartedAt);
    const actionDurationMs = Math.max(0, rawActionDurationMs - scriptedDelayMs);
    const totalDurationMs = Math.max(0, finishedAt - scenarioStartedAt);
    const launchDurationMs = Math.max(0, actionStartedAt - scenarioStartedAt);
    const jsProbe = await page.evaluate(() => ({
      has_window: typeof window !== "undefined",
      ready_state: String(document.readyState || ""),
    }));
    evidence.js_executed = Boolean(jsProbe?.has_window);
    evidence.correlation_ids = Array.from(knownCorrelationIds).sort();

    return {
      ok: true,
      observed_outcome: String(actionResult.observed_outcome || ""),
      detail: String(actionResult.detail || "ok"),
      browser_evidence: evidence,
      diagnostics: {
        ready_state: String(jsProbe?.ready_state || ""),
        action_duration_ms: actionDurationMs,
        action_duration_raw_ms: rawActionDurationMs,
        scripted_delay_ms: scriptedDelayMs,
        launch_duration_ms: launchDurationMs,
        total_duration_ms: totalDurationMs,
      },
    };
  } catch (error) {
    const message = extractErrorMessage(error);
    const failedAt = Date.now();
    const rawActionDurationMs = actionStartedAt
      ? Math.max(0, failedAt - actionStartedAt)
      : 0;
    const actionDurationMs = Math.max(0, rawActionDurationMs - scriptedDelayMs);
    const totalDurationMs = Math.max(0, failedAt - scenarioStartedAt);
    const launchDurationMs = actionStartedAt
      ? Math.max(0, actionStartedAt - scenarioStartedAt)
      : totalDurationMs;
    evidence.correlation_ids = Array.from(knownCorrelationIds).sort();
    return {
      ok: false,
      observed_outcome: null,
      detail: message,
      browser_evidence: evidence,
      diagnostics: {
        error_code: classifyError(message),
        action_duration_ms: actionDurationMs,
        action_duration_raw_ms: rawActionDurationMs,
        scripted_delay_ms: scriptedDelayMs,
        launch_duration_ms: launchDurationMs,
        total_duration_ms: totalDurationMs,
      },
    };
  } finally {
    if (browser) {
      await browser.close().catch(() => {});
    }
  }
}

async function main() {
  try {
    const payload = await readPayloadFromStdin();
    const result = await runScenario(payload);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exit(result.ok ? 0 : 1);
  } catch (error) {
    const message = extractErrorMessage(error);
    const fallback = {
      ok: false,
      observed_outcome: null,
      detail: message,
      browser_evidence: {
        driver_runtime: "playwright_chromium",
        js_executed: false,
        dom_events: 0,
        storage_mode: "stateful_cookie_jar",
        challenge_dom_path: [],
        request_lineage: [],
        correlation_ids: [],
        response_statuses: [],
      },
      diagnostics: {
        error_code: classifyError(message),
        action_duration_ms: 0,
        action_duration_raw_ms: 0,
        scripted_delay_ms: 0,
        launch_duration_ms: 0,
        total_duration_ms: 0,
      },
    };
    process.stdout.write(`${JSON.stringify(fallback)}\n`);
    process.exit(1);
  }
}

await main();
