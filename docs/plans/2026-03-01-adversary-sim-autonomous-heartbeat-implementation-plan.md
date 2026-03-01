# Adversary Sim Autonomous Heartbeat Implementation Plan

Date: 2026-03-01  
Status: Proposed (implementation-ready)

Related:
- [`docs/adr/0010-adversary-sim-autonomous-heartbeat.md`](../adr/0010-adversary-sim-autonomous-heartbeat.md)
- [`docs/adr/0007-adversary-sim-toggle-command-controller.md`](../adr/0007-adversary-sim-toggle-command-controller.md)
- [`docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md`](../adr/0008-realtime-monitoring-cursor-sse-hybrid.md)
- [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](2026-02-26-adversarial-simulation-v2-plan.md)
- [`todos/todo.md`](../../todos/todo.md) (`SIM-DEPLOY-1`, `SIM-DEPLOY-2`, `SIM-LLM-1`)

## Objective

Implement ADR 0010 in bounded, reviewable slices so adversary traffic generation is backend-owned, dashboard-independent, stable under toggle operations, and observable through a single backend-authored freshness contract.

## Explicit Assumptions

1. This repository is pre-launch; we optimize for correctness and clean contracts over backward-compatibility shims.
2. `GET /admin/adversary-sim/status` remains read-only (ADR 0007 and module-boundary requirement).
3. Deterministic lane remains release-blocking oracle; frontier/LLM lane remains discovery/promotable corpus until explicitly elevated.
4. Runtime-off behavior is strict: when simulation is off, there must be no active simulator heartbeat loop, no active generator/supervisor worker, and no simulation traffic emission.

## Non-goals

1. Shipping production-default adversary simulation availability in this tranche.
2. Implementing full frontier/LLM emergent lane orchestration (`SIM-LLM-1` follow-up).
3. Changing monitoring transport architecture beyond freshness ownership cleanup.

## Execution Order (Slice-by-Slice)

1. `SIM-HB-1` Supervisor substrate lock + status contract extension.
2. `SIM-HB-2` Backend-owned heartbeat execution path and capability boundary.
3. `SIM-HB-3` Spawn-on-enable / teardown-on-disable lifecycle + off-state inertness enforcement.
4. `SIM-HB-4` Dashboard decoupling (remove UI tick ownership, remove optimistic-toggle races).
5. `SIM-HB-5` Freshness ownership simplification (backend is source of truth, UI render-only).
6. `SIM-HB-6` Deterministic traffic breadth/cadence hardening (faster and category-diverse traffic).
7. `SIM-HB-7` Verification, docs, rollout guardrails, and rollback drills.

## Slice Details

### SIM-HB-1: Supervisor Substrate Lock and Contract Scaffolding

Scope:
1. Lock the runtime substrate for autonomous heartbeat execution (supervisor logic backend-owned, never dashboard-owned).
2. Extend adversary-sim status payload with explicit supervisor diagnostics and inertness state.
3. Mark dashboard tick contract as deprecated in payload/docs.

Primary touchpoints:
- `src/admin/api.rs`
- `src/admin/adversary_sim.rs`
- `docs/api.md`
- `docs/dashboard.md`
- `docs/testing.md`

Acceptance criteria:
1. Status payload exposes explicit supervisor block (`owner`, `cadence`, `last_heartbeat_at`, `state`, `reason`).
2. Status payload exposes explicit off-state inertness diagnostics (`heartbeat_active=false`, `worker_active=false` when off).
3. Payload/docs stop implying dashboard-owned tick cadence as normal operation.

### SIM-HB-2: Backend Heartbeat Execution Path

Scope:
1. Introduce backend-owned heartbeat tick path with least-authority capability checks.
2. Move generation invocation authority away from dashboard-triggered `POST /admin/adversary-sim/tick`.
3. Keep control-plane/attacker-plane separation strict (control mutates desired state only; heartbeat executes bounded attacker traffic).

Primary touchpoints:
- `src/admin/api.rs`
- `src/admin/adversary_sim.rs`
- `src/admin/adversary_sim_control.rs`
- `src/lib.rs` (route wiring only, no policy logic)

Acceptance criteria:
1. No dashboard request is required for generation ticks while simulation is running.
2. Heartbeat execution path is capability-gated and auditable (actor, reason, outcome, failure taxonomy).
3. `GET /admin/adversary-sim/status` remains non-mutating.

### SIM-HB-3: Lifecycle Inertness (Spawn-On-Enable / Teardown-On-Disable)

Scope:
1. Implement supervisor/generator lifecycle so resources are allocated only while simulation is enabled and running.
2. Enforce hard stop semantics on disable, expiry, and kill-switch paths.
3. Emit explicit degraded/failure states when heartbeat cannot run.

Primary touchpoints:
- `src/admin/adversary_sim.rs`
- `src/admin/api.rs`
- `Makefile` (operator targets only; no dashboard coupling)
- `docs/adversarial-operator-guide.md`

Acceptance criteria:
1. Toggle ON transitions to active heartbeat/generation state within bounded startup window.
2. Toggle OFF guarantees teardown and zero emitted simulation traffic after bounded drain window.
3. While OFF, diagnostics report no active heartbeat loop and no active worker/process.
4. Failure to start or sustain heartbeat reports explicit degraded state; it must not be reported as success.

### SIM-HB-4: Dashboard Decoupling and Toggle Stability

Scope:
1. Remove dashboard-owned tick loop and any UI path that drives traffic generation.
2. Eliminate optimistic toggle writes that race against backend truth.
3. Keep dashboard behavior as control + status + monitoring renderer only.

Primary touchpoints:
- `dashboard/src/routes/+page.svelte`
- `dashboard/src/lib/domain/api-client.js`
- `dashboard/src/lib/runtime/dashboard-adversary-sim.js`
- `e2e/dashboard.modules.unit.test.js`
- `e2e/dashboard.smoke.spec.js`

Acceptance criteria:
1. Dashboard no longer calls `/admin/adversary-sim/tick`.
2. Toggle UI no longer flashes between contradictory states during enable/disable transitions.
3. Only backend status responses can finalize rendered simulator state.

### SIM-HB-5: Freshness Ownership Simplification

Scope:
1. Keep freshness computation backend-owned only.
2. Remove frontend merge/fallback behavior that can synthesize contradictory lag/last-event combinations.
3. Render backend freshness as-is for monitoring and IP-ban surfaces.

Primary touchpoints:
- `src/admin/api.rs`
- `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
- `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
- `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
- `e2e/dashboard.modules.unit.test.js`

Acceptance criteria:
1. UI freshness line cannot flip between incompatible backend states due to frontend synthesis.
2. `lag_ms=0` with `last_event_ts=null` only appears when backend intentionally reports that state.
3. Monitoring and IP-bans tabs use the same freshness ownership model.

### SIM-HB-6: Deterministic Traffic Breadth and Cadence Hardening

Scope:
1. Increase deterministic-lane traffic cadence so monitoring shows meaningful activity promptly.
2. Expand deterministic scenarios to exercise broader defense/event categories (not only honeypot->challenge->ban chain).
3. Preserve replayability and bounded resource use.

Primary touchpoints:
- `src/admin/adversary_sim.rs`
- `scripts/tests/adversarial/scenario_manifest.v2.json`
- `scripts/tests/adversarial/coverage_contract.v2.json`
- `scripts/tests/test_adversarial_simulation_runner.py`
- `src/admin/api.rs` (event/category assertions)

Acceptance criteria:
1. Runtime toggle run produces diverse event families across configured defenses under deterministic profile.
2. Event emission cadence is visibly faster than current baseline and remains bounded by guardrails.
3. Deterministic replay remains stable for regression use.

### SIM-HB-7: Verification, Documentation, and Rollout Safety

Scope:
1. Add regression coverage for heartbeat autonomy, toggle stability, freshness rendering stability, and inertness.
2. Update operator/admin docs and API contracts.
3. Define rollback switches and failure playbook.

Primary touchpoints:
- `src/admin/api.rs` tests
- `e2e/dashboard.modules.unit.test.js`
- `e2e/dashboard.smoke.spec.js`
- `docs/api.md`
- `docs/dashboard.md`
- `docs/testing.md`
- `docs/adversarial-operator-guide.md`

Acceptance criteria:
1. Tests prove generation continues without dashboard tab lifecycle dependence.
2. Tests prove OFF state has no active heartbeat/generator and emits no simulation traffic.
3. Docs clearly separate control plane, heartbeat supervisor, attacker plane, and monitoring renderer responsibilities.
4. Rollback path is documented and includes explicit disable steps and diagnostics checks.

## Verification Strategy (Makefile Canonical)

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-dashboard-e2e` (with `make dev` running)
4. `make test-adversarial-fast` (with `make dev` running)
5. `make test`

## Rollback Strategy

1. Force adversary simulation OFF (`POST /admin/adversary-sim/control`), verify teardown state and zero simulation emission.
2. Disable supervisor heartbeat path via config/feature gate while preserving status/monitoring read surfaces.
3. Keep historical telemetry visible; do not couple rollback with history deletion.
4. Re-enable only after heartbeat diagnostics and toggle/freshness regression suites are green.

## Definition of Done

1. Traffic generation is backend-owned and independent of dashboard lifecycle.
2. OFF state is inert by contract and test-verified.
3. Dashboard has no traffic-generation responsibility and no optimistic toggle race rendering.
4. Freshness has one owner (backend), with UI as renderer only.
5. Deterministic lane remains reliable regression oracle; LLM lane remains explicit follow-up (`SIM-LLM-1`).
