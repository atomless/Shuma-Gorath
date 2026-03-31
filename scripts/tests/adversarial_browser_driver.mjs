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
  "agentic_browser_session",
  "allow_browser_allowlist",
  "not_a_bot_pass",
  "challenge_puzzle_fail_maze",
  "geo_challenge",
  "geo_maze",
  "geo_block",
  "honeypot_deny_temp",
  "header_spoofing_probe",
  "maze_live_js_flow",
  "maze_live_no_js_fallback",
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
const EMPTY_SECONDARY_TRAFFIC_SUMMARY = Object.freeze({
  secondary_capture_mode: "same_origin_request_events",
  secondary_request_count: 0,
  background_request_count: 0,
  subresource_request_count: 0,
});

const DEFAULT_SIM_HEADER_NAMES = Object.freeze({
  run_id: "x-shuma-sim-run-id",
  profile: "x-shuma-sim-profile",
  lane: "x-shuma-sim-lane",
  timestamp: "x-shuma-sim-ts",
  nonce: "x-shuma-sim-nonce",
  signature: "x-shuma-sim-signature",
});

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

export function normalizeProxyConfig(rawProxyUrl) {
  const value = String(rawProxyUrl || "").trim();
  if (!value) {
    return null;
  }
  let parsed;
  try {
    parsed = new URL(value);
  } catch (error) {
    throw new Error(`browser_driver_proxy_url_invalid:${extractErrorMessage(error)}`);
  }
  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw new Error(`browser_driver_proxy_url_protocol_unsupported:${parsed.protocol}`);
  }
  if ((parsed.pathname && parsed.pathname !== "/") || parsed.search || parsed.hash) {
    throw new Error("browser_driver_proxy_url_must_not_include_path_query_or_fragment");
  }
  const username = parsed.username ? decodeURIComponent(parsed.username) : "";
  const password = parsed.password ? decodeURIComponent(parsed.password) : "";
  parsed.username = "";
  parsed.password = "";
  return {
    server: parsed.toString().replace(/\/+$/, ""),
    username,
    password,
  };
}

function mustUseSafePath(path) {
  const normalized = String(path || "").trim();
  if (!normalized.startsWith("/")) {
    throw new Error(`browser_driver_path_must_start_with_slash:${normalized}`);
  }
  if (normalized.startsWith("/shuma/admin/") || normalized === "/shuma/admin") {
    throw new Error(`browser_driver_forbidden_path:${normalized}`);
  }
  return normalized;
}

export function mergeAgenticSessionPaths(requestedPaths, discoveredPaths, actionBudget) {
  const budget = clampInt(actionBudget, 1, 8, 1);
  const merged = [];
  const seen = new Set();
  for (const path of ["/", ...(requestedPaths || []), ...(discoveredPaths || [])]) {
    const normalized = mustUseSafePath(String(path || "").trim() || "/");
    if (seen.has(normalized)) {
      continue;
    }
    seen.add(normalized);
    merged.push(normalized);
    if (merged.length >= budget) {
      break;
    }
  }
  return merged;
}

function sameOrigin(url, baseOrigin) {
  try {
    return new URL(url).origin === baseOrigin;
  } catch {
    return false;
  }
}

function extractRobotsHintPaths(text, baseOrigin) {
  const hintPaths = [];
  const seen = new Set();
  const body = String(text || "");
  const pattern = /^\s*sitemap:\s*(\S+)\s*$/gim;
  let match;
  while ((match = pattern.exec(body)) !== null) {
    try {
      const resolved = new URL(String(match[1] || "").trim(), `${baseOrigin}/`);
      if (resolved.origin !== baseOrigin) {
        continue;
      }
      const path = mustUseSafePath(resolved.pathname || "/");
      if (seen.has(path)) {
        continue;
      }
      seen.add(path);
      hintPaths.push(path);
    } catch {
      // Ignore malformed sitemap hints.
    }
  }
  return hintPaths;
}

function extractSameOriginUrlPaths(text, baseOrigin) {
  const hintPaths = [];
  const seen = new Set();
  const body = String(text || "");
  const pattern = /https?:\/\/[^\s<>"']+|\/[A-Za-z0-9._~!$&'()*+,;=:@%/-]+/gim;
  let match;
  while ((match = pattern.exec(body)) !== null) {
    try {
      const resolved = new URL(String(match[1] || "").trim(), `${baseOrigin}/`);
      if (resolved.origin !== baseOrigin) {
        continue;
      }
      const path = mustUseSafePath(resolved.pathname || "/");
      if (seen.has(path)) {
        continue;
      }
      seen.add(path);
      hintPaths.push(path);
    } catch {
      // Ignore malformed sitemap entries.
    }
  }
  return hintPaths;
}

function normalizeSimIdentity(rawIdentity) {
  const source = rawIdentity && typeof rawIdentity === "object" ? rawIdentity : {};
  const headerNamesRaw =
    source.header_names && typeof source.header_names === "object" ? source.header_names : {};
  const headerNames = {
    run_id: String(headerNamesRaw.run_id || DEFAULT_SIM_HEADER_NAMES.run_id).trim(),
    profile: String(headerNamesRaw.profile || DEFAULT_SIM_HEADER_NAMES.profile).trim(),
    lane: String(headerNamesRaw.lane || DEFAULT_SIM_HEADER_NAMES.lane).trim(),
    timestamp: String(headerNamesRaw.timestamp || DEFAULT_SIM_HEADER_NAMES.timestamp).trim(),
    nonce: String(headerNamesRaw.nonce || DEFAULT_SIM_HEADER_NAMES.nonce).trim(),
    signature: String(headerNamesRaw.signature || DEFAULT_SIM_HEADER_NAMES.signature).trim(),
  };
  const envelopes = Array.isArray(source.envelopes)
    ? source.envelopes
        .map((entry) => ({
          ts: String(entry?.ts || "").trim(),
          nonce: String(entry?.nonce || "").trim(),
          signature: String(entry?.signature || "").trim(),
        }))
        .filter((entry) => entry.ts && entry.nonce && entry.signature)
    : [];
  return {
    runId: String(source.run_id || "").trim(),
    profile: String(source.profile || "").trim(),
    lane: String(source.lane || "").trim(),
    headerNames,
    envelopes,
  };
}

function withSimHeaders(headers, simIdentity, envelope) {
  if (!simIdentity.runId || !envelope) {
    return headers;
  }
  return {
    ...headers,
    [simIdentity.headerNames.run_id]: simIdentity.runId,
    [simIdentity.headerNames.profile]: simIdentity.profile,
    [simIdentity.headerNames.lane]: simIdentity.lane,
    [simIdentity.headerNames.timestamp]: envelope.ts,
    [simIdentity.headerNames.nonce]: envelope.nonce,
    [simIdentity.headerNames.signature]: envelope.signature,
  };
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

function buildWrongChallengeOutput(currentOutput) {
  const normalized = String(currentOutput || "");
  const baseline = normalized || "0".repeat(16);
  const first = baseline[0] === "0" ? "1" : "0";
  return `${first}${baseline.slice(1)}`;
}

export function classifyMazeDocument(status, content) {
  const normalizedStatus = Number(status || 0);
  const body = String(content || "");
  if (
    normalizedStatus === 403 &&
    (body.includes("Access Blocked") || body.includes("Access Restricted"))
  ) {
    return "block";
  }
  if (
    normalizedStatus === 200 &&
    (body.includes("document.cookie = 'js_verified=") ||
      body.includes("Verifying") ||
      body.includes("proof-of-work"))
  ) {
    return "challenge";
  }
  if (
    normalizedStatus === 200 &&
    (body.includes('data-link-kind="maze"') || body.includes("maze-nav-grid"))
  ) {
    return "maze";
  }
  return "unknown";
}

function requestLineageIncludesPath(evidence, targetPath) {
  return evidence.request_lineage.some((row) => String(row?.path || "") === targetPath);
}

async function discoverSameOriginPaths(page, baseOrigin, currentPath) {
  const discovered = await page.evaluate(() => {
    return Array.from(document.querySelectorAll("a[href]")).map((link) =>
      String(link.getAttribute("href") || "").trim(),
    );
  });
  const paths = [];
  for (const href of discovered) {
    if (!href) {
      continue;
    }
    try {
      const resolved = new URL(href, `${baseOrigin}/`);
      if (resolved.origin !== baseOrigin) {
        continue;
      }
      const safePath = mustUseSafePath(resolved.pathname || "/");
      if (safePath === currentPath) {
        continue;
      }
      paths.push(safePath);
    } catch {
      // Ignore malformed anchor hrefs.
    }
  }
  return paths;
}

async function discoverSessionPaths(page, baseOrigin, currentPath, content) {
  const discoveredPaths = await discoverSameOriginPaths(page, baseOrigin, currentPath);
  if (currentPath === "/robots.txt") {
    return [
      ...discoveredPaths,
      ...extractRobotsHintPaths(content, baseOrigin),
    ];
  }
  if (currentPath.endsWith(".xml")) {
    return [
      ...discoveredPaths,
      ...extractSameOriginUrlPaths(content, baseOrigin),
    ];
  }
  return discoveredPaths;
}

export function validateAllowBrowserAllowlistResponse(status, content) {
  const normalizedStatus = Number(status || 0);
  const bodyLower = String(content || "").toLowerCase();
  const gatewayForwardingUnavailable =
    normalizedStatus >= 500 &&
    normalizedStatus < 600 &&
    bodyLower.includes("gateway forwarding unavailable");
  const frictionMarkers = [
    "access blocked",
    "access restricted",
    "rate limit exceeded",
    'data-link-kind="maze"',
    "i am not a bot",
    "puzzle",
    "verifying",
    "proof-of-work",
    "javascript",
  ];
  if (
    (normalizedStatus !== 200 && !gatewayForwardingUnavailable) ||
    frictionMarkers.some((marker) => bodyLower.includes(marker))
  ) {
    throw new Error(`browser_allow_expected_clean_allow status=${normalizedStatus}`);
  }
  return {
    observed_outcome: "allow",
    detail: gatewayForwardingUnavailable ? "gateway_forwarding_unavailable" : "ok",
  };
}

export async function applyChallengePuzzleWrongOutput(page, evidence) {
  const outputGrid = page.locator("#challenge-output-grid");
  if (!(await outputGrid.isVisible())) {
    throw new Error("browser_puzzle_output_grid_missing");
  }
  appendDomPath(evidence, "read", "#challenge-output-grid");

  const outputField = page.locator("#challenge-output");
  if ((await outputField.count()) < 1) {
    throw new Error("browser_puzzle_output_field_missing");
  }

  const currentOutput = String(await outputField.inputValue());
  const wrongOutput = buildWrongChallengeOutput(currentOutput);
  await outputField.evaluate((node, value) => {
    node.value = value;
  }, wrongOutput);
  appendDomPath(evidence, "write", "#challenge-output");
  return wrongOutput;
}

function maybeRecordLineage(evidence, row) {
  if (evidence.request_lineage.length >= MAX_LINEAGE_ENTRIES) {
    return;
  }
  evidence.request_lineage.push(row);
}

function classifyBrowserRequestKind(request, page) {
  const resourceType = String(request.resourceType() || "").trim() || "other";
  let isMainFrame = false;
  try {
    isMainFrame = request.frame() === page.mainFrame();
  } catch {
    isMainFrame = false;
  }
  if (
    request.isNavigationRequest() &&
    isMainFrame &&
    resourceType === "document"
  ) {
    return {
      request_kind: "top_level",
      resource_type: resourceType,
    };
  }
  if (resourceType === "xhr" || resourceType === "fetch") {
    return {
      request_kind: "background",
      resource_type: resourceType,
    };
  }
  return {
    request_kind: "subresource",
    resource_type: resourceType,
  };
}

export function summarizeBrowserSecondaryTraffic(requestLineage) {
  let backgroundRequestCount = 0;
  let subresourceRequestCount = 0;
  for (const row of Array.isArray(requestLineage) ? requestLineage : []) {
    const requestKind = String(row?.request_kind || "").trim();
    if (requestKind === "background") {
      backgroundRequestCount += 1;
      continue;
    }
    if (requestKind === "subresource") {
      subresourceRequestCount += 1;
    }
  }
  return {
    secondary_capture_mode: "same_origin_request_events",
    secondary_request_count: backgroundRequestCount + subresourceRequestCount,
    background_request_count: backgroundRequestCount,
    subresource_request_count: subresourceRequestCount,
  };
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
  const proxyConfig = normalizeProxyConfig(payload.proxy_url);
  const userAgent = String(payload.user_agent || "ShumaAdversarial/1.0 browser-driver");
  const locale = String(payload.locale || "en-US");
  const simIdentity = normalizeSimIdentity(payload.sim_identity);
  const timeoutMs = clampInt(payload.timeout_ms, 1000, 60000, 15000);
  const settleMs = clampInt(payload.settle_ms, 0, 5000, 200);
  const javascriptEnabled = payload.javascript_enabled !== false;
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
    secondary_traffic: { ...EMPTY_SECONDARY_TRAFFIC_SUMMARY },
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
  const baseOrigin = new URL(`${baseUrl}/`).origin;
  const toUrl = (targetPath) => new URL(normalizePath(targetPath), `${baseUrl}/`).toString();

  try {
    browser = await chromium.launch({
      channel: "chromium",
      headless: true,
      args: ["--disable-crashpad", "--disable-crash-reporter"],
      proxy: proxyConfig
        ? {
            server: proxyConfig.server,
            ...(proxyConfig.username ? { username: proxyConfig.username } : {}),
            ...(proxyConfig.password ? { password: proxyConfig.password } : {}),
          }
        : undefined,
    });
    const contextHeaders = { ...headers };
    if (trustedForwardedSecret) {
      // This header is edge-injected by the harness, not attacker-controlled input.
      contextHeaders["X-Shuma-Forwarded-Secret"] = trustedForwardedSecret;
    }
    const context = await browser.newContext({
      userAgent,
      locale,
      extraHTTPHeaders: contextHeaders,
      ignoreHTTPSErrors: true,
      javaScriptEnabled: javascriptEnabled,
    });
    let simEnvelopeIndex = 0;
    await context.route("**/*", async (route) => {
      const request = route.request();
      if (!sameOrigin(request.url(), baseOrigin)) {
        await route.continue();
        return;
      }
      const envelope = simIdentity.envelopes[simEnvelopeIndex] || null;
      if (envelope) {
        simEnvelopeIndex += 1;
      }
      await route.continue({
        headers: withSimHeaders(request.headers(), simIdentity, envelope),
      });
    });
    const page = await context.newPage();

    page.on("domcontentloaded", () => {
      evidence.dom_events += 1;
    });
    page.on("load", () => {
      evidence.dom_events += 1;
    });
    page.on("request", (request) => {
      if (!sameOrigin(request.url(), baseOrigin)) {
        return;
      }
      const requestHeaders = request.headers();
      const simHeaders = summarizeRequestHeaders(requestHeaders);
      const nonce = String(simHeaders.sim_nonce || "");
      if (nonce && !knownCorrelationIds.has(nonce)) {
        knownCorrelationIds.add(nonce);
      }
      const requestPath = new URL(request.url()).pathname;
      const requestClassification = classifyBrowserRequestKind(request, page);
      maybeRecordLineage(evidence, {
        method: request.method(),
        path: requestPath,
        request_kind: requestClassification.request_kind,
        resource_type: requestClassification.resource_type,
        ...simHeaders,
      });
    });
    page.on("response", (response) => {
      if (!sameOrigin(response.url(), baseOrigin)) {
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
      const pageText = await page.evaluate(() => {
        const body = document.body;
        const root = document.documentElement;
        return String(body?.innerText || body?.textContent || root?.textContent || "");
      });
      return { response, content, pageText };
    }

    async function navigateAbsolute(targetUrl) {
      await maybeApplyStoragePolicy();
      const response = await page.goto(targetUrl, {
        waitUntil: "domcontentloaded",
        timeout: timeoutMs,
      });
      if (settleMs > 0) {
        await page.waitForTimeout(settleMs);
        scriptedDelayMs += settleMs;
      }
      const content = await page.content();
      const pageText = await page.evaluate(() => {
        const body = document.body;
        const root = document.documentElement;
        return String(body?.innerText || body?.textContent || root?.textContent || "");
      });
      return { response, content, pageText };
    }

    async function countNamedCookies(cookieName) {
      const cookies = await context.cookies(baseUrl);
      return cookies.filter((cookie) => cookie.name === cookieName).length;
    }

    async function waitForPathChange(previousPath) {
      await page.waitForFunction(
        (oldPath) => {
          try {
            return window.location.pathname !== oldPath;
          } catch {
            return false;
          }
        },
        previousPath,
        { timeout: timeoutMs },
      );
      await page.waitForLoadState("domcontentloaded", { timeout: timeoutMs }).catch(() => null);
      if (settleMs > 0) {
        await page.waitForTimeout(settleMs);
        scriptedDelayMs += settleMs;
      }
    }

    async function requireFirstMazeLink(selector = "[data-link-kind='maze']") {
      const link = page.locator(selector).first();
      if ((await link.count()) < 1 || !(await link.isVisible())) {
        throw new Error(`browser_maze_link_missing selector=${selector}`);
      }
      const href = String((await link.getAttribute("href")) || "").trim();
      if (!href) {
        throw new Error("browser_maze_link_href_missing");
      }
      const powDifficultyRaw = String((await link.getAttribute("data-pow-difficulty")) || "").trim();
      const parsedPowDifficulty = Number.parseInt(powDifficultyRaw, 10);
      appendDomPath(evidence, "read", selector);
      return {
        link,
        href: new URL(href, `${baseUrl}/`).toString(),
        powRequired: Number.isFinite(parsedPowDifficulty) && parsedPowDifficulty > 0,
        powDifficulty: Number.isFinite(parsedPowDifficulty) && parsedPowDifficulty > 0
          ? parsedPowDifficulty
          : null,
      };
    }

    async function performHumanLikeCheckboxActivation(checkbox) {
      const box = await checkbox.boundingBox();
      if (!box) {
        throw new Error("browser_not_a_bot_checkbox_bounds_missing");
      }
      const targetX = box.x + box.width / 2;
      const targetY = box.y + box.height / 2;
      const entryX = Math.max(8, targetX - 48);
      const entryY = Math.max(8, targetY + 26);
      const approachX = targetX - 10;
      const approachY = targetY + 6;

      await page.mouse.move(entryX, entryY);
      await page.waitForTimeout(140);
      scriptedDelayMs += 140;
      await page.mouse.move(approachX, approachY, { steps: 8 });
      await page.waitForTimeout(90);
      scriptedDelayMs += 90;
      await page.mouse.move(targetX, targetY, { steps: 8 });
      await page.waitForTimeout(70);
      scriptedDelayMs += 70;
      await page.mouse.down();
      await page.waitForTimeout(80);
      scriptedDelayMs += 80;
      await page.mouse.up();
    }

    async function executeAction() {
      if (action === "agentic_browser_session") {
        const sessionPlanRaw =
          payload.session_plan && typeof payload.session_plan === "object"
            ? payload.session_plan
            : {};
        const publicHintPaths = Array.isArray(payload.public_hint_paths)
          ? payload.public_hint_paths
          : [];
        const topLevelActionBudget = clampInt(
          sessionPlanRaw.top_level_action_budget,
          1,
          8,
          1,
        );
        const focusedPagePaths = mergeAgenticSessionPaths(
          Array.isArray(sessionPlanRaw.focused_page_paths)
            ? sessionPlanRaw.focused_page_paths
            : [],
          publicHintPaths,
          topLevelActionBudget,
        );
        const dwellIntervalsMs = Array.isArray(sessionPlanRaw.dwell_intervals_ms)
          ? sessionPlanRaw.dwell_intervals_ms
              .map((value) => clampInt(value, 0, 15000, 0))
              .slice(0, Math.max(0, topLevelActionBudget - 1))
          : [];
        const sessionHandles = Array.isArray(sessionPlanRaw.session_handles)
          ? sessionPlanRaw.session_handles.map((value) => String(value || "").trim()).filter(Boolean)
          : ["agentic-browser-session-1"];
        const topLevelActions = [];
        const usedDwells = [];
        let sessionPaths = [...focusedPagePaths];
        let queueIndex = 0;
        let stopReason = "discovery_frontier_exhausted";

        while (topLevelActions.length < topLevelActionBudget) {
          let targetPath = sessionPaths[queueIndex];
          if (!targetPath) {
            const currentPath = (() => {
              try {
                return mustUseSafePath(new URL(page.url()).pathname || "/");
              } catch {
                return "/";
              }
            })();
            const discoveredPaths = await discoverSameOriginPaths(page, baseOrigin, currentPath);
            sessionPaths = mergeAgenticSessionPaths(
              sessionPaths,
              discoveredPaths,
              topLevelActionBudget,
            );
            targetPath = sessionPaths[queueIndex];
          }
          if (!targetPath) {
            break;
          }

          const { response, pageText } = await navigateAbsolute(toUrl(targetPath));
          const status = Number(response?.status() || 0);
          topLevelActions.push({
            action_index: topLevelActions.length + 1,
            action_type: "browser_navigate",
            path: targetPath,
            status,
          });
          appendDomPath(evidence, "read", "body");
          sessionPaths = mergeAgenticSessionPaths(
            sessionPaths,
            await discoverSessionPaths(page, baseOrigin, targetPath, pageText),
            topLevelActionBudget,
          );
          if (topLevelActions.length >= topLevelActionBudget) {
            stopReason = "top_level_budget_exhausted";
            break;
          }
          const nextQueueIndex = queueIndex + 1;
          if (!sessionPaths[nextQueueIndex]) {
            break;
          }
          const dwellMs = dwellIntervalsMs[topLevelActions.length - 1] || 0;
          if (dwellMs > 0) {
            await page.waitForTimeout(dwellMs);
            scriptedDelayMs += dwellMs;
            usedDwells.push(dwellMs);
          }
          queueIndex = nextQueueIndex;
        }

        evidence.agentic_session_paths = topLevelActions.map((row) => row.path);
        return {
          observed_outcome: "browser_session",
          detail: "ok",
          top_level_actions: topLevelActions,
          realism_receipt: {
            schema_version: "sim-lane-realism-receipt.v1",
            profile_id: String(sessionPlanRaw.profile_id || ""),
            capability_state:
              String(sessionPlanRaw.capability_state || "").trim() || "degraded_fallback",
            action_types_attempted: Array.isArray(sessionPlanRaw.action_types_attempted)
              ? sessionPlanRaw.action_types_attempted
                  .map((value) => String(value || "").trim())
                  .filter(Boolean)
              : ["browser_navigate"],
            targeting_strategy:
              String(sessionPlanRaw.targeting_strategy || "").trim() || "archive_walk",
            planned_activity_budget: clampInt(
              sessionPlanRaw.planned_activity_budget,
              1,
              64,
              topLevelActionBudget,
            ),
            effective_activity_budget: topLevelActionBudget,
            activity_count: topLevelActions.length,
            top_level_action_count: topLevelActions.length,
            focused_page_set_size: focusedPagePaths.length,
            dwell_intervals_ms: usedDwells,
            transport_profile:
              String(sessionPlanRaw.transport_profile || "").trim() || "playwright_chromium",
            observed_user_agent_families: Array.isArray(
              sessionPlanRaw.observed_user_agent_families,
            )
              ? sessionPlanRaw.observed_user_agent_families
                  .map((value) => String(value || "").trim())
                  .filter(Boolean)
              : [],
            observed_accept_languages: Array.isArray(
              sessionPlanRaw.observed_accept_languages,
            )
              ? sessionPlanRaw.observed_accept_languages
                  .map((value) => String(value || "").trim())
                  .filter(Boolean)
              : [],
            observed_browser_locales: Array.isArray(
              sessionPlanRaw.observed_browser_locales,
            )
              ? sessionPlanRaw.observed_browser_locales
                  .map((value) => String(value || "").trim())
                  .filter(Boolean)
              : [locale],
            identity_realism_status:
              String(sessionPlanRaw.identity_realism_status || "").trim() || "degraded_local",
            identity_envelope_classes: Array.isArray(sessionPlanRaw.identity_envelope_classes)
              ? sessionPlanRaw.identity_envelope_classes
                  .map((value) => String(value || "").trim())
                  .filter(Boolean)
              : [],
            geo_affinity_mode:
              String(sessionPlanRaw.geo_affinity_mode || "").trim() || "pool_aligned",
            session_stickiness:
              String(sessionPlanRaw.session_stickiness || "").trim() || "stable_per_tick",
            observed_country_codes: Array.isArray(sessionPlanRaw.observed_country_codes)
              ? sessionPlanRaw.observed_country_codes
                  .map((value) => String(value || "").trim().toUpperCase())
                  .filter(Boolean)
              : [],
            session_handles: sessionHandles,
            identity_rotation_count: 0,
            recurrence_strategy:
              String(sessionPlanRaw.recurrence_strategy || "").trim() ||
              String(sessionPlanRaw.recurrence_context?.strategy || "").trim(),
            reentry_scope:
              String(sessionPlanRaw.reentry_scope || "").trim() ||
              String(sessionPlanRaw.recurrence_context?.reentry_scope || "").trim(),
            dormancy_truth_mode:
              String(sessionPlanRaw.dormancy_truth_mode || "").trim() ||
              String(sessionPlanRaw.recurrence_context?.dormancy_truth_mode || "").trim(),
            session_index: clampInt(
              sessionPlanRaw.session_index ?? sessionPlanRaw.recurrence_context?.session_index,
              0,
              64,
              0,
            ),
            reentry_count: clampInt(
              sessionPlanRaw.reentry_count ?? sessionPlanRaw.recurrence_context?.reentry_count,
              0,
              64,
              0,
            ),
            max_reentries_per_run: clampInt(
              sessionPlanRaw.max_reentries_per_run ??
                sessionPlanRaw.recurrence_context?.max_reentries_per_run,
              0,
              64,
              0,
            ),
            planned_dormant_gap_seconds: clampInt(
              sessionPlanRaw.planned_dormant_gap_seconds ??
                sessionPlanRaw.recurrence_context?.planned_dormant_gap_seconds,
              0,
              3600,
              0,
            ),
            representative_dormant_gap_seconds: clampInt(
              sessionPlanRaw.representative_dormant_gap_seconds ??
                sessionPlanRaw.recurrence_context?.representative_dormant_gap_seconds,
              0,
              604800,
              0,
            ),
            stop_reason:
              topLevelActions.length >= topLevelActionBudget
                ? "top_level_budget_exhausted"
                : stopReason,
          },
        };
      }

      if (action === "allow_browser_allowlist") {
        const { response, content } = await navigate("/");
        const result = validateAllowBrowserAllowlistResponse(
          Number(response?.status() || 0),
          content,
        );
        appendDomPath(evidence, "read", "body");
        return result;
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
        const redirectPromise = page
          .waitForURL((url) => {
            try {
              return new URL(url).pathname === "/";
            } catch {
              return false;
            }
          }, { timeout: timeoutMs })
          .catch(() => null);

        await performHumanLikeCheckboxActivation(checkbox);
        appendDomPath(evidence, "click", "#not-a-bot-checkbox");

        const submitRequest = await submitRequestPromise;
        const submitResponse = await submitResponsePromise;
        await redirectPromise;
        const submitStatus = submitResponse.status();
        const submitHeaders = await submitResponse.allHeaders();
        const postData = String(submitRequest.postData() || "");
        if (!postData.includes("checked=1") || !postData.includes("telemetry=")) {
          throw new Error("browser_not_a_bot_submit_missing_checked_or_telemetry");
        }
        const setCookie = String(submitHeaders["set-cookie"] || "");
        const markerCookieCount = await countNamedCookies("shuma_not_a_bot");
        evidence.set_cookie_observed = markerCookieCount;
        if (submitStatus !== 303 || (markerCookieCount < 1 && !setCookie.includes("shuma_not_a_bot="))) {
          throw new Error(
            `browser_not_a_bot_submit_unexpected status=${submitStatus} set_cookie=${setCookie.length > 0} cookie_count=${markerCookieCount}`,
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
        await applyChallengePuzzleWrongOutput(page, evidence);

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
        await navigate("/");
        appendDomPath(evidence, "read", "body");
        await page.evaluate(async (targetPath) => {
          const response = await fetch(targetPath, {
            method: "GET",
            credentials: "include",
          });
          await response.text().catch(() => "");
          return { status: response.status };
        }, honeypotPath);
        appendDomPath(evidence, "fetch", honeypotPath);
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

      if (action === "maze_live_js_flow") {
        const entryPath = normalizePath(String(payload.maze_entry_path || ""));
        const hiddenLinkMin = clampInt(payload.maze_hidden_link_min, 1, 8, 1);
        const replayAttempts = clampInt(payload.maze_replay_attempts, 0, 3, 2);
        const expectPow = Boolean(payload.maze_expect_pow);

        const { response: entryResponse, content: entryContent } = await navigate(entryPath);
        if (classifyMazeDocument(Number(entryResponse?.status() || 0), entryContent) !== "maze") {
          throw new Error(
            `browser_maze_entry_expected status=${Number(entryResponse?.status() || 0)}`,
          );
        }

        const firstMazeLink = await requireFirstMazeLink();
        if (expectPow && !firstMazeLink.powRequired) {
          throw new Error("browser_maze_expected_pow_link");
        }
        evidence.maze_first_link_pow_required = firstMazeLink.powRequired;
        evidence.maze_first_link_pow_difficulty = firstMazeLink.powDifficulty;
        evidence.maze_entry_path = entryPath;

        const entryPathname = new URL(page.url()).pathname;
        await firstMazeLink.link.click();
        appendDomPath(evidence, "click", "[data-link-kind='maze']");
        await waitForPathChange(entryPathname);

        const childContent = await page.content();
        if (classifyMazeDocument(200, childContent) !== "maze") {
          throw new Error("browser_maze_child_expected");
        }
        const bootstrapRaw = await page.locator("#maze-bootstrap").textContent();
        let bootstrap = {};
        try {
          bootstrap = JSON.parse(String(bootstrapRaw || "{}"));
        } catch (error) {
          throw new Error(`browser_maze_bootstrap_invalid:${extractErrorMessage(error)}`);
        }
        const pathPrefix = String(bootstrap.path_prefix || "").trim();
        if (!pathPrefix) {
          throw new Error("browser_maze_path_prefix_missing");
        }
        const checkpointPath = `${pathPrefix.replace(/\/$/, "")}/checkpoint`;
        const issueLinksPath = `${pathPrefix.replace(/\/$/, "")}/issue-links`;

        await page.waitForFunction(
          (minimum) => document.querySelectorAll("a.hidden-link").length >= minimum,
          hiddenLinkMin,
          { timeout: timeoutMs },
        );
        const hiddenLinks = page.locator("a.hidden-link");
        const hiddenCount = await hiddenLinks.count();
        if (hiddenCount < hiddenLinkMin) {
          throw new Error(`browser_maze_hidden_links_missing count=${hiddenCount}`);
        }
        evidence.maze_hidden_link_count = hiddenCount;
        evidence.maze_checkpoint_path_seen = requestLineageIncludesPath(evidence, checkpointPath);
        evidence.maze_issue_links_path_seen = requestLineageIncludesPath(evidence, issueLinksPath);
        if (!evidence.maze_checkpoint_path_seen) {
          throw new Error("browser_maze_checkpoint_not_observed");
        }
        if (!evidence.maze_issue_links_path_seen) {
          throw new Error("browser_maze_issue_links_not_observed");
        }

        const hiddenHref = String((await hiddenLinks.first().getAttribute("href")) || "").trim();
        if (!hiddenHref) {
          throw new Error("browser_maze_hidden_href_missing");
        }
        appendDomPath(evidence, "read", "a.hidden-link");
        const hiddenResult = await navigateAbsolute(new URL(hiddenHref, `${baseUrl}/`).toString());
        if (
          classifyMazeDocument(Number(hiddenResult.response?.status() || 0), hiddenResult.content) !==
          "maze"
        ) {
          throw new Error("browser_maze_hidden_progress_expected");
        }

        const replayOutcomes = [];
        for (let index = 0; index < replayAttempts; index += 1) {
          const replayResult = await navigateAbsolute(firstMazeLink.href);
          const status = Number(replayResult.response?.status() || 0);
          const outcome = classifyMazeDocument(status, replayResult.content);
          replayOutcomes.push({ status, outcome });
          if (!["challenge", "block"].includes(outcome)) {
            throw new Error(`browser_maze_replay_unexpected status=${status} outcome=${outcome}`);
          }
          if (outcome === "block") {
            break;
          }
        }
        evidence.maze_replay_outcomes = replayOutcomes;
        if (replayAttempts > 0 && !replayOutcomes.some((row) => row.outcome === "block")) {
          throw new Error("browser_maze_replay_block_missing");
        }
        return { observed_outcome: "maze", detail: "ok" };
      }

      if (action === "maze_live_no_js_fallback") {
        const entryPath = normalizePath(String(payload.maze_entry_path || ""));
        const expectedFallback = String(payload.maze_expected_fallback || "challenge").trim();
        if (!["challenge", "block"].includes(expectedFallback)) {
          throw new Error(`browser_maze_expected_fallback_invalid:${expectedFallback}`);
        }

        const { response: entryResponse, content: entryContent } = await navigate(entryPath);
        if (classifyMazeDocument(Number(entryResponse?.status() || 0), entryContent) !== "maze") {
          throw new Error(
            `browser_maze_entry_expected status=${Number(entryResponse?.status() || 0)}`,
          );
        }
        const firstMazeLink = await requireFirstMazeLink();
        evidence.maze_entry_path = entryPath;
        const firstResult = await navigateAbsolute(firstMazeLink.href);
        if (classifyMazeDocument(Number(firstResult.response?.status() || 0), firstResult.content) !== "maze") {
          throw new Error("browser_maze_first_follow_expected");
        }

        let childBootstrap = {};
        try {
          const childBootstrapRaw = await page.locator("#maze-bootstrap").textContent();
          childBootstrap = JSON.parse(String(childBootstrapRaw || "{}"));
        } catch (error) {
          throw new Error(`browser_maze_bootstrap_invalid:${extractErrorMessage(error)}`);
        }
        const pathPrefix = String(childBootstrap.path_prefix || "").trim();
        if (!pathPrefix) {
          throw new Error("browser_maze_path_prefix_missing");
        }
        const checkpointPath = `${pathPrefix.replace(/\/$/, "")}/checkpoint`;
        const issueLinksPath = `${pathPrefix.replace(/\/$/, "")}/issue-links`;
        evidence.maze_checkpoint_path_seen = requestLineageIncludesPath(evidence, checkpointPath);
        evidence.maze_issue_links_path_seen = requestLineageIncludesPath(evidence, issueLinksPath);
        if (evidence.maze_checkpoint_path_seen) {
          throw new Error("browser_maze_no_js_checkpoint_should_not_exist");
        }
        if (evidence.maze_issue_links_path_seen) {
          throw new Error("browser_maze_no_js_issue_links_should_not_exist");
        }

        const secondMazeLink = await requireFirstMazeLink();
        const fallbackResult = await navigateAbsolute(secondMazeLink.href);
        const fallbackStatus = Number(fallbackResult.response?.status() || 0);
        const fallbackOutcome = classifyMazeDocument(fallbackStatus, fallbackResult.content);
        evidence.maze_fallback_outcome = fallbackOutcome;
        if (fallbackOutcome !== expectedFallback) {
          throw new Error(
            `browser_maze_fallback_expected expected=${expectedFallback} actual=${fallbackOutcome} status=${fallbackStatus}`,
          );
        }
        return {
          observed_outcome: expectedFallback,
          detail: "ok",
        };
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
    evidence.secondary_traffic = summarizeBrowserSecondaryTraffic(evidence.request_lineage);
    const realismReceipt =
      actionResult.realism_receipt && typeof actionResult.realism_receipt === "object"
        ? {
            ...actionResult.realism_receipt,
            ...evidence.secondary_traffic,
          }
        : null;

    return {
      ok: true,
      observed_outcome: String(actionResult.observed_outcome || ""),
      detail: String(actionResult.detail || "ok"),
      top_level_actions: Array.isArray(actionResult.top_level_actions)
        ? actionResult.top_level_actions
        : [],
      realism_receipt: realismReceipt,
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
    evidence.secondary_traffic = summarizeBrowserSecondaryTraffic(evidence.request_lineage);
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
        secondary_traffic: { ...EMPTY_SECONDARY_TRAFFIC_SUMMARY },
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

const invokedAsScript =
  process.argv[1] && new URL(`file://${process.argv[1]}`).href === import.meta.url;

if (invokedAsScript) {
  await main();
}
