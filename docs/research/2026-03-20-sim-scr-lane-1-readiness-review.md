Date: 2026-03-20
Status: Active readiness review

Related context:

- [`2026-03-20-shared-host-seed-contract-post-implementation-review.md`](./2026-03-20-shared-host-seed-contract-post-implementation-review.md)
- [`2026-03-20-sim-deploy-2-post-implementation-review.md`](./2026-03-20-sim-deploy-2-post-implementation-review.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md`](../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/runtime/dashboard-adversary-sim.js`](../../dashboard/src/lib/runtime/dashboard-adversary-sim.js)

# Purpose

Assess whether `SIM-SCR-LANE-1` is execution-ready after the 2026-03-20 shared-host and production-envelope closeouts, and identify the safest first slices for the runtime lane migration.

# Current Architectural Position

## 1. The external blockers are cleared

`SIM-SCR-LANE-1` was previously gated on two things:

1. a truthful production operating envelope for adversary-sim,
2. and a minimal shared-host scope-and-seed contract for later Scrapling traversal.

Both are now complete as of 2026-03-20, so the next blocker is local to the lane tranche itself rather than to upstream readiness.

## 2. The code still reflects the toggle-only baseline

The current code-truth seam is still the pre-migration model:

1. persisted control state tracks `desired_enabled`, lifecycle phase, and counts only,
2. `POST /admin/adversary-sim/control` accepts `enabled` plus optional `reason`,
3. status reports legacy `lanes.deterministic` and `lanes.containerized` labels plus `active_lane_count`,
4. and the dashboard adapters normalize only that legacy status shape.

This means the first lane slice must be additive and migration-safe rather than a breaking rewrite.

## 3. The highest current risk is blended migration scope

The mature roadmap makes Scrapling the first primary adaptive lane, but that does not mean the worker integration should land first.

The local architectural risk is combining:

1. state-model migration,
2. control-write semantics,
3. worker routing,
4. and dashboard controls

into one patch series.

That would make it hard to tell whether regressions come from contract drift, routing bugs, or UI/client mismatch.

# Readiness Findings

## 1. `SIM-SCR-LANE-1` is startable now

No upstream blocker remains in front of the runtime lane tranche.

The repo now has:

1. the production-safe supervisor baseline,
2. the minimal shared-host scope fence,
3. the minimal shared-host seed contract,
4. and the roadmap decision that traversal telemetry, not a catalog, becomes the reachable-surface map.

## 2. The first slice should be contract scaffolding, not worker code

The current dashboard and API normalization still expect:

1. `active_lane_count`,
2. and `lanes.{deterministic,containerized}`.

So the first runtime-lane slice should add the migration fields without removing or redefining those legacy keys yet.

## 3. Desired and active lane must be separate authorities

The lane migration should mirror the lifecycle split Shuma already uses for enablement:

1. `desired_lane` is the persisted operator choice,
2. `active_lane` is the lane actually executing on the current beat,
3. and lane switches become observable transitions rather than an inferred side effect of lifecycle phase.

Without that split, Scrapling integration would recreate the same class of status/control ambiguity that `SIM-DEPLOY-2` just removed for enablement state.

# Recommended Execution Order

1. `SIM-SCR-0` Add additive status-contract and diagnostics scaffolding.
2. `SIM-SCR-1` Add persisted desired/active lane state plus control-lane validation and audit semantics.
3. `SIM-SCR-6` Add bounded Scrapling worker routing under supervisor heartbeat ownership.
4. `SIM-SCR-7` Add dashboard lane controls and diagnostics after the backend contract is stable.
5. `SIM-SCR-8` Close operator workflow, Make targets, rollout/rollback guidance, and evidence receipts.

# Why This Order Is Correct

## First: additive contract truth

The first backend slice should make the new lane model visible and testable without changing runtime routing yet.

That gives the later worker and dashboard slices one stable contract to target.

## Second: persisted control semantics

Once the contract exists, the control path can safely add lane selection and auditability while preserving the top-level enable/disable behavior.

## Third: worker routing

Only after the state and status contract is stable should the supervisor begin dispatching a non-deterministic lane.

## Fourth: dashboard controls

The UI should project an already-proven backend contract rather than drive its definition.

## Fifth: docs and rollout

Operator workflow and Make target guidance should describe the settled contract, not an intermediate migration state.

# Boundaries For The First Two Slices

1. Keep legacy status fields until dashboard and API-client migration is complete.
2. Keep the top-level `enabled` flag as the first-class ON/OFF control.
3. Allow lane selection to be persisted separately from ON/OFF state.
4. Do not start with Scrapling worker integration, browser-heavy fetch defaults, or new discovery artifacts.
5. Do not reintroduce a public-surface catalog as a prerequisite or a hidden side effect of the lane work.

# Outcome

Treat `SIM-SCR-LANE-1` as execution-ready on 2026-03-20.

The first optimal tranche is not Scrapling execution yet.

It is the backend lane-state migration contract:

1. additive status fields and diagnostics first,
2. then persisted desired/active lane semantics,
3. then worker routing on top of that cleaner base.
