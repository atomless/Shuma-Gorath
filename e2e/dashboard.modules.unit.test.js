const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

function loadBrowserModule(relativePath, overrides = {}) {
  const absolutePath = path.resolve(__dirname, '..', relativePath);
  const source = fs.readFileSync(absolutePath, 'utf8');
  const sandbox = {
    window: {
      ...overrides
    },
    console,
    URL,
    Headers: typeof Headers === 'undefined' ? function HeadersShim() {} : Headers,
    Request: typeof Request === 'undefined' ? function RequestShim() {} : Request,
    Response: typeof Response === 'undefined' ? function ResponseShim() {} : Response
  };
  sandbox.globalThis = sandbox.window;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: absolutePath });
  return sandbox.window;
}

function toPlain(value) {
  return JSON.parse(JSON.stringify(value));
}

test('dashboard API adapters normalize sparse payloads safely', () => {
  const browser = loadBrowserModule('dashboard/modules/api-client.js');
  const api = browser.ShumaDashboardApiClient;
  assert.ok(api);

  const events = api.adaptEvents({ recent_events: null, top_ips: [['198.51.100.8', '3']] });
  assert.deepEqual(toPlain(events.recent_events), []);
  assert.deepEqual(toPlain(events.top_ips), [['198.51.100.8', 3]]);

  const maze = api.adaptMaze({ total_hits: '9', unique_crawlers: '2', top_crawlers: [] });
  assert.equal(maze.total_hits, 9);
  assert.equal(maze.unique_crawlers, 2);
  assert.deepEqual(toPlain(maze.top_crawlers), []);
});

test('dashboard state invalidation scopes are explicit and bounded', () => {
  const browser = loadBrowserModule('dashboard/modules/dashboard-state.js');
  const stateApi = browser.ShumaDashboardState;
  assert.ok(stateApi);

  const state = stateApi.create();
  ['monitoring', 'ip-bans', 'status', 'config', 'tuning'].forEach((tab) => {
    state.markTabUpdated(tab);
  });
  state.invalidate('securityConfig');
  assert.equal(state.isTabStale('status'), true);
  assert.equal(state.isTabStale('config'), true);
  assert.equal(state.isTabStale('tuning'), true);
  assert.equal(state.isTabStale('monitoring'), false);
  assert.equal(state.isTabStale('ip-bans'), false);

  state.markTabUpdated('status');
  assert.equal(state.isTabStale('status'), false);
  assert.equal(state.isTabStale('config'), true);
});

test('tab lifecycle normalizes unknown tabs to monitoring default', () => {
  const browser = loadBrowserModule('dashboard/modules/tab-lifecycle.js');
  const lifecycle = browser.ShumaDashboardTabLifecycle;
  assert.ok(lifecycle);

  assert.equal(lifecycle.normalizeTab('ip-bans'), 'ip-bans');
  assert.equal(lifecycle.normalizeTab('IP-BANS'), 'ip-bans');
  assert.equal(lifecycle.normalizeTab('unknown-tab'), 'monitoring');
  assert.equal(lifecycle.normalizeTab(''), 'monitoring');
});
