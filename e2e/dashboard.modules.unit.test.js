const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

const WORKSPACE_ROOT = path.resolve(__dirname, '..');
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

function createManualScheduler() {
  let now = 0;
  let nextId = 1;
  let tasks = [];

  return {
    nowMs() {
      return now;
    },
    schedule(callback, delayMs = 0) {
      const id = nextId++;
      tasks.push({
        id,
        at: now + Math.max(0, Number(delayMs || 0)),
        callback
      });
      tasks.sort((left, right) => left.at - right.at || left.id - right.id);
      return id;
    },
    cancelScheduled(id) {
      tasks = tasks.filter((task) => task.id !== id);
    },
    async advanceBy(delayMs = 0) {
      const target = now + Math.max(0, Number(delayMs || 0));
      while (tasks.length > 0) {
        tasks.sort((left, right) => left.at - right.at || left.id - right.id);
        const nextTask = tasks[0];
        if (!nextTask || nextTask.at > target) break;
        tasks.shift();
        now = nextTask.at;
        nextTask.callback();
      }
      now = target;
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

    const analytics = api.adaptAnalytics({
      ban_count: '7',
      ban_store_status: 'available',
      shadow_mode: true
    });
    assert.equal(analytics.ban_count, 7);
    assert.equal(analytics.ban_store_status, 'available');
    assert.equal(analytics.shadow_mode, true);
    assert.equal(analytics.fail_mode, 'open');

    const unavailableAnalytics = api.adaptAnalytics({
      ban_count: null,
      ban_store_status: 'unavailable',
      ban_store_message: 'authoritative backend access required'
    });
    assert.equal(unavailableAnalytics.ban_count, null);
    assert.equal(unavailableAnalytics.ban_store_status, 'unavailable');
    assert.equal(unavailableAnalytics.ban_store_message, 'authoritative backend access required');

    const events = api.adaptEvents({
      recent_events: [{ ip: '198.51.100.1' }, null, 'ignored'],
      recent_sim_runs: [{ run_id: 'simrun-1' }, null, 'ignored'],
      top_ips: [['198.51.100.1', '9'], ['198.51.100.2', 4], ['bad']],
      unique_ips: '11'
    });
    assert.equal(events.recent_events.length, 1);
    assert.equal(events.recent_sim_runs.length, 1);
    assert.equal(events.recent_sim_runs[0].run_id, 'simrun-1');
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

    const bans = api.adaptBans({
      bans: [{ ip: '198.51.100.9', reason: 'manual_ban' }, null, 'ignored'],
      status: 'unavailable',
      message: 'authoritative backend access required'
    });
    assert.equal(bans.bans.length, 1);
    assert.equal(bans.bans[0].reason, 'manual_ban');
    assert.equal(bans.status, 'unavailable');
    assert.equal(bans.message, 'authoritative backend access required');

    const maze = api.adaptMaze({
      total_hits: '12',
      unique_crawlers: '3',
      maze_auto_bans: null,
      top_crawlers: [{ ip: '198.51.100.9', hits: 4 }]
    });
    assert.equal(maze.total_hits, 12);
    assert.equal(maze.unique_crawlers, 3);
    assert.equal(maze.maze_auto_bans, null);

    const delta = api.adaptCursorDelta({
      after_cursor: 'c1',
      window_end_cursor: 'c9',
      next_cursor: 'c3',
      has_more: false,
      overflow: 'none',
      events: [{ cursor: 'c3', event: 'Challenge', ts: 1700000000 }],
      recent_sim_runs: [{ run_id: 'simrun-delta-1' }, null, 'ignored'],
      active_bans_status: 'unavailable',
      active_bans_message: 'authoritative backend access required',
      freshness: { state: 'fresh', lag_ms: 120, transport: 'sse' }
    });
    assert.equal(delta.after_cursor, 'c1');
    assert.equal(delta.window_end_cursor, 'c9');
    assert.equal(delta.next_cursor, 'c3');
    assert.equal(delta.events.length, 1);
    assert.equal(delta.recent_sim_runs.length, 1);
    assert.equal(delta.recent_sim_runs[0].run_id, 'simrun-delta-1');
    assert.equal(delta.active_bans_status, 'unavailable');
    assert.equal(delta.active_bans_message, 'authoritative backend access required');
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

test('dashboard API client exposes Retry-After seconds on adversary-sim control errors', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const client = apiModule.create({
      getAdminContext: () => ({
        endpoint: 'https://edge.local',
        apikey: 'test-key',
        sessionAuth: true,
        csrfToken: 'csrf-token'
      }),
      request: async () => ({
        ok: false,
        status: 409,
        headers: new Headers({ 'retry-after': '13', 'content-type': 'application/json' }),
        json: async () => ({ error: 'Adversary simulation controller lease is currently held' }),
        text: async () => '{"error":"Adversary simulation controller lease is currently held"}'
      })
    });

    await assert.rejects(
      () => client.controlAdversarySim(true),
      (error) => {
        assert.equal(error.name, 'DashboardApiError');
        assert.equal(error.status, 409);
        assert.equal(error.retryAfterSeconds, 13);
        return true;
      }
    );
  });
});

test('dashboard API client preserves adversary-sim lane status and diagnostics fields', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const adapted = apiModule.adaptAdversarySimStatus({
      runtime_environment: 'runtime-prod',
      adversary_sim_available: true,
      adversary_sim_enabled: false,
      generation_active: false,
      phase: 'off',
      desired_lane: 'scrapling_traffic',
      active_lane: null,
      lane_switch_seq: 3,
      last_lane_switch_at: 1720,
      last_lane_switch_reason: 'beat_boundary_reconciliation',
      generation_diagnostics: {
        health: 'ok',
        reason: 'persisted_events_observed',
        recommended_action: 'No action required.',
        generated_tick_count: 1,
        generated_request_count: 235,
        last_generated_at: 1715,
        last_generation_error: '',
        truth_basis: 'persisted_event_lower_bound'
      },
      persisted_event_evidence: {
        run_id: 'simrun-red-team-truth',
        lane: 'scrapling_traffic',
        profile: 'baseline',
        monitoring_event_count: 235,
        defense_delta_count: 4,
        ban_outcome_count: 1,
        first_observed_at: 1700,
        last_observed_at: 1715,
        truth_basis: 'persisted_event_lower_bound'
      },
      lane_diagnostics: {
        schema_version: 'v1',
        truth_basis: 'persisted_event_lower_bound',
        lanes: {
          synthetic_traffic: {
            beat_attempts: 4,
            beat_successes: 4,
            beat_failures: 0,
            generated_requests: 10,
            blocked_requests: 0,
            offsite_requests: 0,
            response_bytes: 1024,
            response_status_count: { '200': 10 },
            last_generated_at: 1700,
            last_error: ''
          },
          scrapling_traffic: {
            beat_attempts: 2,
            beat_successes: 1,
            beat_failures: 1,
            generated_requests: 3,
            blocked_requests: 1,
            offsite_requests: 2,
            response_bytes: 2048,
            response_status_count: { '200': 2, '429': 1 },
            last_generated_at: 1710,
            last_error: 'timeout'
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
            last_generated_at: null,
            last_error: ''
          }
        },
        request_failure_classes: {
          cancelled: { count: 0, last_seen_at: null },
          timeout: { count: 1, last_seen_at: 1712 },
          transport: { count: 0, last_seen_at: null },
          http: { count: 2, last_seen_at: 1715 }
        }
      }
    });

    assert.equal(adapted.desired_lane, 'scrapling_traffic');
    assert.equal(adapted.active_lane, '');
    assert.equal(adapted.lane_switch_seq, 3);
    assert.equal(adapted.last_lane_switch_at, 1720);
    assert.equal(adapted.last_lane_switch_reason, 'beat_boundary_reconciliation');
    assert.equal(adapted.generation_diagnostics.generated_request_count, 235);
    assert.equal(adapted.generation_diagnostics.truth_basis, 'persisted_event_lower_bound');
    assert.equal(adapted.lane_diagnostics.schema_version, 'v1');
    assert.equal(adapted.lane_diagnostics.truth_basis, 'persisted_event_lower_bound');
    assert.equal(adapted.lane_diagnostics.lanes.scrapling_traffic.generated_requests, 3);
    assert.equal(adapted.lane_diagnostics.lanes.scrapling_traffic.offsite_requests, 2);
    assert.equal(adapted.lane_diagnostics.request_failure_classes.timeout.count, 1);
    assert.equal(adapted.lane_diagnostics.request_failure_classes.http.last_seen_at, 1715);
    assert.equal(adapted.persisted_event_evidence.run_id, 'simrun-red-team-truth');
    assert.equal(adapted.persisted_event_evidence.monitoring_event_count, 235);
  });
});

test('dashboard API client sends optional adversary-sim lane selection in control writes', { concurrency: false }, async () => {
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
            requested_enabled: false,
            status: {
              adversary_sim_enabled: false,
              generation_active: false,
              phase: 'off',
              desired_lane: 'scrapling_traffic',
              active_lane: null
            },
            config: {},
            runtime: {}
          }),
          text: async () => JSON.stringify({
            requested_enabled: false,
            status: {
              adversary_sim_enabled: false,
              generation_active: false,
              phase: 'off',
              desired_lane: 'scrapling_traffic',
              active_lane: null
            },
            config: {},
            runtime: {}
          })
        };
      }
    });

    const response = await client.controlAdversarySim(false, { lane: 'scrapling_traffic' });

    assert.equal(calls.length, 1);
    assert.match(String(calls[0].url), /\/admin\/adversary-sim\/control$/);
    assert.equal(calls[0].init.method, 'POST');
    assert.deepEqual(JSON.parse(String(calls[0].init.body || '{}')), {
      enabled: false,
      lane: 'scrapling_traffic'
    });
    assert.equal(response.status.desired_lane, 'scrapling_traffic');
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
            active_bans_status: 'unavailable',
            active_bans_message: 'authoritative backend access required',
            freshness: { state: 'fresh', lag_ms: 42 }
          }),
          text: async () =>
            '{"after_cursor":"c1","window_end_cursor":"c9","next_cursor":"c2","has_more":false,"overflow":"none","events":[],"active_bans":[],"active_bans_status":"unavailable","active_bans_message":"authoritative backend access required","freshness":{"state":"fresh","lag_ms":42}}'
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
    assert.equal(ipBansDelta.active_bans_status, 'unavailable');
    assert.equal(ipBansDelta.active_bans_message, 'authoritative backend access required');
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
    ChartMock.defaults = {};
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
    assert.equal(one.defaults.animation.duration, 0);

    adapter.releaseChartRuntime({ window: win });
    assert.equal(typeof win.Chart, 'function');
    adapter.releaseChartRuntime({ window: win });
    assert.equal(win.Chart, undefined);
  });
});

test('chart runtime adapter applies zero-duration animation defaults to preloaded chart runtimes', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adapter = await importBrowserModule('dashboard/src/lib/domain/services/chart-runtime-adapter.js');
    const ChartMock = function ChartMock() {};
    ChartMock.defaults = {};
    const win = { location: { pathname: '/dashboard/index.html' }, Chart: ChartMock };

    const chart = await adapter.acquireChartRuntime({ window: win });

    assert.equal(chart, ChartMock);
    assert.equal(chart.defaults.animation.duration, 0);

    adapter.releaseChartRuntime({ window: win });
    assert.equal(win.Chart, ChartMock);
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

test('half doughnut chart helpers expose shared semicircle and hover readout behavior', { concurrency: false }, async () => {
  const helpers = await importBrowserModule('dashboard/src/lib/domain/half-doughnut-chart.js');
  const format = await importBrowserModule('dashboard/src/lib/domain/core/format.js');

  assert.equal(helpers.HALF_DOUGHNUT_ROTATION, -90);
  assert.equal(helpers.HALF_DOUGHNUT_CIRCUMFERENCE, 180);
  assert.equal(helpers.HALF_DOUGHNUT_CUTOUT, '72%');
  assert.equal(helpers.HALF_DOUGHNUT_LEGEND_OFFSET_PX, 5);
  assert.deepEqual(helpers.EMPTY_HALF_DOUGHNUT_READOUT, {
    label: '',
    value: '',
    active: false
  });
  assert.equal(helpers.HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN.id, 'shuma-half-doughnut-legend-offset');

  const shiftedLegend = {
    top: 20,
    bottom: 40,
    options: { position: 'bottom' },
    legendHitBoxes: [{ top: 22 }, { top: 28 }]
  };
  helpers.HALF_DOUGHNUT_LEGEND_OFFSET_PLUGIN.afterUpdate({ legend: shiftedLegend });
  assert.equal(shiftedLegend.top, 25);
  assert.equal(shiftedLegend.bottom, 45);
  assert.deepEqual(shiftedLegend.legendHitBoxes, [{ top: 27 }, { top: 33 }]);

  const fromCountsObject = helpers.buildHalfDoughnutSeries({
    Maze: 7,
    Challenge: 1200,
    Ban: 42,
    Alpha: 42
  });
  assert.deepEqual(fromCountsObject, {
    entries: [
      { label: 'Challenge', value: 1200 },
      { label: 'Alpha', value: 42 },
      { label: 'Ban', value: 42 },
      { label: 'Maze', value: 7 }
    ],
    labels: ['Challenge', 'Alpha', 'Ban', 'Maze'],
    values: [1200, 42, 42, 7]
  });

  const fromEntryList = helpers.buildHalfDoughnutSeries([
    { label: 'Maze', value: 7 },
    { label: 'Challenge', value: 1200 },
    { label: 'Ban', value: 42 },
    { label: 'Alpha', value: 42 }
  ]);
  assert.deepEqual(fromEntryList, fromCountsObject);

  const activeReadout = helpers.buildHalfDoughnutReadout(
    ['Challenge', 'Maze'],
    [1200, 7],
    [{ index: 0 }]
  );
  assert.deepEqual(activeReadout, {
    label: 'Challenge',
    value: format.formatCompactNumber(1200, '0'),
    active: true
  });

  const inactiveReadout = helpers.buildHalfDoughnutReadout(
    ['Challenge'],
    [5],
    [],
    { inactiveLabel: '' }
  );
  assert.deepEqual(inactiveReadout, {
    label: '',
    value: '',
    active: false
  });

  const nextReadouts = [];
  const options = helpers.buildHalfDoughnutOptions({
    legendColor: 'rgb(10, 20, 30)',
    maintainAspectRatio: false,
    resizeDelay: 0,
    animation: false,
    onReadoutChange(nextReadout) {
      nextReadouts.push(nextReadout);
    }
  });
  assert.equal(options.rotation, -90);
  assert.equal(options.circumference, 180);
  assert.equal(options.cutout, '72%');
  assert.equal(options.maintainAspectRatio, false);
  assert.equal(options.resizeDelay, 0);
  assert.equal(options.animation, false);
  assert.deepEqual(options.plugins.tooltip, { enabled: false });
  assert.equal(options.plugins.legend.position, 'bottom');
  assert.equal(options.plugins.legend.labels.color, 'rgb(10, 20, 30)');
  assert.equal(options.plugins.legend.labels.usePointStyle, true);
  assert.equal(options.plugins.legend.labels.pointStyle, 'circle');

  options.onHover(
    null,
    [{ index: 1 }],
    {
      data: {
        labels: ['Challenge', 'Maze'],
        datasets: [{ data: [12, 34] }]
      }
    }
  );
  assert.deepEqual(nextReadouts, [{
    label: 'Maze',
    value: '34',
    active: true
  }]);

  const syncedReadouts = [];
  helpers.syncHalfDoughnutReadout(
    {
      data: {
        labels: ['Ban'],
        datasets: [{ data: [9] }]
      },
      getActiveElements() {
        return [{ index: 0 }];
      }
    },
    (nextReadout) => {
      syncedReadouts.push(nextReadout);
    }
  );
  assert.deepEqual(syncedReadouts, [{
    label: 'Ban',
    value: '9',
    active: true
  }]);
});

test('dashboard state and store contracts remain immutable and bounded with heartbeat-owned connection telemetry', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const stateModule = await importBrowserModule('dashboard/src/lib/domain/dashboard-state.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    assert.deepEqual(toPlain(stateModule.DASHBOARD_TABS), [
      'monitoring',
      'ip-bans',
      'red-team',
      'tuning',
      'verification',
      'traps',
      'rate-limiting',
      'geo',
      'fingerprinting',
      'policy',
      'status',
      'advanced',
      'diagnostics'
    ]);
    assert.deepEqual(toPlain(storeModule.DASHBOARD_TABS), [
      'monitoring',
      'ip-bans',
      'red-team',
      'tuning',
      'verification',
      'traps',
      'rate-limiting',
      'geo',
      'fingerprinting',
      'policy',
      'status',
      'advanced',
      'diagnostics'
    ]);
    assert.equal(stateModule.normalizeTab('red-team'), 'red-team');

    const initial = stateModule.createInitialState('monitoring');
    const next = stateModule.reduceState(initial, { type: 'set-active-tab', tab: 'verification' });
    assert.notEqual(initial, next);
    assert.equal(initial.activeTab, 'monitoring');
    assert.equal(next.activeTab, 'verification');
    assert.equal(Object.prototype.hasOwnProperty.call(initial.snapshots, 'configRuntime'), true);
    assert.equal(initial.snapshots.configRuntime, null);

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    store.recordRefreshMetrics({ tab: 'diagnostics', reason: 'manual', fetchLatencyMs: 100, renderTimingMs: 10 });
    store.recordRefreshMetrics({ tab: 'diagnostics', reason: 'manual', fetchLatencyMs: 200, renderTimingMs: 20 });
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
    assert.equal(telemetry.refresh.lastTab, 'diagnostics');
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

test('dashboard store heartbeat controller reset clears failure budget without inventing a failure', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });

    store.recordHeartbeatSuccess({ requestId: 'hb-start', path: '/admin/session', method: 'GET' });
    store.recordHeartbeatFailure({
      requestId: 'hb-fail-1',
      path: '/admin/session',
      method: 'GET',
      failureClass: 'transport',
      error: 'network unreachable'
    });
    store.recordHeartbeatControllerReset({
      reason: 'session_cleared'
    });

    const telemetry = store.getRuntimeTelemetry();
    assert.equal(telemetry.connection.state, 'disconnected');
    assert.equal(telemetry.connection.consecutiveFailures, 0);
    assert.equal(telemetry.connection.lastTransitionReason, 'session_cleared');
    assert.equal(telemetry.heartbeat.consecutiveFailures, 0);
    assert.equal(telemetry.heartbeat.lastFailureClass, '');
    assert.equal(telemetry.heartbeat.lastFailureError, '');
    assert.equal(telemetry.heartbeat.lastTransitionReason, 'session_cleared');
    assert.equal(
      telemetry.heartbeat.breadcrumbs[telemetry.heartbeat.breadcrumbs.length - 1].eventType,
      'controller_reset'
    );
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
        analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
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
      deriveMonitoringAnalytics: () => ({ ban_count: 0, shadow_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshMonitoringTab('manual');
    assert.equal(fullFetchCount, 1);
    assert.deepEqual(callOrder.slice(0, 2), ['monitoring_delta', 'monitoring_full']);
    assert.equal(deltaCalls.length, 1);
    assert.equal(deltaCalls[0].limit, 40);

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
    let monitoringDeltaBootstrapCalls = 0;
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
            analytics: { ban_count: 1, shadow_mode: false, fail_mode: 'open' },
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
        if (Number(params.limit || 0) === 40) {
          monitoringDeltaBootstrapCalls += 1;
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
      deriveMonitoringAnalytics: () => ({ ban_count: 1, shadow_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshDashboardForTab('diagnostics', 'auto-refresh');
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(monitoringCalls, 1);
    assert.equal(monitoringDeltaBootstrapCalls, 1);
    assert.equal(monitoringDeltaCalls, 0);
    await runtime.refreshDashboardForTab('diagnostics', 'auto-refresh');
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

test('red-team auto-refresh refreshes monitoring-backed run snapshots without extra config reads', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'red-team' });
    store.setSnapshot('config', { adversary_sim_duration_seconds: 180 });
    store.setSnapshot('configRuntime', {
      runtime_environment: 'runtime-prod',
      gateway_deployment_profile: 'shared-server',
      admin_config_write_enabled: true,
      adversary_sim_available: true
    });
    const storage = {
      getItem() {
        return null;
      },
      setItem() {},
      removeItem() {}
    };

    const now = 1_700_000_240;
    let monitoringCalls = 0;
    let monitoringDeltaBootstrapCalls = 0;
    let monitoringDeltaCalls = 0;
    let configCalls = 0;
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
            analytics: { ban_count: 1, shadow_mode: false, fail_mode: 'open' },
            events: {
              recent_events: [{
                ts: now,
                event: 'Ban',
                ip: '198.51.100.1',
                reason: 'tarpit escalation',
                outcome: 'deny_temp',
                sim_run_id: 'simrun-red-team-1',
                sim_profile: 'runtime_toggle',
                sim_lane: 'deterministic_black_box',
                is_simulation: true
              }],
              recent_sim_runs: [{
                run_id: 'simrun-red-team-1',
                lane: 'deterministic_black_box',
                profile: 'runtime_toggle',
                first_ts: now,
                last_ts: now,
                monitoring_event_count: 1,
                defense_delta_count: 1,
                ban_outcome_count: 1
              }],
              event_counts: { Ban: 1 },
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
        if (Number(params.limit || 0) === 40) {
          monitoringDeltaBootstrapCalls += 1;
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
          events: [{
            ts: now + 1,
            event: 'Challenge',
            ip: '198.51.100.2',
            reason: 'challenge_required',
            outcome: 'challenge',
            sim_run_id: 'simrun-red-team-2',
            sim_profile: 'runtime_toggle',
            sim_lane: 'deterministic_black_box',
            is_simulation: true
          }],
          recent_sim_runs: [
            {
              run_id: 'simrun-red-team-2',
              lane: 'deterministic_black_box',
              profile: 'runtime_toggle',
              first_ts: now + 1,
              last_ts: now + 1,
              monitoring_event_count: 1,
              defense_delta_count: 1,
              ban_outcome_count: 0
            },
            {
              run_id: 'simrun-red-team-1',
              lane: 'deterministic_black_box',
              profile: 'runtime_toggle',
              first_ts: now,
              last_ts: now,
              monitoring_event_count: 1,
              defense_delta_count: 1,
              ban_outcome_count: 1
            }
          ],
          freshness: { state: 'fresh', lag_ms: 0 }
        };
      },
      async getConfig() {
        configCalls += 1;
        return {
          runtime_environment: 'runtime-prod',
          gateway_deployment_profile: 'shared-server',
          local_prod_direct_mode: false,
          admin_config_write_enabled: true,
          kv_store_fail_open: true
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 1, shadow_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshDashboardForTab('red-team', 'auto-refresh');
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(monitoringCalls, 1);
    assert.equal(monitoringDeltaBootstrapCalls, 1);
    assert.equal(monitoringDeltaCalls, 0);
    assert.equal(configCalls, 0);

    await runtime.refreshDashboardForTab('red-team', 'auto-refresh');
    assert.equal(monitoringCalls, 1);
    assert.equal(monitoringDeltaCalls, 1);
    assert.equal(configCalls, 0);

    const eventsSnapshot = store.getSnapshot('events') || {};
    assert.equal(Array.isArray(eventsSnapshot.recent_events), true);
    assert.equal(
      eventsSnapshot.recent_events.some((entry) => String(entry.sim_run_id || '') === 'simrun-red-team-2'),
      true
    );
    assert.equal(Array.isArray(eventsSnapshot.recent_sim_runs), true);
    assert.equal(
      eventsSnapshot.recent_sim_runs.some((entry) => String(entry.run_id || '') === 'simrun-red-team-1'),
      true
    );
    assert.equal(
      eventsSnapshot.recent_sim_runs.some((entry) => String(entry.run_id || '') === 'simrun-red-team-2'),
      true
    );
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
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
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
      deriveMonitoringAnalytics: () => ({ ban_count: 0, shadow_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshMonitoringTab('manual');
    await runtime.refreshMonitoringTab('manual');

    assert.equal(deltaCalls.length, 2);
    assert.equal(deltaCalls[0].limit, 40);
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
      'shuma_dashboard_cache_monitoring_v2',
      JSON.stringify({
        cachedAt: Date.now(),
        payload: {
          monitoring: {
            summary: {},
            details: {
              analytics: { ban_count: 100, shadow_mode: false, fail_mode: 'open' },
              events: { recent_events: [], recent_sim_runs: [] },
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
            analytics: { ban_count: 164, shadow_mode: false, fail_mode: 'open' },
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
      deriveMonitoringAnalytics: (_configSnapshot, _configRuntimeSnapshot, analyticsResponse = {}) => ({
        ban_count: Number(analyticsResponse.ban_count || 0),
        shadow_mode: analyticsResponse.shadow_mode === true,
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
    let configFetchCount = 0;
    let monitoringFetchCount = 0;
    let deferredSeedCursorCount = 0;

    const apiClient = {
      async getMonitoring() {
        monitoringFetchCount += 1;
        return {
          summary: {},
          freshness: { state: 'fresh', transport: 'snapshot_poll' },
          details: {
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
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
          config: {},
          runtime: {
            admin_config_write_enabled: true,
            runtime_environment: 'runtime-prod'
          }
        };
      },
      async getMonitoringDelta(params = {}) {
        if (Number(params.limit || 0) === 1) {
          deferredSeedCursorCount += 1;
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
        shadow_mode: analyticsResponse.shadow_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage: null
    });

    const refreshCompleted = await Promise.race([
      runtime.refreshDashboardForTab('diagnostics', 'session-restored').then(() => true),
      new Promise((resolve) => setTimeout(() => resolve(false), 50))
    ]);

    assert.equal(refreshCompleted, true);
    assert.equal(monitoringFetchCount, 1);
    assert.equal(configFetchCount, 1);
    assert.equal(deferredSeedCursorCount, 1);
    assert.equal(
      (store.getSnapshot('configRuntime') || {}).admin_config_write_enabled,
      true
    );
  });
});

test('red team auto-refresh rehydrates missing config runtime write truth for lane controls', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'red-team' });
    store.setSnapshot('config', { adversary_sim_duration_seconds: 180 });
    store.setSnapshot('configRuntime', {
      runtime_environment: 'runtime-prod',
      adversary_sim_available: true
    });

    let monitoringCallCount = 0;
    let configCallCount = 0;
    const apiClient = {
      async getMonitoring() {
        monitoringCallCount += 1;
        return {
          summary: {},
          freshness: { state: 'fresh', transport: 'snapshot_poll' },
          details: {
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
            events: { recent_events: [] },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getConfig() {
        configCallCount += 1;
        return {
          config: { adversary_sim_duration_seconds: 180 },
          runtime: {
            runtime_environment: 'runtime-prod',
            gateway_deployment_profile: 'shared-server',
            admin_config_write_enabled: true,
            adversary_sim_available: true
          }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: (_configSnapshot, _configRuntimeSnapshot, analyticsResponse = {}) => ({
        ban_count: Number(analyticsResponse.ban_count || 0),
        shadow_mode: analyticsResponse.shadow_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage: null
    });

    await runtime.refreshRedTeamTab('auto-refresh');

    assert.equal(monitoringCallCount, 1);
    assert.equal(configCallCount, 1);
    assert.equal(
      (store.getSnapshot('configRuntime') || {}).admin_config_write_enabled,
      true
    );
  });
});

test('dashboard refresh runtime preserves unavailable ban-state markers instead of coercing them to zero', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'diagnostics' });
    let ipBansDeltaCallCount = 0;
    const apiClient = {
      async getMonitoring() {
        return {
          summary: {},
          freshness: { state: 'fresh', transport: 'snapshot_poll' },
          details: {
            analytics: {
              ban_count: null,
              ban_store_status: 'unavailable',
              ban_store_message: 'authoritative backend access required',
              shadow_mode: false,
              fail_mode: 'closed'
            },
            events: { recent_events: [] },
            bans: {
              bans: [],
              status: 'unavailable',
              message: 'authoritative backend access required'
            },
            maze: { total_hits: 0, unique_crawlers: 0, maze_auto_bans: null, top_crawlers: [] },
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getMonitoringBootstrap() {
        return this.getMonitoring();
      },
      async getConfig() {
        return {
          config: {},
          runtime: {
            admin_config_write_enabled: true,
            runtime_environment: 'runtime-prod'
          }
        };
      },
      async getBans() {
        return { bans: [], status: 'available', message: '' };
      },
      async getIpRangeSuggestions() {
        return { summary: {}, suggestions: [] };
      },
      async getIpBansDelta() {
        ipBansDeltaCallCount += 1;
        return {
          after_cursor: '',
          window_end_cursor: 'cursor-1',
          next_cursor: 'cursor-1',
          has_more: false,
          overflow: 'none',
          events: [],
          active_bans: [],
          active_bans_status: ipBansDeltaCallCount > 1 ? 'unavailable' : 'available',
          active_bans_message: ipBansDeltaCallCount > 1 ? 'authoritative backend access required' : '',
          freshness: { state: 'fresh', transport: 'cursor_delta_poll' }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: (_configSnapshot, _configRuntimeSnapshot, analyticsResponse = {}) => ({
        ban_count: analyticsResponse.ban_count ?? null,
        ban_store_status: String(analyticsResponse.ban_store_status || 'available'),
        ban_store_message: String(analyticsResponse.ban_store_message || ''),
        shadow_mode: analyticsResponse.shadow_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage: null
    });

    await runtime.refreshMonitoringTab('manual-refresh');
    assert.equal((store.getSnapshot('analytics') || {}).ban_count, null);
    assert.equal((store.getSnapshot('analytics') || {}).ban_store_status, 'unavailable');
    assert.equal((store.getSnapshot('bans') || {}).status, 'unavailable');

    await runtime.refreshIpBansTab('manual-refresh');
    await runtime.refreshIpBansTab('auto-refresh');
    assert.equal((store.getSnapshot('bans') || {}).status, 'unavailable');
    assert.equal(
      (store.getSnapshot('bans') || {}).message,
      'authoritative backend access required'
    );
  });
});

test('monitoring tab shows bootstrap telemetry before slow full monitoring details resolve', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    let resolveBootstrapMonitoring;
    const bootstrapMonitoringPromise = new Promise((resolve) => {
      resolveBootstrapMonitoring = resolve;
    });
    let resolveFullMonitoring;
    const fullMonitoringPromise = new Promise((resolve) => {
      resolveFullMonitoring = resolve;
    });
    let bootstrapFetchCount = 0;
    let deltaFetchCount = 0;
    let fullFetchCount = 0;

    const apiClient = {
      async getMonitoringBootstrap() {
        bootstrapFetchCount += 1;
        return bootstrapMonitoringPromise;
      },
      async getMonitoring() {
        fullFetchCount += 1;
        return fullMonitoringPromise;
      },
      async getConfig() {
        return {
          config: {},
          runtime: {
            admin_config_write_enabled: true,
            runtime_environment: 'runtime-prod'
          }
        };
      },
      async getMonitoringDelta() {
        deltaFetchCount += 1;
        return {
          after_cursor: '',
          window_end_cursor: '00000000000000000002|eventlog:v2:1:2-b',
          next_cursor: '00000000000000000002|eventlog:v2:1:2-b',
          has_more: false,
          overflow: 'none',
          events: [
            {
              ts: 2,
              event: 'Challenge',
              reason: 'challenge_served',
              outcome: 'served'
            }
          ],
          freshness: { state: 'fresh', transport: 'cursor_delta_bootstrap' }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: (_configSnapshot, _configRuntimeSnapshot, analyticsResponse = {}) => ({
        ban_count: Number(analyticsResponse.ban_count || 0),
        shadow_mode: analyticsResponse.shadow_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage: null
    });

    const refreshCompleted = await Promise.race([
      runtime.refreshDashboardForTab('diagnostics', 'manual').then(() => true),
      new Promise((resolve) => setTimeout(() => resolve(false), 50))
    ]);

    assert.equal(refreshCompleted, true);
    assert.equal(deltaFetchCount, 1);
    assert.equal(bootstrapFetchCount, 1);
    assert.equal(fullFetchCount, 0);
    assert.equal(
      (store.getSnapshot('events') || {}).recent_events?.[0]?.event,
      'Challenge'
    );

    resolveBootstrapMonitoring({
          summary: { requests_total: 4 },
          freshness: { state: 'fresh', transport: 'snapshot_poll' },
          window_end_cursor: '00000000000000000002|eventlog:v2:1:2-b',
          details: {
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
            events: {
              recent_events: [
                {
                  ts: 2,
                  event: 'Challenge',
                  reason: 'challenge_served',
                  outcome: 'served'
                }
              ],
              event_counts: {},
              top_ips: []
            },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        });

    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal((store.getSnapshot('analytics') || {}).ban_count, 0);
    assert.equal(fullFetchCount, 1);
    assert.equal((store.getSnapshot('events') || {}).recent_events?.[0]?.event, 'Challenge');

    resolveFullMonitoring({
      summary: { requests_total: 8 },
      freshness: { state: 'fresh', transport: 'snapshot_poll' },
      details: {
        analytics: { ban_count: 1, shadow_mode: false, fail_mode: 'open' },
        events: {
          recent_events: [
            {
              ts: 3,
              event: 'Ban',
              reason: 'rate_limit_exceeded',
              outcome: 'blocked'
            }
          ],
          event_counts: { Ban: 1 },
          top_ips: [['203.0.113.4', 1]]
        },
        bans: { bans: [{ ip: '203.0.113.4' }] },
        maze: {},
        cdp: {},
        cdp_events: { events: [] }
      }
    });

    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal((store.getSnapshot('analytics') || {}).ban_count, 1);
    assert.equal((store.getSnapshot('events') || {}).recent_events?.[0]?.event, 'Ban');
  });
});

test('monitoring tab surfaces bootstrap failure as a tab-scoped error when delta bootstrap already rendered telemetry', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const apiClient = {
      async getMonitoringBootstrap() {
        const error = new Error('monitoring pipeline unavailable');
        error.status = 503;
        throw error;
      },
      async getMonitoringDelta() {
        return {
          after_cursor: '',
          window_end_cursor: '00000000000000000002|eventlog:v2:1:2-b',
          next_cursor: '00000000000000000002|eventlog:v2:1:2-b',
          has_more: false,
          overflow: 'none',
          events: [
            {
              ts: 2,
              event: 'Challenge',
              reason: 'challenge_served',
              outcome: 'served'
            }
          ],
          freshness: { state: 'fresh', transport: 'cursor_delta_bootstrap' }
        };
      },
      async getConfig() {
        return {
          config: {},
          runtime: {
            admin_config_write_enabled: true,
            runtime_environment: 'runtime-prod'
          }
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: (_configSnapshot, _configRuntimeSnapshot, analyticsResponse = {}) => ({
        ban_count: Number(analyticsResponse.ban_count || 0),
        shadow_mode: analyticsResponse.shadow_mode === true,
        fail_mode: String(analyticsResponse.fail_mode || 'open')
      }),
      storage: null
    });

    await runtime.refreshDashboardForTab('diagnostics', 'manual');
    await new Promise((resolve) => setTimeout(resolve, 0));

    const monitoringStatus = store.getState().tabStatus.diagnostics;
    assert.equal(monitoringStatus.loading, false);
    assert.equal(monitoringStatus.error, 'monitoring pipeline unavailable');
    assert.equal((store.getSnapshot('events') || {}).recent_events?.[0]?.event, 'Challenge');
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

    let deltaShouldFail = true;
    let fullShouldFail = true;
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
        if (fullShouldFail) {
          fullShouldFail = false;
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
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
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
        if (deltaShouldFail) {
          deltaShouldFail = false;
          throw new Error('bootstrap delta failure');
        }
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
      deriveMonitoringAnalytics: () => ({ ban_count: 0, shadow_mode: false, fail_mode: 'open' }),
      storage
    });

    await runtime.refreshDashboardForTab('diagnostics', 'manual-refresh');
    const failedStatus = store.getState().tabStatus.diagnostics;
    assert.equal(failedStatus.loading, false);
    assert.equal(Boolean(String(failedStatus.error || '').trim()), true);
    assert.equal(
      Number((store.getSnapshot('monitoringFreshness') || {}).last_event_ts || 0),
      1_700_000_000
    );

    await runtime.refreshDashboardForTab('diagnostics', 'manual-refresh');
    const recoveredStatus = store.getState().tabStatus.diagnostics;
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

test('monitoring refresh preserves extended operator summary families in the dashboard snapshot path', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const apiClient = {
      async getConfig() {
        return {
          config: {},
          runtime: {
            admin_config_write_enabled: true,
            runtime_environment: 'runtime-prod'
          }
        };
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
        return {
          freshness: {
            state: 'fresh',
            lag_ms: 0,
            last_event_ts: 1_700_000_050,
            slow_consumer_lag_state: 'normal',
            overflow: 'none',
            transport: 'snapshot_poll'
          },
          summary: {
            honeypot: {},
            challenge: {},
            not_a_bot: {},
            pow: {},
            rate: {},
            geo: {},
            human_friction: {
              segments: [{
                execution_mode: 'enforced',
                segment: 'likely_human',
                denominator_requests: 1,
                friction_requests: 1,
                not_a_bot_requests: 1,
                challenge_requests: 0,
                js_challenge_requests: 0,
                maze_requests: 0,
                not_a_bot_rate: 1,
                challenge_rate: 0,
                js_challenge_rate: 0,
                maze_rate: 0,
                friction_rate: 1
              }]
            },
            defence_funnel: {
              rows: [{
                execution_mode: 'enforced',
                family: 'not_a_bot',
                candidate_requests: 1,
                triggered_requests: 1,
                friction_requests: 1,
                passed_requests: 1,
                failed_requests: 0,
                escalated_requests: 0,
                denied_requests: null,
                suspicious_forwarded_requests: null,
                likely_human_affected_requests: 1
              }]
            },
            request_outcomes: {
              by_scope: [],
              by_lane: [],
              by_response_kind: [{
                traffic_origin: 'live',
                measurement_scope: 'ingress_primary',
                execution_mode: 'enforced',
                value: 'not_a_bot',
                total_requests: 1,
                forwarded_requests: 0,
                short_circuited_requests: 1,
                control_response_requests: 0,
                response_bytes: 45,
                forwarded_response_bytes: 0,
                short_circuited_response_bytes: 45,
                control_response_bytes: 0
              }],
              by_policy_source: [],
              by_route_action_family: []
            }
          },
          details: {
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
            events: { recent_events: [], recent_sim_runs: [] },
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
          window_end_cursor: '',
          next_cursor: '',
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
          window_end_cursor: '',
          next_cursor: '',
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
      deriveMonitoringAnalytics: () => ({ ban_count: 0, shadow_mode: false, fail_mode: 'open' }),
      storage: null
    });

    await runtime.refreshDashboardForTab('diagnostics', 'manual-refresh');
    await new Promise((resolve) => setTimeout(resolve, 0));

    const monitoringSnapshot = store.getSnapshot('monitoring') || {};
    assert.equal(monitoringSnapshot.summary?.human_friction?.segments?.[0]?.segment, 'likely_human');
    assert.equal(monitoringSnapshot.summary?.defence_funnel?.rows?.[0]?.family, 'not_a_bot');
    assert.equal(
      monitoringSnapshot.summary?.request_outcomes?.by_response_kind?.[0]?.value,
      'not_a_bot'
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
    const enforcedChartRows = monitoringModelModule.deriveEnforcedMonitoringChartRows([
      { event: 'challenge', ip: '198.51.100.10' },
      { event: 'challenge', ip: '198.51.100.10', execution_mode: 'shadow', enforcement_applied: false },
      { event: 'ban', ip: '198.51.100.10' },
      { event: 'pow', ip: '203.0.113.77' }
    ]);
    assert.deepEqual(enforcedChartRows.eventCounts, {
      challenge: 1,
      ban: 1,
      pow: 1
    });
    assert.deepEqual(enforcedChartRows.topIps, [
      ['198.51.100.10', 2],
      ['203.0.113.77', 1]
    ]);
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
    assert.equal(Object.hasOwn(tarpitSummary, 'enabled'), false);
    assert.equal(tarpitSummary.activationsProgressive, '11');
    assert.equal(tarpitSummary.progressAdvanced, '7');
    assert.equal(Object.hasOwn(tarpitSummary, 'topActiveBucket'), false);
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
      'source=custom source_id=manual-block action=forbidden_403 matched_cidr=203.0.113.0/24',
      {
        level: 'L11',
        action: 'A_DENY_HARD',
        detection: 'D_IP_RANGE_FORBIDDEN',
        signals: ['S_IP_RANGE_CUSTOM']
      }
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
        outcome: 'source=custom source_id=manual-block action=forbidden_403 matched_cidr=203.0.113.0/24',
        taxonomy: {
          level: 'L11',
          action: 'A_DENY_HARD',
          detection: 'D_IP_RANGE_FORBIDDEN',
          signals: ['S_IP_RANGE_CUSTOM']
        }
      },
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_maze_fallback_block',
        outcome: 'source=custom source_id=manual-bad-range action=maze matched_cidr=198.51.100.0/24',
        taxonomy: {
          level: 'L10',
          action: 'A_DENY_TEMP',
          detection: 'D_IP_RANGE_MAZE',
          signals: ['S_IP_RANGE_CUSTOM']
        }
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

    const summarizedRuns = monitoringModelModule.deriveAdversaryRunRowsFromSummaries([
      {
        run_id: 'run-summary-2',
        lane: 'crawler',
        profile: 'fast_smoke',
        first_ts: 1710000100,
        last_ts: 1710000200,
        monitoring_event_count: 11,
        defense_delta_count: 2,
        ban_outcome_count: 1
      },
      {
        run_id: 'run-summary-1',
        lane: 'browser_realistic',
        profile: 'full_coverage',
        first_ts: 1710000000,
        last_ts: 1710000050,
        monitoring_event_count: 3,
        defense_delta_count: 1,
        ban_outcome_count: 0
      }
    ], [
      { ip: '198.51.100.200' }
    ]);
    assert.deepEqual(
      summarizedRuns.runRows.map((row) => row.runId),
      ['run-summary-2', 'run-summary-1']
    );
    assert.equal(summarizedRuns.runRows[0].defenseDeltaCount, 2);
    assert.equal(summarizedRuns.activeBanCount, 1);

    const compactBotnessDisplay = monitoringModelModule.deriveMonitoringEventDisplay({
      ts: 1710000040,
      event: 'challenge',
      reason: 'botness_gate_not_a_bot',
      outcome_code: 'served',
      botness_score: 4
    });
    assert.equal(compactBotnessDisplay.event, 'Not-a-Bot');
    assert.equal(compactBotnessDisplay.outcome, 'Served');
    assert.equal(compactBotnessDisplay.outcomeToken, 'served');

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
      shadow_mode: false,
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
    const configRuntimeSnapshot = {
      kv_store_fail_open: true,
      runtime_environment: 'runtime-prod',
      gateway_deployment_profile: 'shared-server',
      local_prod_direct_mode: true,
      admin_config_write_enabled: false
    };
    const before = JSON.stringify(configSnapshot);
    const derived = statusModule.deriveStatusSnapshot(configSnapshot, configRuntimeSnapshot);
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
    const testModeItem = statusItems.find((item) => stripHtml(item.title) === 'Shadow Mode');
    assert.equal(Boolean(challengePuzzleItem), true);
    assert.equal(Boolean(challengeNotABotItem), true);
    assert.equal(Boolean(tarpitItem), true);
    assert.equal(Boolean(ipRangeItem), true);
    assert.equal(Boolean(runtimePostureItem), true);
    assert.equal(Boolean(adminWritePostureItem), true);
    assert.equal(statusItems.some((item) => stripHtml(item.title) === 'Retention and Freshness Health'), false);
    assert.equal(Boolean(testModeItem), false);
    assert.equal(challengePuzzleItem?.status, 'ENABLED');
    assert.equal(challengeNotABotItem?.status, 'ENABLED');
    assert.equal(tarpitItem?.status, 'ENABLED');
    assert.equal(ipRangeItem?.status, 'LOGGING-ONLY');
    assert.equal(runtimePostureItem?.status, 'RUNTIME-PROD / LOCAL-DIRECT');
    assert.equal(adminWritePostureItem?.status, 'DISABLED');
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

test('status refresh hydrates monitoring retention/freshness and ip-ban freshness snapshots without tab bootstrap', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const refreshModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-runtime-refresh.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const store = storeModule.createDashboardStore({ initialTab: 'status' });
    let configCalls = 0;
    let monitoringCalls = 0;
    let ipBansDeltaCalls = 0;
    const apiClient = {
      async getConfig() {
        configCalls += 1;
        return {
          config: {},
          runtime: {
            runtime_environment: 'runtime-prod',
            gateway_deployment_profile: 'shared-server',
            local_prod_direct_mode: false,
            admin_config_write_enabled: true,
            kv_store_fail_open: true
          }
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
            analytics: { ban_count: 0, shadow_mode: false, fail_mode: 'open' },
            events: { recent_events: [] },
            bans: { bans: [] },
            maze: {},
            cdp: {},
            cdp_events: { events: [] }
          }
        };
      },
      async getIpBansDelta(params = {}) {
        ipBansDeltaCalls += 1;
        assert.equal(Number(params.limit || 0), 1);
        assert.equal(String(params.after_cursor || ''), '');
        return {
          active_bans: [],
          freshness: {
            state: 'degraded',
            lag_ms: 250,
            last_event_ts: 1_700_000_005,
            transport: 'cursor_delta_poll'
          },
          next_cursor: '',
          has_more: false,
          overflow: 'none'
        };
      }
    };

    const runtime = refreshModule.createDashboardRefreshRuntime({
      normalizeTab: (value) => String(value || ''),
      getApiClient: () => apiClient,
      getStateStore: () => store,
      deriveMonitoringAnalytics: () => ({ ban_count: 0, shadow_mode: false, fail_mode: 'open' })
    });

    await runtime.refreshDashboardForTab('status', 'manual-refresh');

    assert.equal(configCalls, 1);
    assert.equal(monitoringCalls, 1);
    assert.equal(ipBansDeltaCalls, 1);
    assert.equal((store.getSnapshot('monitoring') || {}).retention_health?.state, 'healthy');
    assert.equal((store.getSnapshot('monitoringFreshness') || {}).state, 'fresh');
    assert.equal((store.getSnapshot('ipBansFreshness') || {}).state, 'degraded');
    assert.equal((store.getSnapshot('ipBansFreshness') || {}).transport, 'cursor_delta_poll');
  });
});

test('dashboard class runtime keeps runtime, shadow-mode, and adversary-sim state on html only', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const bodyClassModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-body-classes.js');

    const defaultState = bodyClassModule.deriveDashboardBodyClassState({});
    assert.deepEqual(toPlain(defaultState), {
      runtimeClass: '',
      shadowModeEnabled: false,
      adversarySimEnabled: false,
      connectionState: 'disconnected'
    });

    const explicitDevState = bodyClassModule.deriveDashboardBodyClassState({
      runtime_environment: 'runtime-dev',
      shadow_mode: true,
      adversary_sim_enabled: true
    }, {
      backendConnectionState: 'connected'
    });
    assert.deepEqual(toPlain(explicitDevState), {
      runtimeClass: 'runtime-dev',
      shadowModeEnabled: true,
      adversarySimEnabled: false,
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
    const liveOverrideState = bodyClassModule.deriveDashboardBodyClassState({
      runtime_environment: 'runtime-prod',
      shadow_mode: false,
      adversary_sim_enabled: false
    }, {
      shadowModeEnabled: true,
      adversarySimEnabled: true
    });
    assert.deepEqual(toPlain(liveOverrideState), {
      runtimeClass: 'runtime-prod',
      shadowModeEnabled: true,
      adversarySimEnabled: true,
      connectionState: 'disconnected'
    });

    const classList = createMutableClassList(['runtime-prod', 'shadow-mode', 'adversary-sim', 'connected']);
    const rootClassList = createMutableClassList(['runtime-prod', 'shadow-mode', 'adversary-sim', 'connected']);
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
      shadowModeEnabled: explicitDevState.shadowModeEnabled,
      adversarySimEnabled: explicitDevState.adversarySimEnabled,
      connectionState: explicitDevState.connectionState
    });
    assert.equal(classList.contains('runtime-dev'), false);
    assert.equal(classList.contains('runtime-prod'), false);
    assert.equal(classList.contains('shadow-mode'), false);
    assert.equal(classList.contains('adversary-sim'), false);
    assert.equal(classList.contains('connected'), false);
    assert.equal(classList.contains('degraded'), false);
    assert.equal(classList.contains('disconnected'), false);
    assert.equal(rootClassList.contains('runtime-dev'), true);
    assert.equal(rootClassList.contains('runtime-prod'), false);
    assert.equal(rootClassList.contains('shadow-mode'), true);
    assert.equal(rootClassList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('connected'), true);
    assert.equal(rootClassList.contains('degraded'), false);
    assert.equal(rootClassList.contains('disconnected'), false);

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-dev',
      shadowModeEnabled: true,
      adversarySimEnabled: false,
      connectionState: 'degraded'
    });
    assert.equal(classList.contains('shadow-mode'), false);
    assert.equal(classList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('shadow-mode'), true);
    assert.equal(rootClassList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('connected'), false);
    assert.equal(rootClassList.contains('degraded'), true);
    assert.equal(rootClassList.contains('disconnected'), false);

    bodyClassModule.syncDashboardBodyClasses(doc, {
      runtimeClass: 'runtime-prod',
      shadowModeEnabled: false,
      adversarySimEnabled: false,
      connectionState: 'disconnected'
    });
    assert.equal(classList.contains('runtime-dev'), false);
    assert.equal(classList.contains('runtime-prod'), false);
    assert.equal(classList.contains('shadow-mode'), false);
    assert.equal(classList.contains('adversary-sim'), false);
    assert.equal(classList.contains('connected'), false);
    assert.equal(classList.contains('degraded'), false);
    assert.equal(classList.contains('disconnected'), false);
    assert.equal(rootClassList.contains('runtime-dev'), false);
    assert.equal(rootClassList.contains('runtime-prod'), true);
    assert.equal(rootClassList.contains('shadow-mode'), false);
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
    assert.equal(rootClassList.contains('shadow-mode'), false);
    assert.equal(rootClassList.contains('adversary-sim'), false);
    assert.equal(rootClassList.contains('degraded'), false);
    assert.equal(rootClassList.contains('connected'), false);
    assert.equal(rootClassList.contains('disconnected'), true);

    bodyClassModule.clearDashboardBodyClasses(doc);
    assert.equal(classList.contains('runtime-dev'), false);
    assert.equal(classList.contains('runtime-prod'), false);
    assert.equal(classList.contains('shadow-mode'), false);
    assert.equal(classList.contains('adversary-sim'), false);
    assert.equal(classList.contains('connected'), false);
    assert.equal(classList.contains('degraded'), false);
    assert.equal(classList.contains('disconnected'), false);
    assert.equal(rootClassList.contains('runtime-dev'), false);
    assert.equal(rootClassList.contains('runtime-prod'), false);
    assert.equal(rootClassList.contains('shadow-mode'), false);
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
      desired_lane: 'scrapling_traffic',
      active_lane: 'synthetic_traffic',
      lane_switch_seq: 5,
      last_lane_switch_at: 1100,
      last_lane_switch_reason: 'beat_boundary_reconciliation',
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
        last_generation_error: '',
        truth_basis: 'persisted_event_lower_bound'
      },
      persisted_event_evidence: {
        run_id: 'simrun-123',
        lane: 'scrapling_traffic',
        profile: 'baseline',
        monitoring_event_count: 12,
        defense_delta_count: 2,
        ban_outcome_count: 1,
        first_observed_at: 1090,
        last_observed_at: 1100,
        truth_basis: 'persisted_event_lower_bound'
      },
      lane_diagnostics: {
        schema_version: 'v1',
        truth_basis: 'persisted_event_lower_bound',
        lanes: {
          synthetic_traffic: {
            beat_attempts: 6,
            beat_successes: 5,
            beat_failures: 1,
            generated_requests: 12,
            blocked_requests: 0,
            offsite_requests: 0,
            response_bytes: 4096,
            response_status_count: { '200': 12 },
            last_generated_at: 1100,
            last_error: ''
          },
          scrapling_traffic: {
            beat_attempts: 1,
            beat_successes: 0,
            beat_failures: 1,
            generated_requests: 0,
            blocked_requests: 2,
            offsite_requests: 1,
            response_bytes: 0,
            response_status_count: { '429': 1 },
            last_generated_at: 1099,
            last_error: 'timeout'
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
            last_generated_at: null,
            last_error: ''
          }
        },
        request_failure_classes: {
          cancelled: { count: 0, last_seen_at: null },
          timeout: { count: 1, last_seen_at: 1099 },
          transport: { count: 0, last_seen_at: null },
          http: { count: 0, last_seen_at: null }
        }
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
    assert.equal(normalized.startedAt, 1000);
    assert.equal(normalized.endsAt, 1180);
    assert.equal(normalized.activeRunCount, 1);
    assert.equal(normalized.activeLaneCount, 2);
    assert.equal(normalized.desiredLane, 'scrapling_traffic');
    assert.equal(normalized.activeLane, 'synthetic_traffic');
    assert.equal(normalized.laneSwitchSeq, 5);
    assert.equal(normalized.lastLaneSwitchAt, 1100);
    assert.equal(normalized.lastLaneSwitchReason, 'beat_boundary_reconciliation');
    assert.equal(normalized.remainingSeconds, 120);
    assert.equal(normalized.supervisor.owner, 'backend_autonomous_supervisor');
    assert.equal(normalized.supervisor.cadenceSeconds, 1);
    assert.equal(normalized.supervisor.heartbeatActive, true);
    assert.equal(normalized.generationDiagnostics.health, 'ok');
    assert.equal(normalized.generationDiagnostics.generatedTickCount, 3);
    assert.equal(normalized.generationDiagnostics.generatedRequestCount, 12);
    assert.equal(normalized.generationDiagnostics.truthBasis, 'persisted_event_lower_bound');
    assert.equal(normalized.laneDiagnostics.schemaVersion, 'v1');
    assert.equal(normalized.laneDiagnostics.truthBasis, 'persisted_event_lower_bound');
    assert.equal(normalized.laneDiagnostics.lanes.scraplingTraffic.generatedRequests, 0);
    assert.equal(normalized.laneDiagnostics.lanes.scraplingTraffic.lastError, 'timeout');
    assert.equal(normalized.laneDiagnostics.requestFailureClasses.timeout.count, 1);
    assert.equal(normalized.persistedEventEvidence.runId, 'simrun-123');
    assert.equal(normalized.persistedEventEvidence.monitoringEventCount, 12);

    const renormalized = adversaryModule.normalizeAdversarySimStatus(normalized);
    assert.equal(renormalized.enabled, true);
    assert.equal(renormalized.available, true);
    assert.equal(renormalized.durationSeconds, 180);
    assert.equal(renormalized.startedAt, 1000);
    assert.equal(renormalized.endsAt, 1180);
    assert.equal(renormalized.remainingSeconds, 120);
    assert.equal(renormalized.generationDiagnostics.health, 'ok');
    assert.equal(renormalized.generationDiagnostics.truthBasis, 'persisted_event_lower_bound');
    assert.equal(renormalized.desiredLane, 'scrapling_traffic');
    assert.equal(renormalized.activeLane, 'synthetic_traffic');
    assert.equal(renormalized.laneDiagnostics.lanes.syntheticTraffic.beatAttempts, 6);
    assert.equal(renormalized.laneDiagnostics.truthBasis, 'persisted_event_lower_bound');
    assert.equal(renormalized.persistedEventEvidence.runId, 'simrun-123');
  });
});

test('dashboard adversary-sim control availability follows explicit surface opt-in in both runtime classes', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');

    assert.deepEqual(
      adversaryModule.deriveAdversarySimControlState({
        configRuntimeSnapshot: {
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
        configRuntimeSnapshot: {
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
        configRuntimeSnapshot: {
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

test('dashboard adversary-sim desired-state matcher tolerates eventual-consistency settle windows', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');

    assert.equal(
      adversaryModule.adversarySimStateMatchesDesired({
        adversary_sim_enabled: true,
        phase: 'running'
      }, true),
      true
    );
    assert.equal(
      adversaryModule.adversarySimStateMatchesDesired({
        adversary_sim_enabled: false,
        generation_active: true,
        phase: 'running'
      }, true),
      false
    );
    assert.equal(
      adversaryModule.adversarySimStateMatchesDesired({
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off'
      }, false),
      true
    );
    assert.equal(
      adversaryModule.adversarySimStateMatchesDesired({
        adversary_sim_enabled: false,
        generation_active: true,
        phase: 'stopping'
      }, false),
      false
    );
  });
});

test('dashboard adversary-sim lifecycle copy prioritizes controller convergence over steady-state backend copy', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');

    assert.equal(
      adversaryModule.deriveAdversarySimLifecycleCopy({
        status: {
          adversary_sim_enabled: false,
          generation_active: false,
          historical_data_visible: true,
          history_retention: {
            retention_hours: 168,
            cleanup_command: 'make telemetry-clean'
          },
          phase: 'off'
        },
        controllerState: {
          controllerPhase: 'converging',
          uiDesiredEnabled: true
        }
      }),
      'Starting adversary simulation. Awaiting backend convergence.'
    );

    assert.equal(
      adversaryModule.deriveAdversarySimLifecycleCopy({
        status: {
          adversary_sim_enabled: true,
          generation_active: true,
          phase: 'running'
        },
        controllerState: {
          controllerPhase: 'submitting',
          uiDesiredEnabled: false
        }
      }),
      'Stopping adversary simulation. Awaiting backend convergence.'
    );

    assert.equal(
      adversaryModule.deriveAdversarySimLifecycleCopy({
        status: {
          adversary_sim_enabled: true,
          generation_active: true,
          historical_data_visible: true,
          history_retention: {
            retention_hours: 168,
            cleanup_command: 'make telemetry-clean'
          },
          phase: 'running',
          generation_diagnostics: {
            health: 'ok',
            recommended_action: 'No action required; simulation traffic is being generated.'
          }
        },
        controllerState: {
          controllerPhase: 'idle',
          uiDesiredEnabled: true
        }
      }),
      'Generation active. Auto-off stops new simulation traffic only; retained telemetry stays visible.'
    );

    assert.equal(
      adversaryModule.deriveAdversarySimLifecycleCopy({
        status: {
          adversary_sim_enabled: false,
          generation_active: false,
          historical_data_visible: true,
          history_retention: {
            retention_hours: 168,
            cleanup_command: 'make telemetry-clean'
          },
          phase: 'off'
        },
        controllerState: {
          controllerPhase: 'idle',
          uiDesiredEnabled: false
        }
      }),
      'Generation inactive. Retained telemetry remains visible for 168h or until make telemetry-clean is run.'
    );
  });
});

test('dashboard adversary-sim progress state uses backend run timing and resets immediately when intent turns off', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');

    assert.deepEqual(
      adversaryModule.deriveAdversarySimProgressState({
        status: {
          adversary_sim_enabled: true,
          generation_active: true,
          phase: 'running',
          started_at: 1000,
          ends_at: 1180,
          duration_seconds: 180
        },
        controllerState: {
          controllerPhase: 'idle',
          uiDesiredEnabled: true
        },
        nowMs: 1090_000
      }),
      {
        active: true,
        progressPercent: 50,
        remainingMs: 90_000
      }
    );

    assert.deepEqual(
      adversaryModule.deriveAdversarySimProgressState({
        status: {
          adversary_sim_enabled: true,
          generation_active: true,
          phase: 'running',
          started_at: 1000,
          ends_at: 1180,
          duration_seconds: 180
        },
        controllerState: {
          controllerPhase: 'converging',
          uiDesiredEnabled: false
        },
        nowMs: 1090_000
      }),
      {
        active: false,
        progressPercent: 0,
        remainingMs: 0
      }
    );
  });
});

test('dashboard adversary-sim control retry helper honors retryable lease/throttle responses before succeeding', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');
    const attempts = [];
    const sleeps = [];
    let callCount = 0;

    const result = await adversaryModule.controlAdversarySimWithRetry(
      async (desiredEnabled) => {
        callCount += 1;
        attempts.push(desiredEnabled);
        if (callCount === 1) {
          const error = new Error('lease held');
          error.status = 409;
          error.retryAfterSeconds = 3;
          throw error;
        }
        if (callCount === 2) {
          const error = new Error('throttled');
          error.status = 429;
          error.retryAfterSeconds = 1;
          throw error;
        }
        return { ok: true };
      },
      true,
      {
        timeoutMs: 10_000,
        sleep: async (ms) => {
          sleeps.push(ms);
        }
      }
    );

    assert.deepEqual(attempts, [true, true, true]);
    assert.deepEqual(sleeps, [3250, 1250]);
    assert.deepEqual(result, { ok: true });
  });
});

test('dashboard adversary-sim control retry helper does not swallow non-retryable errors', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adversaryModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-adversary-sim.js');
    let callCount = 0;

    await assert.rejects(
      () => adversaryModule.controlAdversarySimWithRetry(
        async () => {
          callCount += 1;
          const error = new Error('bad request');
          error.status = 400;
          throw error;
        },
        false,
        {
          timeoutMs: 5_000,
          sleep: async () => {
            throw new Error('sleep should not be called');
          }
        }
      ),
      /bad request/
    );

    assert.equal(callCount, 1);
  });
});

test('dashboard red team controller flips desired state immediately and drops a rapid reversal back to confirmed backend state', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    let controllerModule = null;
    try {
      controllerModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-red-team-controller.js');
    } catch (error) {
      assert.fail(`dashboard red team controller module is missing: ${error.message}`);
    }

    const scheduler = createManualScheduler();
    const controlCalls = [];
    const controller = controllerModule.createDashboardRedTeamController({
      initialStatus: {
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off'
      },
      debounceMs: 200,
      nowMs: () => scheduler.nowMs(),
      schedule: (callback, delayMs) => scheduler.schedule(callback, delayMs),
      cancelScheduled: (id) => scheduler.cancelScheduled(id),
      submitControl: async (desiredEnabled) => {
        controlCalls.push(desiredEnabled);
        return {
          status: {
            adversary_sim_enabled: desiredEnabled,
            generation_active: desiredEnabled,
            phase: desiredEnabled ? 'running' : 'off'
          }
        };
      },
      fetchStatus: async () => ({
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off'
      })
    });

    controller.handleToggleIntent(true);
    assert.equal(controller.getState().uiDesiredEnabled, true);
    assert.equal(controller.getState().controllerPhase, 'debouncing');
    assert.deepEqual(controlCalls, []);

    controller.handleToggleIntent(false);
    assert.equal(controller.getState().uiDesiredEnabled, false);

    await scheduler.advanceBy(200);

    assert.deepEqual(controlCalls, []);
    assert.equal(controller.getState().controllerPhase, 'idle');
    assert.equal(controller.getState().uiDesiredEnabled, false);
  });
});

test('dashboard red team controller preserves latest intent when a status poll lands during debounce', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    let controllerModule = null;
    try {
      controllerModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-red-team-controller.js');
    } catch (error) {
      assert.fail(`dashboard red team controller module is missing: ${error.message}`);
    }

    const scheduler = createManualScheduler();
    const controlCalls = [];
    const fetchReasons = [];
    const controller = controllerModule.createDashboardRedTeamController({
      initialStatus: {
        adversary_sim_enabled: true,
        generation_active: true,
        phase: 'running'
      },
      debounceMs: 300,
      pollIntervalMs: 1000,
      nowMs: () => scheduler.nowMs(),
      schedule: (callback, delayMs) => scheduler.schedule(callback, delayMs),
      cancelScheduled: (id) => scheduler.cancelScheduled(id),
      submitControl: async (desiredEnabled) => {
        controlCalls.push(desiredEnabled);
        return {
          status: {
            adversary_sim_enabled: desiredEnabled,
            generation_active: desiredEnabled,
            phase: desiredEnabled ? 'running' : 'off'
          }
        };
      },
      fetchStatus: async (reason) => {
        fetchReasons.push(reason);
        return {
          adversary_sim_enabled: true,
          generation_active: true,
          phase: 'running'
        };
      }
    });

    await controller.bootstrap();
    await scheduler.advanceBy(750);

    controller.handleToggleIntent(false);
    assert.equal(controller.getState().uiDesiredEnabled, false);
    assert.equal(controller.getState().controllerPhase, 'debouncing');

    await scheduler.advanceBy(250);

    assert.deepEqual(fetchReasons, ['poll']);
    assert.equal(controller.getState().uiDesiredEnabled, false);
    assert.equal(controller.getState().controllerPhase, 'debouncing');
    assert.deepEqual(controlCalls, []);

    await scheduler.advanceBy(50);

    assert.deepEqual(controlCalls, [false]);
    assert.equal(controller.getState().uiDesiredEnabled, false);
    assert.equal(controller.getState().controllerPhase, 'idle');
  });
});

test('dashboard red team controller queues a reversal while a control request is in flight', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    let controllerModule = null;
    try {
      controllerModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-red-team-controller.js');
    } catch (error) {
      assert.fail(`dashboard red team controller module is missing: ${error.message}`);
    }

    const scheduler = createManualScheduler();
    const controlCalls = [];
    let resolveFirstControl = null;
    const firstControlPromise = new Promise((resolve) => {
      resolveFirstControl = resolve;
    });
    const controller = controllerModule.createDashboardRedTeamController({
      initialStatus: {
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off'
      },
      debounceMs: 200,
      nowMs: () => scheduler.nowMs(),
      schedule: (callback, delayMs) => scheduler.schedule(callback, delayMs),
      cancelScheduled: (id) => scheduler.cancelScheduled(id),
      submitControl: async (desiredEnabled) => {
        controlCalls.push(desiredEnabled);
        if (controlCalls.length === 1) {
          return firstControlPromise;
        }
        return {
          status: {
            adversary_sim_enabled: false,
            generation_active: false,
            phase: 'off'
          }
        };
      },
      fetchStatus: async () => ({
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off'
      })
    });

    controller.handleToggleIntent(true);
    await scheduler.advanceBy(200);
    assert.deepEqual(controlCalls, [true]);
    assert.equal(controller.getState().controllerPhase, 'submitting');

    controller.handleToggleIntent(false);
    assert.equal(controller.getState().uiDesiredEnabled, false);

    await scheduler.advanceBy(200);
    assert.deepEqual(controlCalls, [true]);

    resolveFirstControl({
      status: {
        adversary_sim_enabled: true,
        generation_active: true,
        phase: 'running'
      }
    });
    await Promise.resolve();
    await Promise.resolve();
    await scheduler.advanceBy(0);
    await Promise.resolve();

    assert.deepEqual(controlCalls, [true, false]);
  });
});

test('dashboard red team controller reports when a submitted desired state has converged', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    let controllerModule = null;
    try {
      controllerModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-red-team-controller.js');
    } catch (error) {
      assert.fail(`dashboard red team controller module is missing: ${error.message}`);
    }

    const scheduler = createManualScheduler();
    const settledStates = [];
    const controller = controllerModule.createDashboardRedTeamController({
      initialStatus: {
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off'
      },
      debounceMs: 200,
      nowMs: () => scheduler.nowMs(),
      schedule: (callback, delayMs) => scheduler.schedule(callback, delayMs),
      cancelScheduled: (id) => scheduler.cancelScheduled(id),
      submitControl: async (desiredEnabled) => ({
        status: {
          adversary_sim_enabled: desiredEnabled,
          generation_active: desiredEnabled,
          phase: desiredEnabled ? 'running' : 'off'
        }
      }),
      fetchStatus: async () => ({
        adversary_sim_enabled: true,
        generation_active: true,
        phase: 'running'
      }),
      onSettled: (desiredEnabled, status) => {
        settledStates.push({
          desiredEnabled,
          phase: status?.phase
        });
      }
    });

    controller.handleToggleIntent(true);
    await scheduler.advanceBy(200);

    assert.deepEqual(settledStates, [
      {
        desiredEnabled: true,
        phase: 'running'
      }
    ]);
    assert.equal(controller.getState().controllerPhase, 'idle');
  });
});

test('dashboard red team controller can replace backend status after lane-only control writes', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const controllerModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-red-team-controller.js');

    const controller = controllerModule.createDashboardRedTeamController({
      initialStatus: {
        adversary_sim_enabled: false,
        generation_active: false,
        phase: 'off',
        desired_lane: 'synthetic_traffic',
        active_lane: null
      }
    });

    controller.replaceBackendStatus({
      adversary_sim_enabled: false,
      generation_active: false,
      phase: 'off',
      desired_lane: 'scrapling_traffic',
      active_lane: null,
      lane_switch_seq: 2
    });

    const state = controller.getState();
    assert.equal(state.uiDesiredEnabled, false);
    assert.equal(state.lastBackendDesiredEnabled, false);
    assert.equal(state.backendStatus.desired_lane, 'scrapling_traffic');
    assert.equal(state.backendStatus.lane_switch_seq, 2);
  });
});

test('dashboard global control helper enables authenticated writable controls before monitoring hydration completes', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const controlModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-global-controls.js');

    assert.equal(
      controlModule.deriveGlobalControlDisabled({
        runtimeMounted: true,
        loggingOut: false,
        saving: false,
        authenticated: true,
        adminConfigWritable: true,
        surfaceAvailable: true
      }),
      false
    );

    assert.equal(
      controlModule.deriveGlobalControlDisabled({
        runtimeMounted: false,
        loggingOut: false,
        saving: false,
        authenticated: true,
        adminConfigWritable: true,
        surfaceAvailable: true
      }),
      true
    );
  });
});

test('dashboard request budgets widen edge-fermyon control and monitoring timeouts', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const budgetModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-request-budgets.js');

    const edgeBudgets = budgetModule.deriveDashboardRequestBudgets({
      gateway_deployment_profile: 'edge-fermyon'
    });
    assert.equal(edgeBudgets.gatewayDeploymentProfile, 'edge-fermyon');
    assert.equal(edgeBudgets.autoHydrateFullMonitoring, false);
    assert.equal(edgeBudgets.monitoringRequestTimeoutMs, 45_000);
    assert.equal(edgeBudgets.monitoringDeltaTimeoutMs, 20_000);
    assert.equal(edgeBudgets.configWriteTimeoutMs, 45_000);
    assert.equal(edgeBudgets.adversarySimControlTimeoutMs, 45_000);
    assert.equal(edgeBudgets.adversarySimEnableTimeoutMs, 90_000);
    assert.equal(edgeBudgets.adversarySimDisableTimeoutMs, 45_000);
    assert.equal(edgeBudgets.adversarySimStatusTimeoutMs, 30_000);

    const sharedBudgets = budgetModule.deriveDashboardRequestBudgets({
      gateway_deployment_profile: 'shared-server'
    });
    assert.equal(sharedBudgets.gatewayDeploymentProfile, 'shared-server');
    assert.equal(sharedBudgets.autoHydrateFullMonitoring, true);
    assert.equal(sharedBudgets.monitoringRequestTimeoutMs, 12_000);
    assert.equal(sharedBudgets.monitoringDeltaTimeoutMs, 12_000);
    assert.equal(sharedBudgets.configWriteTimeoutMs, 12_000);
    assert.equal(sharedBudgets.adversarySimControlTimeoutMs, 15_000);
    assert.equal(sharedBudgets.adversarySimEnableTimeoutMs, 45_000);
    assert.equal(sharedBudgets.adversarySimDisableTimeoutMs, 30_000);
    assert.equal(sharedBudgets.adversarySimStatusTimeoutMs, 12_000);
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
    assert.equal(schema.advancedConfigTemplatePaths.includes('shadow_mode'), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('adversary_sim_enabled'), false);
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
    assert.equal(schema.advancedConfigTemplatePaths.includes('verified_identity.enabled'), true);
    assert.equal(
      schema.advancedConfigTemplatePaths.includes('verified_identity.non_human_traffic_stance'),
      true
    );
    assert.equal(schema.advancedConfigTemplatePaths.includes('verified_identity.named_policies'), true);
    assert.equal(
      schema.advancedConfigTemplatePaths.includes('verified_identity.service_profiles'),
      true
    );
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
    ['provider_backends', parseRustStructFieldNames(apiSource, 'AdminProviderBackendsPatch')],
    ['verified_identity', parseRustStructFieldNames(apiSource, 'AdminVerifiedIdentityPatch')]
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

  const runtimePayloadFnStart = apiSource.indexOf('fn admin_config_runtime_payload(');
  const runtimePayloadFnEnd = apiSource.indexOf('\nfn admin_config_response_payload(', runtimePayloadFnStart);
  if (runtimePayloadFnStart < 0 || runtimePayloadFnEnd < 0) {
    throw new Error('Unable to parse admin_config_runtime_payload function body');
  }
  const runtimePayloadFnSource = apiSource.slice(runtimePayloadFnStart, runtimePayloadFnEnd);
  const insertedReadOnlyTopLevelPaths = Array.from(runtimePayloadFnSource.matchAll(/obj\.insert\(\s*"([^"]+)"/g))
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
    'adversary_sim_enabled',
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

test('dashboard config tabs reuse shared panels, save flows, and owned controls', () => {
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
  const configPathAllowlistSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigPathAllowlistSection.svelte'),
    'utf8'
  );
  const monitoringCdpSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/CdpSection.svelte'),
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
  const statusSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/StatusTab.svelte'),
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
    configNetworkSource,
    configDurationsSource,
    configPathAllowlistSource,
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
  assert.match(ipBansSource, /import HalfDoughnutChart from '\.\/primitives\/HalfDoughnutChart\.svelte';/);
  assert.match(ipBansSource, /buildHalfDoughnutOptions/);
  assert.match(ipBansSource, /syncHalfDoughnutReadout/);
  assert.match(ipBansSource, /canvasId="ip-ban-reasons-chart"/);
  assert.match(ipBansSource, /shellClass="chart-canvas-shell--ip-bans"/);
  assert.match(ipBansSource, /readout=\{banReasonReadout\}/);
  assert.match(ipBansSource, /type: 'doughnut'/);
  assert.doesNotMatch(ipBansSource, /animation: false,/);
  assert.match(ipBansSource, /maintainAspectRatio: false,/);
  assert.match(ipBansSource, /resolveMonitoringChartTheme/);
  assert.match(ipBansSource, /canvasHasRenderableSize\(canvas\)/);
  assert.match(ipBansSource, /window\.addEventListener\('resize', onResize, \{ passive: true \}\);/);
  assert.match(ipBansSource, /if \(browser && nextActive && !wasActive\)/);
  assert.match(ipBansSource, /id="ip-range-suggestions-table"/);
  assert.match(ipBansSource, /id="bypass-allowlists-toggle"/);
  assert.match(ipBansSource, /id="network-allowlist"/);
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
  assert.match(ipBansSource, /disabled=\{!canBan\}/);
  assert.match(ipBansSource, /disabled=\{!canUnban\}/);
  assert.doesNotMatch(ipBansSource, /id="ip-bans-freshness-state"/);
  assert.doesNotMatch(ipBansSource, /id="ip-bans-freshness-meta"/);

  assert.match(statusSource, /export let monitoringFreshnessSnapshot = null;/);
  assert.match(statusSource, /export let ipBansFreshnessSnapshot = null;/);
  assert.match(statusSource, /class="admin-group admin-group--status"/);
  assert.doesNotMatch(statusSource, /<h3>Dashboard and Telemetry Health<\/h3>/);
  assert.match(statusSource, /<h3>Dashboard Connectivity<\/h3>/);
  assert.match(statusSource, /id="status-connection-state"/);
  assert.doesNotMatch(statusSource, /id="status-connection-reason"/);
  assert.match(statusSource, /id="status-connection-last-failure-class"/);
  assert.match(statusSource, /id="status-connection-ignored-cancelled"/);
  assert.match(statusSource, /id="status-connection-ignored-non-heartbeat"/);
  assert.match(statusSource, /id="status-connection-breadcrumbs"/);
  assert.match(statusSource, /<h3>Telemetry Delivery Health<\/h3>/);
  assert.match(statusSource, />Monitoring feed status:</);
  assert.match(statusSource, /id="status-monitoring-freshness-state"/);
  assert.doesNotMatch(statusSource, />Monitoring update path:</);
  assert.match(statusSource, />IP bans feed status:</);
  assert.match(statusSource, /id="status-ip-bans-freshness-state"/);
  assert.doesNotMatch(statusSource, />IP bans update path:</);
  assert.match(statusSource, /<h3>Retention Health<\/h3>/);
  assert.match(statusSource, /id="status-retention-health-state"/);
  assert.match(statusSource, /<h3>Runtime Performance Telemetry<\/h3>/);
  assert.match(statusSource, /Operator thresholds for auto-refresh tabs/);
  assert.match(statusSource, /<code>ip-bans<\/code> and <code>red-team<\/code>/);
  assert.doesNotMatch(statusSource, /<code>diagnostics<\/code>, <code>ip-bans<\/code>, and\s*<code>red-team<\/code>/);
  assert.match(statusSource, />Fetch latency \(last \/ rolling\):</);
  assert.match(statusSource, />Render timing \(last \/ rolling\):</);
  assert.match(statusSource, />Polling skip \/ resume:</);

  assert.match(configSource, /export let onSaveConfig = null;/);
  assert.match(configSource, /await onSaveConfig\(/);
  assert.match(configSource, /import ConfigChallengeSection from '\.\/config\/ConfigChallengeSection\.svelte';/);
  assert.match(configSource, /import SaveChangesBar from '\.\/primitives\/SaveChangesBar\.svelte';/);
  assert.match(configSource, /<ConfigChallengeSection/);
  assert.match(configSource, /<SaveChangesBar/);
  assert.match(configSurfaceSource, /id="preview-challenge-puzzle-link"/);
  assert.match(configSurfaceSource, /id="preview-not-a-bot-link"/);
  assert.equal(
    configSurfaceSource.indexOf('id="not-a-bot-enabled-toggle"')
      < configSurfaceSource.indexOf('id="challenge-puzzle-enabled-toggle"'),
    true
  );
  assert.match(configSource, /\$: notABotScoreFailMaxCap = Math\.max\(0, Number\(notABotScorePassMin\) - 1\);/);
  assert.match(configSource, /\$: notABotScorePassMinFloor = Math\.min\(10, Number\(notABotScoreFailMax\) \+ 1\);/);
  assert.match(configSurfaceSource, /max=\{notABotScoreFailMaxCap\}/);
  assert.match(configSurfaceSource, /Any scores above Fail and below Pass will be shown a tougher challenge\./);
  assert.match(configSource, /buttonId="save-verification-all"/);
  assert.match(configSource, /export let operatorSnapshot = null;/);
  assert.match(configSource, /title="Verified Identity"/);
  assert.match(configSource, /id="verified-identity-enabled-toggle"/);
  assert.match(configSource, /id="verified-identity-native-web-bot-auth-toggle"/);
  assert.match(configSource, /id="verified-identity-provider-assertions-toggle"/);
  assert.match(configSource, /id="verified-identity-replay-window"/);
  assert.match(configSource, /id="verified-identity-clock-skew"/);
  assert.match(configSource, /id="verified-identity-directory-cache-ttl"/);
  assert.match(configSource, /id="verified-identity-directory-freshness-requirement"/);
  assert.match(configSource, /id="verified-identity-top-failure-reasons"/);
  assert.match(configSource, /id="verified-identity-top-schemes"/);
  assert.match(configSource, /id="verified-identity-top-categories"/);
  assert.match(configSource, /patch\.verified_identity = \{/);
  assert.match(configSource, /saveAllConfig\(/);
  assert.match(configSource, /window\.addEventListener\('beforeunload'/);
  assert.match(configSurfaceSource, /id="verification-cdp-enabled-toggle"/);
  assert.match(configSurfaceSource, /id="verification-cdp-threshold-slider"/);
  assert.equal(configSource.includes('{@html'), false);

  assert.match(robotsSurfaceSource, /id="path-allowlist"/);
  assert.match(robotsSurfaceSource, /id="path-allowlist-enabled-toggle"/);
  assert.match(robotsSource, /payload\.path_allowlist_enabled = pathAllowlistEnabled === true;/);
  assert.match(robotsSource, /payload\.path_allowlist = parseListTextarea\(pathAllowlist\);/);

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
  assert.match(robotsSource, /import ConfigDurationsSection from '\.\/config\/ConfigDurationsSection\.svelte';/);
  assert.match(robotsSource, /import ConfigNetworkSection from '\.\/config\/ConfigNetworkSection\.svelte';/);
  assert.match(robotsSource, /import ConfigPathAllowlistSection from '\.\/config\/ConfigPathAllowlistSection\.svelte';/);
  assert.match(robotsSource, /const buildRobotsPreviewPatch = \(\) => \{/);
  assert.match(robotsSource, /await onSaveConfig\(payload/);
  assert.match(robotsSource, /await onFetchRobotsPreview\(patch\);/);
  assert.match(robotsSource, /buttonId="save-policy-config"/);
  assert.match(robotsSource, /window\.addEventListener\('beforeunload'/);
  assert.match(robotsSurfaceSource, /id="open-robots-txt-link"/);
  assert.match(robotsSurfaceSource, /id="preview-robots"/);
  assert.match(robotsSurfaceSource, /dayId="dur-honeypot-days"/);
  assert.match(robotsSurfaceSource, /dayId="dur-ip-range-honeypot-days"/);
  assert.match(robotsSurfaceSource, /dayId="dur-maze-crawler-days"/);
  assert.match(robotsSurfaceSource, /id="browser-policy-toggle"/);
  assert.match(robotsSurfaceSource, /id="browser-block-rules"/);
  assert.match(robotsSurfaceSource, /id="path-allowlist-enabled-toggle"/);
  assert.match(robotsSurfaceSource, /id="path-allowlist"/);

  assert.match(fingerprintingSource, /export let onSaveConfig = null;/);
  assert.match(fingerprintingSource, /await onSaveConfig\(payload/);
  assert.match(fingerprintingSource, /isAkamaiEdgeAvailable/);
  assert.match(fingerprintingSource, /akamaiEdgeAvailable = isAkamaiEdgeAvailable\(runtime\);/);
  assert.match(fingerprintingSource, /id="fingerprinting-akamai-enabled-toggle"/);
  assert.match(fingerprintingSource, /id="fingerprinting-edge-mode-select"/);
  assert.match(fingerprintingSource, /id="fingerprinting-akamai-unavailable-message"/);
  assert.match(fingerprintingSource, /buttonId="save-fingerprinting-config"/);
  assert.match(fingerprintingSource, /window\.addEventListener\('beforeunload'/);
  assert.match(fingerprintingSource, /const AKAMAI_EDGE_ADDITIVE_SIGNAL_KEY = 'fp_akamai_edge_additive';/);
  assert.match(fingerprintingSource, /<h4>Current Akamai Edge Contribution<\/h4>/);
  assert.match(fingerprintingSource, /id="fingerprinting-akamai-signal-list"/);
  assert.match(fingerprintingSource, /key !== AKAMAI_EDGE_ADDITIVE_SIGNAL_KEY/);
  assert.match(fingerprintingSource, /<ConfigPanelHeading title="Botness Scoring Signals">/);
  assert.match(fingerprintingSource, /Additive "botness" fingerprinting signals used to decide how to route bot-like traffic\./);
  assert.match(fingerprintingSource, /id="fingerprinting-botness-signal-list"/);
  assert.match(fingerprintingSource, /js_verification_required/);
  assert.match(fingerprintingSource, /browser_outdated/);
  assert.match(monitoringCdpSource, /title="Detection-Triggered Bans"/);
  assert.match(monitoringCdpSource, /valueId="cdp-total-auto-bans"/);
  assert.match(monitoringCdpSource, /valueId="cdp-fp-ua-client-hint-mismatch"/);
  assert.match(monitoringCdpSource, /valueId="cdp-fp-ua-transport-mismatch"/);
  assert.match(monitoringCdpSource, /valueId="cdp-fp-temporal-transition"/);
  assert.match(monitoringCdpSource, /valueId="cdp-fp-flow-violations"/);

  assert.match(tuningSource, /export let onSaveConfig = null;/);
  assert.match(tuningSource, /await onSaveConfig\(payload/);
  assert.doesNotMatch(tuningSource, /import ConfigDurationsSection from '\.\/config\/ConfigDurationsSection\.svelte';/);
  assert.doesNotMatch(tuningSource, /import ConfigNetworkSection from '\.\/config\/ConfigNetworkSection\.svelte';/);
  assert.doesNotMatch(tuningSource, /ban_durations/);
  assert.doesNotMatch(tuningSource, /browser_policy_enabled/);
  assert.doesNotMatch(tuningSource, /path_allowlist_enabled/);
  assert.match(tuningSource, /buttonId="save-tuning-all"/);
  assert.match(tuningSource, /import SaveChangesBar from '\.\/primitives\/SaveChangesBar\.svelte';/);
  assert.match(tuningSource, /window\.addEventListener\('beforeunload'/);
  assert.doesNotMatch(tuningSurfaceSource, /dayId="dur-honeypot-days"/);
  assert.doesNotMatch(tuningSurfaceSource, /dayId="dur-rate-limit-days"/);
  assert.doesNotMatch(tuningSurfaceSource, /id="browser-policy-toggle"/);
  assert.doesNotMatch(tuningSurfaceSource, /id="browser-block-rules"/);
  assert.doesNotMatch(tuningSurfaceSource, /id="path-allowlist"/);
});

test('ban duration families remain aligned across runtime, config, and policy surfaces', () => {
  const configSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/config/mod.rs'),
    'utf8'
  );
  const adminApiSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/admin/api.rs'),
    'utf8'
  );
  const requestRouterSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/runtime/request_router.rs'),
    'utf8'
  );
  const planBuilderSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/runtime/effect_intents/plan_builder.rs'),
    'utf8'
  );
  const libSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/lib.rs'),
    'utf8'
  );
  const internalProviderSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/providers/internal.rs'),
    'utf8'
  );
  const externalProviderSource = fs.readFileSync(
    path.join(WORKSPACE_ROOT, 'src/providers/external.rs'),
    'utf8'
  );
  const policyTabSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/RobotsTab.svelte'),
    'utf8'
  );
  const durationsPaneSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/config/ConfigDurationsSection.svelte'),
    'utf8'
  );
  const configSchemaSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/domain/config-schema.js'),
    'utf8'
  );
  const statusVarMeanings = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'static/assets/status-var-meanings.json'),
    'utf8'
  );

  const families = [
    'honeypot',
    'ip_range_honeypot',
    'maze_crawler',
    'rate_limit',
    'cdp',
    'edge_fingerprint',
    'tarpit_persistence',
    'not_a_bot_abuse',
    'challenge_puzzle_abuse',
    'admin'
  ];

  families.forEach((family) => {
    assert.match(configSource, new RegExp(`pub ${family}: u64`));
    assert.match(adminApiSource, new RegExp(`${family}: Option<u64>`));
    assert.equal(configSchemaSource.includes(`ban_durations.${family}`), true);
    assert.equal(statusVarMeanings.includes(`ban_durations.${family}`), true);
  });

  assert.match(planBuilderSource, /get_ban_duration\("ip_range_honeypot"\)/);
  assert.match(planBuilderSource, /get_ban_duration\("honeypot"\)/);
  assert.match(planBuilderSource, /get_ban_duration\("rate_limit"\)/);
  assert.match(libSource, /get_ban_duration\("maze_crawler"\)/);
  assert.match(requestRouterSource, /get_ban_duration\(ban_reason\)/);
  assert.doesNotMatch(requestRouterSource, /duration_seconds: CHALLENGE_ABUSE_SHORT_BAN_SECONDS/);
  assert.match(internalProviderSource, /get_ban_duration\("tarpit_persistence"\)/);
  assert.doesNotMatch(internalProviderSource, /TARPIT_ESCALATION_SHORT_BAN_SECONDS/);
  assert.match(externalProviderSource, /get_ban_duration\("edge_fingerprint"\)/);
  assert.match(adminApiSource, /unwrap_or\(cfg\.get_ban_duration\("admin"\)\)/);

  assert.match(policyTabSource, /payload\.ban_durations = \{/);
  assert.match(durationsPaneSource, /Maze Threshold Exceeded/);
  assert.match(durationsPaneSource, /Not-a-Bot Abuse/);
  assert.match(durationsPaneSource, /Challenge Puzzle Abuse/);
  assert.match(durationsPaneSource, /Tarpit Persistence/);
  assert.match(durationsPaneSource, /Edge Fingerprint Automation/);
});

test('config panel provides the shared writable-hide and dirty-state chrome for edit panes', () => {
  const configPanelSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/primitives/ConfigPanel.svelte'),
    'utf8'
  );

  assert.match(configPanelSource, /class:hidden=\{!writable\}/);
  assert.match(configPanelSource, /\$: variantClass = variant === 'export' \? 'config-export-pane' : 'config-edit-pane';/);
  assert.match(configPanelSource, /class:config-edit-pane--dirty=\{variant !== 'export' && dirty\}/);
});

test('red team tab reuses verification-style config panel primitives for its adversary sim pane', () => {
  const redTeamTabSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/RedTeamTab.svelte'),
    'utf8'
  );

  assert.match(redTeamTabSource, /import ConfigPanel from '.\/primitives\/ConfigPanel\.svelte';/);
  assert.match(
    redTeamTabSource,
    /import ConfigPanelHeading from '.\/primitives\/ConfigPanelHeading\.svelte';/
  );
  assert.match(redTeamTabSource, /class="controls-grid controls-grid--config"/);
  assert.match(redTeamTabSource, /<ConfigPanel writable=\{true\} dirty=\{false\}>/);
  assert.match(redTeamTabSource, /<div class="admin-controls">/);
  assert.match(redTeamTabSource, /<div class="input-row"/);
  assert.match(redTeamTabSource, /<label class="control-label control-label--wide" for="adversary-sim-lane-select">Lane<\/label>/);
  assert.match(redTeamTabSource, /<select[\s\S]*id="adversary-sim-lane-select"[\s\S]*class="input-field"[\s\S]*on:change=\{handleLaneChange\}/m);
  assert.match(redTeamTabSource, /<option value="synthetic_traffic">Synthetic Traffic<\/option>/);
  assert.match(redTeamTabSource, /<option value="scrapling_traffic">Scrapling Traffic<\/option>/);
  assert.match(redTeamTabSource, /<option value="bot_red_team" disabled>Bot Red Team \(coming soon\)<\/option>/);
  assert.match(redTeamTabSource, /<p id="adversary-sim-lifecycle-copy" class="control-desc text-muted">\{lifecycleCopy\}<\/p>/);
  assert.match(redTeamTabSource, /class="dashboard-adversary-sim-progress"/);
  assert.match(redTeamTabSource, /class="dashboard-adversary-sim-progress__fill"/);
  assert.match(redTeamTabSource, /export let laneValue = 'synthetic_traffic';/);
  assert.match(redTeamTabSource, /export let laneDisabled = false;/);
  assert.match(redTeamTabSource, /export let laneDisabledReason = '';/);
  assert.match(redTeamTabSource, /export let onLaneChange = null;/);
  assert.match(redTeamTabSource, /function handleLaneChange\(event\)/);
  assert.match(redTeamTabSource, /<div class="status-item">[\s\S]*<h3>Lane State<\/h3>/m);
  assert.match(redTeamTabSource, /Desired lane:/);
  assert.match(redTeamTabSource, /Active lane:/);
  assert.match(redTeamTabSource, /Switch sequence:/);
  assert.match(redTeamTabSource, /<div class="status-item">[\s\S]*<h3>Lane Diagnostics<\/h3>/m);
  assert.match(redTeamTabSource, /Beat attempts:/);
  assert.match(redTeamTabSource, /Generated requests:/);
  assert.match(redTeamTabSource, /Failure classes:/);
});

test('red team adversary-sim progress bar animates stripe motion and respects reduced motion', () => {
  const styleSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'style.css'),
    'utf8'
  );

  assert.match(styleSource, /\.dashboard-adversary-sim-progress__fill\s*\{[\s\S]*animation:\s*dashboard-adversary-sim-progress-stripes\s+1\.5s\s+linear\s+infinite;[\s\S]*\}/m);
  assert.match(styleSource, /@keyframes dashboard-adversary-sim-progress-stripes\s*\{[\s\S]*background-position:\s*0 0;[\s\S]*background-position:\s*0 -198px;[\s\S]*\}/m);
  assert.match(styleSource, /@media\s*\(prefers-reduced-motion:\s*reduce\)\s*\{[\s\S]*\.dashboard-adversary-sim-progress__fill\s*\{[\s\S]*animation:\s*none;[\s\S]*\}[\s\S]*\}/m);
});

test('tab state message supports pane-scoped notices alongside loading and error states', () => {
  const tabStateMessageSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/primitives/TabStateMessage.svelte'),
    'utf8'
  );

  assert.match(tabStateMessageSource, /export let noticeText = '';/);
  assert.match(tabStateMessageSource, /export let noticeKind = 'info';/);
  assert.match(tabStateMessageSource, /data-tab-notice=\{tab\}/);
  assert.match(tabStateMessageSource, /\$:\s+paneNoticeKind = readNoticeKind\(noticeKind\);/);
  assert.match(tabStateMessageSource, /class=\{`message \$\{paneNoticeKind\}`\}/);
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
    /const hasUnsavedConfigChanges = hasVisibleUnsavedConfigChanges\(\);\s+if \(!confirmDiscardUnsavedConfigChanges\(\)\) return;\s+let redirectingToLogin = false;\s+loggingOut = true;\s+try \{\s+suppressBeforeUnloadPrompt = hasUnsavedConfigChanges;\s+routeController\.abortInFlightRefresh\(\);\s+adversarySimController\.dispose\(\);\s+await logoutDashboardSession\(\);/s
  );
});

test('dashboard route lazily loads heavy tabs and keeps orchestration local', () => {
  const source = fs.readFileSync(path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'), 'utf8');

  assert.match(source, /import\('\$lib\/components\/dashboard\/RedTeamTab\.svelte'\)/);
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
  assert.match(source, /\$lib\/runtime\/dashboard-red-team-controller\.js/);
  assert.match(source, /deriveAdversarySimControlState/);
  assert.match(source, /\$:\s+bodyClassSource = \{/);
  assert.match(source, /shadow_mode:\s*configSnapshot\?\.shadow_mode/);
  assert.match(source, /deriveDashboardBodyClassState\(bodyClassSource,\s*\{/);
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
  assert.match(source, /onSaveConfig=\{onSaveConfig\}/);
  assert.match(source, /onValidateConfig=\{onValidateConfig\}/);
  assert.match(source, /onBan=\{onBan\}/);
  assert.match(source, /onUnban=\{onUnban\}/);
  assert.match(source, /configSnapshot=\{snapshots\.config\}/);
  assert.match(source, /ipRangeSuggestionsSnapshot=\{snapshots\.ipRangeSuggestions\}/);
  assert.match(source, /cdpSnapshot=\{snapshots\.cdp\}/);
  assert.match(source, /id="global-shadow-mode-toggle"/);
  assert.match(source, /dashboard-panel-red-team/);
  assert.match(source, /id="connection-status"/);
  assert.match(source, /id="lost-connection"/);
  assert.match(source, /const adversarySimController = createDashboardRedTeamController\(/);
  assert.match(source, /adversarySimController\.subscribe\(/);
  assert.match(source, /await adversarySimController\.bootstrap\(\)/);
  assert.match(source, /void adversarySimController\.handleTabActivated\(\)/);
  assert.match(source, /void adversarySimController\.handleVisibilityResume\(\)/);
  assert.match(
    source,
    /runtimeReady = bootstrapped === true;\s*if \(bootstrapped === true\) \{\s*await adversarySimController\.bootstrap\(\);\s*\}/s
  );
  assert.match(source, /onGlobalShadowModeToggleChange/);
  assert.match(source, /onGlobalAdversarySimToggleChange/);
  assert.match(source, /onAdversarySimLaneChange/);
  assert.match(source, /controlAdversarySimWithRetry/);
  assert.match(source, /deriveDashboardRequestBudgets/);
  assert.match(source, /function isAuthSessionExpiredError\(error\)/);
  assert.match(source, /function withRefreshedSessionOnAuthError\(action\)/);
  assert.match(source, /const restored = await restoreDashboardSession\(\);/);
  assert.match(source, /if \(restored !== true\) throw error;/);
  assert.match(source, /await withRefreshedSessionOnAuthError\(/);
  assert.match(source, /controlAdversarySimWithRetry\(\s*\(\) => withRefreshedSessionOnAuthError\(/);
  assert.match(source, /dashboardRequestBudgets = deriveDashboardRequestBudgets\(configRuntimeSnapshot\)/);
  assert.match(source, /dashboardRequestBudgets\.configWriteTimeoutMs/);
  assert.match(source, /dashboardRequestBudgets\.adversarySimControlTimeoutMs/);
  assert.match(source, /dashboardRequestBudgets\.adversarySimEnableTimeoutMs/);
  assert.match(source, /dashboardRequestBudgets\.adversarySimStatusTimeoutMs/);
  assert.match(source, /deriveGlobalControlDisabled/);
  assert.match(
    source,
    /updateDashboardConfig\(patch \|\| \{\},\s*\{\s*timeoutMs:\s*Math\.max\(\s*1_000,\s*Number\(options\?\.timeoutMs \|\| 0\) \|\| dashboardRequestBudgets\.configWriteTimeoutMs/s
  );
  assert.match(
    source,
    /controlDashboardAdversarySim\(desiredEnabled,\s*\{\s*timeoutMs:\s*dashboardRequestBudgets\.adversarySimControlTimeoutMs/s
  );
  assert.match(
    source,
    /controlAdversarySimWithRetry\(\s*\(\) => withRefreshedSessionOnAuthError\(\s*\(\) => controlDashboardAdversarySim\(desiredEnabled,\s*\{\s*lane:\s*nextLane,\s*timeoutMs:\s*dashboardRequestBudgets\.adversarySimControlTimeoutMs/s
  );
  assert.match(source, /adversarySimController\.replaceBackendStatus\(response\.status\);/);
  assert.match(source, /banDashboardIp\(ip, duration, 'manual_ban',\s*\{/);
  assert.match(source, /unbanDashboardIp\(ip,\s*\{/);
  assert.match(source, /status === 401/);
  assert.match(source, /status !== 403/);
  assert.match(source, /csrf/);
  assert.match(source, /trust boundary/);
  assert.match(source, /Adversary simulation session expired\. Redirecting to login\.\.\./);
  assert.match(source, /deriveAdversarySimLifecycleCopy/);
  assert.match(source, /adversarySimLifecycleCopy = deriveAdversarySimLifecycleCopy\(\{/);
  assert.match(source, /dashboard-global-control-label/);
  assert.match(
    source,
    /if \(checked && autoRefreshSupported && runtimeReady\) \{\s*void routeController\.refreshTab\(activeTabKey, 'auto-refresh'\);/s
  );
  assert.match(
    source,
    /adversarySimToggleEnabled = adversarySimControllerState\??\.uiDesiredEnabled === true/
  );
  assert.match(source, /adversarySimEnabled:\s*normalizedAdversarySimStatus\.enabled === true/);
  assert.match(source, /globalAdversarySimToggleDisabled = deriveGlobalControlDisabled\(/);
  assert.match(source, /globalAdversarySimLaneDisabled = deriveGlobalControlDisabled\(/);
  assert.match(source, /globalAdversarySimLaneDisabledReason = globalAdversarySimLaneDisabled/);
  assert.match(source, /void adversarySimController\.handleToggleIntent\(nextValue\);/);
  assert.match(source, /paneNoticeValues = DASHBOARD_TABS\.reduce\(/);
  assert.match(source, /const notice = paneNotices\[tab\];/);
  assert.match(source, /noticeText=\{paneNoticeValues\['red-team'\]\?\.text \|\| ''\}/);
  assert.match(source, /laneValue=\{normalizedAdversarySimStatus\.desiredLane\}/);
  assert.match(source, /laneDisabled=\{globalAdversarySimLaneDisabled\}/);
  assert.match(source, /laneDisabledReason=\{globalAdversarySimLaneDisabledReason\}/);
  assert.match(source, /onLaneChange=\{onAdversarySimLaneChange\}/);
  assert.match(source, /noticeText=\{paneNoticeValues\['ip-bans'\]\?\.text \|\| ''\}/);
  assert.match(source, /noticeText=\{paneNoticeValues\.verification\?\.text \|\| ''\}/);
});

test('dashboard smoke soak test owns an explicit timeout budget', () => {
  const source = fs.readFileSync(path.join(__dirname, 'dashboard.smoke.spec.js'), 'utf8');

  assert.match(
    source,
    /test\("repeated route remount loops keep polling request fan-out bounded", async \(\{ page \}\) => \{\s+test\.setTimeout\(90_000\);/s
  );
  assert.match(
    source,
    /test\("native remount soak keeps refresh p95 and polling cadence within bounds", async \(\{ page \}\) => \{\s+test\.setTimeout\(90_000\);/s
  );
});

test('dashboard smoke spec keeps the tab information architecture aligned with the canonical registry', () => {
  const source = fs.readFileSync(path.join(__dirname, 'dashboard.smoke.spec.js'), 'utf8');

  assert.match(
    source,
    /const DASHBOARD_TABS = Object\.freeze\(\["monitoring", "ip-bans", "red-team", "tuning", "verification", "traps", "rate-limiting", "geo", "fingerprinting", "policy", "status", "advanced", "diagnostics"\]\);/
  );
  assert.match(
    source,
    /const ADMIN_TABS = Object\.freeze\(\["ip-bans", "red-team", "tuning", "verification", "traps", "rate-limiting", "geo", "fingerprinting", "policy", "status", "advanced", "diagnostics"\]\);/
  );
});

test('dashboard route exposes manual refresh on diagnostics and auto-refresh only on ip-bans and red-team', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(
    source,
    /const MANUAL_REFRESH_TABS = new Set\(\['diagnostics', 'ip-bans', 'red-team'\]\);/
  );
  assert.match(
    source,
    /const AUTO_REFRESH_TABS = new Set\(\['ip-bans', 'red-team'\]\);/
  );
});

test('dashboard monitoring runtime keeps edge-fermyon on bounded monitoring hydration', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/runtime/dashboard-runtime-refresh.js'),
    'utf8'
  );

  assert.match(source, /deriveDashboardRequestBudgets/);
  assert.match(
    source,
    /monitoringRequestOptions = \{\s*\.\.\.requestOptions,\s*timeoutMs: requestBudgets\.monitoringRequestTimeoutMs/s
  );
  assert.match(
    source,
    /monitoringDeltaRequestOptions = \{\s*\.\.\.requestOptions,\s*timeoutMs: requestBudgets\.monitoringDeltaTimeoutMs/s
  );
  assert.match(source, /requestBudgets\.autoHydrateFullMonitoring !== true/);
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
  assert.match(source, /if \(!runtimeStateAvailable\) \{/);
  assert.match(source, /disabled=\{!runtimeStateAvailable\}/);
  assert.match(source, /backendConnectionState:\s*'disconnected'/);
  assert.match(source, /runtime_environment/);
  assert.match(source, /let apiKeyInput = null;/);
  assert.match(source, /apiKeyInput\.focus\(\)/);
});

test('dashboard route keeps a neutral auth gate mounted until session bootstrap authenticates', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(source, /let authBootstrapState = 'pending';/);
  assert.match(source, /id="dashboard-auth-gate"/);
  assert.match(source, /\{#if authBootstrapState !== 'authenticated'\}/);
  assert.doesNotMatch(source, /Checking admin session/);
  assert.match(source, /aria-hidden="true"/);
  assert.match(source, /class="dashboard-auth-gate"/);
  assert.match(source, /\{:else\}\s*<div class="container panel panel-border" data-dashboard-runtime-mode="native">/s);
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

test('diagnostics tab preserves the bounded legacy monitoring surface', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/DiagnosticsTab.svelte'),
    'utf8'
  );

  assert.match(source, /from '\.\.\/\.\.\/domain\/monitoring-normalizers\.js';/);
  assert.match(source, /const RANGE_EVENTS_FETCH_LIMIT = 5000;/);
  assert.match(source, /const RANGE_EVENTS_REQUEST_TIMEOUT_MS = 10000;/);
  assert.match(source, /const RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS = 180000;/);
  assert.match(source, /from '\.\.\/\.\.\/domain\/monitoring-chart-presets\.js';/);
  assert.match(source, /from '\.\.\/\.\.\/domain\/half-doughnut-chart\.js';/);
  assert.match(source, /resolveMonitoringChartTheme/);
  assert.match(source, /buildHalfDoughnutOptions/);
  assert.match(source, /syncHalfDoughnutReadout/);
  assert.match(source, /maintainAspectRatio: false,/);
  assert.match(source, /x: buildMonitoringTimeSeriesXAxis\(\),/);
  assert.match(source, /const fillColor = chartTheme\.timeSeriesFill\.events/);
  assert.match(source, /'challenge'/);
  assert.match(source, /'pow'/);
  assert.match(source, /export let autoRefreshEnabled = false;/);
  assert.match(source, /sameSeries\(chart, trendSeries\.labels, trendSeries\.data\)/);
  assert.match(source, /abortRangeEventsFetch\(\);/);
  assert.match(source, /const isRangeFetchInFlight = selectedRangeWindowState\.loading === true;/);
  assert.match(source, /normalizeReasonRows\(/);
  assert.match(source, /deriveEnforcedMonitoringChartRows\(selectedRangeEvents, \{ topIpLimit: 10 \}\)\.events/);
  assert.match(source, /buildTimeSeries\(enforcedSelectedRangeEvents, selectedTimeRange,/);
  assert.match(source, /deriveMonitoringEventDisplay/);
  assert.match(source, /const rawFeedPayload = \(event = \{\}\) =>/);
  assert.match(source, /Object\.keys\(source\)/);
  assert.match(source, /const buildRawTelemetryFeed = \(events = \[\]\) =>/);
  assert.match(source, /'Puzzle Outcomes'/);
  assert.match(source, /\$: rawRecentEvents = Array\.isArray\(events\.recent_events\)/);
  assert.match(source, /\$: rawTelemetryFeed = buildRawTelemetryFeed\(rawRecentEvents\);/);
  assert.match(source, /\$: eventWindowTotal = toNonNegativeIntOrNull\(events\?\.recent_events_window\?\.total_events_in_window\);/);
  assert.match(source, /\$: totalBans = \(\(\) => \{/);
  assert.match(source, /const byEventType = getEventCountByName\(eventCounts, 'Ban'\);/);
  assert.match(source, /\$: activeBans = banSnapshotStatus === 'unavailable' \? null : bans\.length;/);
  assert.match(source, /eventTypesReadout = EMPTY_HALF_DOUGHNUT_READOUT;/);
});

test('diagnostics tab is decomposed into focused subsection components', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/DiagnosticsTab.svelte'),
    'utf8'
  );
  const diagnosticsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/DiagnosticsSection.svelte'),
    'utf8'
  );
  const rawFeedSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/RawTelemetryFeed.svelte'),
    'utf8'
  );
  const disclosureSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/primitives/DisclosureSection.svelte'),
    'utf8'
  );

  assert.match(source, /import DiagnosticsSection from '\.\/monitoring\/DiagnosticsSection\.svelte';/);
  assert.match(source, /import OverviewStats from '\.\/monitoring\/OverviewStats\.svelte';/);
  assert.match(source, /import PrimaryCharts from '\.\/monitoring\/PrimaryCharts\.svelte';/);
  assert.match(source, /import DefenseTrendBlocks from '\.\/monitoring\/DefenseTrendBlocks\.svelte';/);
  assert.match(source, /import RecentEventsTable from '\.\/monitoring\/RecentEventsTable\.svelte';/);
  assert.match(source, /import ExternalMonitoringSection from '\.\/monitoring\/ExternalMonitoringSection\.svelte';/);
  assert.match(source, /import IpRangeSection from '\.\/monitoring\/IpRangeSection\.svelte';/);
  assert.match(source, /export let ipBansFreshnessSnapshot = null;/);
  assert.match(source, /<OverviewStats/);
  assert.match(source, /<DiagnosticsSection/);
  assert.match(source, /monitoringFreshnessSnapshot=\{monitoringFreshnessSnapshot\}/);
  assert.match(source, /ipBansFreshnessSnapshot=\{ipBansFreshnessSnapshot\}/);
  assert.match(source, /rawTelemetryFeed=\{rawTelemetryFeed\}/);
  assert.match(source, /<PrimaryCharts/);
  assert.match(source, /\{eventTypesReadout\}/);
  assert.match(source, /<DefenseTrendBlocks/);
  assert.match(source, /<ChallengeSection/);
  assert.match(source, /<PowSection/);
  assert.match(source, /<IpRangeSection/);
  assert.equal(source.indexOf('<DiagnosticsSection') < source.indexOf('<ExternalMonitoringSection'), true);
  assert.match(source, /<ExternalMonitoringSection/);
  assert.match(source, /filterOptions=\{eventFilterOptions\}/);
  assert.match(source, /onFilterChange=\{onEventFilterChange\}/);
  assert.match(source, /RAW_FEED_MAX_LINES = 200/);
  assert.doesNotMatch(source, /id="monitoring-freshness-state"/);
  assert.doesNotMatch(source, /id="monitoring-freshness-meta"/);

  assert.match(diagnosticsSource, /import DisclosureSection from '\.\.\/primitives\/DisclosureSection\.svelte';/);
  assert.match(diagnosticsSource, /import RawTelemetryFeed from '\.\/RawTelemetryFeed\.svelte';/);
  assert.match(diagnosticsSource, /title="Telemetry Diagnostics"/);
  assert.match(diagnosticsSource, /<RawTelemetryFeed[\s\S]*wrapped=\{false\}/);
  assert.match(diagnosticsSource, /Monitoring Feed/);
  assert.match(diagnosticsSource, /IP Bans Feed/);

  assert.match(rawFeedSource, /export let wrapped = true;/);
  assert.match(rawFeedSource, /{#if wrapped}/);
  assert.match(rawFeedSource, /{#if !wrapped}/);

  assert.match(disclosureSource, /export let open = false;/);
  assert.match(disclosureSource, /<details/);
  assert.match(disclosureSource, /<summary/);
});

test('monitoring and diagnostics tabs make the accountability-vs-diagnostics split explicit', () => {
  const monitoringSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/MonitoringTab.svelte'),
    'utf8'
  );
  const diagnosticsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/DiagnosticsTab.svelte'),
    'utf8'
  );

  assert.match(monitoringSource, /Closed-Loop Accountability/);
  assert.match(monitoringSource, /data-monitoring-section=\{section\.id\}/);
  assert.match(monitoringSource, /id: 'current-status'/);
  assert.match(monitoringSource, /id: 'recent-loop-progress'/);
  assert.match(monitoringSource, /id: 'outcome-frontier'/);
  assert.match(monitoringSource, /id: 'change-judgment'/);
  assert.match(monitoringSource, /id: 'pressure-sits'/);
  assert.match(monitoringSource, /id: 'trust-and-blockers'/);
  assert.match(monitoringSource, /href="#diagnostics"/);
  assert.doesNotMatch(monitoringSource, /Monitoring Overhaul In Progress/);

  assert.match(diagnosticsSource, /data-diagnostics-section="deep-inspection-intro"/);
  assert.match(diagnosticsSource, /data-diagnostics-section="traffic-overview"/);
  assert.match(diagnosticsSource, /data-diagnostics-section="defense-breakdown"/);
  assert.match(diagnosticsSource, /data-diagnostics-section="recent-external-traffic"/);
  assert.match(diagnosticsSource, /data-diagnostics-section="defense-specific-diagnostics"/);
  assert.match(diagnosticsSource, /data-diagnostics-section="telemetry-diagnostics"/);
  assert.match(diagnosticsSource, /data-diagnostics-section="external-monitoring"/);
  assert.match(diagnosticsSource, />Diagnostics</);
  assert.match(diagnosticsSource, /deep inspection/i);
});

test('tarpit monitoring section centers progression and outcome telemetry', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/TarpitSection.svelte'),
    'utf8'
  );

  assert.match(source, /title="Activations"/);
  assert.match(source, /title="Progress Advanced"/);
  assert.match(source, /<h3>Budget Fallback Outcomes<\/h3>/);
  assert.match(source, /id="tarpit-budget-outcomes-list"/);
  assert.match(source, /<h3>Escalation Outcomes<\/h3>/);
  assert.match(source, /id="tarpit-escalation-outcomes-list"/);
});

test('red team tab renders the recent adversary runs panel with red-team-specific copy', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/RedTeamTab.svelte'),
    'utf8'
  );

  assert.match(source, /import AdversaryRunPanel from '\.\/monitoring\/AdversaryRunPanel\.svelte';/);
  assert.match(source, /import \{ deriveAdversaryRunRowsFromSummaries \} from '\.\/monitoring-view-model\.js';/);
  assert.match(source, /export let eventsSnapshot = null;/);
  assert.match(source, /export let bansSnapshot = null;/);
  assert.match(source, /export let monitoringFreshnessSnapshot = null;/);
  assert.match(source, /\$: rawRecentSimRuns = Array\.isArray\(events\.recent_sim_runs\) \? events\.recent_sim_runs : \[\];/);
  assert.match(source, /\$: adversaryRunSummary = deriveAdversaryRunRowsFromSummaries\(rawRecentSimRuns, bans\);/);
  assert.match(source, /<AdversaryRunPanel/);
  assert.match(source, /title="Recent Red Team Runs"/);
  assert.match(source, /description="Recent adversary simulation runs linked to monitoring and IP ban outcomes\."\/?/);
  assert.match(source, /summaryLabel="Active bans linked to recent runs"/);
  assert.match(source, /emptyText="No recent adversary simulation runs are currently retained in the compact run history\."/);
  assert.match(source, /degradedText="Monitoring freshness is degraded or stale\. Missing red team run rows may indicate delayed telemetry rather than no simulation activity\."/);
  assert.match(source, /<h3>Status Truth<\/h3>/);
  assert.match(source, /id="adversary-sim-generation-truth-basis"/);
  assert.match(source, /id="adversary-sim-lane-diagnostics-truth-basis"/);
  assert.match(source, /id="adversary-sim-persisted-event-evidence"/);
  assert.match(source, /Recovered lower-bound evidence from persisted monitoring events\./);
  assert.match(source, /Direct runtime control counters\./);
});

test('primary charts reuse the shared half doughnut shell for event-type readouts', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/PrimaryCharts.svelte'),
    'utf8'
  );

  assert.match(source, /import HalfDoughnutChart from '\.\.\/primitives\/HalfDoughnutChart\.svelte';/);
  assert.match(source, /export let eventTypesReadout = null;/);
  assert.match(source, /<HalfDoughnutChart/);
  assert.match(source, /canvasId="eventTypesChart"/);
  assert.match(source, /readout=\{eventTypesReadout\}/);
});

test('diagnostics and ip-ban doughnuts share the canonical largest-to-smallest series normalizer', () => {
  const diagnosticsSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/DiagnosticsTab.svelte'),
    'utf8'
  );
  const ipBansSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/IpBansTab.svelte'),
    'utf8'
  );

  assert.match(diagnosticsSource, /buildHalfDoughnutSeries,/);
  assert.match(diagnosticsSource, /const \{ labels, values: data \} = buildHalfDoughnutSeries\(counts\);/);
  assert.match(ipBansSource, /buildHalfDoughnutSeries,/);
  assert.match(ipBansSource, /const \{ labels, values \} = buildHalfDoughnutSeries\(entries\);/);
  assert.match(ipBansSource, /banReasonEntries = buildHalfDoughnutSeries\(Array\.from\(reasonCounts\.entries\(\)\)\)\.entries;/);
});

test('half doughnut readout reuses canonical monitoring typography classes', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/primitives/HalfDoughnutChart.svelte'),
    'utf8'
  );

  assert.match(source, /class="caps-label chart-doughnut-readout__label"/);
  assert.match(source, /class="stat-value chart-doughnut-readout__value"/);
});

test('dashboard stylesheet scales shared half doughnut readouts for larger shells', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'style.css'),
    'utf8'
  );

  assert.match(source, /\.chart-canvas-shell\s*\{[\s\S]*container-type:\s*inline-size;/);
  assert.match(source, /\.chart-doughnut-readout \.stat-value\s*\{[\s\S]*font-size:\s*var\(--half-doughnut-stat-value-size,\s*var\(--text-6xl\)\);/);
  assert.match(
    source,
    /\.chart-canvas-shell--half-doughnut\s*\{[\s\S]*--half-doughnut-stat-value-size:\s*clamp\(var\(--text-6xl\),\s*17cqw,\s*calc\(var\(--text-6xl\)\s*\*\s*3\)\);/
  );
});

test('dashboard stylesheet bottom-aligns half doughnut readouts and tightens label spacing', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'style.css'),
    'utf8'
  );

  assert.match(
    source,
    /\.chart-doughnut-readout\s*\{[\s\S]*bottom:\s*15px;[\s\S]*justify-content:\s*flex-end;[\s\S]*padding-bottom:\s*clamp\(2px,\s*1cqw,\s*8px\);/
  );
  assert.match(source, /\.chart-doughnut-readout \.stat-value\s*\{[\s\S]*line-height:\s*0\.9;/);
});

test('dashboard stylesheet keeps ip-ban half doughnut on the shared rectangular aspect ratio while preserving its larger cap', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'style.css'),
    'utf8'
  );

  assert.match(
    source,
    /\.chart-canvas-shell--ip-bans\s*\{[\s\S]*width:\s*min\(100%,\s*calc\(clamp\(220px,\s*50vh,\s*520px\)\s*\*\s*2\)\);[\s\S]*max-width:\s*calc\(clamp\(220px,\s*50vh,\s*520px\)\s*\*\s*2\);[\s\S]*margin-inline:\s*auto;[\s\S]*aspect-ratio:\s*2 \/ 1;/
  );
});

test('monitoring recent-events filters reuse canonical input-row and input-field styles', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/RecentEventsTable.svelte'),
    'utf8'
  );

  const inputRowMatches = source.match(/class="input-row"/g) || [];
  const selectMatches = source.match(/<select\s+[^>]*class="input-field"/g) || [];

  assert.equal(inputRowMatches.length, 6);
  assert.equal(selectMatches.length, 6);
  assert.match(source, /class="control-label control-label--wide"/);
  assert.match(source, /monitoring-filter-mode/);
});

test('monitoring recent-events table omits admin actor columns from the external-traffic surface', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/monitoring/RecentEventsTable.svelte'),
    'utf8'
  );

  assert.doesNotMatch(source, /caps-label">Admin</);
  assert.doesNotMatch(source, /ev\.admin/);
  assert.match(source, /<TableEmptyRow colspan=\{6\}>/);
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

test('dashboard native runtime owns session heartbeat, tab normalization, and action exports', () => {
  const source = fs.readFileSync(DASHBOARD_NATIVE_RUNTIME_PATH, 'utf8');

  assert.match(source, /const DASHBOARD_TABS = Object\.freeze\(\['monitoring', 'ip-bans', 'red-team', 'tuning', 'verification', 'traps', 'rate-limiting', 'geo', 'fingerprinting', 'policy', 'status', 'advanced', 'diagnostics'\]\);/);
  assert.match(source, /createDashboardRefreshRuntime/);
  assert.match(source, /const CONNECTION_HEARTBEAT_PATH = '\/admin\/session';/);
  assert.match(source, /function runConnectionHeartbeat\(reason = 'manual'\)/);
  assert.match(source, /recordHeartbeatAttemptStarted/);
  assert.match(source, /recordHeartbeatSuccess/);
  assert.match(source, /recordHeartbeatFailure/);
  assert.match(source, /recordHeartbeatControllerReset/);
  assert.doesNotMatch(source, /setBackendConnectionState/);
  assert.match(source, /function hasRuntimeEnvironment\(\)/);
  assert.match(source, /if \(!hasRuntimeEnvironment\(\)\) return false;/);
  assert.match(source, /export async function updateDashboardConfig/);
  assert.match(source, /export async function validateDashboardConfigPatch/);
  assert.match(source, /export async function banDashboardIp/);
  assert.match(source, /export async function unbanDashboardIp/);
  assert.match(source, /dashboardRefreshRuntime\.clearAllCaches/);
});

test('dashboard refresh runtime owns bounded cache, delta, and red-team monitoring refresh flows', () => {
  const source = fs.readFileSync(DASHBOARD_REFRESH_RUNTIME_PATH, 'utf8');

  assert.match(source, /const MONITORING_CACHE_KEY = 'shuma_dashboard_cache_monitoring_v2';/);
  assert.match(source, /const IP_BANS_CACHE_KEY = 'shuma_dashboard_cache_ip_bans_v1';/);
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
  assert.match(source, /if \(!isConfigSnapshotEmpty\(existingConfig\) && !isConfigRuntimeSnapshotEmpty\(existingRuntime\)\) \{/);
  assert.match(source, /async function refreshVerificationTab\(reason = 'manual', runtimeOptions = \{\}\)/);
  assert.match(source, /dashboardApiClient && typeof dashboardApiClient\.getOperatorSnapshot === 'function'/);
  assert.match(source, /applySnapshots\(\{ operatorSnapshot \}\);/);
  assert.match(source, /applySnapshots\(\{ operatorSnapshot: null \}\);/);
  assert.match(source, /const refreshRedTeamTab = async \(reason = 'manual', runtimeOptions = \{\}\) => \{/);
  assert.match(source, /const refreshPolicyTab = \(reason = 'manual', runtimeOptions = \{\}\) =>/);
  assert.match(source, /policy:\s*refreshPolicyTab,/);
  assert.doesNotMatch(source, /refreshRobotsTab/);
  assert.match(source, /if \(reason === 'auto-refresh'\) \{/);
  assert.match(
    source,
    /if \(isConfigSnapshotEmpty\(existingConfig\) \|\| isConfigRuntimeSnapshotEmpty\(existingRuntime\)\) \{\s*await Promise\.all\(\[\s*refreshMonitoringTab\(reason, runtimeOptions\),\s*refreshSharedConfig\(reason, runtimeOptions\)\s*\]\);/s
  );
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
  assert.match(
    source,
    /includeConfigRefresh \? refreshSharedConfig\(reason, runtimeOptions\) : Promise\.resolve\(null\)/
  );
});

test('dashboard verification tab wires verified identity operator snapshot and store state', () => {
  const apiClientSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/domain/api-client.js'),
    'utf8'
  );
  const stateSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/domain/dashboard-state.js'),
    'utf8'
  );
  const routeSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(apiClientSource, /export const adaptOperatorSnapshot = \(payload\) => \{/);
  assert.match(apiClientSource, /const getOperatorSnapshot = async \(requestOptions = \{\}\) =>/);
  assert.match(apiClientSource, /getOperatorSnapshot,/);
  assert.match(stateSource, /'operatorSnapshot'/);
  assert.match(stateSource, /operatorSnapshot: null/);
  assert.match(routeSource, /operatorSnapshot=\{snapshots\.operatorSnapshot\}/);
  assert.match(routeSource, /if \(normalized === 'verification'\) \{/);
  assert.match(routeSource, /state\.snapshots \? state\.snapshots\.operatorSnapshot : null/);
});

test('dashboard route wires native runtime actions with separate manual and auto refresh tab sets', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.match(source, /\$lib\/runtime\/dashboard-native-runtime\.js/);
  assert.match(source, /\$lib\/runtime\/dashboard-route-controller\.js/);
  assert.match(source, /updateDashboardConfig/);
  assert.match(source, /banDashboardIp/);
  assert.match(source, /unbanDashboardIp/);
  assert.match(source, /getDashboardRobotsPreview/);
  assert.match(source, /const MANUAL_REFRESH_TABS = new Set\(\['diagnostics', 'ip-bans', 'red-team'\]\);/);
  assert.match(source, /const AUTO_REFRESH_TABS = new Set\(\['ip-bans', 'red-team'\]\);/);
});

test('dashboard route keeps the shadow-mode eye overlay mounted and lets CSS reveal it when enabled', () => {
  const routeSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );
  const styleSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'style.css'),
    'utf8'
  );

  assert.match(routeSource, /assets\/eye\.png/);
  assert.match(routeSource, /dashboard-shadow-mode-eye/);
  assert.match(routeSource, /Shadow mode active/);

  assert.match(styleSource, /\.shuma-image-wrapper\s*\{\s*position:\s*relative;/m);
  assert.match(styleSource, /\.dashboard-shadow-mode-eye\s*\{[\s\S]*position:\s*absolute;[\s\S]*pointer-events:\s*none;[\s\S]*visibility:\s*hidden;/m);
  assert.match(styleSource, /\.dashboard-shadow-mode-eye-image\s*\{\s*display:\s*block;\s*width:\s*100%;\s*height:\s*auto;/m);
  assert.match(styleSource, /:root\.shadow-mode:not\(\.disconnected\)\s+\.dashboard-shadow-mode-eye\s*\{[\s\S]*visibility:\s*visible;/m);
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
