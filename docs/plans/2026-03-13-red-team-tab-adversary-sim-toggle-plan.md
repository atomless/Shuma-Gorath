# Red Team Tab and Adversary-Sim Toggle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move adversary-sim controls into a dedicated `Red Team` top-level dashboard tab and replace the current multi-authority toggle behavior with an immediate-feeling, debounced intent controller backed by a single write path and a read-only status contract.

**Architecture:** Keep the adversary-sim controller page-scoped and tab-independent. The switch must reflect the latest operator intent immediately, while backend truth is rendered separately as phase/status. The frontend owns only a short human-input coalesce window; the backend continues to own lifecycle safety, start/stop timing, and eventual convergence. Remove generic config as a peer on/off control path and stop the status endpoint from mutating state on read.

**Tech Stack:** Svelte/SvelteKit dashboard, shared dashboard runtime modules, Rust admin API, Spin KV state, Playwright dashboard smoke tests, Makefile verification targets

---

## Scope

This plan covers four coupled changes that must land together:

1. Add a top-level `Red Team` dashboard tab and move adversary-sim controls into it.
2. Replace the current optimistic-but-multi-authority toggle behavior with a dedicated page-scoped controller that separates user intent from backend truth.
3. Make `POST /admin/adversary-sim/control` the sole enable/disable write path.
4. Make `GET /admin/adversary-sim/status` read-only.

## Explicit Assumptions

1. Repository is pre-launch, so we should not preserve the current `adversary_sim_enabled` config-write path as a compatibility shim.
2. The new `Red Team` tab must reuse existing dashboard tab/panel idioms and shared control styles; no new visual language is needed.
3. The HTML root `adversary-sim` class should follow backend truth only, not optimistic pending intent.
4. We do not need push transport in this tranche; polling plus immediate refresh on activation is enough.

## Current-State Shortfalls (Code-Truth)

1. The dashboard toggle is currently derived from three authorities: local pending state, explicit status, and config fallback in `dashboard/src/lib/runtime/dashboard-global-controls.js`.
2. The page-level root class can flip optimistically from `adversarySimPendingEnabled` in `dashboard/src/routes/+page.svelte`, while the lifecycle copy still follows backend `generationActive`.
3. `GET /admin/adversary-sim/status` currently reconciles and persists state in `src/admin/api.rs`, which violates `docs/module-boundaries.md`.
4. `/admin/config` and the Advanced JSON schema still expose `adversary_sim_enabled`, so the sim has multiple write paths.
5. Sim status polling only runs while the page already believes the sim is on/running/stopping, so the UI can remain stale if another writer changes state while this tab thinks the sim is off.

## Target UX Contract

1. The switch moves immediately on every click and represents the latest operator intent.
2. Backend reality is shown separately as phase/status text, not encoded into switch lag.
3. Rapid input is coalesced in the dashboard over a short fixed window; only the final desired state is sent.
4. Backend lifecycle safety remains backend-owned. The frontend must not guess safe stop/start timing.
5. The selected top-level tab must not own sim correctness. Hiding the `Red Team` panel must not pause a running sim controller.
6. If a control request fails, the UI must snap back to the last backend-confirmed desired state and show an explicit error.

## Target State Model

The page-scoped controller owns the following state:

| Field | Meaning | Source |
| --- | --- | --- |
| `uiDesiredEnabled` | Latest user intent reflected by the switch immediately | Frontend |
| `backendStatus` | Last normalized `GET /admin/adversary-sim/status` payload | Backend |
| `lastBackendDesiredEnabled` | Last backend-confirmed desired state | Derived from `backendStatus` |
| `lastSubmittedDesiredEnabled` | Most recent desired value submitted to `/control` | Frontend |
| `inFlightDesiredEnabled` | Desired value currently being submitted or converged | Frontend |
| `queuedDesiredEnabled` | Latest intent captured while another command is in flight | Frontend |
| `controllerPhase` | `idle`, `debouncing`, `submitting`, `converging`, `error` | Frontend |
| `lastError` | Most recent operator-visible control error | Frontend |

## Target State Machine

| Phase | Entry Condition | Exit Condition | UI Contract |
| --- | --- | --- | --- |
| `idle` | No timer, no request, UI intent matches backend-confirmed desired state | User clicks toggle | Switch shows `uiDesiredEnabled`; phase copy shows backend truth |
| `debouncing` | User clicked; coalesce timer running | Timer expires or user clicks again | Switch updates immediately on each click |
| `submitting` | Coalesce timer expired and desired state differs from backend-confirmed desired state | `/control` accepted or failed | Switch stays at latest `uiDesiredEnabled`; show `Applying...` |
| `converging` | `/control` accepted; waiting for status to reflect accepted change | Backend reaches desired state or timeout/error | Show `Starting...` or `Stopping...` from backend phase |
| `error` | Submit or convergence failed | Operator dismisses/retries or controller refreshes to stable backend truth | Snap switch back to last backend-confirmed desired state and show explicit error |

### Controller Transition Rules

1. `click(nextValue)`:
   - set `uiDesiredEnabled = nextValue` immediately
   - clear `lastError`
   - start or reset the coalesce timer
   - enter `debouncing`
2. `debounce_expired`:
   - if `uiDesiredEnabled === lastBackendDesiredEnabled`, go to `idle`
   - if a request is already in flight, set `queuedDesiredEnabled = uiDesiredEnabled`
   - otherwise submit `/admin/adversary-sim/control`
3. `submit_success`:
   - set `lastSubmittedDesiredEnabled`
   - set `inFlightDesiredEnabled`
   - enter `converging`
   - refresh status immediately
4. `status_converged`:
   - update `backendStatus`
   - update `lastBackendDesiredEnabled`
   - if `queuedDesiredEnabled` differs, submit that next
   - otherwise clear in-flight state and return to `idle`
5. `submit_failure` or `convergence_failure`:
   - clear in-flight state
   - clear queued state if it no longer differs from backend truth
   - set `uiDesiredEnabled = lastBackendDesiredEnabled`
   - set `controllerPhase = error`

## Polling Contract

1. The selected dashboard tab does not own adversary-sim correctness.
2. While `controllerPhase` is `submitting` or `converging`, keep sim status polling active regardless of which top-level tab is selected.
3. While backend status reports sim `running` or `stopping`, keep sim status polling active regardless of selected tab.
4. When the whole browser page is hidden, correctness must not depend on timers. On visibility resume, force an immediate status refresh.
5. End-state target:
   - if sim is off, stable, and there is no in-flight command, do not continuously poll in the background
   - refresh sim status on dashboard bootstrap, `Red Team` tab activation, and page-visibility resume
6. Temporary migration note:
   - until the config write path is removed, the controller may need a low-cadence off-state poll to prevent stale UI from external writers

## Rendering Contract

1. Switch checked state: `uiDesiredEnabled`
2. Phase/lifecycle text: normalized backend truth only
3. HTML root `adversary-sim` class: backend truth only (`normalizedAdversarySimStatus.enabled === true`)
4. `Red Team` tab visibility: presentation only
5. Header-level global control strip: remove adversary-sim toggle and lifecycle copy from there; keep it focused on the remaining global controls

## File Touchpoints

### Create

- `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
- `dashboard/src/lib/runtime/dashboard-red-team-controller.js`
- `docs/dashboard-tabs/red-team.md`

### Modify

- `dashboard/src/routes/+page.svelte`
- `dashboard/src/lib/domain/dashboard-state.js`
- `dashboard/src/lib/state/dashboard-store.js`
- `dashboard/src/lib/runtime/dashboard-global-controls.js`
- `dashboard/src/lib/runtime/dashboard-body-classes.js`
- `dashboard/src/lib/runtime/dashboard-adversary-sim.js`
- `dashboard/src/lib/domain/config-schema.js`
- `src/admin/api.rs`
- `docs/api.md`
- `docs/configuration.md`
- `docs/testing.md`
- `Makefile`
- `e2e/dashboard.modules.unit.test.js`
- `e2e/dashboard.smoke.spec.js`

### Modify Only If Needed

- `dashboard/src/lib/domain/api-client.js`
  Use only if the controller needs a small facade change for status/control response shape. Do not widen the API contract unless the controller actually needs it.

## Implementation Order

Land these slices in order. Do not start backend contract cleanup after the UI move and controller split are half-done; the dashboard and API must converge on the same single-authority model.

### Task 1: Add the `Red Team` Top-Level Tab Shell

**Files:**
- Create: `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
- Modify: `dashboard/src/lib/domain/dashboard-state.js`
- Modify: `dashboard/src/lib/state/dashboard-store.js`
- Modify: `dashboard/src/routes/+page.svelte`
- Test: `e2e/dashboard.modules.unit.test.js`

**Step 1: Write the failing test**

Add a dashboard unit test that asserts:
- `DASHBOARD_TABS` includes `red-team`
- the route renders a dedicated `dashboard-panel-red-team`
- adversary-sim controls no longer live in the header strip

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-unit`
Expected: failure because `red-team` is not a known tab and no `RedTeamTab` panel exists yet.

**Step 3: Write minimal implementation**

- Add `red-team` to the canonical dashboard tab list and invalidation scopes.
- Lazy-load and render `RedTeamTab` in `+page.svelte` using the same mounted/hidden panel pattern as the existing tabs.
- Move the current adversary-sim toggle markup and lifecycle copy out of the header and into the new tab shell, preserving existing shared classes where possible.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-unit`
Expected: the new tab/panel contract passes and the route still satisfies existing dashboard tab wiring tests.

**Step 5: Commit**

```bash
git add dashboard/src/lib/components/dashboard/RedTeamTab.svelte dashboard/src/lib/domain/dashboard-state.js dashboard/src/lib/state/dashboard-store.js dashboard/src/routes/+page.svelte e2e/dashboard.modules.unit.test.js
git commit -m "feat: add red team dashboard tab shell"
```

### Task 2: Extract a Page-Scoped Adversary-Sim Controller

**Files:**
- Create: `dashboard/src/lib/runtime/dashboard-red-team-controller.js`
- Modify: `dashboard/src/routes/+page.svelte`
- Modify: `dashboard/src/lib/runtime/dashboard-adversary-sim.js`
- Modify: `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
- Test: `e2e/dashboard.modules.unit.test.js`

**Step 1: Write the failing test**

Add controller-focused dashboard unit tests that assert:
- the switch flips immediately on click
- `off -> on -> off` within the debounce window sends only the final `off`
- `on -> off -> on` within the debounce window sends only the final `on`
- a queued reversal after an in-flight request is submitted after the current request settles

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-unit`
Expected: failure because there is no dedicated controller module and the current route-level logic does not expose these semantics.

**Step 3: Write minimal implementation**

Create `dashboard-red-team-controller.js` with an explicit API similar to:

```js
export function createDashboardRedTeamController(options = {}) {
  return {
    getState,
    subscribe,
    handleToggleIntent,
    refreshStatus,
    handleTabActivated,
    handleVisibilityResume,
    dispose
  };
}
```

Implementation rules:
- keep `uiDesiredEnabled`, `backendStatus`, `queuedDesiredEnabled`, `controllerPhase`, and `lastError` inside the controller
- coalesce rapid clicks with a short fixed timer
- never let the selected top-level tab own correctness
- keep polling while submitting, converging, or while backend phase is running/stopping

**Step 4: Wire the route**

- Instantiate the controller in `+page.svelte`
- replace `adversarySimPendingEnabled`, `waitForAdversarySimStatusConvergence`, and direct toggle orchestration with controller state/actions
- pass the controller snapshot and action handlers into `RedTeamTab`

**Step 5: Run test to verify it passes**

Run: `make test-dashboard-unit`
Expected: the new controller tests pass and the route no longer owns the old scattered toggle orchestration.

**Step 6: Commit**

```bash
git add dashboard/src/lib/runtime/dashboard-red-team-controller.js dashboard/src/routes/+page.svelte dashboard/src/lib/runtime/dashboard-adversary-sim.js dashboard/src/lib/components/dashboard/RedTeamTab.svelte e2e/dashboard.modules.unit.test.js
git commit -m "feat: add page-scoped red team sim controller"
```

### Task 3: Make Dashboard Rendering Truthful

**Files:**
- Modify: `dashboard/src/lib/runtime/dashboard-global-controls.js`
- Modify: `dashboard/src/lib/runtime/dashboard-body-classes.js`
- Modify: `dashboard/src/routes/+page.svelte`
- Modify: `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
- Test: `e2e/dashboard.modules.unit.test.js`

**Step 1: Write the failing test**

Add unit tests that assert:
- the toggle checked state no longer falls back to `configSnapshot.adversary_sim_enabled`
- the HTML root `adversary-sim` class follows backend truth only
- backend phase copy can disagree with switch position while convergence is in progress

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-unit`
Expected: failure because the current helper and root-class wiring still use pending/config-derived state.

**Step 3: Write minimal implementation**

- Remove config-backed toggle derivation from `deriveAdversarySimToggleEnabled`, or replace the helper entirely if it no longer matches the new architecture.
- Ensure `deriveDashboardBodyClassState` and `syncDashboardBodyClasses` receive backend truth only for `adversary-sim`.
- Keep the switch responsive by using `uiDesiredEnabled` only inside `RedTeamTab`; do not reuse that value for global root-class state.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-unit`
Expected: rendering and root-class behavior now reflect the two-channel model: intent in the switch, truth in status/root classes.

**Step 5: Commit**

```bash
git add dashboard/src/lib/runtime/dashboard-global-controls.js dashboard/src/lib/runtime/dashboard-body-classes.js dashboard/src/routes/+page.svelte dashboard/src/lib/components/dashboard/RedTeamTab.svelte e2e/dashboard.modules.unit.test.js
git commit -m "fix: separate sim intent from backend truth in dashboard rendering"
```

### Task 4: Remove Generic Config as an Adversary-Sim Write Path

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `dashboard/src/lib/domain/config-schema.js`
- Modify: `docs/configuration.md`
- Test: `src/admin/api.rs`
- Test: `e2e/dashboard.modules.unit.test.js`

**Step 1: Write the failing test**

Add focused regressions that assert:
- `POST /admin/config` rejects patches containing `adversary_sim_enabled`
- the Advanced JSON schema no longer lists `adversary_sim_enabled`
- dashboard tests stop assuming config-backed provisional sim state

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-unit`
Run: `make test-unit`
Expected: failure because config still accepts the field and the schema still exposes it.

**Step 3: Write minimal implementation**

- In `src/admin/api.rs`, reject `adversary_sim_enabled` in `/admin/config` with a clear `400` explaining that `/admin/adversary-sim/control` is the sole lifecycle write path.
- Remove `adversary_sim_enabled` from `dashboard/src/lib/domain/config-schema.js`.
- Update `docs/configuration.md` to document the field as control-only, not config-writable.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-unit`
Run: `make test-unit`
Expected: config/schema parity is restored and dashboard tests no longer encode config as a toggle truth source.

**Step 5: Commit**

```bash
git add src/admin/api.rs dashboard/src/lib/domain/config-schema.js docs/configuration.md e2e/dashboard.modules.unit.test.js
git commit -m "fix: remove config as adversary sim lifecycle writer"
```

### Task 5: Make `GET /admin/adversary-sim/status` Read-Only

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `docs/api.md`
- Modify: `Makefile`
- Test: `src/admin/api.rs`

**Step 1: Write the failing test**

Replace the current read-path reconciliation expectations with new focused Rust tests that assert:
- `GET /admin/adversary-sim/status` does not persist reconciled state
- `controller_reconciliation_required` is reported when persisted control state is stale
- `POST /admin/adversary-sim/control` remains the sole path that mutates desired enabled state

**Step 2: Run test to verify it fails**

Run: `make test-adversary-sim-lifecycle`
Expected: failure because the current status handler still reconciles and saves on read.

**Step 3: Write minimal implementation**

- Stop `handle_admin_adversary_sim_status` from calling reconcile-and-save logic.
- Build the response from persisted config/state plus read-only diagnostics.
- Preserve `controller_reconciliation_required` so the UI still sees stale-state evidence without hidden mutation.
- Update `docs/api.md` so the status contract explicitly says the endpoint is read-only.
- Update `Makefile` so `test-adversary-sim-lifecycle` runs the replacement Rust lifecycle cases instead of the old read-path reconcile expectations.

**Step 4: Run test to verify it passes**

Run: `make test-adversary-sim-lifecycle`
Expected: the lifecycle gate now enforces the read-only status contract.

**Step 5: Commit**

```bash
git add src/admin/api.rs docs/api.md Makefile
git commit -m "fix: make adversary sim status endpoint read-only"
```

### Task 6: Add Focused Red-Team Smoke Coverage

**Files:**
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `docs/testing.md`

**Step 1: Write the failing test**

Add or update focused Playwright smoke coverage so it asserts:
- the sim controls live under the `Red Team` tab
- the switch moves immediately on click
- rapid reversal within the debounce window does not force a backend flip-flop for the dropped intermediate state
- switching away from `Red Team` while sim is active does not prevent status convergence when returning

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "red team|adversary sim"'`
Expected: failure because the current smoke flow assumes the old header-level control and retry semantics.

**Step 3: Write minimal implementation**

- Replace the old helper assumptions in `dashboard.smoke.spec.js`
- Update `docs/testing.md` so focused dashboard-sim validation points at the `Red Team` tab path and the new intent-vs-truth contract

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "red team|adversary sim"'`
Expected: focused smoke coverage passes against the new contract.

**Step 5: Commit**

```bash
git add e2e/dashboard.smoke.spec.js docs/testing.md
git commit -m "test: cover red team sim toggle debounce flow"
```

### Task 7: Final Focused Verification and End-to-End Confirmation

**Files:**
- Modify only if a focused target or doc reference still points at the old contract

**Step 1: Run focused verification first**

Run:
- `make test-dashboard-unit`
- `make test-adversary-sim-lifecycle`
- `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "red team|adversary sim"'`

Expected: all focused gates pass against the new controller contract.

**Step 2: Run the full suite only once focused gates are green**

Run: `make test`
Expected: full suite passes and proves the migration did not regress adjacent surfaces.

**Step 3: Refresh the full-suite receipt**

Update `.spin/last-full-test-pass.json` only after the successful `make test` run, per repo policy.

**Step 4: Commit**

```bash
git add .spin/last-full-test-pass.json
git commit -m "chore: refresh full test receipt after red team sim migration"
```

## Minimum Regression Coverage

These tests are the minimum contract surface for this migration:

1. Dashboard unit:
   - `DASHBOARD_TABS` includes `red-team`
   - `RedTeamTab` owns the sim controls
   - immediate switch movement on click
   - debounce coalesces `on/off/on` and `off/on/off`
   - queued reversal submits after in-flight request settles
   - no config-backed fallback for toggle checked state
   - HTML root class follows backend truth only
2. Rust lifecycle:
   - config patch rejects `adversary_sim_enabled`
   - status endpoint is read-only
   - control endpoint remains sole desired-state writer
   - stale persisted state is surfaced as `controller_reconciliation_required`
3. Dashboard smoke:
   - `Red Team` tab hosts the control
   - switch moves immediately
   - rapid reversal drops the intermediate backend transition
   - switching away from the tab does not break convergence reporting

## Definition of Done

This tranche is done only when all of the following are true:

1. There is exactly one backend write path for enable/disable: `POST /admin/adversary-sim/control`.
2. `GET /admin/adversary-sim/status` is observably read-only.
3. The switch moves immediately on click and reflects latest user intent.
4. Backend phase/status is rendered separately and stays truthful.
5. A hidden `Red Team` tab panel does not break a running sim controller.
6. Root HTML `adversary-sim` class reflects backend truth only.
7. Focused dashboard and lifecycle tests cover the new contract.
8. Full `make test` passes and the receipt is refreshed.

## Risks and Guardrails

1. Do not combine this migration with unrelated polling/auth/connection-state rewrites. Keep the first slice scoped to sim controller ownership and API contract cleanup.
2. Do not widen the visual language. Reuse existing dashboard panel/toggle classes.
3. Do not preserve the old config-write path "just in case"; that would keep the multi-authority bug alive.
4. Do not leave the status endpoint half-mutating. Either it is read-only or the architecture remains untrustworthy.

