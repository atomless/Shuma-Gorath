const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { pathToFileURL } = require('node:url');

const CHART_LITE_PATH = 'dashboard/static/assets/vendor/chart-lite-1.0.0.min.js';
const DASHBOARD_ROOT = path.resolve(__dirname, '..', 'dashboard');
const DASHBOARD_NATIVE_RUNTIME_PATH = path.join(
  DASHBOARD_ROOT,
  'src',
  'lib',
  'runtime',
  'dashboard-native-runtime.js'
);
const DASHBOARD_REFRESH_RUNTIME_PATH = path.join(
  DASHBOARD_ROOT,
  'src',
  'lib',
  'runtime',
  'dashboard-runtime-refresh.js'
);

function setGlobalValue(key, value) {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, key);
  Object.defineProperty(globalThis, key, {
    configurable: true,
    writable: true,
    value
  });
  return () => {
    if (descriptor) {
      Object.defineProperty(globalThis, key, descriptor);
    } else {
      delete globalThis[key];
    }
  };
}

async function withBrowserGlobals(overrides = {}, fn) {
  const defaultLocation = {
    origin: 'http://127.0.0.1:3000',
    pathname: '/dashboard/index.html',
    search: '',
    hash: ''
  };
  const defaultHistory = {
    replaceState: () => {}
  };
  const defaultDocument = {
    getElementById: () => null,
    querySelector: () => null,
    querySelectorAll: () => [],
    createElement: () => ({
      innerHTML: '',
      classList: { add() {}, remove() {}, toggle() {}, contains() { return false; } }
    })
  };

  const windowValue = {
    ...(overrides.window || {}),
    location: overrides.location || (overrides.window && overrides.window.location) || defaultLocation,
    history: overrides.history || (overrides.window && overrides.window.history) || defaultHistory,
    document: overrides.document || (overrides.window && overrides.window.document) || defaultDocument,
    navigator: overrides.navigator || (overrides.window && overrides.window.navigator) || {},
    fetch: overrides.fetch || (overrides.window && overrides.window.fetch) || globalThis.fetch,
    setTimeout,
    clearTimeout,
    requestAnimationFrame:
      overrides.requestAnimationFrame ||
      (overrides.window && overrides.window.requestAnimationFrame) ||
      ((cb) => setTimeout(cb, 0))
  };

  const restoreFns = [];
  restoreFns.push(setGlobalValue('window', windowValue));
  restoreFns.push(setGlobalValue('document', windowValue.document));
  restoreFns.push(setGlobalValue('location', windowValue.location));
  restoreFns.push(setGlobalValue('history', windowValue.history));
  restoreFns.push(setGlobalValue('navigator', windowValue.navigator));
  if (windowValue.fetch) restoreFns.push(setGlobalValue('fetch', windowValue.fetch));
  restoreFns.push(setGlobalValue('URL', URL));
  if (typeof Headers !== 'undefined') restoreFns.push(setGlobalValue('Headers', Headers));
  if (typeof Request !== 'undefined') restoreFns.push(setGlobalValue('Request', Request));
  if (typeof Response !== 'undefined') restoreFns.push(setGlobalValue('Response', Response));

  try {
    return await fn();
  } finally {
    restoreFns.reverse().forEach((restore) => restore());
  }
}

async function importBrowserModule(relativePath) {
  const absolutePath = path.resolve(__dirname, '..', relativePath);
  const url = pathToFileURL(absolutePath).href;
  const cacheBust = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return import(`${url}?test=${cacheBust}`);
}

function toPlain(value) {
  return JSON.parse(JSON.stringify(value));
}

function listJsFilesRecursively(rootDir) {
  const entries = fs.readdirSync(rootDir, { withFileTypes: true });
  const files = [];
  entries.forEach((entry) => {
    const absolute = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listJsFilesRecursively(absolute));
      return;
    }
    if (entry.isFile() && entry.name.endsWith('.js')) {
      files.push(absolute);
    }
  });
  return files;
}

function stripCommentsAndStrings(source) {
  return source
    .replace(/\/\/.*$/gm, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/`(?:\\.|[^`\\])*`/g, '')
    .replace(/"(?:\\.|[^"\\])*"/g, '')
    .replace(/'(?:\\.|[^'\\])*'/g, '');
}

function parseRelativeImports(source) {
  const imports = [];
  const pattern = /^\s*import\s+[^'"]*['"](.+?)['"]\s*;?\s*$/gm;
  let match = pattern.exec(source);
  while (match) {
    const specifier = String(match[1] || '').trim();
    if (specifier.startsWith('.')) {
      imports.push(specifier);
    }
    match = pattern.exec(source);
  }
  return imports;
}

function parseRustStructFieldNames(source, structName) {
  const pattern = new RegExp(`struct\\s+${structName}\\s*\\{([\\s\\S]*?)\\n\\}`, 'm');
  const match = source.match(pattern);
  if (!match) {
    throw new Error(`Unable to locate Rust struct ${structName}`);
  }
  return match[1]
    .split('\n')
    .map((line) => line.match(/^\s*([a-zA-Z0-9_]+)\s*:\s*.+,\s*$/))
    .filter(Boolean)
    .map((entry) => entry[1]);
}

function detectCycles(adjacency) {
  const visiting = new Set();
  const visited = new Set();
  const stack = [];
  const cycles = [];

  const visit = (node) => {
    if (visited.has(node)) return;
    if (visiting.has(node)) {
      const cycleStart = stack.indexOf(node);
      if (cycleStart >= 0) {
        cycles.push([...stack.slice(cycleStart), node]);
      }
      return;
    }
    visiting.add(node);
    stack.push(node);
    const edges = adjacency.get(node) || [];
    edges.forEach((edge) => visit(edge));
    stack.pop();
    visiting.delete(node);
    visited.add(node);
  };

  Array.from(adjacency.keys()).forEach((node) => visit(node));
  return cycles;
}

function createMockCanvasContext() {
  const calls = {
    fillText: [],
    moveTo: [],
    lineTo: [],
    fillStyle: []
  };
  const gradient = { addColorStop: () => {} };
  const canvas = { clientWidth: 320, clientHeight: 180, width: 320, height: 180 };
  const ctx = {
    canvas,
    save: () => {},
    restore: () => {},
    clearRect: () => {},
    setTransform: () => {},
    beginPath: () => {},
    moveTo: (x, y) => calls.moveTo.push([x, y]),
    lineTo: (x, y) => calls.lineTo.push([x, y]),
    arc: () => {},
    closePath: () => {},
    fill: () => {},
    stroke: () => {},
    fillRect: () => {},
    createLinearGradient: () => gradient,
    fillText: (text) => calls.fillText.push(String(text))
  };
  let fillStyleValue = '';
  Object.defineProperty(ctx, 'fillStyle', {
    get() {
      return fillStyleValue;
    },
    set(value) {
      fillStyleValue = String(value);
      calls.fillStyle.push(fillStyleValue);
    }
  });
  return { ctx, calls };
}

function loadClassicBrowserScript(relativePath, overrides = {}) {
  const absolutePath = path.resolve(__dirname, '..', relativePath);
  const source = fs.readFileSync(absolutePath, 'utf8');
  const sandbox = {
    window: {
      ...overrides
    },
    document: overrides.document,
    location: overrides.location,
    navigator: overrides.navigator,
    fetch: overrides.fetch || (typeof fetch === 'undefined' ? undefined : fetch),
    console,
    URL,
    Headers: typeof Headers === 'undefined' ? function HeadersShim() {} : Headers,
    Request: typeof Request === 'undefined' ? function RequestShim() {} : Request,
    Response: typeof Response === 'undefined' ? function ResponseShim() {} : Response
  };
  if (sandbox.document && !sandbox.window.document) sandbox.window.document = sandbox.document;
  if (sandbox.location && !sandbox.window.location) sandbox.window.location = sandbox.location;
  if (sandbox.navigator && !sandbox.window.navigator) sandbox.window.navigator = sandbox.navigator;
  sandbox.globalThis = sandbox.window;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: absolutePath });
  return sandbox.window;
}

test('dashboard API adapters normalize sparse payloads safely', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const api = await importBrowserModule('dashboard/src/lib/domain/api-client.js');

    const analytics = api.adaptAnalytics({ ban_count: '7', test_mode: true });
    assert.equal(analytics.ban_count, 7);
    assert.equal(analytics.test_mode, true);
    assert.equal(analytics.fail_mode, 'open');

    const events = api.adaptEvents({
      recent_events: [{ ip: '198.51.100.1' }, null, 'ignored'],
      top_ips: [['198.51.100.1', '9'], ['198.51.100.2', 4], ['bad']],
      unique_ips: '11'
    });
    assert.equal(events.recent_events.length, 1);
    assert.deepEqual(toPlain(events.top_ips), [
      ['198.51.100.1', 9],
      ['198.51.100.2', 4]
    ]);
    assert.equal(events.unique_ips, 11);

    const configValidation = api.adaptConfigValidation({
      valid: false,
      issues: [{ field: 'rate_limit', message: 'out of range' }]
    });
    assert.equal(configValidation.valid, false);
    assert.deepEqual(toPlain(configValidation.issues), [
      { field: 'rate_limit', message: 'out of range' }
    ]);
  });
});

test('dashboard API client parses JSON payloads when content-type is missing', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const requests = [];
    const client = apiModule.create({
      getAdminContext: () => ({ endpoint: 'https://edge.local', apikey: '', sessionAuth: false }),
      request: async (url, init = {}) => {
        requests.push({ url, init });
        return {
          ok: true,
          status: 200,
          headers: new Headers(),
          text: async () => '{"recent_events":[{"ip":"203.0.113.7"}],"top_ips":[["203.0.113.7",3]],"unique_ips":1}',
          json: async () => ({
            recent_events: [{ ip: '203.0.113.7' }],
            top_ips: [['203.0.113.7', 3]],
            unique_ips: 1
          })
        };
      }
    });

    const events = await client.getEvents(24);
    assert.equal(events.recent_events.length, 1);
    assert.deepEqual(toPlain(events.top_ips), [['203.0.113.7', 3]]);
    assert.equal(events.unique_ips, 1);
    assert.equal(requests.length, 1);
  });
});

test('dashboard API client adds CSRF + same-origin for session-auth writes and strips empty bearer', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const calls = [];
    const client = apiModule.create({
      getAdminContext: () => ({
        endpoint: 'https://edge.local',
        apikey: '   ',
        sessionAuth: true,
        csrfToken: 'csrf-token'
      }),
      request: async (url, init = {}) => {
        calls.push({ url, init });
        const isValidate = String(url).includes('/admin/config/validate');
        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          json: async () => (
            isValidate
              ? { valid: true, issues: [] }
              : { config: { maze_enabled: true } }
          ),
          text: async () => JSON.stringify(
            isValidate
              ? { valid: true, issues: [] }
              : { config: { maze_enabled: true } }
          )
        };
      }
    });

    await client.updateConfig({ maze_enabled: true });
    await client.validateConfigPatch({ maze_enabled: true });
    assert.equal(calls.length, 2);

    const headers = calls[0].init.headers;
    assert.equal(headers.get('authorization'), null);
    assert.equal(headers.get('x-shuma-csrf'), 'csrf-token');
    assert.equal(calls[0].init.credentials, 'same-origin');
    assert.match(String(calls[1].url), /\/admin\/config\/validate$/);
    assert.equal(calls[1].init.headers.get('x-shuma-csrf'), 'csrf-token');
    assert.equal(calls[1].init.credentials, 'same-origin');
  });
});

test('dashboard API client posts robots preview patch payloads for dirty-state previews', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const calls = [];
    const client = apiModule.create({
      getAdminContext: () => ({
        endpoint: 'https://edge.local',
        apikey: '',
        sessionAuth: true,
        csrfToken: 'csrf-token'
      }),
      request: async (url, init = {}) => {
        calls.push({ url, init });
        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          json: async () => ({ preview: 'User-agent: *\nDisallow: /' }),
          text: async () => '{"preview":"User-agent: *\\nDisallow: /"}'
        };
      }
    });

    const payload = await client.getRobotsPreview({
      robots_enabled: true,
      ai_policy_block_training: false,
      ai_policy_block_search: true,
      ai_policy_allow_search_engines: false,
      robots_crawl_delay: 4
    });

    assert.equal(payload.content.includes('User-agent: *'), true);
    assert.equal(calls.length, 1);
    assert.match(String(calls[0].url), /\/admin\/robots\/preview$/);
    assert.equal(String(calls[0].init.method).toUpperCase(), 'POST');
    assert.equal(calls[0].init.headers.get('x-shuma-csrf'), 'csrf-token');
    assert.equal(calls[0].init.credentials, 'same-origin');
  });
});

test('dashboard API client times out stalled requests with DashboardApiError', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const errors = [];
    const client = apiModule.create({
      getAdminContext: () => ({ endpoint: 'https://edge.local', apikey: '', sessionAuth: false }),
      onApiError: (error) => {
        errors.push(error);
      },
      request: async (_url, init = {}) =>
        new Promise((_resolve, reject) => {
          if (init.signal && typeof init.signal.addEventListener === 'function') {
            init.signal.addEventListener('abort', () => {
              const abortError = new Error('Request aborted');
              abortError.name = 'AbortError';
              reject(abortError);
            }, { once: true });
          }
        })
    });

    await assert.rejects(
      () => client.getEvents(24, { timeoutMs: 25 }),
      (error) => {
        assert.equal(error.name, 'DashboardApiError');
        assert.match(String(error.message || ''), /timed out/i);
        return true;
      }
    );
    assert.equal(errors.length, 1);
    assert.equal(errors[0].name, 'DashboardApiError');
  });
});

test('chart-lite renders doughnut legend labels and dark center fill', () => {
  const script = loadClassicBrowserScript(CHART_LITE_PATH, {});
  const Chart = script.Chart;
  const { ctx, calls } = createMockCanvasContext();

  new Chart(ctx, {
    type: 'doughnut',
    data: {
      labels: ['Allow', 'Challenge', 'Block'],
      datasets: [{ data: [10, 4, 1] }]
    },
    options: { theme: 'dark' }
  });

  assert.equal(calls.fillText.includes('Allow'), true);
  assert.equal(calls.fillStyle.length > 0, true);
  assert.equal(calls.fillStyle.every((entry) => String(entry).toLowerCase() === '#ffffff'), false);
});

test('chart runtime adapter lazily loads once and tears down on final release', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adapter = await importBrowserModule('dashboard/src/lib/domain/services/chart-runtime-adapter.js');
    const win = { location: { pathname: '/dashboard/index.html' }, Chart: undefined };

    const appendedScripts = [];
    const scriptNode = {
      dataset: {},
      attributes: {},
      parentNode: null,
      setAttribute(name, value) {
        this.attributes[name] = value;
      },
      getAttribute(name) {
        return this.attributes[name] || '';
      },
      addEventListener(event, handler) {
        if (event === 'load') {
          this._onload = handler;
        }
      },
      removeEventListener() {}
    };

    const doc = {
      head: {
        appendChild(node) {
          appendedScripts.push(node);
          node.parentNode = this;
          win.Chart = function ChartMock() {};
          if (typeof node._onload === 'function') node._onload();
        },
        removeChild(node) {
          node.parentNode = null;
        }
      },
      body: null,
      createElement() {
        return { ...scriptNode, dataset: {}, attributes: {} };
      },
      querySelectorAll() {
        return [];
      }
    };

    const one = await adapter.acquireChartRuntime({ window: win, document: doc, src: '/dashboard/assets/chart-lite.js' });
    const two = await adapter.acquireChartRuntime({ window: win, document: doc, src: '/dashboard/assets/chart-lite.js' });

    assert.equal(typeof one, 'function');
    assert.equal(one, two);
    assert.equal(appendedScripts.length, 1);

    adapter.releaseChartRuntime({ window: win, document: doc });
    assert.equal(typeof win.Chart, 'function');
    adapter.releaseChartRuntime({ window: win, document: doc });
    assert.equal(win.Chart, undefined);
  });
});

test('dashboard state and store contracts remain immutable and bounded', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const stateModule = await importBrowserModule('dashboard/src/lib/domain/dashboard-state.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const initial = stateModule.createInitialState('monitoring');
    const next = stateModule.reduceState(initial, { type: 'set-active-tab', tab: 'verification' });
    assert.notEqual(initial, next);
    assert.equal(initial.activeTab, 'monitoring');
    assert.equal(next.activeTab, 'verification');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    store.recordRefreshMetrics({ tab: 'monitoring', reason: 'manual', fetchLatencyMs: 100, renderTimingMs: 10 });
    store.recordRefreshMetrics({ tab: 'monitoring', reason: 'manual', fetchLatencyMs: 200, renderTimingMs: 20 });
    store.recordRefreshMetrics({ tab: 'status', reason: 'manual', fetchLatencyMs: 999, renderTimingMs: 999 });

    const telemetry = store.getRuntimeTelemetry();
    assert.equal(telemetry.refresh.fetchLatencyMs.last, 200);
    assert.equal(telemetry.refresh.renderTimingMs.last, 20);
    assert.equal(telemetry.refresh.lastTab, 'monitoring');
    assert.equal(telemetry.refresh.fetchLatencyMs.totalSamples, 2);
    assert.equal(telemetry.refresh.fetchLatencyMs.window.length > 0, true);
  });
});

test('monitoring view model and status module remain pure snapshot transforms', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const monitoringModelModule = await importBrowserModule('dashboard/src/lib/components/dashboard/monitoring-view-model.js');
    const monitoringNormalizers = await importBrowserModule('dashboard/src/lib/domain/monitoring-normalizers.js');
    const ipRangePolicyModule = await importBrowserModule('dashboard/src/lib/domain/ip-range-policy.js');
    const statusModule = await importBrowserModule('dashboard/src/lib/domain/status.js');

    const summary = monitoringModelModule.deriveMonitoringSummaryViewModel({
      honeypot: {
        total_hits: 120,
        unique_crawlers: 4,
        top_crawlers: [{ label: 'bot-a', count: 40 }, { label: 'bot-b', count: 30 }]
      },
      challenge: {
        total_failures: 10,
        unique_offenders: 5,
        top_offenders: [{ label: 'hash-a', count: 4 }],
        reasons: { timeout: 3 },
        trend: []
      },
      not_a_bot: {
        served: 20,
        submitted: 18,
        pass: 12,
        escalate: 4,
        fail: 2,
        replay: 1,
        abandonments_estimated: 2,
        abandonment_ratio: 0.1,
        outcomes: { pass: 12, escalate: 4, fail: 2, replay: 1 },
        solve_latency_buckets: { lt_1s: 3, '1_3s': 8, '3_10s': 5, '10s_plus': 2 }
      },
      pow: {
        total_failures: 5,
        total_successes: 5,
        total_attempts: 10,
        success_ratio: 0.5,
        unique_offenders: 2,
        top_offenders: [{ label: 'hash-pow', count: 3 }],
        reasons: { invalid_proof: 5 },
        outcomes: { success: 5, failure: 5 },
        trend: []
      },
      rate: {
        total_violations: 12,
        unique_offenders: 2,
        top_offenders: [{ label: 'ip-a', count: 8 }],
        outcomes: { block: 7 }
      },
      geo: {
        total_violations: 5,
        actions: { block: 3, challenge: 2, maze: 0 },
        top_countries: [['US', 3]]
      }
    });
    assert.equal(summary.honeypot.totalHits, '120');
    assert.equal(summary.challenge.totalFailures, '10');
    assert.equal(summary.notABot.served, '20');
    assert.equal(summary.notABot.pass, '12');
    assert.equal(summary.notABot.abandonmentRate, '10.0%');
    assert.equal(summary.pow.totalFailures, '5');
    assert.equal(summary.pow.totalSuccesses, '5');
    assert.equal(summary.pow.totalAttempts, '10');
    assert.equal(summary.pow.successRate, '50.0%');
    assert.equal(
      summary.pow.outcomes.some((row) => row[0] === 'success' && Number(row[1]) === 5),
      true
    );
    const helper = monitoringModelModule.derivePrometheusHelperViewModel({
      docs: {
        observability: 'javascript:alert(1)',
        api: 'https://example.com/api'
      }
    });
    assert.equal(helper.observabilityLink, '');
    assert.equal(helper.apiLink, 'https://example.com/api');

    const parsedOutcome = ipRangePolicyModule.parseIpRangeOutcome(
      'source=managed source_id=openai-gptbot action=forbidden_403 matched_cidr=203.0.113.0/24 taxonomy[level=L11 action=A_DENY_HARD detection=D_IP_RANGE_FORBIDDEN signals=S_IP_RANGE_MANAGED]'
    );
    assert.equal(parsedOutcome.source, 'managed');
    assert.equal(parsedOutcome.sourceId, 'openai-gptbot');
    assert.equal(parsedOutcome.action, 'forbidden_403');
    assert.equal(parsedOutcome.detection, 'D_IP_RANGE_FORBIDDEN');
    assert.deepEqual(toPlain(parsedOutcome.signals), ['S_IP_RANGE_MANAGED']);

    const ipRangeSummary = monitoringModelModule.deriveIpRangeMonitoringViewModel([
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_forbidden',
        outcome: 'source=managed source_id=openai-gptbot action=forbidden_403 matched_cidr=203.0.113.0/24 taxonomy[level=L11 action=A_DENY_HARD detection=D_IP_RANGE_FORBIDDEN signals=S_IP_RANGE_MANAGED]'
      },
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_maze_fallback_block',
        outcome: 'source=custom source_id=manual-bad-range action=maze matched_cidr=198.51.100.0/24 taxonomy[level=L10 action=A_DENY_TEMP detection=D_IP_RANGE_MAZE signals=S_IP_RANGE_CUSTOM]'
      }
    ], {
      ip_range_policy_mode: 'enforce',
      ip_range_emergency_allowlist: ['198.51.100.7/32'],
      ip_range_custom_rules: [{ id: 'manual-bad-range', enabled: true }],
      ip_range_managed_policies: [{ set_id: 'openai-gptbot', enabled: true }],
      ip_range_managed_max_staleness_hours: 24,
      ip_range_allow_stale_managed_enforce: false,
      ip_range_managed_catalog_version: '2026-02-20',
      ip_range_managed_catalog_generated_at: '2026-02-20T00:00:00Z',
      ip_range_managed_catalog_generated_at_unix: Math.floor(Date.now() / 1000) - 3600,
      ip_range_managed_sets: [{ set_id: 'openai-gptbot', provider: 'openai', stale: false, entry_count: 42 }]
    });
    assert.equal(ipRangeSummary.totalMatches, 2);
    assert.equal(ipRangeSummary.mode, 'enforce');
    assert.equal(
      ipRangeSummary.actions.some(([label, count]) => label === 'forbidden_403' && Number(count) === 1),
      true
    );
    assert.equal(ipRangeSummary.catalog.managedSetCount, 1);
    assert.equal(monitoringNormalizers.shouldFetchRange('week'), true);
    assert.equal(monitoringNormalizers.shouldFetchRange('day'), false);
    assert.equal(monitoringNormalizers.hoursForRange('month'), 720);
    assert.deepEqual(
      toPlain(monitoringNormalizers.normalizeReasonRows([['invalid_proof', 4]], { invalid_proof: 'Invalid Proof' })),
      [{ key: 'invalid_proof', label: 'Invalid Proof', count: 4 }]
    );
    assert.deepEqual(
      toPlain(monitoringNormalizers.normalizeTopPaths([{ path: '/trap', count: 12 }])),
      [{ path: '/trap', count: 12 }]
    );
    assert.deepEqual(
      toPlain(monitoringNormalizers.normalizeTopCountries([{ country: 'us', count: 9 }])),
      [{ country: 'us', count: 9 }]
    );
    const series = monitoringNormalizers.buildTimeSeries([
      { ts: 1710000000 },
      { ts: 1710000300 }
    ], 'hour', { nowMs: 1710000600 * 1000, maxEvents: 5000 });
    assert.equal(Array.isArray(series.labels), true);
    assert.equal(Array.isArray(series.data), true);
    assert.equal(series.labels.length > 0, true);
    assert.equal(series.data.length, series.labels.length);

    const configSnapshot = {
      kv_store_fail_open: true,
      test_mode: false,
      pow_enabled: true,
      not_a_bot_enabled: true,
      not_a_bot_risk_threshold: 2,
      challenge_puzzle_enabled: true,
      challenge_puzzle_transform_count: 6,
      challenge_puzzle_risk_threshold: 3,
      ip_range_policy_mode: 'advisory',
      ip_range_emergency_allowlist: ['198.51.100.0/24'],
      ip_range_custom_rules: [{ id: 'custom-1', enabled: true }],
      ip_range_managed_policies: [{ set_id: 'openai-gptbot', enabled: true }],
      ip_range_managed_max_staleness_hours: 24,
      ip_range_allow_stale_managed_enforce: false,
      ip_range_managed_catalog_version: '2026-02-20',
      ip_range_managed_catalog_generated_at_unix: Math.floor(Date.now() / 1000) - 7200,
      ip_range_managed_sets: [{ set_id: 'openai-gptbot', stale: false }],
      botness_maze_threshold: 6,
      botness_weights: {
        js_required: 1,
        geo_risk: 2,
        rate_medium: 1,
        rate_high: 2
      }
    };
    const before = JSON.stringify(configSnapshot);
    const derived = statusModule.deriveStatusSnapshot(configSnapshot);
    assert.equal(String(derived.failMode).toLowerCase(), 'open');
    assert.equal(derived.powEnabled, true);
    assert.equal(derived.notABotEnabled, true);
    assert.equal(JSON.stringify(configSnapshot), before);

    const statusItems = statusModule.buildFeatureStatusItems(derived);
    const stripHtml = (value) => String(value || '').replace(/<[^>]+>/g, '');
    const challengePuzzleItem = statusItems.find((item) => stripHtml(item.title) === 'Challenge Puzzle');
    const challengeNotABotItem = statusItems.find((item) => stripHtml(item.title) === 'Challenge Not-A-Bot');
    const tarpitItem = statusItems.find((item) => stripHtml(item.title) === 'Tarpit');
    const ipRangeItem = statusItems.find((item) => stripHtml(item.title) === 'IP Range Policy');
    const testModeItem = statusItems.find((item) => stripHtml(item.title) === 'Test Mode');
    assert.equal(Boolean(challengePuzzleItem), true);
    assert.equal(Boolean(challengeNotABotItem), true);
    assert.equal(Boolean(tarpitItem), true);
    assert.equal(Boolean(ipRangeItem), true);
    assert.equal(Boolean(testModeItem), false);
    assert.equal(challengePuzzleItem?.status, 'ENABLED');
    assert.equal(challengeNotABotItem?.status, 'ENABLED');
    assert.equal(tarpitItem?.status, 'ENABLED');
    assert.equal(ipRangeItem?.status, 'ADVISORY');
    assert.equal(statusItems.some((item) => stripHtml(item.title) === 'Challenge'), false);
  });
});

test('config form utils and JSON object helpers preserve parser contracts', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const formUtils = await importBrowserModule('dashboard/src/lib/domain/config-form-utils.js');
    const tabHelpers = await importBrowserModule('dashboard/src/lib/domain/config-tab-helpers.js');
    const coreMath = await importBrowserModule('dashboard/src/lib/domain/core/math.js');
    const coreStrings = await importBrowserModule('dashboard/src/lib/domain/core/strings.js');
    const coreDateTime = await importBrowserModule('dashboard/src/lib/domain/core/date-time.js');
    const coreValidation = await importBrowserModule('dashboard/src/lib/domain/core/validation.js');
    const json = await importBrowserModule('dashboard/src/lib/domain/core/json-object.js');
    const schema = await importBrowserModule('dashboard/src/lib/domain/config-schema.js');

    assert.deepEqual(formUtils.parseCountryCodesStrict('GB,US'), ['GB', 'US']);
    assert.equal(formUtils.normalizeListTextareaForCompare('a\na\nb'), 'a\nb');
    assert.deepEqual(formUtils.parseBrowserRulesTextarea('Chrome,120\nFirefox,115'), [
      ['Chrome', 120],
      ['Firefox', 115]
    ]);
    assert.deepEqual(formUtils.parseHoneypotPathsTextarea('/instaban\n/trap/%7Ebot'), [
      '/instaban',
      '/trap/%7Ebot'
    ]);
    assert.deepEqual(formUtils.parseHoneypotPathsTextarea('/trap,segment'), ['/trap,segment']);
    assert.throws(
      () => formUtils.parseHoneypotPathsTextarea('/instaban.  gfdgfdgdfgderg.  egfsdfg'),
      /Invalid honeypot path on line 1/
    );
    assert.throws(
      () => formUtils.parseHoneypotPathsTextarea('/good\n/trap?source=bot'),
      /Invalid honeypot path on line 2/
    );
    assert.throws(
      () => formUtils.parseHoneypotPathsTextarea('/trap/%ZZ'),
      /Invalid honeypot path on line 1/
    );
    assert.equal(coreMath.parseInteger('42', 0), 42);
    assert.equal(coreMath.parseFloatNumber('4.5', 0), 4.5);
    assert.equal(coreMath.toBoundedNonNegativeInteger('-1', 10), 0);
    assert.equal(coreMath.toBoundedNonNegativeInteger('99.9', 10), 10);
    assert.equal(coreStrings.normalizeTrimmed('  a  '), 'a');
    assert.equal(coreStrings.normalizeLowerTrimmed('  A  '), 'a');
    assert.equal(coreStrings.sanitizeDisplayText('\u0001alpha\u0007', '-'), 'alpha');
    assert.equal(coreStrings.formatUnknownForDisplay({ ok: true }), '{"ok":true}');
    assert.deepEqual(toPlain(coreDateTime.durationPartsFromSeconds(3660, 60)), {
      days: 0,
      hours: 1,
      minutes: 1
    });
    assert.equal(coreDateTime.durationSeconds(1, 2, 3), 93780);
    assert.equal(coreDateTime.formatUnixSecondsLocal(0, '-'), '-');
    assert.equal(typeof coreDateTime.formatUnixSecondsLocal(1, '-'), 'string');
    assert.equal(coreValidation.inRange('5', 1, 10), true);
    assert.equal(coreValidation.inRange('x', 1, 10), false);
    assert.equal(coreValidation.isNormalizedInSet('Advisory', new Set(['advisory'])), true);
    assert.equal(
      coreValidation.isDurationTupleValid(0, 1, 0, { minSeconds: 60, maxSeconds: 31536000 }),
      true
    );
    assert.equal(tabHelpers.parseInteger('42', 0), 42);
    assert.equal(tabHelpers.parseInteger('nope', 3), 3);
    assert.equal(tabHelpers.parseFloatNumber('0.75', 0), 0.75);
    assert.equal(tabHelpers.normalizeEdgeMode('Additive'), 'additive');
    assert.equal(tabHelpers.normalizeEdgeMode('invalid'), 'off');
    assert.equal(tabHelpers.normalizeIpRangePolicyMode('ENFORCE'), 'enforce');
    assert.equal(tabHelpers.normalizeIpRangePolicyMode('invalid'), 'off');
    assert.equal(tabHelpers.isIpRangePolicyMode('enforce'), true);
    assert.equal(tabHelpers.isIpRangePolicyMode('invalid'), false);
    assert.equal(tabHelpers.geoModeFromToggleState({ scoringEnabled: true, routingEnabled: false }), 'signal');
    assert.deepEqual(toPlain(tabHelpers.geoToggleStateFromMode('both')), {
      scoringEnabled: true,
      routingEnabled: true
    });
    assert.equal(tabHelpers.rateModeFromToggleState({ enforcementEnabled: false }), 'signal');
    assert.equal(tabHelpers.rateEnforcementEnabledFromMode('both'), true);
    assert.equal(tabHelpers.rateEnforcementEnabledFromMode('signal'), false);
    assert.equal(tabHelpers.formatCountryCodes(['gb', 'US', '', null]), 'GB,US');
    assert.equal(tabHelpers.normalizeJsonArrayForCompare('[1,2,3]'), '[1,2,3]');
    assert.equal(tabHelpers.normalizeJsonArrayForCompare('{"bad":true}'), null);
    assert.deepEqual(toPlain(tabHelpers.durationPartsFromSeconds(3660, 60)), {
      days: 0,
      hours: 1,
      minutes: 1
    });
    assert.equal(tabHelpers.durationSeconds(1, 2, 3), 93780);
    assert.equal(tabHelpers.inRange('9', 1, 10), true);
    assert.equal(tabHelpers.inRange('x', 1, 10), false);
    assert.equal(
      tabHelpers.isDurationTupleValid(0, 1, 0, { minSeconds: 60, maxSeconds: 31536000 }),
      true
    );
    assert.equal(
      tabHelpers.isDurationTupleValid(0, 0, 0, { minSeconds: 60, maxSeconds: 31536000 }),
      false
    );
    assert.equal(tabHelpers.formatIssueReceived('abc'), '"abc"');
    assert.equal(tabHelpers.formatIssueReceived(null), 'null');

    const template = json.buildTemplateFromPaths(
      { pow_enabled: true, botness_weights: { geo_risk: 3 }, extra: 1 },
      ['pow_enabled', 'botness_weights.geo_risk']
    );
    assert.deepEqual(toPlain(template), { pow_enabled: true, botness_weights: { geo_risk: 3 } });
    assert.equal(json.normalizeJsonObjectForCompare('{"ok":true}'), '{"ok":true}');
    const parsedObject = json.parseJsonObjectWithDiagnostics('{\n  "rate_limit": 80\n}');
    assert.equal(parsedObject.ok, true);
    assert.deepEqual(toPlain(parsedObject.value), { rate_limit: 80 });
    assert.equal(parsedObject.normalized, '{"rate_limit":80}');

    const parsedInvalid = json.parseJsonObjectWithDiagnostics('{\n  "rate_limit": 80,\n  "pow_enabled":\n}');
    assert.equal(parsedInvalid.ok, false);
    assert.equal(typeof parsedInvalid.issue?.message, 'string');
    assert.equal(Number(parsedInvalid.issue?.line) > 0, true);
    assert.equal(Number(parsedInvalid.issue?.column) > 0, true);

    const fieldLineMap = json.buildJsonFieldLineMap(
      '{\n  "ban_durations": {\n    "honeypot": 300,\n    "rate_limit": 120\n  },\n  "rate_limit": 80\n}'
    );
    assert.equal(fieldLineMap.get('ban_durations'), 2);
    assert.equal(fieldLineMap.get('ban_durations.honeypot'), 3);
    assert.equal(fieldLineMap.get('ban_durations.rate_limit'), 4);
    assert.equal(fieldLineMap.get('rate_limit'), 6);
    assert.equal(json.resolveJsonFieldLine('ban_durations.honeypot', fieldLineMap), 3);
    assert.equal(json.resolveJsonFieldLine('honeypot', fieldLineMap), 3);
    assert.equal(json.resolveJsonFieldLine('missing_field', fieldLineMap), null);
    assert.equal(Array.isArray(schema.advancedConfigTemplatePaths), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('test_mode'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('browser_policy_enabled'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('bypass_allowlists_enabled'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('geo_edge_headers_enabled'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_enabled'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_progress_token_ttl_seconds'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_progress_replay_ttl_seconds'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_hashcash_base_difficulty'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_step_chunk_base_bytes'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_egress_global_bytes_per_window'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_max_concurrent_global'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_max_concurrent_per_ip_bucket'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('tarpit_fallback_action'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('challenge_puzzle_seed_ttl_seconds'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('challenge_puzzle_attempt_limit_per_window'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('challenge_puzzle_attempt_window_seconds'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('robots_block_ai_training'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('robots_block_ai_search'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('robots_allow_search_engines'), true);
  });
});

test('advanced config template paths match writable admin config patch keys', { concurrency: false }, async () => {
  const apiSource = fs.readFileSync(path.join(__dirname, '..', 'src/admin/api.rs'), 'utf8');
  const schema = await importBrowserModule('dashboard/src/lib/domain/config-schema.js');

  const topLevelFields = parseRustStructFieldNames(apiSource, 'AdminConfigPatch');
  const nestedFieldMap = new Map([
    ['ban_durations', parseRustStructFieldNames(apiSource, 'AdminBanDurationsPatch')],
    ['botness_weights', parseRustStructFieldNames(apiSource, 'AdminBotnessWeightsPatch')],
    ['defence_modes', parseRustStructFieldNames(apiSource, 'AdminDefenceModesPatch')],
    ['provider_backends', parseRustStructFieldNames(apiSource, 'AdminProviderBackendsPatch')]
  ]);

  const writablePaths = [];
  topLevelFields.forEach((field) => {
    if (nestedFieldMap.has(field)) {
      nestedFieldMap.get(field).forEach((nestedField) => {
        writablePaths.push(`${field}.${nestedField}`);
      });
      return;
    }
    writablePaths.push(field);
  });

  const writableSet = new Set(writablePaths);
  const advancedSet = new Set(schema.advancedConfigTemplatePaths);

  const missingFromAdvanced = writablePaths
    .filter((pathValue) => !advancedSet.has(pathValue))
    .sort();
  const nonWritableInAdvanced = Array.from(advancedSet)
    .filter((pathValue) => !writableSet.has(pathValue))
    .sort();

  assert.deepEqual(missingFromAdvanced, []);
  assert.deepEqual(nonWritableInAdvanced, []);
});

test('admin endpoint resolver applies loopback override only for local hostnames', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const endpointModule = await importBrowserModule('dashboard/src/lib/domain/services/admin-endpoint.js');

    const localResolver = endpointModule.createAdminEndpointResolver({
      window: {
        location: {
          origin: 'http://127.0.0.1:3000',
          hostname: '127.0.0.1',
          search: '?api_endpoint=http://localhost:7777',
          protocol: 'http:',
          host: '127.0.0.1:3000'
        }
      }
    });
    assert.equal(localResolver().endpoint, 'http://localhost:7777');

    const remoteResolver = endpointModule.createAdminEndpointResolver({
      window: {
        location: {
          origin: 'https://example.com',
          hostname: 'example.com',
          search: '?api_endpoint=http://localhost:7777',
          protocol: 'https:',
          host: 'example.com'
        }
      }
    });
    assert.equal(remoteResolver().endpoint, 'https://example.com');
  });
});

test('ip bans, verification, traps, advanced, rate-limiting, geo, fingerprinting, robots, and tuning tabs are declarative and callback-driven', () => {
  const ipBansSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/IpBansTab.svelte'),
    'utf8'
  );
  const configSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/VerificationTab.svelte'),
    'utf8'
  );
  const trapsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/TrapsTab.svelte'),
    'utf8'
  );
  const advancedSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/AdvancedTab.svelte'),
    'utf8'
  );
  const robotsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/RobotsTab.svelte'),
    'utf8'
  );
  const rateLimitingSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/RateLimitingTab.svelte'),
    'utf8'
  );
  const geoSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/GeoTab.svelte'),
    'utf8'
  );
  const fingerprintingSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/FingerprintingTab.svelte'),
    'utf8'
  );
  const configMazeSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigMazeSection.svelte'),
    'utf8'
  );
  const configChallengeSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigChallengeSection.svelte'),
    'utf8'
  );
  const configNetworkSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigNetworkSection.svelte'),
    'utf8'
  );
  const configDurationsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigDurationsSection.svelte'),
    'utf8'
  );
  const configGeoSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigGeoSection.svelte'),
    'utf8'
  );
  const configExportSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigExportSection.svelte'),
    'utf8'
  );
  const configAdvancedSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigAdvancedSection.svelte'),
    'utf8'
  );
  const configRobotsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigRobotsSection.svelte'),
    'utf8'
  );
  const saveChangesBarSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/primitives/SaveChangesBar.svelte'),
    'utf8'
  );
  const configSurfaceSource = [
    configSource,
    configChallengeSource,
    saveChangesBarSource
  ].join('\n');
  const trapsSurfaceSource = [
    trapsSource,
    configMazeSource,
    configNetworkSource,
    saveChangesBarSource
  ].join('\n');
  const advancedSurfaceSource = [
    advancedSource,
    configExportSource,
    configAdvancedSource,
    saveChangesBarSource
  ].join('\n');
  const robotsSurfaceSource = [
    robotsSource,
    configRobotsSource,
    saveChangesBarSource
  ].join('\n');
  const fingerprintingSurfaceSource = [
    fingerprintingSource,
    saveChangesBarSource
  ].join('\n');
  const tuningSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/TuningTab.svelte'),
    'utf8'
  );
  const tuningSurfaceSource = [
    tuningSource,
    configDurationsSource,
    saveChangesBarSource
  ].join('\n');

  assert.match(ipBansSource, /export let onBan = null;/);
  assert.match(ipBansSource, /export let onUnban = null;/);
  assert.match(ipBansSource, /export let onSaveConfig = null;/);
  assert.match(ipBansSource, /export let configVersion = 0;/);
  assert.match(ipBansSource, /export let configSnapshot = null;/);
  assert.match(ipBansSource, /let banFilter = 'all';/);
  assert.match(ipBansSource, /id="ip-ban-filter"/);
  assert.match(ipBansSource, /id="bypass-allowlists-toggle"/);
  assert.match(ipBansSource, /id="network-allowlist"/);
  assert.match(ipBansSource, /id="path-allowlist"/);
  assert.match(ipBansSource, /buttonId="save-bypass-allowlists"/);
  assert.match(ipBansSource, /id="ip-range-policy-mode"/);
  assert.match(ipBansSource, /buttonId="save-ip-range-policy"/);
  assert.match(ipBansSource, /await onSaveConfig\(payload/);
  assert.match(ipBansSource, /isIpRangeBanLike/);
  assert.match(ipBansSource, /config\.ban_durations\.admin/);
  assert.match(ipBansSource, /applyConfiguredBanDuration\(nextConfig\)/);
  assert.match(ipBansSource, /#each filteredBanRows as row \(row\.key\)/);
  assert.match(ipBansSource, /aria-expanded=\{detailVisible \? 'true' : 'false'\}/);
  assert.match(ipBansSource, /#if detailVisible/);
  assert.equal(ipBansSource.includes('data-target='), false);
  assert.match(ipBansSource, /disabled=\{!canBan\}/);
  assert.match(ipBansSource, /disabled=\{!canUnban\}/);
  assert.equal(ipBansSource.includes('querySelectorAll('), false);

  assert.match(configSource, /export let onSaveConfig = null;/);
  assert.equal(configSource.includes('onFetchRobotsPreview'), false);
  assert.equal(configSource.includes('test_mode'), false);
  assert.equal(configSource.includes('let ipRangePolicyMode = '), false);
  assert.equal(configSource.includes('robotsAllowSearch'), false);
  assert.equal(configSource.includes('onTestModeToggleChange'), false);
  assert.match(configSource, /await onSaveConfig\(/);
  assert.match(configSource, /import ConfigChallengeSection from '\.\/config\/ConfigChallengeSection\.svelte';/);
  assert.match(configSource, /import ConfigNetworkSection from '\.\/config\/ConfigNetworkSection\.svelte';/);
  assert.equal(configSource.includes("import ConfigExportSection from './config/ConfigExportSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigMazeSection from './config/ConfigMazeSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigDurationsSection from './config/ConfigDurationsSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigGeoSection from './config/ConfigGeoSection.svelte';"), false);
  assert.equal(configSource.includes('ConfigRobotsSection'), false);
  assert.match(configSource, /import SaveChangesBar from '\.\/primitives\/SaveChangesBar\.svelte';/);
  assert.match(configSource, /<ConfigChallengeSection/);
  assert.match(configSource, /<ConfigNetworkSection/);
  assert.equal(configSurfaceSource.includes('id="js-browser-allowlist-rules"'), false);
  assert.equal(configNetworkSource.includes('id="browser-allowlist-rules"'), false);
  assert.equal(configSource.includes('<ConfigExportSection'), false);
  assert.equal(configSource.includes('<ConfigMazeSection'), false);
  assert.equal(configSource.includes('<ConfigDurationsSection'), false);
  assert.equal(configSource.includes('<ConfigGeoSection'), false);
  assert.equal(configSource.includes('<ConfigRobotsSection'), false);
  assert.match(configSource, /showHoneypot=\{false\}/);
  assert.match(configSource, /showBrowserPolicy=\{true\}/);
  assert.match(configSource, /<SaveChangesBar/);
  assert.equal(configSurfaceSource.includes('id="test-mode-toggle"'), false);
  assert.match(configSurfaceSource, /id="preview-challenge-puzzle-link"/);
  assert.match(configSurfaceSource, /id="preview-not-a-bot-link"/);
  assert.equal(
    configSurfaceSource.indexOf('id="not-a-bot-enabled-toggle"')
      < configSurfaceSource.indexOf('id="challenge-puzzle-enabled-toggle"'),
    true
  );
  assert.equal(configSurfaceSource.includes('id="not-a-bot-nonce-ttl"'), false);
  assert.equal(configSurfaceSource.includes('id="not-a-bot-marker-ttl"'), false);
  assert.equal(configSurfaceSource.includes('id="not-a-bot-attempt-limit"'), false);
  assert.equal(configSurfaceSource.includes('id="not-a-bot-attempt-window"'), false);
  assert.match(configSource, /\$: notABotScoreFailMaxCap = Math\.max\(0, Number\(notABotScorePassMin\) - 1\);/);
  assert.match(configSource, /\$: notABotScorePassMinFloor = Math\.min\(10, Number\(notABotScoreFailMax\) \+ 1\);/);
  assert.match(configSurfaceSource, /max=\{notABotScoreFailMaxCap\}/);
  assert.match(configSurfaceSource, /Any scores above Fail and below Pass will be shown a tougher challenge\./);
  assert.equal(configSurfaceSource.includes('id="export-current-config-json"'), false);
  assert.match(configSource, /buttonId="save-verification-all"/);
  assert.match(configSource, /saveAllConfig\(/);
  assert.match(configSource, /window\.addEventListener\('beforeunload'/);
  assert.equal(configSurfaceSource.includes('id="ip-range-policy-mode"'), false);
  assert.equal(configSource.includes('ip_range_policy_mode'), false);
  assert.match(configSurfaceSource, /id="verification-cdp-enabled-toggle"/);
  assert.match(configSurfaceSource, /id="verification-cdp-threshold-slider"/);
  assert.equal(configSurfaceSource.includes('id="edge-integration-mode-select"'), false);
  assert.equal(configSurfaceSource.includes('id="bypass-allowlists-toggle"'), false);
  assert.equal(configSurfaceSource.includes('id="geo-scoring-toggle"'), false);
  assert.equal(configSurfaceSource.includes('id="geo-routing-toggle"'), false);
  assert.match(configSource, /browser_policy_enabled/);
  assert.equal(configSource.includes('bypass_allowlists_enabled'), false);
  assert.equal(configSource.includes('(LOGGING ONLY)'), false);
  assert.equal(configSource.includes('(BLOCKING ACTIVE)'), false);
  assert.equal(configSource.includes('Test Mode Active'), false);
  assert.equal(configSource.includes('ENABLED (LOGGING ONLY)'), false);
  assert.equal(configSource.includes('DISABLED (BLOCKING ACTIVE)'), false);
  assert.equal(configSource.includes('id="save-js-required-config"'), false);
  assert.equal(configSource.includes('id="save-test-mode-config"'), false);
  assert.equal(configSource.includes('id="save-advanced-config"'), false);
  assert.equal(configSource.includes('{@html'), false);

  assert.match(trapsSource, /export let onSaveConfig = null;/);
  assert.match(trapsSource, /await onSaveConfig\(payload/);
  assert.match(trapsSource, /import ConfigMazeSection from '\.\/config\/ConfigMazeSection\.svelte';/);
  assert.match(trapsSource, /import ConfigNetworkSection from '\.\/config\/ConfigNetworkSection\.svelte';/);
  assert.match(trapsSource, /showHoneypot=\{true\}/);
  assert.match(trapsSource, /showBrowserPolicy=\{false\}/);
  assert.match(trapsSource, /buttonId="save-traps-config"/);
  assert.match(trapsSource, /window\.addEventListener\('beforeunload'/);
  assert.match(trapsSurfaceSource, /id="maze-enabled-toggle"/);
  assert.match(trapsSurfaceSource, /id="tarpit-enabled-toggle"/);
  assert.match(trapsSurfaceSource, /id="honeypot-enabled-toggle"/);
  assert.match(trapsSurfaceSource, /id="honeypot-paths"/);
  assert.match(trapsSurfaceSource, /id="preview-maze-link"/);
  assert.match(trapsSurfaceSource, /id="preview-tarpit-link"/);

  assert.match(advancedSource, /export let onSaveConfig = null;/);
  assert.match(advancedSource, /export let onValidateConfig = null;/);
  assert.match(advancedSource, /export let configVersion = 0;/);
  assert.match(advancedSource, /export let configSnapshot = null;/);
  assert.match(advancedSource, /import ConfigExportSection from '\.\/config\/ConfigExportSection\.svelte';/);
  assert.match(advancedSource, /import ConfigAdvancedSection from '\.\/config\/ConfigAdvancedSection\.svelte';/);
  assert.match(advancedSource, /await onSaveConfig\(patch/);
  assert.match(advancedSource, /await onValidateConfig\(advancedPatch\)/);
  assert.match(advancedSource, /id="status-vars-groups"/);
  assert.match(advancedSource, /<ConfigExportSection/);
  assert.match(advancedSource, /buttonId="save-advanced-config"/);
  assert.match(advancedSource, /window\.addEventListener\('beforeunload'/);
  assert.match(advancedSurfaceSource, /id="export-current-config-json"/);
  assert.match(advancedSurfaceSource, /Download the current JSON configuration/);
  assert.match(advancedSurfaceSource, /id="advanced-config-json-error"/);
  assert.match(advancedSurfaceSource, /id="advanced-config-json-issue-list"/);
  assert.match(advancedSurfaceSource, /id="advanced-config-json-validating"/);
  assert.match(advancedSurfaceSource, /id="advanced-config-json-docs-link"/);

  assert.match(rateLimitingSource, /export let onSaveConfig = null;/);
  assert.match(rateLimitingSource, /await onSaveConfig\(payload/);
  assert.match(rateLimitingSource, /id="rate-limiting-enabled-toggle"/);
  assert.match(rateLimitingSource, /id="rate-limit-threshold"/);
  assert.match(rateLimitingSource, /id="rate-akamai-enabled-toggle"/);
  assert.match(rateLimitingSource, /buttonId="save-rate-limiting-config"/);
  assert.match(rateLimitingSource, /window\.addEventListener\('beforeunload'/);

  assert.match(geoSource, /export let onSaveConfig = null;/);
  assert.match(geoSource, /await onSaveConfig\(payload/);
  assert.match(geoSource, /<ConfigGeoSection/);
  assert.match(configGeoSource, /id="geo-scoring-toggle"/);
  assert.match(configGeoSource, /id="geo-routing-toggle"/);
  assert.match(geoSource, /id="geo-akamai-enabled-toggle"/);
  assert.match(geoSource, /geo_edge_headers_enabled/);
  assert.match(geoSource, /buttonId="save-geo-config"/);
  assert.match(geoSource, /window\.addEventListener\('beforeunload'/);

  assert.match(robotsSource, /export let onSaveConfig = null;/);
  assert.match(robotsSource, /export let onFetchRobotsPreview = null;/);
  assert.match(robotsSource, /const buildRobotsPreviewPatch = \(\) => \{/);
  assert.match(robotsSource, /await onSaveConfig\(payload/);
  assert.match(robotsSource, /await onFetchRobotsPreview\(patch\);/);
  assert.match(robotsSource, /buttonId="save-robots-config"/);
  assert.match(robotsSource, /window\.addEventListener\('beforeunload'/);
  assert.match(robotsSurfaceSource, /id="open-robots-txt-link"/);
  assert.match(robotsSurfaceSource, /id="preview-robots"/);

  assert.match(fingerprintingSource, /export let cdpSnapshot = null;/);
  assert.match(fingerprintingSource, /export let onSaveConfig = null;/);
  assert.match(fingerprintingSource, /await onSaveConfig\(payload/);
  assert.match(fingerprintingSource, /id="fingerprinting-akamai-enabled-toggle"/);
  assert.match(fingerprintingSource, /id="fingerprinting-edge-mode-select"/);
  assert.equal(fingerprintingSource.includes('id="fingerprinting-provider-backend-select"'), false);
  assert.equal(fingerprintingSource.includes('id="fingerprinting-cdp-enabled-toggle"'), false);
  assert.equal(fingerprintingSource.includes('id="fingerprinting-cdp-threshold-slider"'), false);
  assert.match(fingerprintingSource, /buttonId="save-fingerprinting-config"/);
  assert.match(fingerprintingSource, /window\.addEventListener\('beforeunload'/);
  assert.match(fingerprintingSurfaceSource, /id="fingerprinting-total-detections"/);

  assert.match(tuningSource, /export let onSaveConfig = null;/);
  assert.match(tuningSource, /await onSaveConfig\(payload/);
  assert.match(tuningSource, /import ConfigDurationsSection from '\.\/config\/ConfigDurationsSection\.svelte';/);
  assert.match(tuningSource, /<ConfigDurationsSection/);
  assert.match(tuningSource, /ban_durations/);
  assert.match(tuningSource, /buttonId="save-tuning-all"/);
  assert.match(tuningSource, /import SaveChangesBar from '\.\/primitives\/SaveChangesBar\.svelte';/);
  assert.match(tuningSource, /window\.addEventListener\('beforeunload'/);
  assert.equal(tuningSource.includes('id="save-botness-config"'), false);
  assert.match(tuningSurfaceSource, /dayId="dur-honeypot-days"/);
  assert.match(tuningSurfaceSource, /dayId="dur-rate-limit-days"/);
});

test('dashboard route lazily loads heavy tabs and keeps orchestration local', () => {
  const source = fs.readFileSync(path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'), 'utf8');

  assert.match(source, /import\('\$lib\/components\/dashboard\/VerificationTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/TrapsTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/AdvancedTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/RateLimitingTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/GeoTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/FingerprintingTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/RobotsTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/TuningTab\.svelte'\)/);
  assert.match(source, /\$lib\/runtime\/dashboard-route-controller\.js/);
  assert.match(source, /<svelte:window on:hashchange=\{onWindowHashChange\} \/>/);
  assert.match(source, /<svelte:document on:visibilitychange=\{onDocumentVisibilityChange\} \/>/);
  assert.match(source, /use:registerTabLink=\{tab\}/);
  assert.match(source, /buildDashboardLoginPath/);
  assert.match(source, /const AUTO_REFRESH_INTERVAL_MS = 60000;/);
  assert.match(source, /isAutoRefreshEnabled: \(\) => autoRefreshEnabled === true/);
  assert.match(source, /shouldRefreshOnActivate: \(\{ tab, store \}\) =>/);
  assert.equal(source.includes('requestNextFrame,'), false);
  assert.equal(source.includes('nowMs,'), false);
  assert.equal(source.includes('readHashTab,'), false);
  assert.equal(source.includes('writeHashTab,'), false);
  assert.equal(source.includes('isPageVisible,'), false);
  assert.equal(source.includes('createDashboardActions'), false);
  assert.equal(source.includes('createDashboardEffects'), false);
  assert.match(source, /onSaveConfig=\{onSaveConfig\}/);
  assert.match(source, /onValidateConfig=\{onValidateConfig\}/);
  assert.match(source, /onBan=\{onBan\}/);
  assert.match(source, /onUnban=\{onUnban\}/);
  assert.match(source, /configSnapshot=\{snapshots\.config\}/);
  assert.match(source, /cdpSnapshot=\{snapshots\.cdp\}/);
  assert.match(source, /id="global-test-mode-toggle"/);
  assert.match(source, /onGlobalTestModeToggleChange/);
  assert.match(source, /dashboard-global-control-label/);
  assert.match(source, /id="admin-msg"/);
});

test('monitoring tab applies bounded sanitization and redraw guards', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/MonitoringTab.svelte'),
    'utf8'
  );

  assert.match(source, /from '\.\.\/\.\.\/domain\/monitoring-normalizers\.js';/);
  assert.match(source, /const RANGE_EVENTS_FETCH_LIMIT = 5000;/);
  assert.match(source, /const RANGE_EVENTS_REQUEST_TIMEOUT_MS = 10000;/);
  assert.match(source, /const RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS = 180000;/);
  assert.match(source, /export let autoRefreshEnabled = false;/);
  assert.match(source, /sameSeries\(chart, trendSeries\.labels, trendSeries\.data\)/);
  assert.match(source, /abortRangeEventsFetch\(\);/);
  assert.match(source, /normalizeReasonRows\(/);
  assert.match(source, /buildTimeSeries\(selectedRangeEvents, selectedTimeRange,/);
});

test('monitoring tab is decomposed into focused subsection components', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/MonitoringTab.svelte'),
    'utf8'
  );

  assert.match(source, /import OverviewStats from '\.\/monitoring\/OverviewStats\.svelte';/);
  assert.match(source, /import PrimaryCharts from '\.\/monitoring\/PrimaryCharts\.svelte';/);
  assert.match(source, /import RecentEventsTable from '\.\/monitoring\/RecentEventsTable\.svelte';/);
  assert.match(source, /import ExternalMonitoringSection from '\.\/monitoring\/ExternalMonitoringSection\.svelte';/);
  assert.match(source, /import IpRangeSection from '\.\/monitoring\/IpRangeSection\.svelte';/);
  assert.match(source, /<OverviewStats/);
  assert.match(source, /<PrimaryCharts/);
  assert.match(source, /<ChallengeSection/);
  assert.match(source, /<PowSection/);
  assert.match(source, /<IpRangeSection/);
  assert.match(source, /<ExternalMonitoringSection/);
});

test('dashboard runtime is slim and free of legacy DOM-id wiring layers', () => {
  const source = fs.readFileSync(DASHBOARD_NATIVE_RUNTIME_PATH, 'utf8');

  assert.equal(source.includes('document.getElementById'), false);
  assert.equal(source.includes('config-controls'), false);
  assert.equal(source.includes('config-ui-state'), false);
  assert.equal(source.includes('input-validation'), false);
  assert.equal(source.includes('tab-lifecycle'), false);
  assert.equal(source.includes('createDashboardSessionRuntime'), false);
  assert.equal(source.includes('createDashboardTabRuntime'), false);
  assert.match(source, /createDashboardRefreshRuntime/);
  assert.equal(source.includes('createDashboardTabStateRuntime'), false);
  assert.match(source, /export async function updateDashboardConfig/);
  assert.match(source, /export async function validateDashboardConfigPatch/);
  assert.match(source, /export async function banDashboardIp/);
  assert.match(source, /export async function unbanDashboardIp/);
  assert.match(source, /dashboardRefreshRuntime\.clearAllCaches/);
});

test('dashboard refresh runtime remains snapshot-only and excludes legacy config UI glue', () => {
  const source = fs.readFileSync(DASHBOARD_REFRESH_RUNTIME_PATH, 'utf8');

  assert.equal(source.includes('updateConfigModeUi'), false);
  assert.equal(source.includes('invokeConfigUiState'), false);
  assert.equal(source.includes('refreshAllDirtySections'), false);
  assert.equal(source.includes('refreshDirtySections'), false);
  assert.equal(source.includes('getMessageNode'), false);
  assert.match(source, /const MONITORING_CACHE_KEY = 'shuma_dashboard_cache_monitoring_v1';/);
  assert.match(source, /const IP_BANS_CACHE_KEY = 'shuma_dashboard_cache_ip_bans_v1';/);
  assert.equal(source.includes('shuma_dashboard_cache_config_v1'), false);
  assert.match(source, /const MONITORING_CACHE_MAX_RECENT_EVENTS = 25;/);
  assert.match(source, /const MONITORING_CACHE_MAX_CDP_EVENTS = 50;/);
  assert.match(source, /const MONITORING_CACHE_MAX_BANS = 100;/);
  assert.match(source, /function clearAllCaches\(\) \{/);
  assert.match(source, /writeCache\(MONITORING_CACHE_KEY, \{ monitoring: compactMonitoring \}\);/);
  assert.match(source, /if \(hasConfigSnapshot\(existingConfig\)\) \{/);
  assert.equal(source.includes("? { monitoring: monitoringData }"), false);
  assert.match(source, /const refreshVerificationTab = \(reason = 'manual'/);
  assert.match(source, /async function refreshFingerprintingTab\(reason = 'manual'/);
  assert.match(source, /dashboardApiClient\.getCdp\(requestOptions\)/);
  assert.match(source, /const includeConfigRefresh = reason !== 'auto-refresh';/);
  assert.match(
    source,
    /includeConfigRefresh \? refreshSharedConfig\(reason, runtimeOptions\) : Promise\.resolve\(null\)/
  );
});

test('dashboard route imports native runtime actions directly', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.equal(source.includes("$lib/runtime/dashboard-runtime.js"), false);
  assert.match(source, /\$lib\/runtime\/dashboard-native-runtime\.js/);
  assert.match(source, /\$lib\/runtime\/dashboard-route-controller\.js/);
  assert.match(source, /updateDashboardConfig/);
  assert.match(source, /banDashboardIp/);
  assert.match(source, /unbanDashboardIp/);
  assert.match(source, /getDashboardRobotsPreview/);
});

test('dashboard route controller gates polling to auto-enabled eligible tabs', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/runtime/dashboard-route-controller.js'),
    'utf8'
  );

  assert.match(source, /const isAutoRefreshEnabled =/);
  assert.match(source, /const isAutoRefreshTab =/);
  assert.match(source, /recordPollingSkip\('auto-refresh-disabled'/);
  assert.match(source, /recordPollingSkip\('tab-not-auto-refreshable'/);
  assert.match(source, /const shouldRefreshOnActivate =/);
});

test('dashboard module graph is layered with no cycles', () => {
  const moduleRoot = path.join(DASHBOARD_ROOT, 'src/lib/domain');
  const moduleFiles = listJsFilesRecursively(moduleRoot);
  const runtimeFiles = listJsFilesRecursively(path.join(DASHBOARD_ROOT, 'src/lib/runtime'));
  const allFiles = [DASHBOARD_NATIVE_RUNTIME_PATH, ...runtimeFiles, ...moduleFiles];

  const relativeOf = (absolutePath) =>
    path.relative(DASHBOARD_ROOT, absolutePath).split(path.sep).join('/');
  const rankOf = (relativePath) => {
    if (relativePath === 'src/lib/runtime/dashboard-native-runtime.js') return 4;
    if (relativePath.startsWith('src/lib/runtime/')) return 3;
    if (relativePath.startsWith('src/lib/domain/core/')) return 0;
    if (relativePath.startsWith('src/lib/domain/services/')) return 1;
    if (
      relativePath === 'src/lib/domain/api-client.js' ||
      relativePath === 'src/lib/domain/dashboard-state.js' ||
      relativePath === 'src/lib/domain/config-schema.js' ||
      relativePath === 'src/lib/domain/config-form-utils.js'
    ) {
      return 1;
    }
    if (relativePath.startsWith('src/lib/domain/')) return 2;
    return 99;
  };

  const knownFiles = new Set(allFiles.map((filePath) => relativeOf(filePath)));
  const adjacency = new Map();
  const rankErrors = [];

  allFiles.forEach((filePath) => {
    const fromRel = relativeOf(filePath);
    const fromDir = path.dirname(filePath);
    const fromRank = rankOf(fromRel);
    const source = fs.readFileSync(filePath, 'utf8');
    const imports = parseRelativeImports(source);
    const edges = [];

    imports.forEach((specifier) => {
      const resolvedAbsolute = path.resolve(fromDir, specifier);
      const withJs = `${resolvedAbsolute}.js`;
      const candidateAbsolute =
        fs.existsSync(resolvedAbsolute) && fs.statSync(resolvedAbsolute).isFile()
          ? resolvedAbsolute
          : (fs.existsSync(withJs) ? withJs : null);
      if (!candidateAbsolute) return;

      const toRel = relativeOf(candidateAbsolute);
      if (!knownFiles.has(toRel)) return;
      edges.push(toRel);

      const toRank = rankOf(toRel);
      if (toRank > fromRank) {
        rankErrors.push(`${fromRel} imports higher layer ${toRel}`);
      }
    });

    adjacency.set(fromRel, edges);
  });

  assert.deepEqual(rankErrors, [], `layering violations:\n${rankErrors.join('\n')}`);
  const cycles = detectCycles(adjacency);
  assert.equal(cycles.length, 0, `module import cycles found:\n${JSON.stringify(cycles, null, 2)}`);
});

test('dashboard modules are reachable from route/runtime entry graph (no dead wrappers)', () => {
  const moduleRoot = path.join(DASHBOARD_ROOT, 'src/lib/domain');
  const moduleFiles = listJsFilesRecursively(moduleRoot);

  const routeAndRuntimeEntries = [
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    path.join(DASHBOARD_ROOT, 'src/routes/login.html/+page.svelte'),
    ...fs
      .readdirSync(path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard'))
      .filter((name) => name.endsWith('.svelte'))
      .map((name) => path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard', name)),
    ...listJsFilesRecursively(path.join(DASHBOARD_ROOT, 'src/lib/runtime')),
    ...listJsFilesRecursively(path.join(DASHBOARD_ROOT, 'src/lib/state'))
  ];

  const queue = routeAndRuntimeEntries.filter((absolutePath) => fs.existsSync(absolutePath));
  const visited = new Set();

  const resolveRelativeImport = (fromPath, specifier) => {
    const base = path.resolve(path.dirname(fromPath), specifier);
    const candidates = [base, `${base}.js`, `${base}.svelte`, path.join(base, 'index.js')];
    for (const candidate of candidates) {
      if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) return candidate;
    }
    return null;
  };

  while (queue.length > 0) {
    const current = queue.pop();
    if (visited.has(current)) continue;
    visited.add(current);
    const source = fs.readFileSync(current, 'utf8');
    const imports = parseRelativeImports(source);
    imports.forEach((specifier) => {
      const resolved = resolveRelativeImport(current, specifier);
      if (resolved && !visited.has(resolved)) {
        queue.push(resolved);
      }
    });
  }

  const unreachableModules = moduleFiles
    .filter((absolutePath) => !visited.has(absolutePath))
    .map((absolutePath) => path.relative(DASHBOARD_ROOT, absolutePath).split(path.sep).join('/'));

  assert.deepEqual(
    unreachableModules,
    [],
    `unreachable dashboard modules found:\n${unreachableModules.join('\n')}`
  );
});
