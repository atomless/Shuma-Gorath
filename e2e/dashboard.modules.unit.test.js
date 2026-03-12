const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

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

function createMutableClassList(initial = []) {
  const classes = new Set(Array.isArray(initial) ? initial : []);
  return {
    add(...tokens) {
      tokens.forEach((token) => {
        const normalized = String(token || '').trim();
        if (normalized) classes.add(normalized);
      });
    },
    remove(...tokens) {
      tokens.forEach((token) => {
        classes.delete(String(token || '').trim());
      });
    },
    toggle(token, force = undefined) {
      const normalized = String(token || '').trim();
      if (!normalized) return false;
      if (force === undefined) {
        if (classes.has(normalized)) {
          classes.delete(normalized);
          return false;
        }
        classes.add(normalized);
        return true;
      }
      if (force) {
        classes.add(normalized);
        return true;
      }
      classes.delete(normalized);
      return false;
    },
    contains(token) {
      return classes.has(String(token || '').trim());
    },
    values() {
      return [...classes.values()];
    }
  };
}

function createRecordingClassList(initial = []) {
  const classes = new Set(Array.isArray(initial) ? initial : []);
  const operations = [];
  const record = (op, token) => {
    operations.push(`${op}:${token}`);
  };
  return {
    add(...tokens) {
      tokens.forEach((token) => {
        const normalized = String(token || '').trim();
        if (!normalized) return;
        if (!classes.has(normalized)) {
          classes.add(normalized);
          record('add', normalized);
        }
      });
    },
    remove(...tokens) {
      tokens.forEach((token) => {
        const normalized = String(token || '').trim();
        if (!normalized) return;
        if (classes.delete(normalized)) {
          record('remove', normalized);
        }
      });
    },
    toggle(token, force = undefined) {
      const normalized = String(token || '').trim();
      if (!normalized) return false;
      if (force === undefined) {
        if (classes.has(normalized)) {
          classes.delete(normalized);
          record('remove', normalized);
          return false;
        }
        classes.add(normalized);
        record('add', normalized);
        return true;
      }
      if (force) {
        if (!classes.has(normalized)) {
          classes.add(normalized);
          record('add', normalized);
        }
        return true;
      }
      if (classes.delete(normalized)) {
        record('remove', normalized);
      }
      return false;
    },
    contains(token) {
      return classes.has(String(token || '').trim());
    },
    values() {
      return [...classes.values()];
    },
    operationCount() {
      return operations.length;
    },
    operationSnapshot() {
      return [...operations];
    }
  };
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

    const monitoring = api.adaptMonitoring({
      summary: {},
      freshness_slo: { visibility_delay_ms: { p95_target: 300 } },
      load_envelope: { query_budget_requests_per_second_per_client: 1 },
      freshness: { state: 'fresh', lag_ms: 125, last_event_ts: 1700000000, transport: 'snapshot_poll' },
      retention_health: {
        state: 'degraded',
        retention_hours: 168,
        oldest_retained_ts: 1699999000,
        purge_lag_hours: 2.5,
        pending_expired_buckets: 3
      },
      details: {
        tarpit: {
          enabled: true,
          metrics: {
            activations: { progressive: 2 }
          }
        }
      }
    });
    assert.equal(monitoring.details.tarpit.enabled, true);
    assert.equal(
      Number(monitoring.details.tarpit.metrics.activations.progressive || 0),
      2
    );
    assert.equal(monitoring.freshness.state, 'fresh');
    assert.equal(monitoring.freshness.last_event_ts, 1700000000);
    assert.equal(monitoring.retention_health.state, 'degraded');
    assert.equal(monitoring.retention_health.retention_hours, 168);

    const suggestions = api.adaptIpRangeSuggestions({
      generated_at: 1700000000,
      hours: 24,
      summary: { suggestions_total: 1, low_risk: 1, medium_risk: 0, high_risk: 0 },
      suggestions: [{
        cidr: '198.51.100.0/24',
        ip_family: 'ipv4',
        bot_evidence_score: '18.5',
        human_evidence_score: 0,
        collateral_risk: 0.02,
        confidence: 0.91,
        risk_band: 'low',
        recommended_action: 'deny_temp',
        recommended_mode: 'enforce',
        evidence_counts: { honeypot: 12 },
        safer_alternatives: ['198.51.100.0/25'],
        guardrail_notes: []
      }]
    });
    assert.equal(suggestions.summary.suggestions_total, 1);
    assert.equal(suggestions.suggestions.length, 1);
    assert.equal(suggestions.suggestions[0].recommended_action, 'deny_temp');

    const delta = api.adaptCursorDelta({
      after_cursor: 'c1',
      window_end_cursor: 'c9',
      next_cursor: 'c3',
      has_more: false,
      overflow: 'none',
      events: [{ cursor: 'c3', event: 'Challenge', ts: 1700000000 }],
      freshness: { state: 'fresh', lag_ms: 120, transport: 'sse' }
    });
    assert.equal(delta.after_cursor, 'c1');
    assert.equal(delta.window_end_cursor, 'c9');
    assert.equal(delta.next_cursor, 'c3');
    assert.equal(delta.events.length, 1);
    assert.equal(delta.freshness.state, 'fresh');
  });
});

test('request failure classifier distinguishes timeout/cancelled/transport/http failures', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const failureModule = await importBrowserModule('dashboard/src/lib/domain/core/request-failure.js');

    const timeoutError = new Error('timed out');
    assert.equal(
      failureModule.classifyRequestFailure(timeoutError, { didTimeout: true }),
      failureModule.REQUEST_FAILURE_CLASSES.timeout
    );

    const abortError = new Error('Request aborted');
    abortError.name = 'AbortError';
    assert.equal(
      failureModule.classifyRequestFailure(abortError),
      failureModule.REQUEST_FAILURE_CLASSES.cancelled
    );

    const httpError = new Error('Service unavailable');
    httpError.status = 503;
    assert.equal(
      failureModule.classifyRequestFailure(httpError),
      failureModule.REQUEST_FAILURE_CLASSES.http
    );

    const transportError = new Error('Network down');
    assert.equal(
      failureModule.classifyRequestFailure(transportError),
      failureModule.REQUEST_FAILURE_CLASSES.transport
    );
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

test('dashboard API client bypasses caches for adversary-sim status reads', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const calls = [];
    const client = apiModule.create({
      getAdminContext: () => ({
        endpoint: 'https://edge.local',
        apikey: 'test-key',
        sessionAuth: true,
        csrfToken: 'csrf-token'
      }),
      request: async (url, init = {}) => {
        calls.push({ url, init });
        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          json: async () => ({
            adversary_sim_enabled: false,
            generation_active: false,
            phase: 'off'
          }),
          text: async () => JSON.stringify({
            adversary_sim_enabled: false,
            generation_active: false,
            phase: 'off'
          })
        };
      }
    });

    await client.getAdversarySimStatus();
    assert.equal(calls.length, 1);
    assert.match(String(calls[0].url), /\/admin\/adversary-sim\/status$/);
    assert.equal(calls[0].init.cache, 'no-store');
  });
});

test('dashboard API client exposes cursor-delta and stream URL helpers for realtime tabs', { concurrency: false }, async () => {
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
          json: async () => ({
            after_cursor: 'c1',
            window_end_cursor: 'c9',
            next_cursor: 'c2',
            has_more: false,
            overflow: 'none',
            events: [],
            active_bans: [],
            freshness: { state: 'fresh', lag_ms: 42 }
          }),
          text: async () =>
            '{"after_cursor":"c1","window_end_cursor":"c9","next_cursor":"c2","has_more":false,"overflow":"none","events":[],"active_bans":[],"freshness":{"state":"fresh","lag_ms":42}}'
        };
      }
    });

    const monitoringDelta = await client.getMonitoringDelta({
      hours: 6,
      limit: 99,
      after_cursor: 'cursor-123'
    });
    const ipBansDelta = await client.getIpBansDelta({
      hours: 3,
      limit: 77,
      after_cursor: 'cursor-abc'
    });
    assert.equal(monitoringDelta.next_cursor, 'c2');
    assert.equal(ipBansDelta.freshness.state, 'fresh');
    assert.match(String(calls[0].url), /\/admin\/monitoring\/delta\?/);
    assert.match(String(calls[0].url), /hours=6/);
    assert.match(String(calls[0].url), /limit=99/);
    assert.match(String(calls[0].url), /after_cursor=cursor-123/);
    assert.match(String(calls[1].url), /\/admin\/ip-bans\/delta\?/);
    assert.match(String(calls[1].url), /after_cursor=cursor-abc/);

    const monitoringStreamUrl = client.getMonitoringStreamUrl({
      hours: 4,
      limit: 55,
      after_cursor: 'resume-token'
    });
    const ipBansStreamUrl = client.getIpBansStreamUrl({
      hours: 2,
      limit: 44,
      after_cursor: 'resume-ban'
    });
    assert.match(monitoringStreamUrl, /^https:\/\/edge\.local\/admin\/monitoring\/stream\?/);
    assert.match(monitoringStreamUrl, /hours=4/);
    assert.match(monitoringStreamUrl, /limit=55/);
    assert.match(monitoringStreamUrl, /after_cursor=resume-token/);
    assert.match(ipBansStreamUrl, /^https:\/\/edge\.local\/admin\/ip-bans\/stream\?/);
    assert.match(ipBansStreamUrl, /after_cursor=resume-ban/);
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
    const telemetryEvents = [];
    const disconnectEvents = [];
    const client = apiModule.create({
      getAdminContext: () => ({ endpoint: 'https://edge.local', apikey: '', sessionAuth: false }),
      onApiError: (error) => {
        errors.push(error);
      },
      onBackendDisconnected: (error) => {
        disconnectEvents.push(error);
      },
      onRequestTelemetry: (event) => {
        telemetryEvents.push(event);
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
    assert.equal(disconnectEvents.length, 1);
    assert.equal(telemetryEvents.length, 1);
    assert.equal(telemetryEvents[0].failureClass, 'timeout');
    assert.equal(telemetryEvents[0].source, 'api-client');
    assert.equal(String(telemetryEvents[0].path || '').includes('/admin/events'), true);
  });
});

test('dashboard API client classifies AbortError as cancelled and does not emit backend disconnect callback', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const telemetryEvents = [];
    const disconnectEvents = [];
    const controller = new AbortController();
    const client = apiModule.create({
      getAdminContext: () => ({ endpoint: 'https://edge.local', apikey: '', sessionAuth: false }),
      onBackendDisconnected: (error) => {
        disconnectEvents.push(error);
      },
      onRequestTelemetry: (event) => {
        telemetryEvents.push(event);
      },
      request: async (_url, init = {}) =>
        new Promise((_resolve, reject) => {
          if (init.signal && typeof init.signal.addEventListener === 'function') {
            init.signal.addEventListener('abort', () => {
              const abortError = new Error('Request aborted by caller');
              abortError.name = 'AbortError';
              reject(abortError);
            }, { once: true });
          }
        })
    });

    const requestPromise = client.getEvents(24, {
      signal: controller.signal,
      telemetry: {
        tab: 'monitoring',
        reason: 'auto-refresh',
        source: 'tab-refresh'
      }
    });
    controller.abort();
    await assert.rejects(() => requestPromise);
    assert.equal(disconnectEvents.length, 0);
    assert.equal(telemetryEvents.length, 1);
    assert.equal(telemetryEvents[0].failureClass, 'cancelled');
    assert.equal(telemetryEvents[0].tab, 'monitoring');
    assert.equal(telemetryEvents[0].reason, 'auto-refresh');
    assert.equal(telemetryEvents[0].source, 'tab-refresh');
  });
});

test('chart runtime adapter lazily loads once and tears down on final release', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adapter = await importBrowserModule('dashboard/src/lib/domain/services/chart-runtime-adapter.js');
    const win = { location: { pathname: '/dashboard/index.html' }, Chart: undefined };
    let loaderCalls = 0;
    const ChartMock = function ChartMock() {};
    const loader = async () => {
      loaderCalls += 1;
      return { Chart: ChartMock };
    };

    const one = await adapter.acquireChartRuntime({ window: win, loader });
    const two = await adapter.acquireChartRuntime({ window: win, loader });

    assert.equal(typeof one, 'function');
    assert.equal(one, two);
    assert.equal(one, ChartMock);
    assert.equal(loaderCalls, 1);

    adapter.releaseChartRuntime({ window: win });
    assert.equal(typeof win.Chart, 'function');
    adapter.releaseChartRuntime({ window: win });
    assert.equal(win.Chart, undefined);
  });
});

test('monitoring chart presets enforce shared palette and stable x-axis layout', { concurrency: false }, async () => {
  const presets = await importBrowserModule('dashboard/src/lib/domain/monitoring-chart-presets.js');
  assert.equal(Array.isArray(presets.MONITORING_CHART_PALETTE), true);
  assert.equal(presets.MONITORING_CHART_PALETTE.length >= 3, true);
  assert.equal(presets.MONITORING_CHART_PALETTE.every((color) => /^hsl\(/.test(String(color))), true);
  assert.equal(presets.MONITORING_TIME_SERIES_FILL.events, presets.MONITORING_CHART_PALETTE[0]);
  assert.equal(presets.MONITORING_TIME_SERIES_FILL.challenge, presets.MONITORING_CHART_PALETTE[1]);
  assert.equal(presets.MONITORING_TIME_SERIES_FILL.pow, presets.MONITORING_CHART_PALETTE[2]);
  assert.equal(presets.MONITORING_RUNTIME_HUES['runtime-dev'], 310);
  assert.equal(presets.MONITORING_RUNTIME_HUES['runtime-prod'], 210);

  const prodPalette = presets.buildMonitoringChartPalette(210);
  assert.equal(Array.isArray(prodPalette), true);
  assert.equal(prodPalette.length, presets.MONITORING_CHART_PALETTE.length);
  assert.equal(/^hsl\(210,/.test(String(prodPalette[0])), true);

  const classListWithProd = {
    contains(className) {
      return className === 'runtime-prod';
    }
  };
  const prodTheme = presets.resolveMonitoringChartTheme({
    documentRef: {
      documentElement: { classList: classListWithProd },
      body: {}
    },
    windowRef: {
      getComputedStyle() {
        return {
          getPropertyValue(name) {
            if (name === '--hue') return '';
            if (name === '--muted-fg') return 'rgb(89, 69, 85)';
            return '';
          }
        };
      }
    }
  });
  assert.equal(prodTheme.hue, 210);
  assert.equal(prodTheme.palette[0], prodPalette[0]);
  assert.equal(prodTheme.legendColor, 'rgb(89, 69, 85)');

  const xAxis = presets.buildMonitoringTimeSeriesXAxis();
  assert.equal(xAxis.ticks.autoSkip, true);
  assert.equal(xAxis.ticks.maxTicksLimit, presets.MONITORING_TIME_SERIES_TICK_MAX);
  assert.equal(xAxis.ticks.minRotation, 0);
  assert.equal(xAxis.ticks.maxRotation, 0);
  assert.equal(xAxis.ticks.autoSkipPadding, presets.MONITORING_TIME_SERIES_TICK_PADDING);
  assert.equal(presets.MONITORING_TIME_SERIES_OMIT_FINAL_TICK_LABEL, true);

  const tickLabel = xAxis.ticks.callback.call(
    { getLabelForValue: () => 'September 29 13:45' },
    0,
    0,
    [{ value: 0 }, { value: 1 }]
  );
  assert.equal(typeof tickLabel, 'string');
  assert.equal(tickLabel.length <= (presets.MONITORING_TIME_SERIES_TICK_MAX_CHARS + 3), true);
  const finalTickLabel = xAxis.ticks.callback.call(
    { getLabelForValue: () => 'September 29 14:00' },
    1,
    1,
    [{ value: 0 }, { value: 1 }]
  );
  assert.equal(finalTickLabel, '');

  const scale = { height: 999 };
  xAxis.afterFit(scale);
  assert.equal(scale.height, presets.MONITORING_TIME_SERIES_AXIS_HEIGHT_PX);

  const countAxisSmall = presets.buildMonitoringCountYAxis([0, 1, 2, 3]);
  assert.equal(countAxisSmall.beginAtZero, true);
  assert.equal(countAxisSmall.ticks.stepSize, 1);
  assert.equal(countAxisSmall.ticks.maxTicksLimit, presets.MONITORING_COUNT_AXIS_TICK_MAX);
  assert.equal(countAxisSmall.ticks.precision, 0);

  const countAxisLarge = presets.buildMonitoringCountYAxis([9233]);
  assert.equal(countAxisLarge.ticks.stepSize > 1, true);
  const estimatedLargeTickCount = Math.floor(9233 / countAxisLarge.ticks.stepSize) + 1;
  assert.equal(estimatedLargeTickCount <= countAxisLarge.ticks.maxTicksLimit, true);
});

test('dashboard state and store contracts remain immutable and bounded with heartbeat-owned connection telemetry', { concurrency: false }, async () => {
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
    store.recordRequestTelemetry({
      requestId: 'req-failure-1',
      path: '/admin/events?hours=24',
      method: 'GET',
      tab: 'monitoring',
      reason: 'auto-refresh',
      source: 'tab-refresh',
      outcome: 'failure',
      failureClass: 'cancelled',
      statusCode: 0,
      aborted: true
    });
    store.recordHeartbeatAttemptStarted({
      requestId: 'hb-1',
      path: '/admin/session',
      method: 'GET',
      reason: 'interval'
    });
    store.recordHeartbeatFailure({
      requestId: 'hb-1',
      path: '/admin/session',
      method: 'GET',
      failureClass: 'timeout',
      error: 'Request timed out after 2500ms'
    });
    store.recordHeartbeatSuccess({
      requestId: 'hb-2',
      path: '/admin/session',
      method: 'GET',
      statusCode: 200
    });

    const telemetry = store.getRuntimeTelemetry();
    assert.equal(telemetry.refresh.fetchLatencyMs.last, 200);
    assert.equal(telemetry.refresh.renderTimingMs.last, 20);
    assert.equal(telemetry.refresh.lastTab, 'monitoring');
    assert.equal(telemetry.refresh.fetchLatencyMs.totalSamples, 2);
    assert.equal(telemetry.refresh.fetchLatencyMs.window.length > 0, true);
    assert.equal(telemetry.connection.state, 'connected');
    assert.equal(telemetry.connection.disconnectThreshold, 3);
    assert.equal(telemetry.connection.lastTransitionReason, 'heartbeat_ok');
    assert.equal(telemetry.connection.lastFailureAt.length > 0, true);
    assert.equal(telemetry.connection.lastSuccessAt.length > 0, true);
    assert.equal(telemetry.heartbeat.lastFailureClass, '');
    assert.equal(telemetry.heartbeat.ignoredCancelledCount, 1);
    assert.equal(telemetry.heartbeat.ignoredNonHeartbeatFailureCount, 1);
    assert.equal(Array.isArray(telemetry.heartbeat.breadcrumbs), true);
    assert.equal(telemetry.heartbeat.breadcrumbs.length >= 2, true);
    assert.equal(telemetry.requests.total >= 1, true);
    assert.equal(telemetry.requests.cancelledCount, 1);
  });
});

test('dashboard store heartbeat failure hysteresis transitions connected -> degraded -> disconnected and ignores cancelled failures', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });

    store.recordHeartbeatSuccess({ requestId: 'hb-start', path: '/admin/session', method: 'GET' });
    assert.equal(store.getRuntimeTelemetry().connection.state, 'connected');

    store.recordHeartbeatFailure({
      requestId: 'hb-fail-1',
      path: '/admin/session',
      method: 'GET',
      failureClass: 'transport',
      error: 'network unreachable'
    });
    assert.equal(store.getRuntimeTelemetry().connection.state, 'degraded');

    store.recordHeartbeatFailure({
      requestId: 'hb-cancelled',
      path: '/admin/session',
      method: 'GET',
      failureClass: 'cancelled',
      error: 'aborted'
    });
    const afterCancelled = store.getRuntimeTelemetry();
    assert.equal(afterCancelled.connection.state, 'degraded');
    assert.equal(afterCancelled.connection.consecutiveFailures, 1);
    assert.equal(afterCancelled.heartbeat.ignoredCancelledCount >= 1, true);

    store.recordHeartbeatFailure({
      requestId: 'hb-fail-2',
      path: '/admin/session',
      method: 'GET',
      failureClass: 'timeout',
      error: 'Request timed out'
    });
    assert.equal(store.getRuntimeTelemetry().connection.state, 'degraded');
    assert.equal(store.getRuntimeTelemetry().connection.consecutiveFailures, 2);

    store.recordHeartbeatFailure({
      requestId: 'hb-fail-3',
      path: '/admin/session',
      method: 'GET',
      failureClass: 'http',
      statusCode: 503,
      error: 'status 503'
    });
    const disconnectedTelemetry = store.getRuntimeTelemetry();
    assert.equal(disconnectedTelemetry.connection.state, 'disconnected');
    assert.equal(disconnectedTelemetry.connection.consecutiveFailures, 3);
  });
});

test('refresh runtime bootstraps monitoring baseline before cursor deltas and keeps manual refresh effective', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const storage = {
      getItem() {
        return null;
      },
      setItem() {},
      removeItem() {}
    };

    const now = 1_700_000_000;
    const callOrder = [];
    let fullFetchCount = 0;
    const deltaCalls = [];
    const buildMonitoringPayload = (reason) => ({
      freshness: {
        state: 'fresh',
        lag_ms: 0,
        last_event_ts: now,
        transport: 'snapshot_poll'
      },
      summary: {
        honeypot: { total_hits: 1, unique_crawlers: 1, top_crawlers: [], top_paths: [] },
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
      details: {
        analytics: { ban_count: 0, test_mode: false, fail_mode: 'open' },
        events: {
          recent_events: [{
            ts: now,
            event: 'Challenge',
            ip: '198.51.100.1',
            reason,
            outcome: 'served',
            admin: 'ops'
          }],
          event_counts: { Challenge: 1 },
          top_ips: [['198.51.100.1', 1]],
          unique_ips: 1
        },
        bans: { bans: [] },
        maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
        cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
        cdp_events: { events: [] }
      }
    });

    const apiClient = {
      async getMonitoring() {
        callOrder.push('monitoring_full');
        const reason = fullFetchCount === 0 ? 'historical-baseline' : 'manual-refresh-full';
        fullFetchCount += 1;
        return buildMonitoringPayload(reason);
      },
      async getMonitoringDelta(params = {}) {
        callOrder.push('monitoring_delta');
        deltaCalls.push({
          limit: Number(params.limit || 0),
          after_cursor: String(params.after_cursor || '')
        });
        if (Number(params.limit || 0) === 1) {
          return {
            after_cursor: '',
            window_end_cursor: 'cursor-1',
            next_cursor: 'cursor-1',
            has_more: false,
            overflow: 'none',
            events: [],
            freshness: { state: 'fresh' }
          };
        }
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'cursor-2',
          next_cursor: 'cursor-2',
          has_more: false,
          overflow: 'none',
          events: [{
            cursor: 'cursor-2',
            ts: now + 1,
            event: 'Challenge',
            ip: '198.51.100.2',
            reason: 'manual-refresh-delta',
            outcome: 'served',
            admin: 'ops'
          }],
          freshness: { state: 'fresh' }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 0, test_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshMonitoringTab('manual');
    assert.equal(fullFetchCount, 1);
    assert.deepEqual(callOrder.slice(0, 2), ['monitoring_full', 'monitoring_delta']);
    assert.equal(deltaCalls.length, 1);
    assert.equal(deltaCalls[0].limit, 1);

    const baselineEvents = (store.getSnapshot('events') || {}).recent_events || [];
    assert.equal(
      baselineEvents.some((entry) => String(entry.reason || '') === 'historical-baseline'),
      true
    );
    const baselineFreshness = store.getSnapshot('monitoringFreshness') || {};
    assert.equal(baselineFreshness.state, 'fresh');
    assert.equal(Number(baselineFreshness.last_event_ts || 0), now);

    await runtime.refreshMonitoringTab('manual-refresh');
    assert.equal(fullFetchCount, 1);
    assert.equal(deltaCalls.length, 2);

    const updatedEvents = (store.getSnapshot('events') || {}).recent_events || [];
    assert.equal(
      updatedEvents.some((entry) => String(entry.reason || '') === 'manual-refresh-delta'),
      true
    );
  });
});

test('monitoring auto-refresh refreshes monitoring snapshots without extra ip-bans side reads', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const storage = {
      getItem() {
        return null;
      },
      setItem() {},
      removeItem() {}
    };

    const now = 1_700_000_200;
    let monitoringCalls = 0;
    let monitoringDeltaSeedCalls = 0;
    let monitoringDeltaCalls = 0;
    let bansCalls = 0;
    let suggestionsCalls = 0;
    let ipBansSeedCalls = 0;
    const apiClient = {
      async getMonitoring(params = {}) {
        monitoringCalls += 1;
        assert.equal(Number(params.limit || 0), 200);
        return {
          freshness: {
            state: 'fresh',
            lag_ms: 0,
            last_event_ts: now,
            transport: 'snapshot_poll'
          },
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
          details: {
            analytics: { ban_count: 1, test_mode: false, fail_mode: 'open' },
            events: {
              recent_events: [{
                ts: now,
                event: 'Challenge',
                ip: '198.51.100.1',
                reason: 'rate',
                outcome: 'challenge',
                admin: null
              }],
              event_counts: { Challenge: 1 },
              top_ips: [['198.51.100.1', 1]],
              unique_ips: 1
            },
            bans: {
              bans: [{
                ip: '198.51.100.55',
                reason: 'rate',
                banned_at: now,
                expires: now + 300
              }]
            },
            maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] },
            cdp: { stats: { total_detections: 0, auto_bans: 0 }, config: {}, fingerprint_stats: {} },
            cdp_events: { events: [] }
          }
        };
      },
      async getMonitoringDelta(params = {}) {
        if (Number(params.limit || 0) === 1) {
          monitoringDeltaSeedCalls += 1;
          return {
            after_cursor: '',
            window_end_cursor: 'monitoring-window-end',
            next_cursor: 'monitoring-window-end',
            has_more: false,
            overflow: 'none',
            events: [],
            freshness: { state: 'fresh', lag_ms: 0 }
          };
        }
        monitoringDeltaCalls += 1;
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'monitoring-window-end',
          next_cursor: 'monitoring-window-end',
          has_more: false,
          overflow: 'none',
          events: [],
          freshness: { state: 'fresh', lag_ms: 0 }
        };
      },
      async getBans() {
        bansCalls += 1;
        return {
          bans: [{
            ip: '198.51.100.55',
            reason: 'rate',
            banned_at: now,
            expires: now + 300
          }]
        };
      },
      async getIpRangeSuggestions() {
        suggestionsCalls += 1;
        return {
          summary: {
            suggestions_total: 1,
            low_risk: 1,
            medium_risk: 0,
            high_risk: 0
          },
          suggestions: [{
            cidr: '198.51.100.0/24',
            ip_family: 'ipv4',
            bot_evidence_score: 18.1,
            human_evidence_score: 0,
            collateral_risk: 0.05,
            confidence: 0.91,
            risk_band: 'low',
            recommended_action: 'deny_temp',
            recommended_mode: 'enforce',
            evidence_counts: { honeypot: 12 },
            safer_alternatives: [],
            guardrail_notes: []
          }]
        };
      },
      async getIpBansDelta(params = {}) {
        if (Number(params.limit || 0) === 1) {
          ipBansSeedCalls += 1;
        }
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'ip-bans-window-end',
          next_cursor: 'ip-bans-window-end',
          has_more: false,
          overflow: 'none',
          active_bans: [],
          freshness: {
            state: 'fresh',
            lag_ms: 0,
            last_event_ts: now,
            transport: 'cursor_delta_poll'
          }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 1, test_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshDashboardForTab('monitoring', 'auto-refresh');

    assert.equal(monitoringCalls, 1);
    assert.equal(monitoringDeltaSeedCalls, 1);
    assert.equal(monitoringDeltaCalls, 0);
    await runtime.refreshDashboardForTab('monitoring', 'auto-refresh');
    assert.equal(monitoringCalls, 1);
    assert.equal(monitoringDeltaCalls, 1);
    assert.equal(bansCalls, 0);
    assert.equal(suggestionsCalls, 0);
    assert.equal(ipBansSeedCalls, 0);
    const bansSnapshot = store.getSnapshot('bans') || {};
    assert.equal(Array.isArray(bansSnapshot.bans), true);
    assert.equal(bansSnapshot.bans.length, 1);
  });
});

test('refresh runtime seeds monitoring cursor from window end instead of replaying oldest page cursor', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const storage = {
      getItem() {
        return null;
      },
      setItem() {},
      removeItem() {}
    };

    const deltaCalls = [];
    const apiClient = {
      async getMonitoring() {
        return {
          summary: {},
          details: {
            analytics: { ban_count: 0, test_mode: false, fail_mode: 'open' },
            events: { recent_events: [] },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getMonitoringDelta(params = {}) {
        deltaCalls.push({
          limit: Number(params.limit || 0),
          after_cursor: String(params.after_cursor || '')
        });
        if (Number(params.limit || 0) === 1) {
          return {
            after_cursor: '',
            window_end_cursor: 'cursor-window-end',
            next_cursor: 'cursor-oldest-page',
            has_more: true,
            overflow: 'limit_exceeded',
            events: [],
            freshness: { state: 'fresh' }
          };
        }
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'cursor-window-end',
          next_cursor: 'cursor-window-end',
          has_more: false,
          overflow: 'none',
          events: [],
          freshness: { state: 'fresh' }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 0, test_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshMonitoringTab('manual');
    await runtime.refreshMonitoringTab('manual');

    assert.equal(deltaCalls.length, 2);
    assert.equal(deltaCalls[0].limit, 1);
    assert.equal(deltaCalls[1].after_cursor, 'cursor-window-end');
  });
});

test('manual refresh bypasses monitoring cache while passive reasons honor cached baseline', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const storageState = new Map();
    const storage = {
      getItem(key) {
        return storageState.has(key) ? storageState.get(key) : null;
      },
      setItem(key, value) {
        storageState.set(key, String(value));
      },
      removeItem(key) {
        storageState.delete(key);
      }
    };

    const now = 1_700_000_500;
    const compactedBans = Array.from({ length: 100 }, (_, index) => ({
      ip: `198.51.100.${index}`,
      reason: 'honeypot',
      banned_at: now - index,
      expires: now + 3600
    }));
    storage.setItem(
      'shuma_dashboard_cache_monitoring_v1',
      JSON.stringify({
        cachedAt: Date.now(),
        payload: {
          monitoring: {
            summary: {},
            details: {
              analytics: { ban_count: 100, test_mode: false, fail_mode: 'open' },
              events: { recent_events: [] },
              bans: { bans: compactedBans },
              maze: {},
              cdp: {},
              cdp_events: { events: [] }
            }
          }
        }
      })
    );

    let fullFetchCount = 0;
    const apiClient = {
      async getMonitoring() {
        fullFetchCount += 1;
        return {
          summary: {},
          details: {
            analytics: { ban_count: 164, test_mode: false, fail_mode: 'open' },
            events: { recent_events: [] },
            bans: { bans: compactedBans },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getMonitoringDelta(params = {}) {
        if (Number(params.limit || 0) === 1) {
          return {
            after_cursor: '',
            window_end_cursor: 'cursor-1',
            next_cursor: 'cursor-1',
            has_more: false,
            overflow: 'none',
            events: [],
            freshness: { state: 'fresh' }
          };
        }
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'cursor-1',
          next_cursor: 'cursor-1',
          has_more: false,
          overflow: 'limit_exceeded',
          events: [],
          freshness: { state: 'fresh' }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: (_configSnapshot, analyticsResponse = {}) => ({
        ban_count: Number(analyticsResponse.ban_count || 0),
        test_mode: analyticsResponse.test_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage
    });

    await runtime.refreshMonitoringTab('session-restored');
    assert.equal(fullFetchCount, 0);
    assert.equal(
      Number((store.getSnapshot('analytics') || {}).ban_count || 0),
      100
    );

    await runtime.refreshMonitoringTab('manual-refresh');
    assert.equal(fullFetchCount, 1);
    assert.equal(
      Number((store.getSnapshot('analytics') || {}).ban_count || 0),
      164
    );
  });
});

test('monitoring bootstrap does not wait for cursor seeding before config-backed controls can become ready', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    let resolveSeedCursor;
    const seedCursorPromise = new Promise((resolve) => {
      resolveSeedCursor = resolve;
    });
    let configFetchCount = 0;
    let monitoringFetchCount = 0;
    let seedCursorCount = 0;

    const apiClient = {
      async getMonitoring() {
        monitoringFetchCount += 1;
        return {
          summary: {},
          freshness: { state: 'fresh', transport: 'snapshot_poll' },
          details: {
            analytics: { ban_count: 0, test_mode: false, fail_mode: 'open' },
            events: { recent_events: [] },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getConfig() {
        configFetchCount += 1;
        return {
          admin_config_write_enabled: true,
          runtime_environment: 'runtime-prod'
        };
      },
      async getMonitoringDelta(params = {}) {
        if (Number(params.limit || 0) === 1) {
          seedCursorCount += 1;
          return seedCursorPromise;
        }
        return {
          after_cursor: '',
          window_end_cursor: '',
          next_cursor: '',
          has_more: false,
          overflow: 'none',
          events: [],
          freshness: { state: 'fresh' }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: (_configSnapshot, analyticsResponse = {}) => ({
        ban_count: Number(analyticsResponse.ban_count || 0),
        test_mode: analyticsResponse.test_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage: null
    });

    const refreshCompleted = await Promise.race([
      runtime.refreshDashboardForTab('monitoring', 'session-restored').then(() => true),
      new Promise((resolve) => setTimeout(() => resolve(false), 50))
    ]);

    assert.equal(refreshCompleted, true);
    assert.equal(monitoringFetchCount, 1);
    assert.equal(configFetchCount, 1);
    assert.equal(seedCursorCount, 1);
    assert.equal(
      (store.getSnapshot('config') || {}).admin_config_write_enabled,
      true
    );

    resolveSeedCursor({
      after_cursor: '',
      window_end_cursor: 'cursor-1',
      next_cursor: 'cursor-1',
      has_more: false,
      overflow: 'none',
      events: [],
      freshness: { state: 'fresh' }
    });
  });
});

test('monitoring refresh recovers cleanly after transient failure without synthetic freshness overwrite', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    store.setSnapshot('monitoringFreshness', {
      state: 'stale',
      lag_ms: 212000,
      last_event_ts: 1_700_000_000,
      slow_consumer_lag_state: 'normal',
      overflow: 'none',
      transport: 'snapshot_poll'
    });

    const storageState = new Map();
    const storage = {
      getItem(key) {
        return storageState.has(key) ? storageState.get(key) : null;
      },
      setItem(key, value) {
        storageState.set(key, String(value));
      },
      removeItem(key) {
        storageState.delete(key);
      }
    };

    let shouldFail = true;
    const apiClient = {
      async getConfig() {
        return {};
      },
      async getBans() {
        return { bans: [] };
      },
      async getIpRangeSuggestions() {
        return {
          summary: {
            suggestions_total: 0,
            low_risk: 0,
            medium_risk: 0,
            high_risk: 0
          },
          suggestions: []
        };
      },
      async getMonitoring() {
        if (shouldFail) {
          shouldFail = false;
          throw new Error('transient monitoring read failure');
        }
        return {
          freshness: {
            state: 'fresh',
            lag_ms: 0,
            last_event_ts: 1_700_000_050,
            slow_consumer_lag_state: 'normal',
            overflow: 'none',
            transport: 'snapshot_poll'
          },
          summary: {},
          details: {
            analytics: { ban_count: 0, test_mode: false, fail_mode: 'open' },
            events: {
              recent_events: [{
                ts: 1_700_000_050,
                event: 'Challenge',
                ip: '198.51.100.9',
                reason: 'recovered-refresh',
                outcome: 'served'
              }]
            },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getMonitoringDelta(params = {}) {
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'cursor-after-recovery',
          next_cursor: 'cursor-after-recovery',
          has_more: false,
          overflow: 'none',
          events: [],
          freshness: {
            state: 'fresh',
            lag_ms: 0,
            last_event_ts: 1_700_000_050,
            slow_consumer_lag_state: 'normal',
            overflow: 'none',
            transport: 'cursor_delta_poll'
          }
        };
      },
      async getIpBansDelta(params = {}) {
        return {
          after_cursor: String(params.after_cursor || ''),
          window_end_cursor: 'ip-bans-cursor-after-recovery',
          next_cursor: 'ip-bans-cursor-after-recovery',
          has_more: false,
          overflow: 'none',
          active_bans: [],
          freshness: {
            state: 'fresh',
            lag_ms: 0,
            last_event_ts: 1_700_000_050,
            slow_consumer_lag_state: 'normal',
            overflow: 'none',
            transport: 'cursor_delta_poll'
          }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 0, test_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshDashboardForTab('monitoring', 'manual-refresh');
    const failedStatus = store.getState().tabStatus.monitoring;
    assert.equal(failedStatus.loading, false);
    assert.equal(Boolean(String(failedStatus.error || '').trim()), true);
    assert.equal(
      Number((store.getSnapshot('monitoringFreshness') || {}).last_event_ts || 0),
      1_700_000_000
    );

    await runtime.refreshDashboardForTab('monitoring', 'manual-refresh');
    const recoveredStatus = store.getState().tabStatus.monitoring;
    assert.equal(recoveredStatus.loading, false);
    assert.equal(String(recoveredStatus.error || ''), '');
    assert.equal(
      Number((store.getSnapshot('monitoringFreshness') || {}).last_event_ts || 0),
      1_700_000_050
    );
    assert.equal(
      ((store.getSnapshot('events') || {}).recent_events || []).some((row) => row.reason === 'recovered-refresh'),
      true
    );
  });
});

test('monitoring view model and status module remain pure snapshot transforms', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const monitoringModelModule = await importBrowserModule('dashboard/src/lib/components/dashboard/monitoring-view-model.js');
    const monitoringNormalizers = await importBrowserModule('dashboard/src/lib/domain/monitoring-normalizers.js');
    const ipRangePolicyModule = await importBrowserModule('dashboard/src/lib/domain/ip-range-policy.js');
    const statusModule = await importBrowserModule('dashboard/src/lib/domain/status.js');

    const summary = monitoringModelModule.deriveMonitoringSummaryViewModel({
      shadow: {
        total_actions: 9,
        pass_through_total: 24,
        actions: { challenge: 5, block: 4 }
      },
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
    assert.equal(summary.shadow.totalActions, '9');
    assert.equal(summary.shadow.passThroughTotal, '24');
    assert.equal(summary.shadow.topAction.value, 'Would Challenge (5)');
    assert.equal(summary.shadow.actions[0]?.label, 'Would Challenge');
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
    const tarpitSummary = monitoringModelModule.deriveTarpitViewModel({
      enabled: true,
      active: {
        top_buckets: [{ bucket: '198.51.100.0/24', active: 6 }]
      },
      metrics: {
        activations: { progressive: 11 },
        progress_outcomes: { advanced: 7 },
        budget_outcomes: { fallback_maze: 2, fallback_block: 1 },
        escalation_outcomes: { short_ban: 3, block: 1 }
      }
    });
    assert.equal(tarpitSummary.enabled, true);
    assert.equal(tarpitSummary.activationsProgressive, '11');
    assert.equal(tarpitSummary.progressAdvanced, '7');
    assert.equal(tarpitSummary.topActiveBucket.value, '198.51.100.0/24');
    assert.equal(
      tarpitSummary.budgetOutcomes.some((row) => row[0] === 'fallback_maze' && Number(row[1]) === 2),
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
      'source=custom source_id=manual-block action=forbidden_403 matched_cidr=203.0.113.0/24 taxonomy[level=L11 action=A_DENY_HARD detection=D_IP_RANGE_FORBIDDEN signals=S_IP_RANGE_CUSTOM]'
    );
    assert.equal(parsedOutcome.source, 'custom');
    assert.equal(parsedOutcome.sourceId, 'manual-block');
    assert.equal(parsedOutcome.action, 'forbidden_403');
    assert.equal(parsedOutcome.detection, 'D_IP_RANGE_FORBIDDEN');
    assert.deepEqual(toPlain(parsedOutcome.signals), ['S_IP_RANGE_CUSTOM']);

    const ipRangeSummary = monitoringModelModule.deriveIpRangeMonitoringViewModel([
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_forbidden',
        outcome: 'source=custom source_id=manual-block action=forbidden_403 matched_cidr=203.0.113.0/24 taxonomy[level=L11 action=A_DENY_HARD detection=D_IP_RANGE_FORBIDDEN signals=S_IP_RANGE_CUSTOM]'
      },
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_maze_fallback_block',
        outcome: 'source=custom source_id=manual-bad-range action=maze matched_cidr=198.51.100.0/24 taxonomy[level=L10 action=A_DENY_TEMP detection=D_IP_RANGE_MAZE signals=S_IP_RANGE_CUSTOM]'
      }
    ], {
      ip_range_policy_mode: 'enforce',
      ip_range_emergency_allowlist: ['198.51.100.7/32'],
      ip_range_custom_rules: [{ id: 'manual-bad-range', enabled: true }]
    });
    assert.equal(ipRangeSummary.totalMatches, 2);
    assert.equal(ipRangeSummary.mode, 'enforce');
    assert.equal(
      ipRangeSummary.actions.some(([label, count]) => label === 'forbidden_403' && Number(count) === 1),
      true
    );
    assert.equal(ipRangeSummary.catalog.customRuleCount, 1);

    const gc10Events = [
      {
        ts: 1710000000,
        event: 'challenge',
        ip: '198.51.100.10',
        reason: 'challenge_required',
        outcome: 'challenge',
        scenario_id: 'challenge_puzzle_fail_maze',
        sim_run_id: 'run-1',
        sim_lane: 'browser_realistic',
        sim_profile: 'full_coverage',
        is_simulation: true
      },
      {
        ts: 1710000005,
        event: 'challenge',
        ip: '198.51.100.10',
        reason: 'shadow_mode_preview',
        outcome: 'served',
        execution_mode: 'shadow',
        shadow_source: 'test_mode',
        intended_action: 'challenge',
        enforcement_applied: false,
        sim_run_id: 'run-1',
        sim_lane: 'browser_realistic',
        sim_profile: 'full_coverage',
        is_simulation: true
      },
      {
        ts: 1710000010,
        event: 'ban',
        ip: '198.51.100.10',
        reason: 'tarpit escalation',
        outcome: 'deny_temp',
        sim_run_id: 'run-1',
        sim_lane: 'browser_realistic',
        sim_profile: 'full_coverage',
        is_simulation: true
      },
      {
        ts: 1710000020,
        event: 'pow',
        ip: '203.0.113.77',
        reason: 'scenario=pow_invalid_proof',
        outcome: 'failure',
        sim_run_id: 'run-2',
        sim_lane: 'crawler',
        sim_profile: 'fast_smoke',
        is_simulation: true
      },
      {
        ts: 1710000030,
        event: 'config_patch',
        ip: '127.0.0.1',
        reason: 'manual_threshold_update',
        outcome: 'allow',
        admin: 'ops'
      }
    ];
    const gc10FilterOptions = monitoringModelModule.deriveRecentEventFilterOptions(gc10Events);
    assert.equal(gc10FilterOptions.origins.some((row) => row.value === 'sim'), true);
    assert.equal(gc10FilterOptions.origins.some((row) => row.value === 'manual'), true);
    assert.equal(gc10FilterOptions.modes.some((row) => row.value === 'shadow'), true);
    assert.equal(gc10FilterOptions.modes.some((row) => row.value === 'enforced'), true);
    assert.equal(gc10FilterOptions.lanes.some((row) => row.value === 'browser_realistic'), true);
    assert.equal(gc10FilterOptions.lanes.some((row) => row.value === 'crawler'), true);
    assert.equal(gc10FilterOptions.scenarios.some((row) => row.value === 'challenge_puzzle_fail_maze'), true);

    const gc10FilteredRows = monitoringModelModule.filterRecentEvents(gc10Events, {
      origin: 'sim',
      mode: 'shadow',
      lane: 'browser_realistic',
      defense: 'challenge',
      outcome: 'would_challenge'
    });
    assert.equal(gc10FilteredRows.length, 1);
    assert.equal(gc10FilteredRows[0].sim_run_id, 'run-1');
    const shadowDisplay = monitoringModelModule.deriveMonitoringEventDisplay(gc10FilteredRows[0]);
    assert.equal(shadowDisplay.executionModeLabel, 'Shadow');
    assert.equal(shadowDisplay.outcome, 'Would Challenge');
    assert.equal(shadowDisplay.event, 'Puzzle');

    const defenseTrendRows = monitoringModelModule.deriveDefenseTrendRows(gc10Events);
    const challengeTrend = defenseTrendRows.find((row) => row.defense === 'challenge');
    const tarpitTrend = defenseTrendRows.find((row) => row.defense === 'tarpit');
    assert.equal(Boolean(challengeTrend), true);
    assert.equal(challengeTrend?.triggerCount, 2);
    assert.equal(challengeTrend?.label, 'Puzzle');
    assert.equal(challengeTrend?.escalationCount, 2);
    assert.equal(challengeTrend?.hasOutcomeBreakdown, true);
    assert.equal(challengeTrend?.sourceRows.some((row) => row.source === 'sim' && row.count === 2), true);
    assert.equal(challengeTrend?.modeRows.some((row) => row.mode === 'shadow' && row.count === 1), true);
    assert.equal(challengeTrend?.modeRows.some((row) => row.mode === 'enforced' && row.count === 1), true);
    assert.equal(Boolean(tarpitTrend), true);
    assert.equal(tarpitTrend?.triggerCount, 1);
    assert.equal(tarpitTrend?.hasOutcomeBreakdown, false);
    assert.equal(tarpitTrend?.passCount, 0);
    assert.equal(tarpitTrend?.failCount, 0);
    assert.equal(tarpitTrend?.escalationCount, 0);
    const defenseTrendAccumulator = monitoringModelModule.createDefenseTrendAccumulator();
    monitoringModelModule.appendDefenseTrendEvent(defenseTrendAccumulator, gc10Events[0]);
    monitoringModelModule.appendDefenseTrendEvent(defenseTrendAccumulator, gc10Events[1]);
    let accumulatedDefenseRows =
      monitoringModelModule.deriveDefenseTrendRowsFromAccumulator(defenseTrendAccumulator);
    let accumulatedChallenge = accumulatedDefenseRows.find((row) => row.defense === 'challenge');
    assert.equal(accumulatedChallenge?.triggerCount, 2);
    monitoringModelModule.appendDefenseTrendEvent(defenseTrendAccumulator, gc10Events[3]);
    accumulatedDefenseRows =
      monitoringModelModule.deriveDefenseTrendRowsFromAccumulator(defenseTrendAccumulator);
    accumulatedChallenge = accumulatedDefenseRows.find((row) => row.defense === 'challenge');
    const accumulatedPow = accumulatedDefenseRows.find((row) => row.defense === 'pow');
    assert.equal(accumulatedChallenge?.triggerCount, 2);
    assert.equal(accumulatedPow?.triggerCount, 1);

    const runSummary = monitoringModelModule.deriveAdversaryRunRows(gc10Events, [
      { ip: '198.51.100.200' },
      { ip: '203.0.113.200' }
    ]);
    const runOne = runSummary.runRows.find((row) => row.runId === 'run-1');
    assert.equal(Boolean(runOne), true);
    assert.equal(runOne?.monitoringEventCount, 3);
    assert.equal(runOne?.banOutcomeCount, 1);
    assert.equal(runSummary.activeBanCount, 2);

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
    const bucketTs = 1710000000 * 1000;
    const hourLabel = monitoringNormalizers.formatBucketLabel('hour', bucketTs);
    const dayLabel = monitoringNormalizers.formatBucketLabel('day', bucketTs);
    assert.equal(dayLabel, hourLabel);

    const configSnapshot = {
      kv_store_fail_open: true,
      test_mode: false,
      runtime_environment: 'runtime-prod',
      gateway_deployment_profile: 'shared-server',
      local_prod_direct_mode: true,
      admin_config_write_enabled: false,
      pow_enabled: true,
      not_a_bot_enabled: true,
      not_a_bot_risk_threshold: 2,
      challenge_puzzle_enabled: true,
      challenge_puzzle_transform_count: 6,
      challenge_puzzle_risk_threshold: 3,
      ip_range_policy_mode: 'advisory',
      ip_range_emergency_allowlist: ['198.51.100.0/24'],
      ip_range_custom_rules: [{ id: 'custom-1', enabled: true }],
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

    const statusItems = statusModule.buildFeatureStatusItems(derived, {
      statusOperationalSnapshot: {
        freshness: {
          state: 'degraded',
          lag_ms: 900,
          last_event_ts: 1700000000
        },
        retention_health: {
          state: 'degraded',
          retention_hours: 168,
          oldest_retained_ts: 1699996400,
          purge_lag_hours: 2.5,
          pending_expired_buckets: 3,
          last_purge_success_ts: 1699998200,
          last_purge_error: ''
        }
      }
    });
    const stripHtml = (value) => String(value || '').replace(/<[^>]+>/g, '');
    const challengePuzzleItem = statusItems.find((item) => stripHtml(item.title) === 'Challenge Puzzle');
    const challengeNotABotItem = statusItems.find((item) => stripHtml(item.title) === 'Challenge Not-A-Bot');
    const tarpitItem = statusItems.find((item) => stripHtml(item.title) === 'Tarpit');
    const ipRangeItem = statusItems.find((item) => stripHtml(item.title) === 'IP Range Policy');
    const runtimePostureItem = statusItems.find((item) => stripHtml(item.title) === 'Runtime and Deployment Posture');
    const adminWritePostureItem = statusItems.find((item) => stripHtml(item.title) === 'Admin Config Write Posture');
    const retentionFreshnessItem = statusItems.find((item) => stripHtml(item.title) === 'Retention and Freshness Health');
    const testModeItem = statusItems.find((item) => stripHtml(item.title) === 'Test Mode');
    assert.equal(Boolean(challengePuzzleItem), true);
    assert.equal(Boolean(challengeNotABotItem), true);
    assert.equal(Boolean(tarpitItem), true);
    assert.equal(Boolean(ipRangeItem), true);
    assert.equal(Boolean(runtimePostureItem), true);
    assert.equal(Boolean(adminWritePostureItem), true);
    assert.equal(Boolean(retentionFreshnessItem), true);
    assert.equal(Boolean(testModeItem), false);
    assert.equal(challengePuzzleItem?.status, 'ENABLED');
    assert.equal(challengeNotABotItem?.status, 'ENABLED');
    assert.equal(tarpitItem?.status, 'ENABLED');
    assert.equal(ipRangeItem?.status, 'LOGGING-ONLY');
    assert.equal(runtimePostureItem?.status, 'RUNTIME-PROD / LOCAL-DIRECT');
    assert.equal(adminWritePostureItem?.status, 'DISABLED');
    assert.equal(retentionFreshnessItem?.status, 'DEGRADED');
    assert.match(
      String(runtimePostureItem?.description || ''),
      /https:\/\/github\.com\/atomless\/Shuma-Gorath\/blob\/main\/docs\/quick-reference\.md#runtime-and-deployment-posture-matrix/
    );
    assert.equal(statusItems.some((item) => stripHtml(item.title) === 'Challenge'), false);
  });
});

test('quick reference documents the runtime and deployment posture matrix', () => {
  const source = fs.readFileSync(path.resolve(__dirname, '..', 'docs', 'quick-reference.md'), 'utf8');
  assert.match(source, /## 🐙 Runtime and Deployment Posture Matrix/);
  assert.match(source, /\| Posture \| `SHUMA_RUNTIME_ENV` \| `SHUMA_DEBUG_HEADERS` \| `SHUMA_ADMIN_IP_ALLOWLIST` \| `SHUMA_ENFORCE_HTTPS` \| `SHUMA_GATEWAY_UPSTREAM_ORIGIN` \| `SHUMA_LOCAL_PROD_DIRECT_MODE` \|/);
  assert.match(source, /\| `make dev` \| `runtime-dev` \| `true` \| empty by default \| `false` by default \| not required \| `false` \(normally\) \|/);
  assert.match(source, /\| `make dev-prod` \| `runtime-prod` \| `false` \| empty by default \| `false` by default \| not required \(local-direct\) \| `true` \|/);
  assert.match(source, /\| deployed production \| `runtime-prod` \| `false` \| required and must be narrow \| `true` \| required \| `false` \|/);
});

test('status refresh hydrates monitoring retention/freshness snapshot without monitoring-tab bootstrap', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'status' });
    let configCalls = 0;
    let monitoringCalls = 0;
    const apiClient = {
      async getConfig() {
        configCalls += 1;
        return {
          runtime_environment: 'runtime-prod',
          gateway_deployment_profile: 'shared-server',
          local_prod_direct_mode: false,
          admin_config_write_enabled: true,
          kv_store_fail_open: true
        };
      },
      async getMonitoring(params = {}) {
        monitoringCalls += 1;
        assert.equal(Number(params.limit || 0), 1);
        return {
          freshness: {
            state: 'fresh',
            lag_ms: 125,
            last_event_ts: 1_700_000_000,
            transport: 'snapshot_poll'
          },
          retention_health: {
            state: 'healthy',
            retention_hours: 168,
            oldest_retained_ts: 1_699_999_000,
            purge_lag_hours: 0,
            pending_expired_buckets: 0,
            last_purge_success_ts: 1_700_000_000,
            last_purge_error: ''
          },
          summary: {},
          details: {
            analytics: { ban_count: 0, test_mode: false, fail_mode: 'open' },
            events: { recent_events: [] },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 0, test_mode: false, fail_mode: 'open' })
    });

    await runtime.refreshDashboardForTab('status', 'manual-refresh');

    assert.equal(configCalls, 1);
    assert.equal(monitoringCalls, 1);
    assert.equal((store.getSnapshot('monitoring') || {}).retention_health?.state, 'healthy');
    assert.equal((store.getSnapshot('monitoringFreshness') || {}).state, 'fresh');
  });
});

test('dashboard class runtime keeps exactly one environment class on html and adversary-sim state on body', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const bodyClassModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-body-classes.js');

    const defaultState = bodyClassModule.deriveDashboardBodyClassState({});
    assert.deepEqual(toPlain(defaultState), {
      runtimeClass: '',
      adversarySimEnabled: false,
      connectionState: 'disconnected'
    });

    const explicitDevState = bodyClassModule.deriveDashboardBodyClassState({
      runtime_environment: 'runtime-dev',
      adversary_sim_enabled: true
    }, {
      backendConnectionState: 'connected'
    });
    assert.deepEqual(toPlain(explicitDevState), {
      runtimeClass: 'runtime-dev',
      adversarySimEnabled: true,
      connectionState: 'connected'
    });

    const hintedRuntimeState = bodyClassModule.deriveDashboardBodyClassState({}, {
      runtimeClassHint: 'runtime-dev'
    });
    assert.equal(hintedRuntimeState.runtimeClass, 'runtime-dev');
    const invalidHintedRuntimeState = bodyClassModule.deriveDashboardBodyClassState({}, {
      runtimeClassHint: 'runtime-unknown'
    });
    assert.equal(invalidHintedRuntimeState.runtimeClass, '');

    const classList = createMutableClassList(['runtime-prod', 'adversary-sim', 'connected']);
    const rootClassList = createMutableClassList(['runtime-prod', 'adversary-sim', 'connected']);
    const doc = {
      body: {
        classList
      },
      documentElement: {
        classList: rootClassList
      }
    };

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: explicitDevState.runtimeClass,
      adversarySimEnabled: explicitDevState.adversarySimEnabled,
      connectionState: explicitDevState.connectionState
    });
    assert.equal(classList.contains('runtime-dev'), false);
    assert.equal(classList.contains('runtime-prod'), false);
    assert.equal(classList.contains('adversary-sim'), true);
    assert.equal(classList.contains('connected'), false);
    assert.equal(classList.contains('degraded'), false);
    assert.equal(classList.contains('disconnected'), false);
    assert.equal(rootClassList.contains('runtime-dev'), true);
    assert.equal(rootClassList.contains('runtime-prod'), false);
    assert.equal(rootClassList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('connected'), true);
    assert.equal(rootClassList.contains('degraded'), false);
    assert.equal(rootClassList.contains('disconnected'), false);

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-dev',
      adversarySimEnabled: false,
      connectionState: 'degraded'
    });
    assert.equal(rootClassList.contains('connected'), false);
    assert.equal(rootClassList.contains('degraded'), true);
    assert.equal(rootClassList.contains('disconnected'), false);

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-prod',
      adversarySimEnabled: false,
      connectionState: 'disconnected'
    });
    assert.equal(classList.contains('runtime-dev'), false);
    assert.equal(classList.contains('runtime-prod'), false);
    assert.equal(classList.contains('adversary-sim'), false);
    assert.equal(classList.contains('connected'), false);
    assert.equal(classList.contains('degraded'), false);
    assert.equal(classList.contains('disconnected'), false);
    assert.equal(rootClassList.contains('runtime-dev'), false);
    assert.equal(rootClassList.contains('runtime-prod'), true);
    assert.equal(rootClassList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('connected'), false);
    assert.equal(rootClassList.contains('degraded'), false);
    assert.equal(rootClassList.contains('disconnected'), true);
    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: '',
      adversarySimEnabled: false,
      connectionState: 'degraded'
    });
    assert.equal(rootClassList.contains('runtime-dev'), false);
    assert.equal(rootClassList.contains('runtime-prod'), false);
    assert.equal(rootClassList.contains('degraded'), false);
    assert.equal(rootClassList.contains('connected'), false);
    assert.equal(rootClassList.contains('disconnected'), true);

    bodyClassModule.clearDashboardBodyClasses(doc);
    assert.equal(classList.contains('runtime-dev'), false);
    assert.equal(classList.contains('runtime-prod'), false);
    assert.equal(classList.contains('adversary-sim'), false);
    assert.equal(classList.contains('connected'), false);
    assert.equal(classList.contains('degraded'), false);
    assert.equal(classList.contains('disconnected'), false);
    assert.equal(rootClassList.contains('runtime-dev'), false);
    assert.equal(rootClassList.contains('runtime-prod'), false);
    assert.equal(rootClassList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('connected'), false);
    assert.equal(rootClassList.contains('degraded'), false);
    assert.equal(rootClassList.contains('disconnected'), false);
  });
});

test('dashboard class sync is mutation-stable when target runtime/connection state is unchanged', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const bodyClassModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-body-classes.js');
    const rootClassList = createRecordingClassList(['runtime-prod', 'connected']);
    const bodyClassList = createRecordingClassList([]);
    const doc = {
      body: { classList: bodyClassList },
      documentElement: { classList: rootClassList }
    };

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-prod',
      adversarySimEnabled: false,
      connectionState: 'connected'
    });
    assert.equal(rootClassList.operationCount(), 0);
    assert.equal(bodyClassList.operationCount(), 0);

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-prod',
      adversarySimEnabled: false,
      connectionState: 'connected'
    });
    assert.equal(rootClassList.operationCount(), 0);
    assert.equal(bodyClassList.operationCount(), 0);

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-dev',
      adversarySimEnabled: false,
      connectionState: 'connected'
    });
    assert.equal(rootClassList.contains('runtime-dev'), true);
    assert.equal(rootClassList.contains('runtime-prod'), false);
    assert.equal(rootClassList.operationSnapshot().includes('remove:runtime-prod'), true);
    assert.equal(rootClassList.operationSnapshot().includes('add:runtime-dev'), true);
  });
});

test('dashboard adversary-sim runtime normalizes orchestration status', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');
    const normalized = adversaryModule.normalizeAdversarySimStatus({
      runtime_environment: 'runtime-dev',
      adversary_sim_available: true,
      adversary_sim_enabled: true,
      generation_active: true,
      historical_data_visible: true,
      history_retention: {
        retention_hours: 168,
        cleanup_supported: true,
        cleanup_command: 'make telemetry-clean'
      },
      phase: 'running',
      run_id: 'simrun-123',
      started_at: 1000,
      ends_at: 1180,
      duration_seconds: 180,
      remaining_seconds: 120,
      active_run_count: 1,
      active_lane_count: 2,
      supervisor: {
        owner: 'backend_autonomous_supervisor',
        cadence_seconds: 1,
        max_catchup_ticks_per_invocation: 8,
        heartbeat_active: true,
        worker_active: true,
        last_heartbeat_at: 1100,
        idle_seconds: 0,
        off_state_inert: false,
        trigger_surface: 'internal_beat_endpoint'
      },
      generation_diagnostics: {
        health: 'ok',
        reason: 'traffic_observed',
        recommended_action: 'No action required; simulation traffic is being generated.',
        generated_tick_count: 3,
        generated_request_count: 12,
        last_generated_at: 1100,
        last_generation_error: ''
      }
    });

    assert.equal(normalized.runtimeEnvironment, 'runtime-dev');
    assert.equal(normalized.available, true);
    assert.equal(normalized.enabled, true);
    assert.equal(normalized.generationActive, true);
    assert.equal(normalized.historicalDataVisible, true);
    assert.equal(normalized.historyRetentionHours, 168);
    assert.equal(normalized.historyCleanupSupported, true);
    assert.equal(normalized.historyCleanupCommand, 'make telemetry-clean');
    assert.equal(normalized.phase, 'running');
    assert.equal(normalized.runId, 'simrun-123');
    assert.equal(normalized.activeRunCount, 1);
    assert.equal(normalized.activeLaneCount, 2);
    assert.equal(normalized.supervisor.owner, 'backend_autonomous_supervisor');
    assert.equal(normalized.supervisor.cadenceSeconds, 1);
    assert.equal(normalized.supervisor.heartbeatActive, true);
    assert.equal(normalized.generationDiagnostics.health, 'ok');
    assert.equal(normalized.generationDiagnostics.generatedTickCount, 3);
    assert.equal(normalized.generationDiagnostics.generatedRequestCount, 12);

    const renormalized = adversaryModule.normalizeAdversarySimStatus(normalized);
    assert.equal(renormalized.enabled, true);
    assert.equal(renormalized.available, true);
    assert.equal(renormalized.durationSeconds, 180);
    assert.equal(renormalized.generationDiagnostics.health, 'ok');
  });
});

test('dashboard adversary-sim control availability follows explicit surface opt-in in both runtime classes', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');

    assert.deepEqual(
      adversaryModule.deriveAdversarySimControlState({
        configSnapshot: {
          runtime_environment: 'runtime-prod',
          adversary_sim_available: true
        }
      }),
      {
        runtimeEnvironment: 'runtime-prod',
        surfaceAvailable: true,
        controlAvailable: true
      }
    );

    assert.deepEqual(
      adversaryModule.deriveAdversarySimControlState({
        configSnapshot: {
          runtime_environment: 'runtime-dev',
          adversary_sim_available: true
        }
      }),
      {
        runtimeEnvironment: 'runtime-dev',
        surfaceAvailable: true,
        controlAvailable: true
      }
    );

    assert.deepEqual(
      adversaryModule.deriveAdversarySimControlState({
        configSnapshot: {
          runtime_environment: 'runtime-prod',
          adversary_sim_available: false
        },
        adversarySimStatus: {
          runtime_environment: 'runtime-prod',
          adversary_sim_available: false
        }
      }),
      {
        runtimeEnvironment: 'runtime-prod',
        surfaceAvailable: false,
        controlAvailable: false
      }
    );
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
    assert.equal(schema.advancedConfigTemplatePaths.includes('adversary_sim_duration_seconds'), true);
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
    assert.equal(schema.advancedConfigTemplatePaths.includes('ai_policy_block_training'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('ai_policy_block_search'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('ai_policy_allow_search_engines'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('robots_block_ai_training'), false);
    assert.equal(schema.advancedConfigTemplatePaths.includes('robots_block_ai_search'), false);
    assert.equal(schema.advancedConfigTemplatePaths.includes('robots_allow_search_engines'), false);
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

test('runtime variable inventory meanings match writable and read-only admin config payload paths', { concurrency: false }, async () => {
  const apiSource = fs.readFileSync(path.join(__dirname, '..', 'src/admin/api.rs'), 'utf8');
  const schema = await importBrowserModule('dashboard/src/lib/domain/config-schema.js');
  const statusVarMeanings = JSON.parse(
    fs.readFileSync(path.join(DASHBOARD_ROOT, 'static/assets/status-var-meanings.json'), 'utf8')
  );

  const payloadFnStart = apiSource.indexOf('fn admin_config_payload(');
  const payloadFnEnd = apiSource.indexOf('\n#[derive(Debug, Deserialize, Default)]', payloadFnStart);
  if (payloadFnStart < 0 || payloadFnEnd < 0) {
    throw new Error('Unable to parse admin_config_payload function body');
  }
  const payloadFnSource = apiSource.slice(payloadFnStart, payloadFnEnd);
  const insertedReadOnlyTopLevelPaths = Array.from(payloadFnSource.matchAll(/obj\.insert\(\s*"([^"]+)"/g))
    .map((match) => String(match[1] || '').trim())
    .filter((value) => value.length > 0);

  const expectedReadOnlyPaths = insertedReadOnlyTopLevelPaths.filter(
    (pathValue) => pathValue !== 'defence_modes_effective' && pathValue !== 'botness_signal_definitions'
  );
  ['rate', 'geo', 'js'].forEach((moduleName) => {
    expectedReadOnlyPaths.push(`defence_modes_effective.${moduleName}.configured`);
    expectedReadOnlyPaths.push(`defence_modes_effective.${moduleName}.signal_enabled`);
    expectedReadOnlyPaths.push(`defence_modes_effective.${moduleName}.action_enabled`);
    expectedReadOnlyPaths.push(`defence_modes_effective.${moduleName}.note`);
  });
  expectedReadOnlyPaths.push('botness_signal_definitions.scored_signals');
  expectedReadOnlyPaths.push('botness_signal_definitions.terminal_signals');
  [
    'ip_range_suggestions_min_observations',
    'ip_range_suggestions_min_bot_events',
    'ip_range_suggestions_min_confidence_percent',
    'ip_range_suggestions_low_collateral_percent',
    'ip_range_suggestions_high_collateral_percent',
    'ip_range_suggestions_ipv4_min_prefix_len',
    'ip_range_suggestions_ipv6_min_prefix_len',
    'ip_range_suggestions_likely_human_sample_percent'
  ].forEach((pathValue) => expectedReadOnlyPaths.push(pathValue));

  const expectedInventoryPaths = new Set([
    ...schema.advancedConfigTemplatePaths,
    ...expectedReadOnlyPaths
  ]);
  const statusVarMeaningPaths = Object.keys(statusVarMeanings || {});

  const missingMeaningPaths = Array.from(expectedInventoryPaths)
    .filter((pathValue) => !Object.prototype.hasOwnProperty.call(statusVarMeanings, pathValue))
    .sort();
  const staleMeaningPaths = statusVarMeaningPaths
    .filter((pathValue) => !expectedInventoryPaths.has(pathValue))
    .sort();

  assert.deepEqual(missingMeaningPaths, []);
  assert.deepEqual(staleMeaningPaths, []);
  [
    'ip_range_suggestions_min_observations',
    'ip_range_suggestions_min_bot_events',
    'ip_range_suggestions_min_confidence_percent',
    'ip_range_suggestions_low_collateral_percent',
    'ip_range_suggestions_high_collateral_percent',
    'ip_range_suggestions_ipv4_min_prefix_len',
    'ip_range_suggestions_ipv6_min_prefix_len',
    'ip_range_suggestions_likely_human_sample_percent'
  ].forEach((pathValue) => {
    assert.equal(Object.prototype.hasOwnProperty.call(statusVarMeanings, pathValue), true);
  });
  [
    'robots_block_ai_training',
    'robots_block_ai_search',
    'robots_allow_search_engines'
  ].forEach((pathValue) => {
    assert.equal(Object.prototype.hasOwnProperty.call(statusVarMeanings, pathValue), false);
  });
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
    configNetworkSource,
    configDurationsSource,
    saveChangesBarSource
  ].join('\n');

  assert.match(ipBansSource, /export let onBan = null;/);
  assert.match(ipBansSource, /export let onUnban = null;/);
  assert.match(ipBansSource, /export let onSaveConfig = null;/);
  assert.match(ipBansSource, /export let configVersion = 0;/);
  assert.match(ipBansSource, /export let ipRangeSuggestionsVersion = 0;/);
  assert.match(ipBansSource, /export let configSnapshot = null;/);
  assert.match(ipBansSource, /export let ipRangeSuggestionsSnapshot = null;/);
  assert.match(ipBansSource, /let banFilter = 'all';/);
  assert.match(ipBansSource, /id="ip-ban-filter"/);
  assert.match(ipBansSource, /class="chart-canvas-shell chart-canvas-shell--ip-bans"/);
  assert.match(ipBansSource, /id="ip-ban-reasons-chart"/);
  assert.match(ipBansSource, /type: 'doughnut'/);
  assert.match(ipBansSource, /animation: false,/);
  assert.match(ipBansSource, /maintainAspectRatio: false,/);
  assert.match(ipBansSource, /resolveMonitoringChartTheme/);
  assert.equal(ipBansSource.includes('CHART_COLORS'), false);
  assert.match(ipBansSource, /canvasHasRenderableSize\(canvas\)/);
  assert.match(ipBansSource, /window\.addEventListener\('resize', onResize, \{ passive: true \}\);/);
  assert.match(ipBansSource, /if \(browser && nextActive && !wasActive\)/);
  assert.match(ipBansSource, /id="ip-range-suggestions-table"/);
  assert.match(ipBansSource, /id="bypass-allowlists-toggle"/);
  assert.match(ipBansSource, /id="network-allowlist"/);
  assert.equal(ipBansSource.includes('id="path-allowlist"'), false);
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
  assert.equal(configSource.includes("import ConfigNetworkSection from './config/ConfigNetworkSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigExportSection from './config/ConfigExportSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigMazeSection from './config/ConfigMazeSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigDurationsSection from './config/ConfigDurationsSection.svelte';"), false);
  assert.equal(configSource.includes("import ConfigGeoSection from './config/ConfigGeoSection.svelte';"), false);
  assert.equal(configSource.includes('ConfigRobotsSection'), false);
  assert.match(configSource, /import SaveChangesBar from '\.\/primitives\/SaveChangesBar\.svelte';/);
  assert.match(configSource, /<ConfigChallengeSection/);
  assert.equal(configSource.includes('<ConfigNetworkSection'), false);
  assert.equal(configSurfaceSource.includes('id="js-browser-allowlist-rules"'), false);
  assert.equal(configNetworkSource.includes('id="browser-allowlist-rules"'), false);
  assert.equal(configSource.includes('<ConfigExportSection'), false);
  assert.equal(configSource.includes('<ConfigMazeSection'), false);
  assert.equal(configSource.includes('<ConfigDurationsSection'), false);
  assert.equal(configSource.includes('<ConfigGeoSection'), false);
  assert.equal(configSource.includes('<ConfigRobotsSection'), false);
  assert.equal(configSource.includes('showHoneypot={false}'), false);
  assert.equal(configSource.includes('showBrowserPolicy={true}'), false);
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
  assert.equal(configSource.includes('browser_policy_enabled'), false);
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

  assert.match(tuningSource, /id="path-allowlist"/);
  assert.match(tuningSource, /id="path-allowlist-enabled-toggle"/);
  assert.match(tuningSource, /payload\.path_allowlist_enabled = pathAllowlistEnabled === true;/);
  assert.match(tuningSource, /payload\.path_allowlist = parseListTextarea\(pathAllowlist\);/);

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
  assert.match(rateLimitingSource, /id="rate-edge-unavailable-message"/);
  assert.match(rateLimitingSource, /buttonId="save-rate-limiting-config"/);
  assert.match(rateLimitingSource, /window\.addEventListener\('beforeunload'/);

  assert.match(geoSource, /export let onSaveConfig = null;/);
  assert.match(geoSource, /await onSaveConfig\(payload/);
  assert.match(geoSource, /<ConfigGeoSection/);
  assert.match(configGeoSource, /id="geo-scoring-toggle"/);
  assert.match(configGeoSource, /id="geo-routing-toggle"/);
  assert.match(geoSource, /id="geo-edge-unavailable-message"/);
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
  assert.match(fingerprintingSource, /config\.akamai_edge_available === true/);
  assert.match(fingerprintingSource, /id="fingerprinting-akamai-enabled-toggle"/);
  assert.match(fingerprintingSource, /id="fingerprinting-edge-mode-select"/);
  assert.match(fingerprintingSource, /id="fingerprinting-akamai-unavailable-message"/);
  assert.equal(fingerprintingSource.includes('id="fingerprinting-provider-backend-select"'), false);
  assert.equal(fingerprintingSource.includes('id="fingerprinting-cdp-enabled-toggle"'), false);
  assert.equal(fingerprintingSource.includes('id="fingerprinting-cdp-threshold-slider"'), false);
  assert.match(fingerprintingSource, /buttonId="save-fingerprinting-config"/);
  assert.match(fingerprintingSource, /window\.addEventListener\('beforeunload'/);
  assert.match(fingerprintingSurfaceSource, /id="fingerprinting-total-detections"/);

  assert.match(tuningSource, /export let onSaveConfig = null;/);
  assert.match(tuningSource, /await onSaveConfig\(payload/);
  assert.match(tuningSource, /import ConfigDurationsSection from '\.\/config\/ConfigDurationsSection\.svelte';/);
  assert.match(tuningSource, /import ConfigNetworkSection from '\.\/config\/ConfigNetworkSection\.svelte';/);
  assert.match(tuningSource, /<ConfigDurationsSection/);
  assert.match(tuningSource, /<ConfigNetworkSection/);
  assert.match(tuningSource, /showHoneypot=\{false\}/);
  assert.match(tuningSource, /showBrowserPolicy=\{true\}/);
  assert.match(tuningSource, /browser_policy_enabled/);
  assert.match(tuningSource, /ban_durations/);
  assert.match(tuningSource, /buttonId="save-tuning-all"/);
  assert.match(tuningSource, /import SaveChangesBar from '\.\/primitives\/SaveChangesBar\.svelte';/);
  assert.match(tuningSource, /window\.addEventListener\('beforeunload'/);
  assert.equal(tuningSource.includes('id="save-botness-config"'), false);
  assert.match(tuningSurfaceSource, /dayId="dur-honeypot-days"/);
  assert.match(tuningSurfaceSource, /dayId="dur-rate-limit-days"/);
  assert.match(tuningSurfaceSource, /id="browser-policy-toggle"/);
  assert.match(tuningSurfaceSource, /id="browser-block-rules"/);
});

test('dashboard route does not add unapproved read-only chrome to config tabs', () => {
  const configPanelSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/primitives/ConfigPanel.svelte'),
    'utf8'
  );
  const dashboardRouteSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(configPanelSource, /class:hidden=\{!writable\}/);
  assert.equal(configPanelSource.includes('config-panel--read-only'), false);
  assert.equal(configPanelSource.includes('config-panel__fieldset'), false);
  assert.equal(dashboardRouteSource.includes('dashboard-read-only-hint'), false);
  assert.equal(dashboardRouteSource.includes('This deployment is read-only.'), false);
});

test('dashboard route preflights dirty config logout before mutating session state', () => {
  const dashboardRouteSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(dashboardRouteSource, /function hasVisibleUnsavedConfigChanges\(/);
  assert.match(dashboardRouteSource, /\.config-save-bar:not\(\.hidden\)/);
  assert.match(dashboardRouteSource, /window\.confirm\(/);
  assert.match(dashboardRouteSource, /stopImmediatePropagation\(\)/);
  assert.match(
    dashboardRouteSource,
    /const hasUnsavedConfigChanges = hasVisibleUnsavedConfigChanges\(\);\s+if \(!confirmDiscardUnsavedConfigChanges\(\)\) return;\s+let redirectingToLogin = false;\s+loggingOut = true;\s+try \{\s+suppressBeforeUnloadPrompt = hasUnsavedConfigChanges;\s+routeController\.abortInFlightRefresh\(\);\s+clearAdversarySimStatusPollTimer\(\);\s+await logoutDashboardSession\(\);/s
  );
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
  assert.match(source, /\$lib\/runtime\/dashboard-body-classes\.js/);
  assert.match(source, /\$lib\/runtime\/dashboard-adversary-sim\.js/);
  assert.match(source, /deriveAdversarySimControlState/);
  assert.match(source, /deriveDashboardBodyClassState\(configSnapshot,\s*\{/);
  assert.match(source, /runtimeClassHint/);
  assert.match(source, /const DASHBOARD_LOADED_CLASS = 'dashboard-loaded';/);
  assert.match(source, /backendConnectionState/);
  assert.match(source, /dashboardLoaded && backendConnectionState === 'disconnected'/);
  assert.match(source, /classList\.add\(DASHBOARD_LOADED_CLASS\)/);
  assert.match(source, /classList\.remove\(DASHBOARD_LOADED_CLASS\)/);
  assert.match(source, /syncDashboardBodyClasses\(document, bodyClassState\)/);
  assert.match(source, /clearDashboardBodyClasses\(document\)/);
  assert.match(source, /<svelte:window on:hashchange=\{onWindowHashChange\} \/>/);
  assert.match(source, /<svelte:document on:visibilitychange=\{onDocumentVisibilityChange\} \/>/);
  assert.match(source, /use:registerTabLink=\{tab\}/);
  assert.match(source, /buildDashboardLoginPath/);
  assert.match(source, /const AUTO_REFRESH_INTERVAL_MS = 1000;/);
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
  assert.match(source, /ipRangeSuggestionsSnapshot=\{snapshots\.ipRangeSuggestions\}/);
  assert.match(source, /cdpSnapshot=\{snapshots\.cdp\}/);
  assert.match(source, /id="global-test-mode-toggle"/);
  assert.match(source, /id="global-adversary-sim-toggle"/);
  assert.match(source, /id="adversary-sim-lifecycle-copy"/);
  assert.match(source, /id="connection-status"/);
  assert.match(source, /id="lost-connection"/);
  assert.match(source, /let adversarySimStatusRequestInFlight = null;/);
  assert.match(source, /if \(adversarySimStatusRequestInFlight\) \{/);
  assert.match(source, /return adversarySimStatusRequestInFlight;/);
  assert.match(
    source,
    /if \(runtimeReady && bootstrapAdversarySimControlState\.controlAvailable\) \{\s*await refreshAdversarySimStatus\('bootstrap'\);/s
  );
  assert.match(source, /onGlobalTestModeToggleChange/);
  assert.match(source, /onGlobalAdversarySimToggleChange/);
  assert.match(source, /function isAuthSessionExpiredError\(error\)/);
  assert.match(source, /function withRefreshedSessionOnAuthError\(action\)/);
  assert.match(source, /const restored = await restoreDashboardSession\(\);/);
  assert.match(source, /if \(restored !== true\) throw error;/);
  assert.match(source, /await withRefreshedSessionOnAuthError\(/);
  assert.match(source, /controlDashboardAdversarySim\(nextValue,\s*\{/);
  assert.match(source, /updateDashboardConfig\(patch \|\| \{\},\s*\{/);
  assert.match(source, /banDashboardIp\(ip, duration, 'manual_ban',\s*\{/);
  assert.match(source, /unbanDashboardIp\(ip,\s*\{/);
  assert.match(source, /status === 401/);
  assert.match(source, /status !== 403/);
  assert.match(source, /csrf/);
  assert.match(source, /trust boundary/);
  assert.match(source, /Adversary simulation control session expired\. Redirecting to login\.\.\./);
  assert.match(source, /dashboard-global-control-label/);
  assert.equal(source.includes("await routeController.refreshTab(activeTabKey, 'auto-refresh');"), false);
  assert.equal(source.includes('adversary-sim-progress-line'), false);
  assert.match(source, /id="admin-msg"/);
});

test('dashboard stylesheet applies disconnected visual treatment via root class', () => {
  const source = fs.readFileSync(path.join(DASHBOARD_ROOT, 'style.css'), 'utf8');
  assert.match(source, /:root\.disconnected\s*\{/);
  assert.match(source, /filter:\s*saturate\(0\);/);
  assert.match(source, /:root\.disconnected\.dashboard-loaded #lost-connection\s*\{/);
  assert.match(source, /#connection-status\s*\{/);
  assert.match(source, /#lost-connection\s*\{/);
});

test('login route syncs disconnected + runtime classes onto html root and gates submit on runtime state', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/login.html/+page.svelte'),
    'utf8'
  );
  assert.match(source, /deriveDashboardBodyClassState/);
  assert.match(source, /syncDashboardBodyClasses/);
  assert.match(source, /let runtimeStateAvailable = false;/);
  assert.match(source, /normalizeRuntimeEnvironment/);
  assert.equal(source.includes('inferRuntimeEnvironment'), false);
  assert.match(source, /if \(!runtimeStateAvailable\) \{/);
  assert.match(source, /disabled=\{!runtimeStateAvailable\}/);
  assert.match(source, /backendConnectionState:\s*'disconnected'/);
  assert.match(source, /runtime_environment/);
  assert.match(source, /let apiKeyInput = null;/);
  assert.match(source, /apiKeyInput\.focus\(\)/);
});

test('login route exposes native password-manager-friendly form-post semantics', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/login.html/+page.svelte'),
    'utf8'
  );
  const loadSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/login.html/+page.js'),
    'utf8'
  );

  assert.match(source, /resolveDashboardAssetPath/);
  assert.match(source, /export let data;/);
  assert.match(source, /const dashboardBasePath = typeof data\?\.dashboardBasePath === 'string'/);
  assert.match(source, /const faviconHref = typeof data\?\.faviconHref === 'string'/);
  assert.match(source, /<link rel="icon" type="image\/png" href=\{faviconHref\}>/);
  assert.match(loadSource, /export function load\(\)/);
  assert.match(loadSource, /dashboardBasePath/);
  assert.match(loadSource, /faviconHref/);
  assert.match(loadSource, /assets\/shuma-gorath-pencil-closed\.png/);
  assert.match(source, /const passwordManagerIdentity = 'admin';/);
  assert.match(source, /<form id="login-form" class="login-form" method="POST" action="\/admin\/login">/);
  assert.match(source, /<label class="control-label" for="username">Account<\/label>/);
  assert.match(source, /name="username"/);
  assert.match(source, /id="username"/);
  assert.match(source, /type="text"/);
  assert.match(source, /class="input-field"/);
  assert.match(source, /autocomplete="username"/);
  assert.match(source, /value=\{passwordManagerIdentity\}/);
  assert.match(source, /readonly/);
  assert.equal(source.includes('visually-hidden" for="username"'), false);
  assert.match(source, /name="next"/);
  assert.match(source, /value=\{nextPath\}/);
  assert.match(source, /id="current-password"/);
  assert.match(source, /name="password"/);
  assert.match(source, /autocomplete="current-password"/);
  assert.equal(source.includes('autocomplete="off"'), false);
  assert.equal(source.includes("fetch('/admin/login'"), false);
  assert.equal(source.includes('JSON.stringify({ api_key: normalized })'), false);
  assert.equal(source.includes('event.preventDefault()'), false);
});

test('dashboard routes advertise an explicit dashboard-scoped favicon', () => {
  const mainSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(mainSource, /const faviconHref = resolveDashboardAssetPath\(\s*dashboardBasePath,\s*'assets\/shuma-gorath-pencil-closed\.png'\s*\)/s);
  assert.match(mainSource, /<link rel="icon" type="image\/png" href=\{faviconHref\}>/);
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
  assert.match(source, /from '\.\.\/\.\.\/domain\/monitoring-chart-presets\.js';/);
  assert.match(source, /resolveMonitoringChartTheme/);
  assert.match(source, /x: buildMonitoringTimeSeriesXAxis\(\),/);
  assert.match(source, /const fillColor = chartTheme\.timeSeriesFill\.events/);
  assert.match(source, /'challenge'/);
  assert.match(source, /'pow'/);
  assert.match(source, /export let autoRefreshEnabled = false;/);
  assert.match(source, /sameSeries\(chart, trendSeries\.labels, trendSeries\.data\)/);
  assert.match(source, /abortRangeEventsFetch\(\);/);
  assert.match(source, /const isRangeFetchInFlight = selectedRangeWindowState\.loading === true;/);
  assert.match(source, /normalizeReasonRows\(/);
  assert.match(source, /buildTimeSeries\(selectedRangeEvents, selectedTimeRange,/);
  assert.match(source, /deriveMonitoringEventDisplay/);
  assert.match(source, /const normalizeEventForDisplay = \(event = \{\}\) =>/);
  assert.match(source, /const buildRawTelemetryFeed = \(events = \[\]\) =>/);
  assert.equal(source.includes('rangeEventsSnapshot.range'), false);
  assert.match(source, /'Puzzle Outcomes'/);
  assert.match(source, /\$: rawRecentEvents = Array\.isArray\(events\.recent_events\)/);
  assert.match(source, /\$: rawTelemetryFeed = buildRawTelemetryFeed\(rawRecentEvents\);/);
  assert.match(source, /\$: eventWindowTotal = toNonNegativeIntOrNull\(events\?\.recent_events_window\?\.total_events_in_window\);/);
  assert.match(source, /\$: totalBans = \(\(\) => \{/);
  assert.match(source, /const byEventType = getEventCountByName\(eventCounts, 'Ban'\);/);
  assert.match(source, /\$: activeBans = bans\.length;/);
});

test('monitoring tab is decomposed into focused subsection components', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/MonitoringTab.svelte'),
    'utf8'
  );

  assert.match(source, /import RawTelemetryFeed from '\.\/monitoring\/RawTelemetryFeed\.svelte';/);
  assert.match(source, /import OverviewStats from '\.\/monitoring\/OverviewStats\.svelte';/);
  assert.match(source, /import PrimaryCharts from '\.\/monitoring\/PrimaryCharts\.svelte';/);
  assert.match(source, /import AdversaryRunPanel from '\.\/monitoring\/AdversaryRunPanel\.svelte';/);
  assert.match(source, /import DefenseTrendBlocks from '\.\/monitoring\/DefenseTrendBlocks\.svelte';/);
  assert.match(source, /import RecentEventsTable from '\.\/monitoring\/RecentEventsTable\.svelte';/);
  assert.match(source, /import ShadowSection from '\.\/monitoring\/ShadowSection\.svelte';/);
  assert.match(source, /import ExternalMonitoringSection from '\.\/monitoring\/ExternalMonitoringSection\.svelte';/);
  assert.match(source, /import IpRangeSection from '\.\/monitoring\/IpRangeSection\.svelte';/);
  assert.match(source, /<OverviewStats/);
  assert.match(source, /<RawTelemetryFeed/);
  assert.match(source, /<PrimaryCharts/);
  assert.match(source, /<AdversaryRunPanel/);
  assert.match(source, /<DefenseTrendBlocks/);
  assert.match(source, /<ShadowSection/);
  assert.match(source, /<ChallengeSection/);
  assert.match(source, /<PowSection/);
  assert.match(source, /<IpRangeSection/);
  assert.match(source, /<ExternalMonitoringSection/);
  assert.match(source, /filterOptions=\{eventFilterOptions\}/);
  assert.match(source, /onFilterChange=\{onEventFilterChange\}/);
  assert.match(source, /RAW_FEED_MAX_LINES = 200/);
});

test('monitoring recent-events filters reuse canonical input-row and input-field styles', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/RecentEventsTable.svelte'),
    'utf8'
  );

  const inputRowMatches = source.match(/class="input-row"/g) || [];
  const selectMatches = source.match(/<select\s+[^>]*class="input-field"/g) || [];

  assert.equal(source.includes('field-row'), false);
  assert.equal(inputRowMatches.length, 6);
  assert.equal(selectMatches.length, 6);
  assert.match(source, /class="control-label control-label--wide"/);
  assert.match(source, /monitoring-filter-mode/);
});

test('monitoring overview stats labels retain explicit window semantics', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/OverviewStats.svelte'),
    'utf8'
  );
  assert.match(source, /title="Bans \(24h\)"/);
  assert.match(source, /title="Active Bans"/);
  assert.match(source, /title="Events \(24h\)"/);
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
  assert.match(source, /const CONNECTION_HEARTBEAT_PATH = '\/admin\/session';/);
  assert.match(source, /function runConnectionHeartbeat\(reason = 'manual'\)/);
  assert.match(source, /recordHeartbeatAttemptStarted/);
  assert.match(source, /recordHeartbeatSuccess/);
  assert.match(source, /recordHeartbeatFailure/);
  assert.match(source, /function hasRuntimeEnvironment\(\)/);
  assert.match(source, /if \(!hasRuntimeEnvironment\(\)\) return false;/);
  assert.equal(source.includes('next.adversary_sim_enabled = status.adversary_sim_enabled;'), false);
  assert.equal(source.includes('onBackendConnected'), false);
  assert.equal(source.includes('onBackendDisconnected'), false);
  assert.match(source, /export async function updateDashboardConfig/);
  assert.match(source, /export async function validateDashboardConfigPatch/);
  assert.match(source, /export async function banDashboardIp/);
  assert.match(source, /export async function unbanDashboardIp/);
  assert.match(source, /dashboardRefreshRuntime\.clearAllCaches/);
});

test('dashboard refresh runtime enforces cursor-delta + SSE helpers and excludes legacy config UI glue', () => {
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
  assert.match(source, /const IP_BANS_CACHE_MAX_SUGGESTIONS = 50;/);
  assert.match(source, /const MONITORING_DELTA_LIMIT = 120;/);
  assert.match(source, /const IP_BANS_DELTA_LIMIT = 120;/);
  assert.match(source, /const MONITORING_FULL_RECENT_EVENTS_LIMIT = 200;/);
  assert.match(source, /const IP_RANGE_SUGGESTIONS_HOURS = 24;/);
  assert.match(source, /const IP_RANGE_SUGGESTIONS_LIMIT = 20;/);
  assert.match(source, /const STREAMABLE_TABS = Object\.freeze\(new Set\(\['monitoring', 'ip-bans'\]\)\);/);
  assert.match(source, /function clearAllCaches\(\) \{/);
  assert.match(source, /closeAllStreams\(\);/);
  assert.match(source, /monitoringFreshness/);
  assert.match(source, /ipBansFreshness/);
  assert.match(source, /dashboardApiClient\.getMonitoringDelta\(/);
  assert.match(source, /dashboardApiClient\.getIpBansDelta\(/);
  assert.match(source, /function startRealtimeStream\(tab\) \{/);
  assert.match(source, /typeof EventSource !== 'function'/);
  assert.match(source, /new EventSource\(streamUrl, \{ withCredentials: true \}\)/);
  assert.match(source, /applyMonitoringDeltaSnapshots\(payload, 'sse'\)/);
  assert.match(source, /applyIpBansDeltaSnapshots\(payload, 'sse'\)/);
  assert.match(source, /updateFreshnessSnapshot\(/);
  assert.match(source, /writeCache\(MONITORING_CACHE_KEY, \{ monitoring: compactMonitoring \}\);/);
  assert.match(source, /if \(hasConfigSnapshot\(existingConfig\)\) \{/);
  assert.equal(source.includes("? { monitoring: monitoringData }"), false);
  assert.match(source, /const refreshVerificationTab = \(reason = 'manual'/);
  assert.match(source, /async function refreshFingerprintingTab\(reason = 'manual'/);
  assert.match(source, /dashboardApiClient\.getCdp\(requestOptions\)/);
  assert.match(
    source,
    /dashboardApiClient\.getIpRangeSuggestions\(\s*\{ hours: IP_RANGE_SUGGESTIONS_HOURS, limit: IP_RANGE_SUGGESTIONS_LIMIT \},\s*requestOptions\s*\)/
  );
  assert.match(
    source,
    /writeCache\(IP_BANS_CACHE_KEY, \{\s*bans: compactBans,\s*ipRangeSuggestions: compactSuggestions\s*\}\);/m
  );
  assert.match(source, /const includeConfigRefresh = reason !== 'auto-refresh';/);
  assert.equal(source.includes("reason === 'adversary-sim-toggle'"), false);
  assert.match(
    source,
    /includeConfigRefresh \? refreshSharedConfig\(reason, runtimeOptions\) : Promise\.resolve\(null\)/
  );
  assert.equal(source.includes("refreshIpBansTab('auto-refresh', runtimeOptions)"), false);
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
  assert.equal(source.includes("routeController.refreshTab(activeTabKey, 'adversary-sim-toggle')"), false);
});

test('dashboard route overlays a test-mode eye on the header image only when test mode is enabled', () => {
  const routeSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );
  const styleSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'style.css'),
    'utf8'
  );

  assert.match(routeSource, /assets\/eye\.png/);
  assert.match(routeSource, /\{#if testModeEnabled\}/);
  assert.match(routeSource, /dashboard-test-mode-eye/);
  assert.match(routeSource, /Test mode active/);
  assert.equal(routeSource.includes("<style>"), false);

  assert.match(styleSource, /\.shuma-image-wrapper\s*\{\s*position:\s*relative;/m);
  assert.match(styleSource, /\.dashboard-test-mode-eye\s*\{[\s\S]*position:\s*absolute;[\s\S]*pointer-events:\s*none;/m);
  assert.match(styleSource, /\.dashboard-test-mode-eye-image\s*\{\s*display:\s*block;\s*width:\s*100%;\s*height:\s*auto;/m);
  assert.equal(styleSource.includes("drop-shadow"), false);
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
  assert.match(source, /runtimeEnvironmentRaw/);
  assert.match(source, /runtimeEnvironment/);
  assert.match(source, /if \(authenticated && !runtimeEnvironment\)/);
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
